#!/usr/bin/env python3
"""Append and summarize local work-ledger events."""

from __future__ import annotations

import argparse
import contextlib
import json
import sys
import tempfile
import uuid
from datetime import UTC, datetime
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "lib"))

from agent_roster import ReceiptValidationError, validate_usage  # noqa: E402

DEFAULT_STORE = Path(".harness-kit/work/ledger.jsonl")
RECORD_TYPE = "work-ledger-event"
ACTIVE_STATUSES = {"active", "blocked"}
VALID_STATUSES = {"active", "blocked", "completed", "failed", "superseded"}
VALID_EVENT_TYPES = {
    "phase_started",
    "phase_completed",
    "blocker_added",
    "next_action_changed",
}


def now_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def event_id() -> str:
    return f"work-{uuid.uuid4()}"


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


def build_event(args: argparse.Namespace) -> dict[str, object]:
    event: dict[str, object] = {
        "schema_version": 1,
        "record_type": RECORD_TYPE,
        "event_id": event_id(),
        "created_at": now_iso(),
        "event_type": args.event_type,
        "work_id": args.work_id,
        "parent_work_id": args.parent_work_id,
        "backlog_ref": args.backlog,
        "branch": args.branch,
        "owning_skill": args.owning_skill,
        "phase": args.phase,
        "evidence_refs": args.evidence_ref,
        "blockers": args.blocker,
        "spawned_agents": args.spawned_agent,
        "trace_refs": args.trace_ref,
        "next_action": args.next_action,
        "status": args.status,
    }
    if args.usage is not None:
        event["usage"] = args.usage
    return event


def append_event(store: Path, event: dict[str, object]) -> None:
    store.parent.mkdir(parents=True, exist_ok=True)
    with store.open("a", encoding="utf-8") as handle:
        with file_lock(handle):
            handle.write(json.dumps(event, sort_keys=True, separators=(",", ":")))
            handle.write("\n")


def read_events(store: Path) -> list[dict[str, object]]:
    if not store.exists():
        return []
    events: list[dict[str, object]] = []
    for lineno, line in enumerate(store.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        event = json.loads(line)
        if not isinstance(event, dict):
            raise ValueError(f"{store}:{lineno}: event must be a JSON object")
        events.append(event)
    return events


def latest_by_work(events: list[dict[str, object]]) -> dict[str, dict[str, object]]:
    latest: dict[str, dict[str, object]] = {}
    for event in events:
        latest[str(event["work_id"])] = event
    return latest


def format_list(values: object) -> str:
    if not values:
        return "none"
    if isinstance(values, list):
        return ", ".join(str(value) for value in values) if values else "none"
    return str(values)


def summary_text(events: list[dict[str, object]]) -> str:
    active = [
        event
        for event in latest_by_work(events).values()
        if str(event.get("status", "")) in ACTIVE_STATUSES
    ]
    if not active:
        return "No active work ledger entries."

    lines = ["Work ledger"]
    for event in sorted(active, key=lambda row: str(row.get("created_at", ""))):
        evidence_refs = event.get("evidence_refs")
        latest_evidence = "none"
        if isinstance(evidence_refs, list) and evidence_refs:
            latest_evidence = str(evidence_refs[-1])
        lines.extend(
            [
                f"- work_id: {event.get('work_id', '')}",
                f"  branch: {event.get('branch', '')}",
                f"  backlog: {event.get('backlog_ref', '')}",
                f"  event_type: {event.get('event_type', '')}",
                f"  owning_skill: {event.get('owning_skill', '')}",
                f"  phase: {event.get('phase', '')}",
                f"  status: {event.get('status', '')}",
                f"  latest_evidence: {latest_evidence}",
                f"  blockers: {format_list(event.get('blockers'))}",
                f"  spawned_agents: {format_list(event.get('spawned_agents'))}",
                f"  trace_refs: {format_list(event.get('trace_refs'))}",
                f"  next_action: {event.get('next_action', '')}",
            ]
        )
    return "\n".join(lines)


def append_command(args: argparse.Namespace) -> int:
    args.usage = _parse_usage_json(args.usage_json)
    event = build_event(args)
    append_event(args.store, event)
    print(json.dumps({"event_id": event["event_id"], "store": str(args.store)}, sort_keys=True))
    return 0


def summary_command(args: argparse.Namespace) -> int:
    try:
        events = read_events(args.store)
    except (json.JSONDecodeError, ValueError) as error:
        print(f"work-ledger: {error}", file=sys.stderr)
        return 2
    print(summary_text(events))
    return 0


def self_test() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        store = Path(tmp) / "ledger.jsonl"
        args = argparse.Namespace(
            store=store,
            event_type="phase_started",
            work_id="058",
            parent_work_id="",
            backlog="058",
            branch="deliver/058-work-ledger-mission-control",
            owning_skill="deliver",
            phase="review",
            evidence_ref=[".harness-kit/traces/delegations.jsonl#abc"],
            blocker=["waiting for critic"],
            spawned_agent=["grok-build:critic"],
            trace_ref=[".harness-kit/traces/work-records.jsonl#trace-abc"],
            next_action="address critic output",
            status="active",
            usage={
                "input_tokens": 100,
                "output_tokens": 25,
                "total_tokens": 125,
                "cost_usd": 0.01,
                "cost_source": "manual",
            },
        )
        append_event(store, build_event(args))
        text = summary_text(read_events(store))
        assert "backlog: 058" in text
        assert "phase: review" in text
        assert "trace_refs: .harness-kit/traces/work-records.jsonl#trace-abc" in text

        args.phase = "done"
        args.event_type = "phase_completed"
        args.status = "completed"
        args.blocker = []
        args.spawned_agent = []
        args.trace_ref = []
        args.evidence_ref = []
        args.next_action = "none"
        args.usage = None
        append_event(store, build_event(args))
        assert summary_text(read_events(store)) == "No active work ledger entries."
    print("work-ledger self-test ok")
    return 0


def _parse_usage_json(value: str | None) -> dict[str, object] | None:
    if not value:
        return None
    try:
        usage = json.loads(value)
    except json.JSONDecodeError as error:
        raise SystemExit(f"work-ledger: invalid --usage-json: {error}") from error
    try:
        validate_usage(usage)
    except ReceiptValidationError as error:
        raise SystemExit(f"work-ledger: invalid --usage-json: {error}") from error
    return usage


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--self-test", action="store_true")
    subparsers = parser.add_subparsers(dest="command")

    append = subparsers.add_parser("append", help="append a phase transition event")
    append.add_argument("--store", type=Path, default=DEFAULT_STORE)
    append.add_argument("--event-type", choices=sorted(VALID_EVENT_TYPES), required=True)
    append.add_argument("--work-id", required=True)
    append.add_argument("--parent-work-id", default="")
    append.add_argument("--backlog", required=True)
    append.add_argument("--branch", required=True)
    append.add_argument("--owning-skill", required=True)
    append.add_argument("--phase", required=True)
    append.add_argument("--evidence-ref", action="append", default=[])
    append.add_argument("--blocker", action="append", default=[])
    append.add_argument("--spawned-agent", action="append", default=[])
    append.add_argument("--trace-ref", action="append", default=[])
    append.add_argument("--next-action", required=True)
    append.add_argument("--status", choices=sorted(VALID_STATUSES), default="active")
    append.add_argument("--usage-json", default=None)
    append.set_defaults(func=append_command)

    summary = subparsers.add_parser("summary", help="print active work summary")
    summary.add_argument("--store", type=Path, default=DEFAULT_STORE)
    summary.set_defaults(func=summary_command)
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
