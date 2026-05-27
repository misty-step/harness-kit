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

DELEGATION_FLOOR_REQUIREMENTS = {
    "roster floor": ["two or more"],
    "direct-work exceptions": [
        "mechanical",
        "emergency",
        "user-forbidden",
        "fewer than two",
    ],
    "lane responsibilities": ["lane"],
    "context boundary": ["context", "give", "scope"],
    "output evidence": ["receipt", "evidence"],
    "lead verification": ["lead"],
}

RUNTIME_REFERENCES = {
    "Claude Code": Path("harnesses/claude/README.md"),
    "Codex": Path("harnesses/codex/README.md"),
    "Antigravity": Path("harnesses/antigravity-cli/README.md"),
    "Pi": Path("harnesses/pi/README.md"),
}

ADVERSARIAL_REVIEW_SKILLS = [
    "code-review",
    "implement",
    "qa",
    "settle",
    "shape",
]


def delegation_floor_section(text: str) -> str:
    start = text.find("## Delegation Floor")
    if start == -1:
        return ""
    end = text.find("\n## ", start + 1)
    if end == -1:
        return text[start:]
    return text[start:end]


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
        section = delegation_floor_section(text)
        if not section:
            missing.append(str(path))
            continue
        lowered_section = section.lower()
        has_roster_contract = (
            "provider roster is available" in section
            or ".spellbook/agents.yaml" in section
        )
        missing_requirements = [
            name
            for name, phrases in DELEGATION_FLOOR_REQUIREMENTS.items()
            if not any(phrase in lowered_section for phrase in phrases)
        ]
        if not has_roster_contract:
            missing_requirements.append("roster availability")
        if missing_requirements:
            weak.append(f"{path} ({', '.join(missing_requirements)})")
            continue
        if "explicit user waivers" in lowered_section:
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


def validate_runtime_delegation_references() -> None:
    issues = []
    for runtime, path in RUNTIME_REFERENCES.items():
        if not path.exists():
            issues.append(f"{path}: missing {runtime} dynamic delegation reference")
            continue
        text = path.read_text().lower()
        missing = [
            phrase
            for phrase in [
                "dynamic delegation",
                "roster",
                "receipt",
                "evidence",
                "lead",
            ]
            if phrase not in text
        ]
        if missing:
            issues.append(f"{path}: missing phrase(s): {', '.join(missing)}")
    if issues:
        raise SystemExit("; ".join(issues))


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


def validate_adversarial_done_review() -> None:
    shared_path = Path("harnesses/shared/AGENTS.md")
    shared_text = shared_path.read_text().lower()
    shared_required = [
        "adversarial",
        "embarrass us in production",
        "automatic veto",
        "lead accepts or",
    ]
    missing = [phrase for phrase in shared_required if phrase not in shared_text]
    if missing:
        raise SystemExit(
            f"{shared_path}: missing adversarial review phrase(s): "
            + ", ".join(missing)
        )

    issues = []
    for skill in ADVERSARIAL_REVIEW_SKILLS:
        path = Path("skills") / skill / "SKILL.md"
        text = path.read_text().lower()
        if "adversarial" not in text:
            issues.append(f"{path}: missing adversarial review stance")
            continue
        if "embarrass us" not in text and "production embarrassment" not in text:
            issues.append(f"{path}: missing production-embarrassment calibration")
    if issues:
        raise SystemExit("; ".join(issues))


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
    validate_runtime_delegation_references()
    validate_shared_roster_doctrine()
    validate_adversarial_done_review()
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
    print(f"skills/: {len(ADVERSARIAL_REVIEW_SKILLS)} adversarial review stance(s) valid")
    print(f"harnesses/: {len(RUNTIME_REFERENCES)} runtime delegation reference(s) valid")
    print("source repo: no repo-local skill bridges")
    print(f"{summary_script}: report helper valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
