#!/usr/bin/env python3
"""Validate and probe Spellbook's agent-provider roster."""

from __future__ import annotations

import json
import os
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "lib"))

from agent_roster import (  # noqa: E402
    append_receipt,
    build_probe_receipts,
    load_roster,
    parse_common_args,
    validate_roster,
)


def main() -> int:
    parser = parse_common_args("Probe the configured agent-provider roster.")
    parser.add_argument("--validate-only", action="store_true")
    parser.add_argument("--write-receipts", action="store_true")
    parser.add_argument("--path-env", default=None)
    parser.add_argument("--lead-harness", default="unknown")
    parser.add_argument("--lead-provider", default="unknown")
    parser.add_argument("--input-ref", default=".spellbook/agents.yaml")
    parser.add_argument("--objective", default="probe agent-provider roster")
    parser.add_argument("--backlog-ref", default="")
    args = parser.parse_args()

    roster = load_roster(args.roster)
    validate_roster(roster)
    if args.validate_only:
        print(f"{args.roster}: roster valid")
        return 0

    path_env = args.path_env if args.path_env is not None else os.environ.get("PATH", "")
    receipts = build_probe_receipts(
        roster,
        path_env=path_env,
        lead_harness=args.lead_harness,
        lead_provider=args.lead_provider,
        input_ref=args.input_ref,
        objective=args.objective,
        backlog_ref=args.backlog_ref,
    )
    for receipt in receipts:
        print(json.dumps(receipt, sort_keys=True))
        if args.write_receipts:
            append_receipt(args.receipt_output, receipt)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
