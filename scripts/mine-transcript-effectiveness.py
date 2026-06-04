#!/usr/bin/env python3
"""Mine redacted transcript refs into local effectiveness signals."""

from __future__ import annotations

import argparse
import importlib.util
import json
import re
import sys
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_SKILL_LOG = Path.home() / ".claude" / "skill-invocations.jsonl"
DEFAULT_WORK_LEDGER = Path(".harness-kit/work/ledger.jsonl")
DEFAULT_DELEGATIONS = Path(".harness-kit/traces/delegations.jsonl")
DEFAULT_REVIEW_SCORES = Path(".groom/review-scores.ndjson")

UNRESOLVED_SECRET_PATTERNS = [
    re.compile(r"-----BEGIN [A-Z ]*PRIVATE KEY-----"),
    re.compile(r"\b(?:sk|rk|ghp|glpat|xox[abprs]?)-[A-Za-z0-9_\-]{16,}\b"),
    re.compile(r"\bgithub_pat_[A-Za-z0-9_]{20,}\b"),
    re.compile(r"\bAKIA[0-9A-Z]{16}\b"),
    re.compile(r"(?i)\b(?:authorization|cookie|x-api-key|api[_-]?key|password)\s*[:=]\s*(?!\[REDACTED)[^\s`'\"]{8,}"),
    re.compile(r"(?i)\bprivate[_ -]?customer[_ -]?data\b"),
]

CATEGORY_PATTERNS = {
    "user_corrections": re.compile(r"(?i)\b(wrong|not what i|actually|instead|still|again|revert|stop)\b"),
    "skill_missed_opportunities": re.compile(r"(?i)\b(should have used|forgot to use|missed .*skill|use /[a-z-]+)\b"),
    "repeated_tool_failure": re.compile(r"(?i)\b(error|failed|traceback|exception|command not found|timed out)\b"),
    "cost_token_concern": re.compile(r"(?i)\b(cost|token|budget|too expensive|spend)\b"),
    "insufficient_evidence_claim": re.compile(r"(?i)\b(no evidence|unverified|did not run|without checking|claimed|validated)\b"),
    "privacy_secret_risk": re.compile(r"(?i)\b(secret|credential|private key|token leak|redact|privacy)\b"),
    "successful_skill_usage": re.compile(r"(?i)(<command-name>[^<]+</command-name>|\bSkill\b|/[a-z][a-z-]+)"),
}


def load_agent_transcript_module():
    path = ROOT / "skills" / "agent-transcript" / "scripts" / "agent_transcript.py"
    spec = importlib.util.spec_from_file_location("agent_transcript", path)
    if spec is None or spec.loader is None:
        raise RuntimeError(f"cannot load {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


AGENT_TRANSCRIPT = load_agent_transcript_module()


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


def extract_text(content: Any) -> str:
    if isinstance(content, str):
        return content
    if isinstance(content, list):
        parts = []
        for item in content:
            if isinstance(item, str):
                parts.append(item)
            elif isinstance(item, dict):
                if isinstance(item.get("text"), str):
                    parts.append(item["text"])
                elif item.get("type") == "tool_use":
                    parts.append(f"tool_use:{item.get('name', 'unknown')}")
        return "\n".join(part for part in parts if part)
    if isinstance(content, dict) and isinstance(content.get("text"), str):
        return content["text"]
    return ""


def assert_no_unresolved_secret(text: str, source: Path) -> None:
    for pattern in UNRESOLVED_SECRET_PATTERNS:
        if pattern.search(text):
            raise SystemExit(f"{source}: unresolved secret-like transcript content refused")


def safe_line(raw: str, source: Path) -> str:
    redacted = AGENT_TRANSCRIPT.redact(raw)
    AGENT_TRANSCRIPT.assert_safe(redacted)
    assert_no_unresolved_secret(redacted, source)
    return redacted


def parse_transcript(path: Path) -> dict[str, Any]:
    turns: list[dict[str, str]] = []
    malformed = 0
    redactions = Counter()
    session_ids: set[str] = set()
    backlog_refs: set[str] = set()
    work_ids: set[str] = set()
    branches: set[str] = set()
    projects: set[str] = set()

    for lineno, raw_line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
        if not raw_line.strip():
            continue
        try:
            row = json.loads(raw_line)
        except json.JSONDecodeError:
            malformed += 1
            text = safe_line(raw_line, path)
            if text:
                turns.append({"role": "unknown", "text": text, "lineno": str(lineno)})
            continue
        if not isinstance(row, dict):
            malformed += 1
            continue
        message = row.get("message") if isinstance(row.get("message"), dict) else row
        role = str(message.get("role") or row.get("type") or "unknown")
        raw_text = extract_text(message.get("content"))
        if not raw_text:
            continue
        text = safe_line(raw_text, path)
        if not text:
            continue
        if "[REDACTED" in text:
            redactions["redacted_segments"] += text.count("[REDACTED")
        turns.append({"role": role, "text": text, "lineno": str(lineno)})
        for key, target in (
            ("sessionId", session_ids),
            ("session_id", session_ids),
            ("backlog_ref", backlog_refs),
            ("work_id", work_ids),
            ("gitBranch", branches),
        ):
            value = row.get(key)
            if isinstance(value, str) and value.strip():
                target.add(value.strip())
        cwd = row.get("cwd")
        if isinstance(cwd, str) and cwd.strip():
            projects.add(Path(cwd).name)

    return {
        "path": str(path),
        "turns": turns,
        "malformed": malformed,
        "redactions": dict(redactions),
        "session_ids": sorted(session_ids),
        "backlog_refs": sorted(backlog_refs),
        "work_ids": sorted(work_ids),
        "branches": sorted(branches),
        "projects": sorted(projects),
    }


def transcript_paths(paths: list[Path], source_roots: list[Path]) -> list[Path]:
    resolved: list[Path] = []
    for path in paths:
        if not path.exists():
            raise SystemExit(f"transcript path missing: {path}")
        if path.is_dir():
            raise SystemExit(f"use --source-root for transcript directories: {path}")
        resolved.append(path)
    for root in source_roots:
        if not root.exists() or not root.is_dir():
            raise SystemExit(f"source root missing or not a directory: {root}")
        for path in sorted(root.rglob("*.jsonl")):
            if "subagents" in path.parts:
                continue
            resolved.append(path)
    unique = []
    seen = set()
    for path in resolved:
        key = str(path.resolve())
        if key not in seen:
            seen.add(key)
            unique.append(path)
    return unique


def categorize(turns: list[dict[str, str]], *, allow_excerpts: bool) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for category, pattern in CATEGORY_PATTERNS.items():
        refs = []
        excerpts = []
        for turn in turns:
            if pattern.search(turn["text"]):
                refs.append({"role": turn["role"], "lineno": turn["lineno"]})
                if allow_excerpts and len(excerpts) < 3:
                    excerpts.append(turn["text"][:180])
        result[category] = {"count": len(refs), "refs": refs[:10]}
        if allow_excerpts:
            result[category]["redacted_excerpts"] = excerpts
    return result


def row_matches(row: dict[str, Any], keys: dict[str, set[str]]) -> bool:
    for field in ("session_id", "backlog_ref", "work_id"):
        value = row.get(field)
        if isinstance(value, str) and value in keys[field]:
            return True
    project = row.get("project")
    if isinstance(project, str) and project in keys["projects"]:
        return True
    return False


def join_evidence(
    transcripts: list[dict[str, Any]],
    skill_rows: list[dict[str, Any]],
    work_rows: list[dict[str, Any]],
    delegation_rows: list[dict[str, Any]],
    review_rows: list[dict[str, Any]],
) -> dict[str, Any]:
    keys = {
        "session_id": set().union(*(set(t["session_ids"]) for t in transcripts)),
        "backlog_ref": set().union(*(set(t["backlog_refs"]) for t in transcripts)),
        "work_id": set().union(*(set(t["work_ids"]) for t in transcripts)),
        "projects": set().union(*(set(t["projects"]) for t in transcripts)),
        "branches": set().union(*(set(t["branches"]) for t in transcripts)),
    }
    skill_matches = [row for row in skill_rows if row_matches(row, keys)]
    work_matches = [row for row in work_rows if row_matches(row, keys)]
    delegation_matches = [row for row in delegation_rows if row_matches(row, keys)]
    review_matches = [
        row for row in review_rows
        if isinstance(row.get("branch"), str) and row["branch"] in keys["branches"]
    ]
    return {
        "skill_invocations": {
            "matched": len(skill_matches),
            "missing": len(skill_rows) - len(skill_matches),
            "refs": sorted({str(row.get("skill") or "unknown") for row in skill_matches}),
        },
        "work_ledger": {
            "matched": len(work_matches),
            "missing": len(work_rows) - len(work_matches),
            "refs": sorted({str(row.get("work_id") or row.get("backlog_ref") or "unknown") for row in work_matches}),
        },
        "delegations": {
            "matched": len(delegation_matches),
            "missing": len(delegation_rows) - len(delegation_matches),
            "refs": sorted({str(row.get("delegation_id") or "unknown") for row in delegation_matches}),
        },
        "review_scores": {
            "matched": len(review_matches),
            "missing": len(review_rows) - len(review_matches),
            "refs": sorted({str(row.get("branch") or "unknown") for row in review_matches}),
            "trend_status": "insufficient_data" if len(review_rows) < 5 else "available",
        },
    }


def proposed_actions(categories: dict[str, dict[str, Any]], joins: dict[str, Any]) -> list[str]:
    actions = []
    if categories["user_corrections"]["count"]:
        actions.append("Run /reflect prompt-debt on repeated correction categories before editing skills.")
    if categories["skill_missed_opportunities"]["count"]:
        actions.append("Review skill trigger descriptions for missed or late invocation patterns.")
    if joins["skill_invocations"]["matched"] == 0:
        actions.append("Collect skill invocation rows before making effectiveness claims.")
    if joins["review_scores"]["trend_status"] == "insufficient_data":
        actions.append("Treat review-score effectiveness as insufficient data until at least 5 entries exist.")
    return actions or ["No codification action proposed from this small sample."]


def build_report(args: argparse.Namespace) -> dict[str, Any]:
    paths = transcript_paths(args.transcript, args.source_root)
    if not paths:
        raise SystemExit("provide at least one --transcript path or explicit --source-root")
    transcripts = [parse_transcript(path) for path in paths]
    all_turns = [turn for transcript in transcripts for turn in transcript["turns"]]
    categories = categorize(all_turns, allow_excerpts=args.allow_redacted_excerpts)

    skill_rows, skill_coverage, skill_warnings = read_jsonl(args.skill_log, "skill invocation")
    work_rows, work_coverage, work_warnings = read_jsonl(args.work_ledger, "work ledger")
    delegation_rows, delegation_coverage, delegation_warnings = read_jsonl(args.delegations, "delegation")
    review_rows, review_coverage, review_warnings = read_jsonl(args.review_scores, "review scores")

    joins = join_evidence(transcripts, skill_rows, work_rows, delegation_rows, review_rows)
    redaction_summary = Counter()
    for transcript in transcripts:
        redaction_summary.update(transcript["redactions"])

    return {
        "schema_version": 1,
        "report_type": "transcript_effectiveness_mining",
        "transcripts": [
            {
                "path": transcript["path"],
                "turn_count": len(transcript["turns"]),
                "malformed_count": transcript["malformed"],
                "session_ids": transcript["session_ids"],
                "backlog_refs": transcript["backlog_refs"],
                "work_ids": transcript["work_ids"],
                "branches": transcript["branches"],
                "projects": transcript["projects"],
            }
            for transcript in transcripts
        ],
        "categories": categories,
        "joins": joins,
        "source_coverage": {
            "claude_transcripts": {"present": True, "rows": len(paths), "path": ",".join(str(path) for path in paths)},
            "codex_sessions": {"present": False, "rows": 0, "path": "unsupported explicit export in this slice"},
            "skill_invocations": skill_coverage,
            "work_ledger": work_coverage,
            "delegations": delegation_coverage,
            "review_scores": review_coverage,
        },
        "redaction_summary": {
            "redacted_segments": redaction_summary.get("redacted_segments", 0),
            "excerpts_included": args.allow_redacted_excerpts,
        },
        "warnings": skill_warnings + work_warnings + delegation_warnings + review_warnings,
        "proposed_actions": proposed_actions(categories, joins),
    }


def render_markdown(report: dict[str, Any]) -> str:
    lines = ["# Transcript Effectiveness Mining", "", "## Category Counts", ""]
    lines.extend(["| Category | Count | Evidence Refs |", "|---|---:|---|"])
    for name, data in report["categories"].items():
        refs = ", ".join(f"{ref['role']}:{ref['lineno']}" for ref in data["refs"]) or "none"
        lines.append(f"| {name} | {data['count']} | {refs} |")
    lines.extend(["", "## Source Coverage", "", "| Source | Present | Rows | Path |", "|---|---|---:|---|"])
    for name, coverage in report["source_coverage"].items():
        lines.append(f"| {name} | {coverage['present']} | {coverage['rows']} | {coverage['path']} |")
    lines.extend(["", "## Joins", "", "| Store | Matched | Missing | Refs |", "|---|---:|---:|---|"])
    for name, data in report["joins"].items():
        lines.append(f"| {name} | {data['matched']} | {data['missing']} | {', '.join(data['refs']) or 'none'} |")
    redaction = report["redaction_summary"]
    lines.extend(
        [
            "",
            "## Redaction Summary",
            "",
            f"- redacted_segments: {redaction['redacted_segments']}",
            f"- excerpts_included: {redaction['excerpts_included']}",
            "",
            "## Proposed Actions",
        ]
    )
    lines.extend(f"- {action}" for action in report["proposed_actions"])
    lines.extend(["", "## Warnings"])
    lines.extend(f"- {warning}" for warning in report["warnings"] or ["none"])
    return "\n".join(lines)


def self_test() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        transcript = root / "session.jsonl"
        transcript.write_text(
            "\n".join(
                [
                    json.dumps({"type": "user", "sessionId": "sess-1", "gitBranch": "feat/test", "cwd": "/tmp/harness-kit", "message": {"role": "user", "content": "This is wrong, use /reflect instead."}}),
                    json.dumps({"type": "assistant", "sessionId": "sess-1", "backlog_ref": "091", "work_id": "work-091", "cwd": "/tmp/harness-kit", "message": {"role": "assistant", "content": "Tool failed, then Skill reflect succeeded. Authorization: Bearer sk-test_1234567890abcdef /Users/alice/project"}}),
                ]
            )
            + "\n",
            encoding="utf-8",
        )
        skill_log = root / "skills.jsonl"
        skill_log.write_text(json.dumps({"session_id": "sess-1", "skill": "reflect", "project": "harness-kit"}) + "\n")
        work_ledger = root / "work.jsonl"
        work_ledger.write_text(json.dumps({"work_id": "work-091", "owning_skill": "deliver"}) + "\n")
        delegations = root / "delegations.jsonl"
        delegations.write_text(json.dumps({"backlog_ref": "091", "delegation_id": "del-1"}) + "\n")
        review_scores = root / "review.ndjson"
        review_scores.write_text(json.dumps({"branch": "feat/test", "correctness": 8}) + "\n")
        args = argparse.Namespace(
            transcript=[transcript],
            source_root=[],
            skill_log=skill_log,
            work_ledger=work_ledger,
            delegations=delegations,
            review_scores=review_scores,
            allow_redacted_excerpts=False,
            format="json",
        )
        report = build_report(args)
        rendered = render_markdown(report)
        assert report["categories"]["user_corrections"]["count"] == 1
        assert report["joins"]["skill_invocations"]["matched"] == 1
        assert report["joins"]["review_scores"]["matched"] == 1
        assert report["redaction_summary"]["redacted_segments"] >= 1
        assert "sk-test" not in json.dumps(report)
        assert "Authorization: Bearer" not in rendered
        assert "wrong, use" not in rendered

        unsafe = root / "unsafe.jsonl"
        unsafe.write_text(json.dumps({"message": {"role": "user", "content": "private_customer_data"}}) + "\n")
        args.transcript = [unsafe]
        try:
            build_report(args)
        except SystemExit as error:
            assert "unresolved secret-like" in str(error)
        else:
            raise AssertionError("unresolved private customer data should fail closed")
    print("transcript effectiveness mining self-test ok")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--transcript", action="append", type=Path, default=[])
    parser.add_argument("--source-root", action="append", type=Path, default=[])
    parser.add_argument("--skill-log", type=Path, default=DEFAULT_SKILL_LOG)
    parser.add_argument("--work-ledger", type=Path, default=DEFAULT_WORK_LEDGER)
    parser.add_argument("--delegations", type=Path, default=DEFAULT_DELEGATIONS)
    parser.add_argument("--review-scores", type=Path, default=DEFAULT_REVIEW_SCORES)
    parser.add_argument("--allow-redacted-excerpts", action="store_true")
    parser.add_argument("--format", choices=("json", "markdown"), default="markdown")
    parser.add_argument("--self-test", action="store_true")
    return parser


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    if args.self_test:
        return self_test()
    report = build_report(args)
    if args.format == "json":
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        print(render_markdown(report))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
