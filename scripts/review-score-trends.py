#!/usr/bin/env python3
"""Summarize .groom/review-scores.ndjson into review-quality trend signal."""

from __future__ import annotations

import argparse
import json
import sys
import tempfile
from pathlib import Path
from statistics import mean
from typing import Any


DIMENSIONS = ["correctness", "depth", "simplicity", "craft"]
REQUIRED_SCORE_FIELDS = [
    "date",
    "branch",
    "sha",
    "correctness",
    "depth",
    "simplicity",
    "craft",
    "verdict",
    "providers",
    "findings_total",
    "findings_accepted",
    "findings_false_positive",
    "post_merge_bugs_found",
]
TUNING_TARGETS = {
    "correctness": "skills/code-review/SKILL.md executable-path and blocking-finding instructions",
    "depth": "skills/code-review/references/deep-review-lens.md reviewer prompts",
    "simplicity": "skills/code-review/SKILL.md Thermo / Deslop Lens",
    "craft": "skills/code-review/SKILL.md Completion Gate and evidence formatting",
}


def load_rows(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    if not path.exists():
        return rows
    for lineno, line in enumerate(path.read_text().splitlines(), start=1):
        if not line.strip():
            continue
        try:
            row = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError(f"{path}:{lineno}: invalid JSON: {error}") from error
        if not isinstance(row, dict):
            raise ValueError(f"{path}:{lineno}: row must be a JSON object")
        rows.append(row)
    return rows


def numeric(row: dict[str, Any], field: str) -> float | None:
    value = row.get(field)
    if isinstance(value, bool):
        return None
    if isinstance(value, (int, float)):
        return float(value)
    return None


def missing_schema_fields(rows: list[dict[str, Any]]) -> dict[str, int]:
    counts = {field: 0 for field in REQUIRED_SCORE_FIELDS}
    for row in rows:
        for field in REQUIRED_SCORE_FIELDS:
            if field not in row:
                counts[field] += 1
    return {field: count for field, count in counts.items() if count}


def regression_lines(rows: list[dict[str, Any]]) -> list[str]:
    if len(rows) < 5:
        return []
    window = rows[-5:]
    lines: list[str] = []
    for dimension in DIMENSIONS:
        values = [numeric(row, dimension) for row in window]
        if any(value is None for value in values):
            continue
        first = mean(value for value in values[:2] if value is not None)
        last = mean(value for value in values[2:] if value is not None)
        delta = last - first
        if delta <= -2.0:
            target = TUNING_TARGETS[dimension]
            lines.append(
                f"- {dimension} regression: last-3 avg {last:.1f} vs "
                f"first-2 avg {first:.1f}. Proposed skill target: {target}."
            )
    return lines


def false_positive_summary(rows: list[dict[str, Any]]) -> tuple[str, bool]:
    totals = [
        (
            numeric(row, "findings_total"),
            numeric(row, "findings_false_positive"),
        )
        for row in rows[-20:]
    ]
    usable = [(total, false) for total, false in totals if total is not None and false is not None and total > 0]
    if not usable:
        return "False-positive rate: unavailable (no calibrated finding counts).", False
    total_findings = sum(total for total, _ in usable)
    total_false = sum(false for _, false in usable)
    rate = total_false / total_findings if total_findings else 0.0
    needs_tuning = rate >= 0.25 and total_findings >= 4
    return (
        f"False-positive rate: {rate * 100:.1f}% "
        f"({int(total_false)}/{int(total_findings)} calibrated findings).",
        needs_tuning,
    )


def report(rows: list[dict[str, Any]], path: Path) -> str:
    lines = [
        "Review Score Trend",
        f"- Source: {path}",
        f"- Entries: {len(rows)}",
    ]
    if not rows:
        lines.append("- Status: no review scores recorded.")
        return "\n".join(lines)

    missing = missing_schema_fields(rows)
    if missing:
        fields = ", ".join(f"{field} missing in {count}" for field, count in sorted(missing.items()))
        lines.append(f"- Schema coverage: legacy or incomplete rows ({fields}).")
    else:
        lines.append("- Schema coverage: all rows include required feedback-loop fields.")

    fp_line, fp_needs_tuning = false_positive_summary(rows)
    lines.append(f"- {fp_line}")

    if len(rows) < 5:
        lines.append(f"- Status: insufficient trend data ({len(rows)}/5 entries).")
        return "\n".join(lines)

    lines.append("- Rolling window: last 5 entries.")
    for dimension in DIMENSIONS:
        values = [numeric(row, dimension) for row in rows[-5:]]
        present = [value for value in values if value is not None]
        if present:
            lines.append(f"- {dimension} avg: {mean(present):.1f}")

    regressions = regression_lines(rows)
    if regressions:
        lines.append("Skill tuning suggestions:")
        lines.extend(regressions)
    elif fp_needs_tuning:
        lines.append("Skill tuning suggestions:")
        lines.append(
            "- false-positive rate high: tighten finding acceptance and "
            "rejection-after-steelman guidance in skills/code-review/SKILL.md."
        )
    else:
        lines.append("Skill tuning suggestions: none from current score window.")
    return "\n".join(lines)


def self_test() -> int:
    rows = [
        {"date": "2026-06-01", "branch": "a", "sha": "1", "correctness": 9, "depth": 8, "simplicity": 8, "craft": 8, "verdict": "ship", "providers": ["codex"], "findings_total": 4, "findings_accepted": 4, "findings_false_positive": 0, "post_merge_bugs_found": 0},
        {"date": "2026-06-01", "branch": "b", "sha": "2", "correctness": 9, "depth": 8, "simplicity": 8, "craft": 8, "verdict": "ship", "providers": ["codex"], "findings_total": 4, "findings_accepted": 3, "findings_false_positive": 1, "post_merge_bugs_found": 0},
        {"date": "2026-06-01", "branch": "c", "sha": "3", "correctness": 6, "depth": 8, "simplicity": 8, "craft": 8, "verdict": "conditional", "providers": ["codex"], "findings_total": 4, "findings_accepted": 2, "findings_false_positive": 2, "post_merge_bugs_found": 1},
        {"date": "2026-06-01", "branch": "d", "sha": "4", "correctness": 6, "depth": 8, "simplicity": 8, "craft": 8, "verdict": "conditional", "providers": ["codex"], "findings_total": 4, "findings_accepted": 3, "findings_false_positive": 1, "post_merge_bugs_found": 0},
        {"date": "2026-06-01", "branch": "e", "sha": "5", "correctness": 6, "depth": 8, "simplicity": 8, "craft": 8, "verdict": "dont-ship", "providers": ["codex"], "findings_total": 4, "findings_accepted": 3, "findings_false_positive": 1, "post_merge_bugs_found": 1},
    ]
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "review-scores.ndjson"
        path.write_text("\n".join(json.dumps(row, sort_keys=True) for row in rows) + "\n")
        output = report(load_rows(path), path)
    required = ["correctness regression", "False-positive rate: 25.0%", "Skill tuning suggestions"]
    missing = [token for token in required if token not in output]
    if missing:
        print(output, file=sys.stderr)
        print(f"FAIL: self-test missing token(s): {', '.join(missing)}", file=sys.stderr)
        return 1
    print("OK: review-score trend self-test passed")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("path", nargs="?", default=".groom/review-scores.ndjson")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    path = Path(args.path)
    try:
        rows = load_rows(path)
    except ValueError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print(report(rows, path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
