#!/usr/bin/env python3
"""Parse Claude Code JSONL transcripts into skill instruction packets."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any


def iter_jsonl(path: Path) -> tuple[list[dict[str, Any]], int]:
    rows: list[dict[str, Any]] = []
    malformed = 0
    for line in path.read_text().splitlines():
        if not line.strip():
            continue
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            malformed += 1
            continue
        if isinstance(value, dict):
            rows.append(value)
    return rows, malformed


def extract_text(content: Any) -> str:
    if isinstance(content, str):
        return content.strip()
    if isinstance(content, list):
        parts = []
        for item in content:
            if isinstance(item, str):
                parts.append(item)
            elif isinstance(item, dict) and isinstance(item.get("text"), str):
                parts.append(item["text"])
            elif isinstance(item, dict) and isinstance(item.get("content"), str):
                parts.append(item["content"])
        return "\n".join(part.strip() for part in parts if part.strip()).strip()
    if isinstance(content, dict) and isinstance(content.get("text"), str):
        return content["text"].strip()
    return ""


def extract_turns(rows: list[dict[str, Any]]) -> list[dict[str, str]]:
    turns: list[dict[str, str]] = []
    for row in rows:
        message = row.get("message") if isinstance(row.get("message"), dict) else row
        role = message.get("role") or row.get("type")
        if role not in {"user", "assistant", "system"}:
            continue
        text = extract_text(message.get("content"))
        if text:
            turns.append({"role": str(role), "text": text})
    return turns


def candidate_instructions(turns: list[dict[str, str]]) -> list[str]:
    candidates = []
    for turn in turns:
        text = turn["text"]
        lowered = text.lower()
        if turn["role"] == "assistant" and any(
            marker in lowered
            for marker in ("use ", "avoid ", "must ", "when ", "workflow", "skill", "contract")
        ):
            candidates.append(text)
    return candidates


def parse_transcript(path: Path) -> dict[str, object]:
    rows, malformed = iter_jsonl(path)
    turns = extract_turns(rows)
    return {
        "source": str(path),
        "turns": turns,
        "candidate_instructions": candidate_instructions(turns),
        "evidence": {
            "row_count": len(rows),
            "turn_count": len(turns),
            "malformed_count": malformed,
        },
    }


def claude_project_keys(cwd: Path) -> set[str]:
    resolved = str(cwd.resolve())
    stripped = resolved.strip("/")
    return {
        resolved.replace("/", "-"),
        stripped.replace("/", "-"),
    }


def find_current_transcript(projects_dir: Path, cwd: Path) -> Path:
    keys = claude_project_keys(cwd)
    candidates = []
    for path in projects_dir.rglob("*.jsonl"):
        if not path.is_file() or path.parent.name not in keys:
            continue
        candidates.append((path.stat().st_mtime, path))
    if not candidates:
        raise FileNotFoundError(f"no Claude JSONL transcripts found for {cwd} under {projects_dir}")
    candidates.sort(reverse=True)
    return candidates[0][1]


def resolve_transcript(
    transcript: Path | None,
    *,
    from_current: bool,
    projects_dir: Path,
    cwd: Path,
) -> Path:
    if transcript and from_current:
        raise ValueError("provide either a transcript path or --from-current, not both")
    if transcript:
        return transcript
    if from_current:
        return find_current_transcript(projects_dir, cwd)
    raise ValueError("provide a transcript path or --from-current")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("transcript", type=Path, nargs="?")
    parser.add_argument(
        "--from-current",
        action="store_true",
        help="Use the latest Claude Code JSONL transcript for the current project.",
    )
    parser.add_argument(
        "--claude-projects-dir",
        type=Path,
        default=Path.home() / ".claude" / "projects",
        help="Claude Code projects directory. Defaults to ~/.claude/projects.",
    )
    args = parser.parse_args(argv)
    try:
        transcript = resolve_transcript(
            args.transcript,
            from_current=args.from_current,
            projects_dir=args.claude_projects_dir,
            cwd=Path.cwd(),
        )
    except (FileNotFoundError, ValueError) as exc:
        print(f"parse-transcript: {exc}", file=sys.stderr)
        return 2
    print(json.dumps(parse_transcript(transcript), sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
