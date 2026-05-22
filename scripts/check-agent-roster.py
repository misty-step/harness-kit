#!/usr/bin/env python3
"""Validate committed agent-roster config and receipt fixtures."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "lib"))

from agent_roster import load_roster, read_receipts, validate_roster  # noqa: E402


def main() -> int:
    roster_path = Path(".spellbook/agents.yaml")
    fixture_path = Path(".spellbook/examples/delegation-receipt.jsonl")
    gitignore_path = Path(".gitignore")

    validate_roster(load_roster(roster_path))
    receipts = read_receipts(fixture_path)
    if not receipts:
        raise SystemExit(f"{fixture_path}: must contain at least one receipt fixture")

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
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
