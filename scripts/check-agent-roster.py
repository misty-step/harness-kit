#!/usr/bin/env python3
"""Validate committed agent-roster config and receipt fixtures."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "lib"))

from agent_roster import load_roster, read_receipts, validate_roster  # noqa: E402

CORE_WORKFLOW_SKILLS = [
    "ci",
    "code-review",
    "deliver",
    "demo",
    "diagnose",
    "flywheel",
    "groom",
    "hardening",
    "harness",
    "implement",
    "monitor",
    "qa",
    "refactor",
    "reflect",
    "research",
    "settle",
    "shape",
    "ship",
    "yeet",
]


def validate_delegation_floor() -> None:
    missing = []
    weak = []
    ambiguous = []
    root = Path("skills")
    for skill in CORE_WORKFLOW_SKILLS:
        path = root / skill / "SKILL.md"
        if not path.exists():
            continue
        text = path.read_text()
        if "## Delegation Floor" not in text:
            missing.append(str(path))
            continue
        has_roster_contract = (
            "provider roster is available" in text or ".spellbook/agents.yaml" in text
        )
        if "two or more" not in text or not has_roster_contract:
            weak.append(str(path))
        if skill in {"shape", "research", "harness"}:
            lowered = text.lower()
            if "native in-thread subagents" not in lowered or "do not" not in lowered:
                ambiguous.append(str(path))

    errors = []
    if missing:
        errors.append("missing delegation floor: " + ", ".join(missing))
    if weak:
        errors.append("weak delegation floor: " + ", ".join(weak))
    if ambiguous:
        errors.append(
            "ambiguous roster/subagent boundary: " + ", ".join(ambiguous)
        )
    if errors:
        raise SystemExit("; ".join(errors))


def validate_shared_roster_doctrine() -> None:
    path = Path("harnesses/shared/AGENTS.md")
    text = path.read_text().lower()
    required = [
        "native in-thread subagents",
        "satisfy the roster floor",
        "configured provider ids",
        "a probe is not a provider attempt",
    ]
    missing = [phrase for phrase in required if phrase not in text]
    if missing:
        raise SystemExit(
            f"{path}: missing roster doctrine phrase(s): {', '.join(missing)}"
        )


def validate_no_source_skill_bridges() -> None:
    forbidden = [
        Path(".agents/skills"),
        Path(".codex/skills"),
        Path(".claude/skills"),
        Path(".pi/skills"),
    ]
    present = [str(path) for path in forbidden if path.exists()]
    if present:
        raise SystemExit(
            "source repo must not commit repo-local skill bridges: "
            + ", ".join(present)
        )


def main() -> int:
    roster_path = Path(".spellbook/agents.yaml")
    fixture_path = Path(".spellbook/examples/delegation-receipt.jsonl")
    gitignore_path = Path(".gitignore")
    summary_script = Path("scripts/summarize-delegations.py")

    validate_roster(load_roster(roster_path))
    validate_delegation_floor()
    validate_shared_roster_doctrine()
    validate_no_source_skill_bridges()
    receipts = read_receipts(fixture_path)
    if not receipts:
        raise SystemExit(f"{fixture_path}: must contain at least one receipt fixture")
    if not summary_script.exists():
        raise SystemExit(f"{summary_script}: missing roster report helper")
    completed = subprocess.run(
        ["python3", str(summary_script), "--format", "text", str(fixture_path)],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.returncode != 0 or "Roster delegation report" not in completed.stdout:
        detail = (completed.stderr or completed.stdout).strip().splitlines()
        suffix = f": {detail[-1]}" if detail else ""
        raise SystemExit(f"{summary_script}: roster report helper failed{suffix}")

    gitignore = gitignore_path.read_text()
    if ".spellbook/traces/*.jsonl" not in gitignore:
        raise SystemExit(".gitignore must ignore runtime delegation JSONL traces")

    forbidden_dirs = [
        ".spellbook/auth",
        ".spellbook/sessions",
        ".spellbook/provider-sessions",
        ".spellbook/raw-transcripts",
    ]
    present = [path for path in forbidden_dirs if Path(path).exists()]
    if present:
        raise SystemExit(f"forbidden provider runtime directories: {', '.join(present)}")

    print(f"{roster_path}: valid")
    print(f"{fixture_path}: {len(receipts)} receipt fixture(s) valid")
    print(f"skills/: {len(CORE_WORKFLOW_SKILLS)} delegation floor(s) valid")
    print("source repo: no repo-local skill bridges")
    print(f"{summary_script}: report helper valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
