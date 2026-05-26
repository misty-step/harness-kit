#!/usr/bin/env python3
"""Print an operator-facing summary for delegation receipts."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "lib"))

from agent_roster import default_receipt_path, summarize_receipts  # noqa: E402


def _format_counts(counts: dict[str, int]) -> str:
    if not counts:
        return "none"
    return ", ".join(f"{key}={value}" for key, value in sorted(counts.items()))


def print_text_report(summary: dict[str, object]) -> None:
    backlog_ref = summary.get("backlog_ref") or "all receipts"
    print("Roster delegation report")
    print(f"backlog_ref: {backlog_ref}")
    print(f"total_receipts: {summary['total']}")
    print("providers:")
    providers = summary.get("providers", {})
    provider_statuses = summary.get("provider_statuses", {})
    if isinstance(providers, dict) and providers:
        for provider, attempts in sorted(providers.items()):
            statuses = provider_statuses.get(provider, {}) if isinstance(provider_statuses, dict) else {}
            print(
                f"  - {provider}: attempts[{_format_counts(attempts)}]; "
                f"status[{_format_counts(statuses)}]"
            )
    else:
        print("  - none")
    print(f"lead_verdicts: {_format_counts(summary.get('lead_verdicts', {}))}")
    print(f"worktrees: {_format_counts(summary.get('worktrees', {}))}")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path", nargs="?", type=Path, default=None)
    parser.add_argument("--backlog-ref", default="")
    parser.add_argument("--format", choices=("json", "text"), default="json")
    args = parser.parse_args()

    path = args.path if args.path is not None else default_receipt_path()
    summary = summarize_receipts(path, backlog_ref=args.backlog_ref)
    if args.format == "text":
        print_text_report(summary)
    else:
        print(json.dumps(summary, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
