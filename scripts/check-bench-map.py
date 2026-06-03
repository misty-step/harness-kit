#!/usr/bin/env python3
"""Validate /code-review bench-map reviewer ids and replacement behavior."""

from __future__ import annotations

import fnmatch
import re
import sys
from pathlib import Path

import yaml


BENCH_MAP = Path("skills/code-review/references/bench-map.yaml")
LENSES = Path("harnesses/shared/references/lenses.md")
AGENTS = Path("agents")


def lens_names() -> set[str]:
    text = LENSES.read_text()
    return {
        match.group(1)
        for match in re.finditer(r"^## ([a-z][a-z0-9-]*)\s*$", text, re.MULTILINE)
        if match.group(1) != "adding-a-lens"
    }


def known_reviewers() -> set[str]:
    agents = {path.stem for path in AGENTS.glob("*.md")}
    return lens_names() | agents


def unique(items: list[str]) -> list[str]:
    seen: set[str] = set()
    result: list[str] = []
    for item in items:
        if item in seen:
            continue
        seen.add(item)
        result.append(item)
    return result


def matches_any(path: str, patterns: list[str]) -> bool:
    return any(fnmatch.fnmatch(path, pattern) for pattern in patterns)


def selected_reviewers(changed_files: list[str], config: dict) -> list[str]:
    selected = list(config["default"])
    for rule in config.get("rules", []):
        if not any(matches_any(path, rule.get("paths", [])) for path in changed_files):
            continue
        replace = set(rule.get("replace", []))
        selected = [reviewer for reviewer in selected if reviewer not in replace]
        selected.extend(rule.get("add", []))
    selected = unique(selected)
    if "critic" not in selected:
        selected.insert(0, "critic")
    if len(selected) > 5:
        pinned = ["critic"] if "critic" in selected else []
        rest = [reviewer for reviewer in selected if reviewer != "critic"]
        selected = pinned + rest[: 5 - len(pinned)]
    return selected


def validate_ids(config: dict) -> list[str]:
    allowed = known_reviewers()
    errors: list[str] = []
    unknown = sorted(set(config.get("default", [])) - allowed)
    if unknown:
        errors.append(f"default: unknown reviewer id(s): {', '.join(unknown)}")
    for rule in config.get("rules", []):
        for field in ["add", "replace"]:
            unknown = sorted(set(rule.get(field, [])) - allowed)
            if unknown:
                errors.append(
                    f"rule {rule.get('name', '<unnamed>')} {field}: "
                    f"unknown reviewer id(s): {', '.join(unknown)}"
                )
    return errors


def main() -> int:
    config = yaml.safe_load(BENCH_MAP.read_text())
    errors = validate_ids(config)

    security = selected_reviewers(["src/auth/login.ts"], config)
    if "security" not in security:
        errors.append("security fixture did not select security")
    if "grug" in security:
        errors.append("security fixture did not replace grug")
    if len(security) > 5:
        errors.append("security fixture exceeded bench cap")

    tests = selected_reviewers(["tests/auth.spec.ts"], config)
    if "cooper" not in tests:
        errors.append("tests fixture did not select cooper")
    if len(tests) > 5:
        errors.append("tests fixture exceeded bench cap")

    if errors:
        for error in errors:
            print(f"FAIL: {error}", file=sys.stderr)
        return 1

    print("OK: bench-map reviewer ids and replacement fixtures valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
