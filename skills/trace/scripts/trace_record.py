#!/usr/bin/env python3
"""Append sanitized agent-session trace records."""

from __future__ import annotations

import argparse
import contextlib
import json
import re
import sys
import tempfile
import uuid
from datetime import UTC, datetime
from pathlib import Path
from typing import Iterable

DEFAULT_STORE = Path(".harness-kit/traces/work-records.jsonl")
RECORD_TYPE = "agent-session-trace"

SECRET_RE = re.compile(
    r"(?i)(api[_-]?key|token|secret|password|credential|"
    r"xai[_-]?api[_-]?key|exa[_-]?api[_-]?key|anthropic[_-]?api[_-]?key|"
    r"bearer\s+[a-z0-9._~+/-]+|-----BEGIN [A-Z ]*PRIVATE KEY-----|"
    r"private[_ -]?customer[_ -]?data)"
)


def now_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def trace_id() -> str:
    return f"trace-{uuid.uuid4()}"


def reject_secret_like(values: Iterable[object]) -> None:
    for value in values:
        if value is None:
            continue
        if isinstance(value, dict):
            reject_secret_like(value.keys())
            reject_secret_like(value.values())
            continue
        if isinstance(value, (list, tuple, set)):
            reject_secret_like(value)
            continue
        text = str(value)
        if SECRET_RE.search(text):
            raise ValueError(f"secret-like value refused: {text[:80]}")


def parse_metadata(entries: list[str]) -> dict[str, str]:
    metadata: dict[str, str] = {}
    for entry in entries:
        if "=" not in entry:
            raise ValueError(f"metadata must be key=value: {entry}")
        key, value = entry.split("=", 1)
        key = key.strip()
        if not key:
            raise ValueError(f"metadata key must be non-empty: {entry}")
        metadata[key] = value
    return metadata


def build_record(args: argparse.Namespace) -> dict[str, object]:
    metadata = parse_metadata(args.metadata)
    if not args.transcript_ref and not args.waiver_reason:
        raise ValueError("provide at least one --transcript-ref or --waiver-reason")

    record: dict[str, object] = {
        "schema_version": 1,
        "record_type": RECORD_TYPE,
        "trace_id": trace_id(),
        "created_at": now_iso(),
        "backlog_ref": args.backlog,
        "spec_ref": args.spec_ref,
        "branch": args.branch,
        "commits": args.commit,
        "reviewer_verdict_refs": args.reviewer_verdict_ref,
        "qa_refs": args.qa_ref,
        "demo_refs": args.demo_ref,
        "transcript_refs": args.transcript_ref,
        "shipped_ref": args.shipped_ref,
        "waiver_reason": args.waiver_reason,
        "metadata": metadata,
    }
    reject_secret_like(record.values())
    return record


def append_record(store: Path, record: dict[str, object]) -> None:
    store.parent.mkdir(parents=True, exist_ok=True)
    with store.open("a", encoding="utf-8") as handle:
        with file_lock(handle):
            handle.write(json.dumps(record, sort_keys=True, separators=(",", ":")))
            handle.write("\n")


@contextlib.contextmanager
def file_lock(handle):
    try:
        import fcntl  # type: ignore
    except ImportError:
        yield
        return

    fcntl.flock(handle.fileno(), fcntl.LOCK_EX)
    try:
        yield
    finally:
        fcntl.flock(handle.fileno(), fcntl.LOCK_UN)


def append_command(args: argparse.Namespace) -> int:
    try:
        record = build_record(args)
        append_record(args.store, record)
    except ValueError as error:
        print(f"trace_record: {error}", file=sys.stderr)
        return 2
    print(json.dumps({"store": str(args.store), "trace_id": record["trace_id"]}, sort_keys=True))
    return 0


def self_test() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        store = Path(tmp) / "records.jsonl"
        args = argparse.Namespace(
            store=store,
            backlog="056",
            spec_ref="backlog.d/056-agent-session-trace-lifecycle.md",
            branch="deliver/056-agent-session-trace-lifecycle",
            commit=["abc1234"],
            reviewer_verdict_ref=[".harness-kit/traces/delegations.jsonl#abc"],
            qa_ref=[".evidence/qa/056.md"],
            demo_ref=[".evidence/demo/056.gif"],
            transcript_ref=[".harness-kit/traces/transcripts/056.md"],
            shipped_ref="master@deadbeef",
            waiver_reason="",
            metadata=["source=self-test"],
        )
        record = build_record(args)
        append_record(store, record)
        rows = [json.loads(line) for line in store.read_text(encoding="utf-8").splitlines()]
        assert len(rows) == 1
        assert rows[0]["backlog_ref"] == "056"
        assert rows[0]["metadata"] == {"source": "self-test"}

        args.transcript_ref = []
        args.waiver_reason = ""
        try:
            build_record(args)
        except ValueError as error:
            assert "transcript-ref" in str(error)
        else:
            raise AssertionError("missing transcript ref or waiver should fail")

        args.waiver_reason = "No safe transcript export available."
        args.metadata = ["API_TOKEN=leak"]
        try:
            build_record(args)
        except ValueError as error:
            assert "secret-like" in str(error)
        else:
            raise AssertionError("secret-like metadata should fail")

    print("trace_record self-test ok")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    subparsers = parser.add_subparsers(dest="command")

    append = subparsers.add_parser("append", help="append a trace work record")
    append.add_argument("--store", type=Path, default=DEFAULT_STORE)
    append.add_argument("--backlog", required=True, help="backlog or work item id")
    append.add_argument("--spec-ref", default="", help="spec/context packet path or id")
    append.add_argument("--branch", required=True)
    append.add_argument("--commit", action="append", default=[])
    append.add_argument("--reviewer-verdict-ref", action="append", default=[])
    append.add_argument("--qa-ref", action="append", default=[])
    append.add_argument("--demo-ref", action="append", default=[])
    append.add_argument("--transcript-ref", action="append", default=[])
    append.add_argument("--shipped-ref", default="")
    append.add_argument("--waiver-reason", default="")
    append.add_argument("--metadata", action="append", default=[], help="key=value")
    append.set_defaults(func=append_command)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    if args.self_test:
        return self_test()
    if not hasattr(args, "func"):
        parser.error("choose a command or --self-test")
    return args.func(args)


if __name__ == "__main__":
    raise SystemExit(main())
