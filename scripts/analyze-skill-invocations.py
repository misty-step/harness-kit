#!/usr/bin/env python3
"""Analyze local skill invocation, work-ledger, and delegation JSONL evidence."""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from collections import Counter, defaultdict
from datetime import UTC, datetime, timedelta
from pathlib import Path
from typing import Any

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "scripts" / "lib"))

from agent_roster import ReceiptValidationError, validate_usage  # noqa: E402

DEFAULT_SKILL_LOG = Path.home() / ".claude" / "skill-invocations.jsonl"
DEFAULT_WORK_LEDGER = Path(".harness-kit/work/ledger.jsonl")
DEFAULT_DELEGATIONS = Path(".harness-kit/traces/delegations.jsonl")
DEFAULT_HARNESSES = ("claude", "codex", "pi", "antigravity-cli")
HARNESS_SUPPORT = {
    "claude": {
        "status": "supported",
        "source_protocols": ("post_tool_use",),
        "reason": "Claude PostToolUse Skill hook is installed by bootstrap.",
    },
    "codex": {
        "status": "unsupported",
        "source_protocols": (),
        "reason": "No verified Codex skill/tool hook API is configured in this repo.",
    },
    "pi": {
        "status": "unsupported",
        "source_protocols": (),
        "reason": "Pi is a roster dispatch lane; no skill event hook is configured.",
    },
    "antigravity-cli": {
        "status": "unsupported",
        "source_protocols": (),
        "reason": "Antigravity hook shape exists as a concept, but no stable JSON stdin contract is wired here.",
    },
}


def parse_ts(value: object) -> datetime | None:
    if not isinstance(value, str) or not value:
        return None
    try:
        return datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError:
        return None


def parse_since(value: str) -> datetime | None:
    if not value:
        return None
    unit = value[-1]
    amount_text = value[:-1]
    if unit not in {"d", "h"} or not amount_text.isdigit():
        raise SystemExit("--since must look like 7d, 30d, or 12h")
    amount = int(amount_text)
    delta = timedelta(days=amount) if unit == "d" else timedelta(hours=amount)
    return datetime.now(UTC) - delta


def parse_harnesses(value: str) -> list[str]:
    harnesses = [item.strip() for item in value.split(",") if item.strip()]
    return harnesses or list(DEFAULT_HARNESSES)


def read_jsonl(path: Path, label: str) -> tuple[list[dict[str, Any]], dict[str, Any], list[str]]:
    coverage = {"path": str(path), "present": path.exists(), "rows": 0}
    warnings: list[str] = []
    if not path.exists():
        warnings.append(f"{label} store missing: {path}")
        return [], coverage, warnings

    rows: list[dict[str, Any]] = []
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as error:
            raise SystemExit(f"{path}:{lineno}: invalid JSON: {error}") from error
        if not isinstance(row, dict):
            raise SystemExit(f"{path}:{lineno}: row must be a JSON object")
        rows.append(row)
    coverage["rows"] = len(rows)
    return rows, coverage, warnings


def repo_id(row: dict[str, Any]) -> str:
    project = str(row.get("project") or "").strip()
    if project:
        return project
    cwd = str(row.get("cwd") or "").strip()
    return Path(cwd).name if cwd else "unknown"


def source_protocol(row: dict[str, Any]) -> str:
    protocol = str(row.get("source_protocol") or "").strip()
    return protocol or "legacy_unknown"


def passes_filters(row: dict[str, Any], args: argparse.Namespace, since: datetime | None) -> bool:
    if since is not None:
        ts = parse_ts(row.get("ts") or row.get("created_at"))
        if ts is None or ts < since:
            return False
    if args.project and str(row.get("project", "")) != args.project:
        return False
    if args.repo and args.repo not in {repo_id(row), Path(str(row.get("cwd") or "")).name}:
        return False
    if args.skill and str(row.get("skill") or row.get("owning_skill") or "") != args.skill:
        return False
    return True


def usage_summary(rows: list[dict[str, Any]]) -> dict[str, object]:
    known = 0
    unknown = 0
    total_tokens = 0
    cost_usd = 0.0
    cost_sources: Counter[str] = Counter()
    for row in rows:
        usage = row.get("usage")
        if not isinstance(usage, dict):
            unknown += 1
            continue
        try:
            validate_usage(usage)
        except ReceiptValidationError as error:
            raise SystemExit(f"invalid usage payload: {error}") from error
        known += 1
        if isinstance(usage.get("total_tokens"), int):
            total_tokens += int(usage["total_tokens"])
        if isinstance(usage.get("cost_usd"), int | float):
            cost_usd += float(usage["cost_usd"])
        if isinstance(usage.get("cost_source"), str):
            cost_sources[usage["cost_source"]] += 1
    return {
        "known_count": known,
        "unknown_count": unknown,
        "total_tokens": total_tokens if known else None,
        "cost_usd": round(cost_usd, 6) if known else None,
        "cost_sources": dict(sorted(cost_sources.items())),
    }


def harness_coverage(
    rows: list[dict[str, Any]], expected_harnesses: list[str]
) -> tuple[list[dict[str, Any]], list[str]]:
    by_harness: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in rows:
        harness = str(row.get("harness") or "unknown").strip() or "unknown"
        by_harness[harness].append(row)

    names = sorted(set(expected_harnesses) | set(by_harness))
    coverage: list[dict[str, Any]] = []
    warnings: list[str] = []
    for harness in names:
        support = HARNESS_SUPPORT.get(
            harness,
            {
                "status": "unknown",
                "source_protocols": (),
                "reason": "No support entry is defined for this harness.",
            },
        )
        harness_rows = by_harness.get(harness, [])
        protocols = sorted({source_protocol(row) for row in harness_rows})
        status = str(support["status"])
        if status != "supported" and not harness_rows:
            warnings.append(f"{harness} skill invocation adapter unsupported/unavailable: {support['reason']}")
        elif status != "supported" and harness_rows:
            warnings.append(
                f"{harness} has {len(harness_rows)} skill invocation row(s) but no verified adapter: "
                f"protocols={','.join(protocols)}"
            )
        elif any(protocol == "legacy_unknown" for protocol in protocols):
            warnings.append(f"{harness} has legacy skill invocation row(s) without source_protocol")
        coverage.append(
            {
                "harness": harness,
                "status": status,
                "rows": len(harness_rows),
                "source_protocols": protocols,
                "supported_protocols": list(support["source_protocols"]),
                "reason": support["reason"],
            }
        )
    return coverage, warnings


def classify(count: int) -> str:
    if count > 10:
        return "hot"
    if count >= 3:
        return "warm"
    if count >= 1:
        return "cold"
    return "dead"


def analyze(args: argparse.Namespace) -> dict[str, Any]:
    since = parse_since(args.since)
    expected_harnesses = parse_harnesses(args.harnesses)
    skill_rows, skill_coverage, warnings = read_jsonl(args.skill_log, "skill invocation")
    work_rows, work_coverage, work_warnings = read_jsonl(args.work_ledger, "work ledger")
    delegation_rows, delegation_coverage, delegation_warnings = read_jsonl(
        args.delegations, "delegation"
    )
    warnings.extend(work_warnings)
    warnings.extend(delegation_warnings)

    skill_rows = [row for row in skill_rows if passes_filters(row, args, since)]
    work_rows = [row for row in work_rows if passes_filters(row, args, since)]
    delegation_rows = [row for row in delegation_rows if passes_filters(row, args, since)]

    by_skill: dict[str, list[dict[str, Any]]] = defaultdict(list)
    sessions: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for row in skill_rows:
        skill = str(row.get("skill") or "unknown")
        by_skill[skill].append(row)
        session_id = str(row.get("session_id") or "unknown")
        sessions[session_id].append(row)

    skills: list[dict[str, Any]] = []
    for skill, rows in sorted(by_skill.items(), key=lambda item: (-len(item[1]), item[0])):
        timestamps = [parse_ts(row.get("ts")) for row in rows]
        timestamps = [ts for ts in timestamps if ts is not None]
        skills.append(
            {
                "skill": skill,
                "count": len(rows),
                "health": classify(len(rows)),
                "last_used": max(timestamps).isoformat() if timestamps else "unknown",
                "projects": sorted({repo_id(row) for row in rows}),
                "usage": usage_summary(rows),
            }
        )

    transitions: Counter[tuple[str, str]] = Counter()
    for rows in sessions.values():
        ordered = sorted(rows, key=lambda row: str(row.get("ts") or ""))
        names = [str(row.get("skill") or "unknown") for row in ordered]
        for before, after in zip(names, names[1:]):
            transitions[(before, after)] += 1

    transition_rows = [
        {"from": before, "to": after, "count": count}
        for (before, after), count in sorted(
            transitions.items(), key=lambda item: (-item[1], item[0][0], item[0][1])
        )
    ]

    sequences: dict[str, list[str]] = defaultdict(list)
    for row in skill_rows:
        ref = str(row.get("backlog_ref") or row.get("work_id") or "")
        if ref:
            sequences[ref].append(str(row.get("skill") or "unknown"))
    for row in work_rows:
        ref = str(row.get("backlog_ref") or row.get("work_id") or "")
        skill = str(row.get("owning_skill") or "")
        if ref and skill:
            sequences[ref].append(skill)

    unmatched_skill_rows = sum(1 for row in skill_rows if not row.get("backlog_ref") and not row.get("work_id"))
    if unmatched_skill_rows:
        warnings.append(f"{unmatched_skill_rows} skill invocation row(s) lack backlog_ref/work_id")
    if delegation_rows and not work_rows:
        warnings.append("delegation rows are present but work ledger rows are absent")
    harness_rows, harness_warnings = harness_coverage(skill_rows, expected_harnesses)
    warnings.extend(harness_warnings)

    delegation_usage = usage_summary(delegation_rows)
    return {
        "skills": skills,
        "transitions": transition_rows,
        "work_sequences": [
            {"ref": ref, "skills": values}
            for ref, values in sorted(sequences.items())
        ],
        "delegation_usage": delegation_usage,
        "coverage": {
            "skill_log": skill_coverage,
            "work_ledger": work_coverage,
            "delegations": delegation_coverage,
            "harnesses": harness_rows,
        },
        "warnings": warnings,
    }


def unknown(value: object) -> str:
    return "unknown" if value is None else str(value)


def render_markdown(report: dict[str, Any]) -> str:
    lines = ["# Skill Invocation Analytics", "", "## Skill Frequency", ""]
    lines.append("| Skill | Count | Health | Last Used | Projects | Tokens | Cost |")
    lines.append("|---|---:|---|---|---|---:|---:|")
    for row in report["skills"]:
        usage = row["usage"]
        lines.append(
            "| {skill} | {count} | {health} | {last_used} | {projects} | {tokens} | {cost} |".format(
                skill=row["skill"],
                count=row["count"],
                health=row["health"],
                last_used=row["last_used"],
                projects=", ".join(row["projects"]),
                tokens=unknown(usage["total_tokens"]),
                cost=unknown(usage["cost_usd"]),
            )
        )
    if not report["skills"]:
        lines.append("| none | 0 | dead | unknown | unknown | unknown | unknown |")

    lines.extend(["", "## Skill Transitions", "", "| From | To | Count |", "|---|---|---:|"])
    for row in report["transitions"]:
        lines.append(f"| {row['from']} | {row['to']} | {row['count']} |")
    if not report["transitions"]:
        lines.append("| none | none | 0 |")

    lines.extend(["", "## Work Sequences", "", "| Ref | Skills |", "|---|---|"])
    for row in report["work_sequences"]:
        lines.append(f"| {row['ref']} | {' -> '.join(row['skills'])} |")
    if not report["work_sequences"]:
        lines.append("| none | none |")

    lines.extend(["", "## Source Coverage", "", "| Store | Present | Rows | Path |", "|---|---|---:|---|"])
    for name, coverage in report["coverage"].items():
        if name == "harnesses":
            continue
        lines.append(f"| {name} | {coverage['present']} | {coverage['rows']} | {coverage['path']} |")

    lines.extend(
        [
            "",
            "## Harness Coverage",
            "",
            "| Harness | Status | Rows | Source Protocols | Supported Protocols |",
            "|---|---|---:|---|---|",
        ]
    )
    for row in report["coverage"]["harnesses"]:
        protocols = ", ".join(row["source_protocols"]) or "none"
        supported = ", ".join(row["supported_protocols"]) or "none"
        lines.append(f"| {row['harness']} | {row['status']} | {row['rows']} | {protocols} | {supported} |")

    usage = report["delegation_usage"]
    lines.extend(
        [
            "",
            "## Delegation Usage",
            "",
            f"- known: {usage['known_count']}",
            f"- unknown: {usage['unknown_count']}",
            f"- total_tokens: {unknown(usage['total_tokens'])}",
            f"- cost_usd: {unknown(usage['cost_usd'])}",
            "",
            "## Warnings",
        ]
    )
    lines.extend(f"- {warning}" for warning in report["warnings"])
    if not report["warnings"]:
        lines.append("- none")
    return "\n".join(lines)


def render_text(report: dict[str, Any]) -> str:
    lines = ["Skill invocation analytics"]
    for row in report["skills"]:
        usage = row["usage"]
        lines.append(
            f"- {row['skill']}: count={row['count']} health={row['health']} "
            f"projects={','.join(row['projects'])} total_tokens={unknown(usage['total_tokens'])} "
            f"cost_usd={unknown(usage['cost_usd'])}"
        )
    lines.append("transitions:")
    lines.extend(f"- {row['from']} -> {row['to']}: {row['count']}" for row in report["transitions"])
    lines.append("coverage:")
    for name, coverage in report["coverage"].items():
        if name == "harnesses":
            continue
        lines.append(f"- {name}: present={coverage['present']} rows={coverage['rows']} path={coverage['path']}")
    lines.append("harness coverage:")
    for row in report["coverage"]["harnesses"]:
        protocols = ",".join(row["source_protocols"]) or "none"
        lines.append(
            f"- {row['harness']}: status={row['status']} rows={row['rows']} "
            f"source_protocols={protocols}"
        )
    lines.append("warnings:")
    lines.extend(f"- {warning}" for warning in report["warnings"] or ["none"])
    return "\n".join(lines)


def self_test() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        skill_log = root / "skill-invocations.jsonl"
        work_ledger = root / "work-ledger.jsonl"
        delegations = root / "delegations.jsonl"
        skill_log.write_text(
            "\n".join(
                [
                    json.dumps(
                        {
                            "schema_version": 2,
                            "event_type": "skill_invocation",
                            "ts": "2026-06-04T00:00:00Z",
                            "harness": "claude",
                            "source_protocol": "post_tool_use",
                            "skill": "shape",
                            "args": "088",
                            "session_id": "s1",
                            "cwd": "/tmp/harness-kit",
                            "project": "harness-kit",
                            "backlog_ref": "088",
                            "work_id": "work-088",
                            "usage": {
                                "input_tokens": 10,
                                "output_tokens": 5,
                                "total_tokens": 15,
                                "cost_usd": 0.001,
                                "cost_source": "provider_reported",
                            },
                        }
                    ),
                    json.dumps(
                        {
                            "schema_version": 2,
                            "event_type": "skill_invocation",
                            "ts": "2026-06-04T00:01:00Z",
                            "harness": "claude",
                            "source_protocol": "post_tool_use",
                            "skill": "implement",
                            "args": "088",
                            "session_id": "s1",
                            "cwd": "/tmp/harness-kit",
                            "project": "harness-kit",
                            "backlog_ref": "088",
                        }
                    ),
                    json.dumps(
                        {
                            "schema_version": 2,
                            "event_type": "skill_invocation",
                            "ts": "2026-06-04T00:02:00Z",
                            "harness": "codex",
                            "source_protocol": "external_import",
                            "skill": "shape",
                            "args": "090",
                            "session_id": "s2",
                            "cwd": "/tmp/harness-kit",
                            "project": "harness-kit",
                        }
                    ),
                ]
            )
            + "\n",
            encoding="utf-8",
        )
        work_ledger.write_text(
            json.dumps(
                {
                    "created_at": "2026-06-04T00:00:00Z",
                    "owning_skill": "deliver",
                    "backlog_ref": "088",
                    "work_id": "work-088",
                    "usage": {
                        "input_tokens": None,
                        "output_tokens": None,
                        "total_tokens": None,
                        "cost_usd": None,
                        "cost_source": "unknown",
                    },
                }
            )
            + "\n",
            encoding="utf-8",
        )
        delegations.write_text(
            json.dumps(
                {
                    "created_at": "2026-06-04T00:00:00Z",
                    "provider_target": "codex",
                    "backlog_ref": "088",
                }
            )
            + "\n",
            encoding="utf-8",
        )
        args = argparse.Namespace(
            skill_log=skill_log,
            work_ledger=work_ledger,
            delegations=delegations,
            since="",
            repo="",
            project="",
            skill="",
            harnesses="claude,codex,pi,antigravity-cli",
        )
        report = analyze(args)
        assert report["skills"][0]["skill"] == "shape"
        assert {"from": "shape", "to": "implement", "count": 1} in report["transitions"]
        assert report["delegation_usage"]["total_tokens"] is None
        assert any(row["harness"] == "codex" and row["status"] == "unsupported" for row in report["coverage"]["harnesses"])
        assert any("pi skill invocation adapter unsupported" in warning for warning in report["warnings"])
        assert "unknown" in render_markdown(report)
        missing_args = argparse.Namespace(
            skill_log=root / "missing-skill.jsonl",
            work_ledger=root / "missing-work.jsonl",
            delegations=root / "missing-delegations.jsonl",
            since="",
            repo="",
            project="",
            skill="",
            harnesses="claude,codex,pi,antigravity-cli",
        )
        missing_report = analyze(missing_args)
        assert missing_report["coverage"]["skill_log"]["present"] is False
        assert missing_report["warnings"]
    print("analyze-skill-invocations self-test ok")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--skill-log", type=Path, default=DEFAULT_SKILL_LOG)
    parser.add_argument("--work-ledger", type=Path, default=DEFAULT_WORK_LEDGER)
    parser.add_argument("--delegations", type=Path, default=DEFAULT_DELEGATIONS)
    parser.add_argument("--since", default="")
    parser.add_argument("--repo", default="")
    parser.add_argument("--project", default="")
    parser.add_argument("--skill", default="")
    parser.add_argument(
        "--harnesses",
        default=",".join(DEFAULT_HARNESSES),
        help="Comma-separated harnesses to show in coverage; defaults to Harness Kit primary harnesses.",
    )
    parser.add_argument("--format", choices=("json", "text", "markdown"), default="markdown")
    parser.add_argument("--self-test", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.self_test:
        return self_test()
    report = analyze(args)
    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    elif args.format == "text":
        print(render_text(report))
    else:
        print(render_markdown(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
