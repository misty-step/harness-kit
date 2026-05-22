#!/usr/bin/env python3
"""Append one normalized delegation receipt row."""

from __future__ import annotations

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "lib"))

from agent_roster import (  # noqa: E402
    append_receipt,
    build_attempt_receipt,
    load_roster,
    parse_common_args,
    validate_roster,
)


def main() -> int:
    parser = parse_common_args("Record a delegation attempt.")
    parser.add_argument("--provider-target", required=True)
    parser.add_argument("--provider-status", required=True)
    parser.add_argument("--attempt-status", required=True)
    parser.add_argument("--objective", required=True)
    parser.add_argument("--input-ref", required=True)
    parser.add_argument("--evidence-ref", action="append", default=[])
    parser.add_argument("--lead-verdict", default="pending")
    parser.add_argument("--worktree-id", required=True)
    parser.add_argument("--backlog-ref", default="")
    parser.add_argument("--lead-harness", default="unknown")
    parser.add_argument("--lead-provider", default="unknown")
    parser.add_argument("--summary", default="")
    args = parser.parse_args()

    validate_roster(load_roster(args.roster))
    receipt = build_attempt_receipt(
        provider_target=args.provider_target,
        provider_status=args.provider_status,
        attempt_status=args.attempt_status,
        objective=args.objective,
        input_ref=args.input_ref,
        evidence_refs=args.evidence_ref,
        lead_verdict=args.lead_verdict,
        worktree_id=args.worktree_id,
        backlog_ref=args.backlog_ref,
        lead_harness=args.lead_harness,
        lead_provider=args.lead_provider,
        summary=args.summary,
    )
    append_receipt(args.receipt_output, receipt)
    print(receipt["delegation_id"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
