#!/usr/bin/env python3
"""Report first-party skill quality coverage.

Read-only by design: this script surfaces gaps for /groom audit. It does not
rewrite skills, add evals, or mutate backlog state.
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable


GENERIC_DESCRIPTION_RE = re.compile(
    r"^\s*(use this skill to|this skill|helps?|guides?)\b", re.IGNORECASE
)
TRIGGER_RE = re.compile(
    r"\b(use when|use for|triggered by|when the user|applies when|invoke when|"
    r"for tasks involving)\b|\btriggers?:",
    re.IGNORECASE,
)
TESTING_RE = re.compile(
    r"^#{1,3}\s*(testing|verification|evals?)\b", re.IGNORECASE | re.MULTILINE
)


@dataclass(frozen=True)
class Check:
    passed: bool
    detail: str


@dataclass(frozen=True)
class SkillAudit:
    name: str
    frontmatter: Check
    trigger: Check
    tests: Check
    routed: Check

    @property
    def failures(self) -> int:
        return sum(
            1
            for check in (self.frontmatter, self.trigger, self.tests, self.routed)
            if not check.passed
        )


def repo_root(start: Path) -> Path:
    result = subprocess.run(
        ["git", "-C", str(start), "rev-parse", "--show-toplevel"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        raise SystemExit(result.stderr.strip() or "not inside a git repository")
    return Path(result.stdout.strip()).resolve()


def frontmatter(text: str) -> dict[str, str]:
    if not text.startswith("---\n"):
        return {}
    try:
        raw = text.split("---", 2)[1]
    except IndexError:
        return {}

    data: dict[str, str] = {}
    current: str | None = None
    for line in raw.splitlines():
        if not line.strip():
            continue
        if line.startswith((" ", "\t")) and current:
            data[current] = f"{data[current]} {line.strip()}".strip()
            continue
        if ":" not in line:
            current = None
            continue
        key, value = line.split(":", 1)
        key = key.strip()
        value = value.strip()
        if value in {"|", ">"}:
            value = ""
        data[key] = value.strip('"').strip("'")
        current = key
    return data


def has_testing_evidence(skill_dir: Path, body: str) -> Check:
    evidence: list[str] = []
    for dirname in ("tests", "test", "evals"):
        path = skill_dir / dirname
        if path.is_dir() and any(child.is_file() for child in path.rglob("*")):
            evidence.append(f"{dirname}/")
    scripts_dir = skill_dir / "scripts"
    if scripts_dir.is_dir():
        for pattern in ("test_*.sh", "*_test.sh", "test_*.py", "*_test.py"):
            if any(scripts_dir.rglob(pattern)):
                evidence.append(f"scripts/{pattern}")
                break
    if TESTING_RE.search(body):
        evidence.append("SKILL.md testing section")

    if evidence:
        return Check(True, ", ".join(sorted(set(evidence))))
    return Check(False, "no tests/, evals/, test script, or Testing/Verification section")


def is_routed(name: str, docs: Iterable[tuple[Path, str]]) -> Check:
    needles = (f"/{name}", f"skills/{name}", f"`{name}`", f"| `{name}`", f"| /{name}")
    hits = [str(path) for path, text in docs if any(needle in text for needle in needles)]
    if hits:
        return Check(True, ", ".join(hits))
    return Check(False, "not referenced from harnesses/shared/AGENTS.md routing")


def audit_skill(skill_dir: Path, routing_docs: list[tuple[Path, str]]) -> SkillAudit:
    skill_md = skill_dir / "SKILL.md"
    text = skill_md.read_text(encoding="utf-8")
    meta = frontmatter(text)
    name = skill_dir.name
    desc = " ".join(str(meta.get("description", "")).split())
    fm_missing = [field for field in ("name", "description") if not meta.get(field)]
    name_mismatch = meta.get("name") not in (None, "", name)

    if fm_missing:
        fm = Check(False, f"missing frontmatter field(s): {', '.join(fm_missing)}")
    elif name_mismatch:
        fm = Check(False, f"frontmatter name {meta.get('name')!r} differs from directory {name!r}")
    else:
        fm = Check(True, "name and description present")

    if not desc:
        trigger = Check(False, "description missing")
    elif GENERIC_DESCRIPTION_RE.search(desc) and not TRIGGER_RE.search(desc):
        trigger = Check(False, "description is generic and lacks trigger language")
    elif not TRIGGER_RE.search(desc):
        trigger = Check(False, "description lacks concrete trigger phrase")
    else:
        trigger = Check(True, "description has trigger language")

    return SkillAudit(
        name=name,
        frontmatter=fm,
        trigger=trigger,
        tests=has_testing_evidence(skill_dir, text),
        routed=is_routed(name, routing_docs),
    )


def render_report(audits: list[SkillAudit], routing_paths: list[Path]) -> str:
    ordered = sorted(audits, key=lambda item: (-item.failures, item.name))
    counts = {
        failures: sum(1 for item in audits if item.failures == failures)
        for failures in range(5)
    }

    lines = [
        "# Skill Quality Audit",
        "",
        f"Skills audited: {len(audits)}",
        f"Routing docs: {', '.join(str(path) for path in routing_paths) or 'none'}",
        "",
        "## Summary",
        "",
        "| Failed dimensions | Skills |",
        "|---:|---:|",
    ]
    for failures in range(4, -1, -1):
        lines.append(f"| {failures} | {counts[failures]} |")

    lines.extend(["", "## Findings", ""])
    for audit in ordered:
        verdict = "PASS" if audit.failures == 0 else f"FAIL {audit.failures}/4"
        lines.append(f"### {audit.name} - {verdict}")
        for label, check in (
            ("frontmatter", audit.frontmatter),
            ("trigger", audit.trigger),
            ("tests", audit.tests),
            ("routing", audit.routed),
        ):
            mark = "PASS" if check.passed else "FAIL"
            lines.append(f"- {label}: {mark} - {check.detail}")
        lines.append("")
    return "\n".join(lines).rstrip() + "\n"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", default=".", help="Path inside the target repository.")
    parser.add_argument("--output", help="Optional path to write the report.")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    root = repo_root(Path(args.repo).expanduser().resolve())
    skills_root = root / "skills"
    routing_paths = [root / "harnesses" / "shared" / "AGENTS.md"]
    routing_paths = [path for path in routing_paths if path.is_file()]
    routing_docs = [
        (path.relative_to(root), path.read_text(encoding="utf-8"))
        for path in routing_paths
    ]
    audits = [
        audit_skill(skill_dir, routing_docs)
        for skill_dir in sorted(skills_root.iterdir())
        if (skill_dir / "SKILL.md").is_file()
    ]
    report = render_report(audits, [path.relative_to(root) for path in routing_paths])
    if args.output:
        output = Path(args.output)
        if not output.is_absolute():
            output = root / output
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(report, encoding="utf-8")
    else:
        print(report, end="")
    return 0


if __name__ == "__main__":
    sys.exit(main())
