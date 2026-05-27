#!/usr/bin/env python3
"""Run one configured roster provider with timeout and receipt capture."""

from __future__ import annotations

import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent / "lib"))

from agent_roster import (  # noqa: E402
    default_receipt_path,
    dispatch_provider_lane,
    load_roster,
    parse_common_args,
)


def main() -> int:
    parser = parse_common_args("Dispatch one provider lane from the configured roster.")
    parser.add_argument("--provider-target", required=True)
    parser.add_argument("--objective", required=True)
    parser.add_argument("--input-ref", required=True)
    parser.add_argument("--prompt-file", type=Path, required=True)
    parser.add_argument("--backlog-ref", default="")
    parser.add_argument("--lead-harness", default="unknown")
    parser.add_argument("--lead-provider", default="unknown")
    parser.add_argument("--timeout-s", type=float, default=600)
    parser.add_argument("--grace-s", type=float, default=2)
    parser.add_argument("--max-prompt-bytes", type=int, default=128 * 1024)
    parser.add_argument(
        "--transcript-dir",
        type=Path,
        default=default_receipt_path().parent / "provider-lanes",
    )
    parser.add_argument("--path-env", default=None)
    args = parser.parse_args()

    if args.prompt_file.stat().st_size > args.max_prompt_bytes:
        parser.error(f"--prompt-file exceeds --max-prompt-bytes ({args.max_prompt_bytes})")

    prompt = args.prompt_file.read_text()
    receipt = dispatch_provider_lane(
        load_roster(args.roster),
        provider_target=args.provider_target,
        prompt=prompt,
        objective=args.objective,
        input_ref=args.input_ref,
        backlog_ref=args.backlog_ref,
        transcript_dir=args.transcript_dir,
        receipt_output=args.receipt_output,
        timeout_s=args.timeout_s,
        grace_s=args.grace_s,
        lead_harness=args.lead_harness,
        lead_provider=args.lead_provider,
        path_env=args.path_env,
    )
    print(json.dumps(receipt, sort_keys=True))
    return 0 if receipt["attempt_status"] == "succeeded" else 1


if __name__ == "__main__":
    raise SystemExit(main())
