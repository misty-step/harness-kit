#!/usr/bin/env python3
"""Validate harness runtime projection primitives."""

from __future__ import annotations

import json
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
CLAUDE_SETTINGS = ROOT / "harnesses" / "claude" / "settings.json"
CLAUDE_HOOKS = ROOT / "harnesses" / "claude" / "hooks"
SKILL_INVOCATION_FIXTURE = ROOT / ".harness-kit" / "examples" / "skill-invocations.jsonl"

LIVE_PROTOCOLS = {
    "claude": {"post_tool_use"},
    "codex": set(),
    "pi": set(),
    "antigravity-cli": set(),
}
IMPORT_PROTOCOLS = {"external_import"}


def hook_target(command: str) -> Path | None:
    for part in command.split():
        if part.startswith("~/.claude/hooks/"):
            return CLAUDE_HOOKS / Path(part).name
    return None


def check_claude_settings() -> None:
    data = json.loads(CLAUDE_SETTINGS.read_text(encoding="utf-8"))
    hooks = data.get("hooks")
    if not isinstance(hooks, dict):
        raise SystemExit(f"{CLAUDE_SETTINGS}: missing hooks object")

    checked = 0
    for event, groups in hooks.items():
        if not isinstance(groups, list):
            raise SystemExit(f"{CLAUDE_SETTINGS}: hooks.{event} must be a list")
        for group in groups:
            if not isinstance(group, dict):
                raise SystemExit(f"{CLAUDE_SETTINGS}: hooks.{event} entries must be objects")
            entries = group.get("hooks", [])
            if not isinstance(entries, list):
                raise SystemExit(f"{CLAUDE_SETTINGS}: hooks.{event}.hooks must be a list")
            for entry in entries:
                if not isinstance(entry, dict):
                    raise SystemExit(f"{CLAUDE_SETTINGS}: hook entry must be an object")
                target = hook_target(str(entry.get("command") or ""))
                if target is None:
                    continue
                checked += 1
                if not target.is_file():
                    raise SystemExit(f"{CLAUDE_SETTINGS}: stale hook target {target}")
    if checked == 0:
        raise SystemExit(f"{CLAUDE_SETTINGS}: no ~/.claude/hooks targets validated")


def check_skill_invocation_protocols() -> None:
    rows = 0
    for lineno, line in enumerate(
        SKILL_INVOCATION_FIXTURE.read_text(encoding="utf-8").splitlines(),
        start=1,
    ):
        if not line.strip():
            continue
        rows += 1
        record = json.loads(line)
        harness = str(record.get("harness") or "")
        protocol = str(record.get("source_protocol") or "")
        allowed = LIVE_PROTOCOLS.get(harness, set()) | IMPORT_PROTOCOLS
        if protocol not in allowed:
            raise SystemExit(
                f"{SKILL_INVOCATION_FIXTURE}:{lineno}: {harness}/{protocol} "
                "is not a verified live hook or import protocol"
            )
    if rows == 0:
        raise SystemExit(f"{SKILL_INVOCATION_FIXTURE}: no skill invocation fixture rows")


def run_claude_hook_tests() -> None:
    completed = subprocess.run(
        ["python3", "-m", "unittest", "discover", "-s", str(CLAUDE_HOOKS), "-p", "test_*.py"],
        cwd=ROOT,
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if completed.returncode != 0:
        output = (completed.stdout + completed.stderr).strip()
        raise SystemExit(output or "Claude hook tests failed")


def main() -> int:
    check_claude_settings()
    check_skill_invocation_protocols()
    run_claude_hook_tests()
    print("runtime primitives valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
