#!/usr/bin/env python3
"""Validate committed agent-roster config and receipt fixtures."""

from __future__ import annotations

import re
import subprocess
import sys
from datetime import UTC, datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "lib"))

from agent_roster import load_roster, read_receipts, validate_roster  # noqa: E402

CORE_WORKFLOW_SKILLS = [
    "ci",
    "code-review",
    "create-repo-skill",
    "deliver",
    "demo",
    "design",
    "diagnose",
    "flywheel",
    "groom",
    "hardening",
    "harness-engineering",
    "implement",
    "monitor",
    "qa",
    "refactor",
    "reflect",
    "research",
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

DELEGATION_FLOOR_COMMITMENTS = {
    "two-provider commitment": [
        r"\b(dispatch\w*|uses?|requires?|verif\w*|show)\b[^.]{0,160}\btwo or more\b",
        r"\btwo or more\b[^.]{0,160}\b(dispatch\w*|uses?|requires?|verif\w*|show)\b",
    ],
    "direct-work exception commitment": [
        r"\b(direct\w*|lead-only)\b[^.]{0,160}\b(is for|limited to|only|exceptions?|mechanical|emergency|user(?:-forbidden| forbids)|fewer than two)\b",
        r"\b(except for|limited to)\b[^.]{0,160}\b(mechanical|emergency|user(?:-forbidden| forbids)|fewer than two)\b",
    ],
    "scoped lane handoff": [
        r"\b(give|gives)\b[^.]{0,80}\b(lane|lanes|each lane|providers?|members?|reviewers?|them)\b[^.]{0,160}\b(scoped|scope|context|files|questions|commands|output|evidence|receipt|sources|methods|risk|boundar\w*)\b",
        r"\bscoped\b[^.]{0,80}\b(lane|lanes|each lane|providers?|members?|reviewers?)\b",
        r"\buse\b[^.]{0,80}\blanes?\b[^.]{0,160}\b(scoped|scope|context|files|questions|commands|output|evidence|receipt|sources|methods|risk|boundar\w*)\b",
    ],
    "lead-owned synthesis": [
        r"\bthe lead(?: agent| model)?\b(?=[^.]{0,160}\b(owns|verif\w*|records?|accepts?|keeps|synthesis|final)\b)",
        r"\blead agent\b(?=[^.]{0,160}\b(owns|verif\w*|records?|accepts?|keeps|synthesis|final)\b)",
        r"\blead synthesis\b",
    ],
}

HEDGED_COMMITMENT_PATTERN = re.compile(
    r"\b(may|might|optional|whether|if available|at [^.]{0,40} discretion|"
    r"decide later|reminders only|only matters)\b"
)

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
    "shape",
]

COMPLETION_EVIDENCE_CORE_SKILLS = [
    "code-review",
    "deliver",
    "implement",
    "refactor",
]

DOMAIN_COMPLETION_GATE_SKILLS = [
    "demo",
    "design",
    "hardening",
    "qa",
]

CLEAN_CLOSEOUT_POINTER_PATHS = [
    Path("AGENTS.md"),
    Path("skills/deliver/SKILL.md"),
    Path("skills/ship/SKILL.md"),
    Path("skills/yeet/SKILL.md"),
]

COMPLETION_EVIDENCE_REQUIREMENTS = {
    "behavior": ["behavior", "end-user", "developer", "operator"],
    "live evidence": ["live evidence"],
    "exercised surface": ["command", "path", "route", "artifact", "surface"],
    "repo fit": ["repo-fit"],
    "residual risk": ["residual", "waiver", "follow-up"],
}

RETIRED_PROVIDER_REFERENCE_PATHS = [
    # Active sources only. Closed backlog and trace receipts remain historical
    # records and may mention retired providers.
    Path(".harness-kit/agents.yaml"),
    Path("scripts/lib/agent_roster.py"),
    Path("harnesses/shared/AGENTS.md"),
    Path("docs/copy/site.json"),
    Path("harnesses/pi/README.md"),
    Path("harnesses/pi/settings.json"),
    Path("skills/harness-engineering/references/open-model-roster.md"),
]
RETIRED_PROVIDER_PATTERN = r"\bopen[- ]?code\b|\bopencode\b"
OPEN_MODEL_ROSTER_PATH = Path("skills/harness-engineering/references/open-model-roster.md")


def markdown_section(text: str, heading: str) -> str:
    start = text.find(heading)
    if start == -1:
        return ""
    end = text.find("\n## ", start + 1)
    return text[start:] if end == -1 else text[start:end]


def delegation_floor_section(text: str) -> str:
    return markdown_section(text, "## Delegation Floor")


def has_delegation_floor_pointer(section: str) -> bool:
    """A skill may point to the shared single source instead of restating the
    full floor (backlog 081). The pointer must still say the floor applies
    (one-line restatement) AND link the canonical source — never a bare 'see X'.
    The canonical phrase requirements are validated once against
    harnesses/shared/AGENTS.md ## Roster in validate_shared_roster_doctrine()."""
    low = section.lower()
    return (
        "delegation floor applies" in low
        and "harnesses/shared/agents.md" in low
        and "roster" in low
    )


def has_local_lane_guidance(section: str) -> bool:
    """Pointer-mode skill sections must preserve local phase guidance.

    Backlog 081 intentionally removes generic delegation-floor boilerplate, not
    the skill-specific sentence that tells operators what lanes to use.
    """
    match = re.search(r"(?im)^local lane guidance:\s*(.+)$", section)
    return bool(match and match.group(1).strip())


def delegation_contract_gaps(section: str) -> list:
    """Requirement labels missing from a full delegation-floor / roster section."""
    lowered = section.lower()
    missing = [
        name
        for name, phrases in DELEGATION_FLOOR_REQUIREMENTS.items()
        if not any(phrase in lowered for phrase in phrases)
    ]
    if (
        "provider roster is available" not in lowered
        and ".harness-kit/agents.yaml" not in lowered
    ):
        missing.append("roster availability")
    flattened = re.sub(r"\s+", " ", lowered)

    def has_unhedged_match(patterns: list[str]) -> bool:
        for pattern in patterns:
            for match in re.finditer(pattern, flattened):
                sentence_end = flattened.find(".", match.start())
                if sentence_end == -1:
                    sentence_end = min(len(flattened), match.end() + 160)
                window = flattened[max(0, match.start() - 40) : sentence_end]
                if not HEDGED_COMMITMENT_PATTERN.search(window):
                    return True
        return False

    missing.extend(
        name
        for name, patterns in DELEGATION_FLOOR_COMMITMENTS.items()
        if not has_unhedged_match(patterns)
    )
    return missing


def phrase_group_gaps(text: str, requirements: dict[str, list[str]]) -> list[str]:
    lowered = text.lower()
    return [
        name
        for name, phrases in requirements.items()
        if not any(phrase in lowered for phrase in phrases)
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
        section = delegation_floor_section(text)
        if not section:
            missing.append(str(path))
            continue
        if skill in {"shape", "research", "harness-engineering", "create-repo-skill"}:
            lowered = text.lower()
            if "native in-thread subagents" not in lowered or "do not" not in lowered:
                ambiguous.append(str(path))
                continue
        # A skill may EITHER restate the full floor OR point to the shared
        # single source (harnesses/shared/AGENTS.md ## Roster). Pointer mode
        # passes here; the canonical phrases are validated once against the
        # shared source in validate_shared_roster_doctrine().
        if has_delegation_floor_pointer(section):
            if not has_local_lane_guidance(section):
                weak.append(f"{path} (missing local lane guidance)")
            continue
        lowered_section = section.lower()
        missing_requirements = delegation_contract_gaps(section)
        if missing_requirements:
            weak.append(f"{path} ({', '.join(missing_requirements)})")
            continue
        if "explicit user waivers" in lowered_section:
            weak.append(str(path))

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
    # The shared ## Roster section is the single source for the delegation
    # floor (backlog 081): skills may point to it instead of restating it, so
    # validate the full contract here, once.
    roster_section = markdown_section(path.read_text(), "## Roster")
    if not roster_section:
        raise SystemExit(f"{path}: missing '## Roster' single-source section")
    gaps = delegation_contract_gaps(roster_section)
    if gaps:
        raise SystemExit(
            f"{path} (## Roster): missing delegation-contract requirement(s): "
            + ", ".join(gaps)
        )


def validate_completion_evidence() -> None:
    shared_path = Path("harnesses/shared/AGENTS.md")
    shared_section = markdown_section(shared_path.read_text(), "## Completion Evidence")
    if not shared_section:
        raise SystemExit(f"{shared_path}: missing '## Completion Evidence' section")
    gaps = phrase_group_gaps(shared_section, COMPLETION_EVIDENCE_REQUIREMENTS)
    if gaps:
        raise SystemExit(
            f"{shared_path} (## Completion Evidence): missing requirement(s): "
            + ", ".join(gaps)
        )

    issues = []
    for skill in COMPLETION_EVIDENCE_CORE_SKILLS:
        path = Path("skills") / skill / "SKILL.md"
        text = path.read_text().lower()
        missing = [
            phrase
            for phrase in [
                "completion evidence core applies",
                "harnesses/shared/agents.md",
                "completion evidence",
                "local fields",
            ]
            if phrase not in text
        ]
        if missing:
            issues.append(
                f"{path}: missing completion evidence pointer ({', '.join(missing)})"
            )

    for skill in DOMAIN_COMPLETION_GATE_SKILLS:
        path = Path("skills") / skill / "SKILL.md"
        text = path.read_text().lower()
        if "## completion gate" not in text and "### completion gate" not in text:
            issues.append(f"{path}: missing local completion gate")
        if (
            "harnesses/shared/agents.md" not in text
            or "completion evidence" not in text
        ):
            issues.append(f"{path}: missing shared Completion Evidence pointer")
    if issues:
        raise SystemExit("; ".join(issues))


def validate_clean_closeout_pointers() -> None:
    shared_path = Path("harnesses/shared/AGENTS.md")
    shared_section = markdown_section(shared_path.read_text(), "## Closeout")
    required = [
        "single source for clean-tree closeout",
        "git status --short --untracked-files=all",
        "committed, deleted, moved out, or durably ignored",
    ]
    missing = [phrase for phrase in required if phrase not in shared_section.lower()]
    if missing:
        raise SystemExit(
            f"{shared_path} (## Closeout): missing phrase(s): " + ", ".join(missing)
        )

    issues = []
    for path in CLEAN_CLOSEOUT_POINTER_PATHS:
        text = path.read_text().lower()
        if "harnesses/shared/agents.md" not in text or "closeout" not in text:
            issues.append(f"{path}: missing shared Closeout pointer")
    if issues:
        raise SystemExit("; ".join(issues))


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


def validate_no_retired_provider_references() -> None:
    hits = []
    for path in RETIRED_PROVIDER_REFERENCE_PATHS:
        if not path.exists():
            continue
        for lineno, line in enumerate(path.read_text().splitlines(), start=1):
            if "RETIRED_RECEIPT_PROVIDER_IDS" in line:
                continue
            if re.search(RETIRED_PROVIDER_PATTERN, line, flags=re.IGNORECASE):
                hits.append(f"{path}:{lineno}")
    if hits:
        raise SystemExit(
            "retired provider reference(s) in active roster/docs: "
            + ", ".join(hits)
        )


def validate_open_model_roster_review_due() -> None:
    text = OPEN_MODEL_ROSTER_PATH.read_text()
    match = re.search(r"^roster_review_due:\s*(\d{4}-\d{2}-\d{2})$", text, re.MULTILINE)
    if not match:
        raise SystemExit(f"{OPEN_MODEL_ROSTER_PATH}: missing roster_review_due")
    review_due = datetime.strptime(match.group(1), "%Y-%m-%d").date()
    today = datetime.now(UTC).date()
    if today > review_due:
        raise SystemExit(
            f"{OPEN_MODEL_ROSTER_PATH}: roster review overdue since {review_due.isoformat()}"
        )


def main() -> int:
    roster_path = Path(".harness-kit/agents.yaml")
    fixture_path = Path(".harness-kit/examples/delegation-receipt.jsonl")
    gitignore_path = Path(".gitignore")
    summary_script = Path("scripts/summarize-delegations.py")

    validate_roster(load_roster(roster_path))
    validate_delegation_floor()
    validate_runtime_delegation_references()
    validate_shared_roster_doctrine()
    validate_completion_evidence()
    validate_clean_closeout_pointers()
    validate_adversarial_done_review()
    validate_no_source_skill_bridges()
    validate_no_retired_provider_references()
    validate_open_model_roster_review_due()
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
    if ".harness-kit/traces/*.jsonl" not in gitignore:
        raise SystemExit(".gitignore must ignore runtime delegation JSONL traces")

    forbidden_dirs = [
        ".harness-kit/auth",
        ".harness-kit/sessions",
        ".harness-kit/provider-sessions",
        ".harness-kit/raw-transcripts",
    ]
    present = [path for path in forbidden_dirs if Path(path).exists()]
    if present:
        raise SystemExit(f"forbidden provider runtime directories: {', '.join(present)}")

    print(f"{roster_path}: valid")
    print(f"{fixture_path}: {len(receipts)} receipt fixture(s) valid")
    print(f"skills/: {len(CORE_WORKFLOW_SKILLS)} delegation floor(s) valid")
    print(
        f"skills/: {len(COMPLETION_EVIDENCE_CORE_SKILLS)} "
        "completion evidence pointer(s) valid"
    )
    print(f"skills/: {len(DOMAIN_COMPLETION_GATE_SKILLS)} local completion gate(s) valid")
    print(f"closeout: {len(CLEAN_CLOSEOUT_POINTER_PATHS)} shared pointer(s) valid")
    print(f"skills/: {len(ADVERSARIAL_REVIEW_SKILLS)} adversarial review stance(s) valid")
    print(f"harnesses/: {len(RUNTIME_REFERENCES)} runtime delegation reference(s) valid")
    print("source repo: no repo-local skill bridges")
    print("active roster/docs: no retired provider references")
    print(f"{OPEN_MODEL_ROSTER_PATH}: review due date valid")
    print(f"{summary_script}: report helper valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
