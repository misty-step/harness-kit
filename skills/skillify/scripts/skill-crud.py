#!/usr/bin/env python3
"""Deterministic CRUD for Harness Kit skill filesystem primitives."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
import tempfile
from pathlib import Path


FRONTMATTER_RE = re.compile(r"\A---\s*\n(.*?)\n---\s*(?:\n|\Z)", re.DOTALL)
NAME_RE = re.compile(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$")
FORBIDDEN_PORTABILITY_RE = re.compile(r"\b(SendUserMessage|Edit|Skill|bash)\b")


def normalize_name(name: str) -> str:
    if "/" in name or "\\" in name or ".." in name:
        raise ValueError(f"invalid skill name: {name!r}")
    normalized = name.strip().lower().replace("_", "-")
    normalized = re.sub(r"[^a-z0-9-]+", "-", normalized)
    normalized = re.sub(r"-+", "-", normalized).strip("-")
    if not normalized or not NAME_RE.fullmatch(normalized):
        raise ValueError(f"invalid skill name: {name!r}")
    return normalized


def skill_dir(skills_root: Path, name: str) -> Path:
    root = skills_root.resolve()
    target = (root / normalize_name(name)).resolve()
    if root not in target.parents and target != root:
        raise ValueError("skill path escapes skills root")
    return target


def build_skill_md(name: str, description: str, body: str) -> str:
    clean_name = normalize_name(name)
    description = " ".join(description.strip().split())
    if not description:
        raise ValueError("description is required")
    body = body.strip() + "\n"
    return (
        "---\n"
        f"name: {clean_name}\n"
        "description: |\n"
        f"  {description}\n"
        f"argument-hint: \"[{clean_name}-args]\"\n"
        "---\n\n"
        f"# /{clean_name}\n\n"
        f"{body}"
    )


def _atomic_write(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile("w", delete=False, dir=path.parent) as handle:
        handle.write(text)
        tmp = Path(handle.name)
    tmp.replace(path)


def create_skill(skills_root: Path, name: str, description: str, body: str) -> dict[str, str]:
    target = skill_dir(skills_root, name)
    if target.exists():
        raise ValueError(f"skill already exists: {target}")
    target.mkdir(parents=True)
    _atomic_write(target / "SKILL.md", build_skill_md(name, description, body))
    return {"status": "created", "name": normalize_name(name), "path": str(target)}


def read_skill(skills_root: Path, name: str) -> dict[str, object]:
    target = skill_dir(skills_root, name)
    skill_md = target / "SKILL.md"
    if not skill_md.exists():
        raise ValueError(f"missing SKILL.md for {normalize_name(name)}")
    files = sorted(str(path.relative_to(target)) for path in target.rglob("*") if path.is_file())
    return {
        "status": "read",
        "name": normalize_name(name),
        "path": str(target),
        "files": files,
        "skill_md": skill_md.read_text(),
    }


def update_skill(
    skills_root: Path,
    name: str,
    *,
    description: str | None = None,
    body: str | None = None,
) -> dict[str, str]:
    target = skill_dir(skills_root, name)
    current = read_skill(skills_root, name)["skill_md"]
    frontmatter = parse_frontmatter(current)
    current_body = current.split("---", 2)[2].lstrip()
    new_description = description or frontmatter["description"]
    new_body = body if body is not None else re.sub(r"^# .+?\n\n", "", current_body, count=1, flags=re.DOTALL)
    _atomic_write(target / "SKILL.md", build_skill_md(name, new_description, new_body))
    return {"status": "updated", "name": normalize_name(name), "path": str(target)}


def delete_skill(skills_root: Path, name: str) -> dict[str, str]:
    target = skill_dir(skills_root, name)
    if not target.exists():
        raise ValueError(f"skill does not exist: {normalize_name(name)}")
    shutil.rmtree(target)
    return {"status": "deleted", "name": normalize_name(name), "path": str(target)}


def parse_frontmatter(text: str) -> dict[str, str]:
    match = FRONTMATTER_RE.match(text)
    if not match:
        raise ValueError("missing or malformed frontmatter")
    values: dict[str, str] = {}
    lines = match.group(1).splitlines()
    index = 0
    while index < len(lines):
        line = lines[index]
        if ":" not in line:
            index += 1
            continue
        key, raw = line.split(":", 1)
        key = key.strip()
        raw = raw.strip()
        if raw == "|":
            parts = []
            index += 1
            while index < len(lines) and lines[index].startswith("  "):
                parts.append(lines[index][2:])
                index += 1
            values[key] = "\n".join(parts).strip()
            continue
        values[key] = raw.strip('"')
        index += 1
    return values


def validate_portability(text: str) -> None:
    if FORBIDDEN_PORTABILITY_RE.search(text) and "fallback" not in text.lower():
        raise ValueError("harness-specific operation appears without fallback")


def validate_skill(skills_root: Path, name: str) -> dict[str, str]:
    target = skill_dir(skills_root, name)
    skill_md = target / "SKILL.md"
    if not skill_md.exists():
        raise ValueError("missing SKILL.md")
    text = skill_md.read_text()
    frontmatter = parse_frontmatter(text)
    expected = normalize_name(name)
    if frontmatter.get("name") != expected:
        raise ValueError("frontmatter name does not match skill directory")
    description = frontmatter.get("description", "")
    if "Trigger:" not in description:
        raise ValueError("description must include Trigger:")
    if not re.search(r'Use (?:when|for):.*"', description, re.IGNORECASE | re.DOTALL):
        raise ValueError("description must include quoted Use when/for phrases")
    validate_portability(text)
    return {"status": "valid", "name": expected, "path": str(target)}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("command", choices=["create", "read", "update", "delete", "validate"])
    parser.add_argument("--skills-root", type=Path, default=Path("skills"))
    parser.add_argument("--name", required=True)
    parser.add_argument("--description", default="")
    parser.add_argument("--body", default="")
    args = parser.parse_args(argv)

    try:
        if args.command == "create":
            result = create_skill(args.skills_root, args.name, args.description, args.body)
        elif args.command == "read":
            result = read_skill(args.skills_root, args.name)
        elif args.command == "update":
            result = update_skill(args.skills_root, args.name, description=args.description or None, body=args.body or None)
        elif args.command == "delete":
            result = delete_skill(args.skills_root, args.name)
        else:
            result = validate_skill(args.skills_root, args.name)
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
