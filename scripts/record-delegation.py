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
    validate_usage,
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
    parser.add_argument("--model-id", default=None)
    parser.add_argument("--duration-ms", type=int, default=None)
    parser.add_argument("--transcript-bytes", type=int, default=None)
    parser.add_argument("--input-tokens", type=int, default=None)
    parser.add_argument("--output-tokens", type=int, default=None)
    parser.add_argument("--total-tokens", type=int, default=None)
    parser.add_argument("--cost-usd", type=float, default=None)
    parser.add_argument(
        "--cost-source",
        choices=("provider_reported", "estimated", "manual", "unknown"),
        default=None,
    )
    args = parser.parse_args()

    validate_roster(load_roster(args.roster))
    usage = _usage_from_args(args)
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
        model_id=args.model_id,
        duration_ms=args.duration_ms,
        usage=usage,
        transcript_bytes=args.transcript_bytes,
    )
    append_receipt(args.receipt_output, receipt)
    print(receipt["delegation_id"])
    return 0


def _usage_from_args(args) -> dict[str, object] | None:
    if not any(
        value is not None
        for value in (
            args.input_tokens,
            args.output_tokens,
            args.total_tokens,
            args.cost_usd,
            args.cost_source,
        )
    ):
        return None
    usage = {
        "input_tokens": args.input_tokens,
        "output_tokens": args.output_tokens,
        "total_tokens": args.total_tokens,
        "cost_usd": args.cost_usd,
        "cost_source": args.cost_source or ("manual" if args.cost_usd is not None else "unknown"),
    }
    validate_usage(usage)
    return usage


if __name__ == "__main__":
    raise SystemExit(main())
