#!/usr/bin/env python3
"""Validate opt-in /reflect checkpoint artifacts."""

from __future__ import annotations

import argparse
import json
import re
import sys
import tempfile
from datetime import UTC, datetime
from pathlib import Path
from typing import Any


REQUIRED_FIELDS = {
    "topic",
    "source_refs",
    "question",
    "operator_restatement",
    "lead_verdict",
    "gaps",
    "next_action",
    "timestamp",
}
VALID_VERDICTS = {"pass", "partial", "fail"}
REQUIRED_MARKER_RE = re.compile(r"^Comprehension-required:\s*(.+?)\s*$", re.I | re.M)
SECRET_OR_RAW_RE = re.compile(
    r"(?i)(raw transcript|system prompt|developer prompt|raw tool output|"
    r"private_customer_data|api[_-]?key\s*[:=]|authorization\s*[:=]|"
    r"bearer\s+[a-z0-9._-]+|-----BEGIN [A-Z ]*PRIVATE KEY-----)"
)


class CheckpointError(ValueError):
    pass


def read_json(path: Path) -> dict[str, Any]:
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise CheckpointError(f"cannot read checkpoint {path}: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise CheckpointError(f"checkpoint {path} is not valid JSON: {exc}") from exc
    if not isinstance(data, dict):
        raise CheckpointError("checkpoint must be a JSON object")
    return data


def required_topics(packet: Path | None) -> set[str]:
    if packet is None:
        return set()
    try:
        text = packet.read_text(encoding="utf-8")
    except OSError as exc:
        raise CheckpointError(f"cannot read packet {packet}: {exc}") from exc
    return {match.group(1).strip() for match in REQUIRED_MARKER_RE.finditer(text)}


def validate_checkpoint(data: dict[str, Any]) -> None:
    missing = REQUIRED_FIELDS - set(data)
    if missing:
        raise CheckpointError(f"checkpoint missing field(s): {', '.join(sorted(missing))}")

    extra = set(data) - REQUIRED_FIELDS
    if extra:
        raise CheckpointError(f"checkpoint has unknown field(s): {', '.join(sorted(extra))}")

    topic = require_text(data, "topic")
    require_text(data, "question")
    restatement = require_text(data, "operator_restatement")
    next_action = require_text(data, "next_action")
    validate_refs(data["source_refs"])
    gaps = validate_gaps(data["gaps"])
    validate_timestamp(data["timestamp"])

    verdict = data["lead_verdict"]
    if verdict not in VALID_VERDICTS:
        raise CheckpointError("lead_verdict must be pass, partial, or fail")
    if verdict == "pass" and gaps:
        raise CheckpointError("lead_verdict pass requires empty gaps")
    if verdict in {"partial", "fail"} and not gaps:
        raise CheckpointError("lead_verdict partial/fail requires at least one gap")

    for field_name, value in (
        ("topic", topic),
        ("question", data["question"]),
        ("operator_restatement", restatement),
        ("next_action", next_action),
        ("gaps", "\n".join(gaps)),
    ):
        if SECRET_OR_RAW_RE.search(str(value)):
            raise CheckpointError(f"checkpoint {field_name} contains raw/private content")
    if len(restatement) > 1000:
        raise CheckpointError("operator_restatement must be short; store refs, not transcripts")


def require_text(data: dict[str, Any], field: str) -> str:
    value = data[field]
    if not isinstance(value, str) or not value.strip():
        raise CheckpointError(f"{field} must be a non-empty string")
    return value.strip()


def validate_refs(value: Any) -> None:
    if not isinstance(value, list) or not value:
        raise CheckpointError("source_refs must be a non-empty list")
    for ref in value:
        if not isinstance(ref, str) or not ref.strip():
            raise CheckpointError("source_refs must contain non-empty strings")
        if SECRET_OR_RAW_RE.search(ref):
            raise CheckpointError("source_refs contain raw/private content")


def validate_gaps(value: Any) -> list[str]:
    if not isinstance(value, list):
        raise CheckpointError("gaps must be a list")
    gaps: list[str] = []
    for gap in value:
        if not isinstance(gap, str) or not gap.strip():
            raise CheckpointError("gaps must contain non-empty strings")
        gaps.append(gap.strip())
    return gaps


def validate_timestamp(value: Any) -> None:
    if not isinstance(value, str) or not value.strip():
        raise CheckpointError("timestamp must be an ISO-8601 string")
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise CheckpointError("timestamp must be ISO-8601") from exc
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=UTC)
    if parsed.astimezone(UTC) > datetime.now(UTC):
        raise CheckpointError("timestamp must not be in the future")


def gate(checkpoint: Path | None, topic: str, packet: Path | None) -> None:
    topics = required_topics(packet)
    if packet is None or topic not in topics:
        return
    if checkpoint is None:
        raise CheckpointError(f"checkpoint required for topic {topic!r}")
    data = read_json(checkpoint)
    validate_checkpoint(data)
    if data["topic"] != topic:
        raise CheckpointError("checkpoint topic does not match gate topic")
    if data["lead_verdict"] != "pass":
        raise CheckpointError("checkpoint gate requires lead_verdict pass")
    if data["gaps"]:
        raise CheckpointError("checkpoint gate requires empty gaps")


def self_test() -> None:
    now = "2026-01-01T00:00:00Z"
    passing = {
        "topic": "load-bearing-decision",
        "source_refs": ["backlog.d/096-reflect-teach-back-checkpoints.md"],
        "question": "What decision did we make, what can fail, and what happens next?",
        "operator_restatement": "We keep this opt-in, record refs only, and continue after a pass.",
        "lead_verdict": "pass",
        "gaps": [],
        "next_action": "Continue the session.",
        "timestamp": now,
    }
    partial = {**passing, "lead_verdict": "partial", "gaps": ["Next action was unclear."]}
    failed = {**passing, "lead_verdict": "fail", "gaps": ["Failure mode was not named."]}
    missing_restatement = {**passing, "operator_restatement": ""}
    invalid_verdict = {**passing, "lead_verdict": "seems understood"}
    raw = {**passing, "operator_restatement": "raw transcript: user said ok"}

    validate_checkpoint(passing)
    expect_error(lambda: validate_checkpoint(missing_restatement), "missing restatement")
    expect_error(lambda: validate_checkpoint(invalid_verdict), "invalid verdict")
    expect_error(lambda: validate_checkpoint(raw), "raw content")

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        pass_path = write_json(root / "pass.json", passing)
        partial_path = write_json(root / "partial.json", partial)
        fail_path = write_json(root / "fail.json", failed)
        required_packet = root / "required.md"
        required_packet.write_text("Comprehension-required: load-bearing-decision\n", encoding="utf-8")
        unrelated_packet = root / "unrelated.md"
        unrelated_packet.write_text("Comprehension-required: other-topic\n", encoding="utf-8")

        gate(pass_path, "load-bearing-decision", required_packet)
        gate(None, "load-bearing-decision", None)
        gate(partial_path, "load-bearing-decision", unrelated_packet)
        expect_error(lambda: gate(partial_path, "load-bearing-decision", required_packet), "partial gate")
        expect_error(lambda: gate(fail_path, "load-bearing-decision", required_packet), "fail gate")
        expect_error(lambda: gate(None, "load-bearing-decision", required_packet), "missing checkpoint")

    print("reflect checkpoint self-test ok")


def write_json(path: Path, data: dict[str, Any]) -> Path:
    path.write_text(json.dumps(data), encoding="utf-8")
    return path


def expect_error(fn, label: str) -> None:
    try:
        fn()
    except CheckpointError:
        return
    raise AssertionError(f"expected CheckpointError for {label}")


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="command")
    validate = subparsers.add_parser("validate", help="validate a checkpoint artifact")
    validate.add_argument("checkpoint", nargs="?", type=Path)
    validate.add_argument("--gate", metavar="TOPIC")
    validate.add_argument("--packet", type=Path)
    parser.add_argument("--self-test", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.self_test:
        self_test()
        return 0
    if args.command != "validate":
        parser.error("choose validate or --self-test")
    try:
        if args.gate:
            gate(args.checkpoint, args.gate, args.packet)
        else:
            if args.checkpoint is None:
                parser.error("validate requires a checkpoint path unless --gate has a non-required packet")
            validate_checkpoint(read_json(args.checkpoint))
    except CheckpointError as exc:
        print(str(exc), file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
