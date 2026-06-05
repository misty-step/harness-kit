# Skill invocation tracking

Priority: medium
Status: done
Estimate: S

## Goal

Track which skills are invoked, when, how often, and in what context.
Currently zero telemetry. This enables /reflect to ground its analysis in
actual tool usage data rather than conversation heuristics alone.

## Non-Goals

- No dashboards, no aggregation CLI, no remote reporting
- No blocking behavior -- purely passive, append-only
- No tracking of non-Skill tool uses (scope creep)

## Design

Three changes: a PostToolUse hook script, a settings.json entry, and a
surgical addition to /reflect's conversation archaeology phase.

---

### 1. Hook Script

**Path:** `harness-kit-checks claude-hook skill-invocation-tracker`
(symlinked to `~/.claude/hooks/` by bootstrap.sh via `link_dir_entries_if_present`)

```python
#!/usr/bin/env python3
"""
PostToolUse hook: append skill invocation records to a JSONL log.

Passive telemetry -- exits 0 with no stdout. Never influences tool behavior.
Reads the Claude Code hook protocol JSON from stdin, extracts Skill tool
invocations, appends one JSONL line per invocation.
"""

import json
import os
import sys
from datetime import datetime, timezone
from pathlib import Path

LOG_PATH = Path(os.path.expanduser("~/.claude/skill-invocations.jsonl"))


def main():
    try:
        data = json.load(sys.stdin)
    except (json.JSONDecodeError, EOFError):
        sys.exit(0)

    tool_name = data.get("tool_name", "")
    if tool_name != "Skill":
        sys.exit(0)

    tool_input = data.get("tool_input") or {}
    skill = tool_input.get("skill", "")
    if not skill:
        sys.exit(0)

    cwd = data.get("cwd", "")
    entry = {
        "ts": datetime.now(timezone.utc).isoformat(),
        "skill": skill,
        "args": tool_input.get("args", ""),
        "session_id": data.get("session_id", ""),
        "cwd": cwd,
        "project": os.path.basename(cwd) if cwd else "",
    }

    try:
        LOG_PATH.parent.mkdir(parents=True, exist_ok=True)
        with LOG_PATH.open("a") as f:
            f.write(json.dumps(entry, separators=(",", ":")) + "\n")
    except OSError:
        pass  # Never fail the hook -- telemetry is best-effort

    sys.exit(0)


if __name__ == "__main__":
    main()
```

**Key decisions:**
- UTC timestamps (consistent across timezones, no ambiguity)
- Compact JSON separators (no spaces) to keep log lean
- `OSError` swallowed -- a full disk or permission error must not block the session
- No dependencies beyond stdlib
- Filters on `tool_name == "Skill"` early to skip irrelevant events fast

---

### 2. Settings Config

**Path:** `harnesses/claude/settings.json`

Add a new entry to the `hooks` object under a new `PostToolUse` key:

```diff
     "SessionStart": [
       {
         "matcher": "",
         "hooks": [
           {
             "type": "command",
             "command": "harness-kit-checks claude-hook time-context",
             "timeout": 5
           }
         ]
       }
-    ]
+    ],
+    "PostToolUse": [
+      {
+        "matcher": "Skill",
+        "hooks": [
+          {
+            "type": "command",
+            "command": "harness-kit-checks claude-hook skill-invocation-tracker",
+            "timeout": 5
+          }
+        ]
+      }
+    ]
   },
```

**Note:** `settings.json` is COPIED by bootstrap (not symlinked) because Claude
modifies it at runtime. After building this, re-run `bootstrap.sh` to install.

The spec originally called for `"async": true` but the Claude Code hooks schema
uses `"timeout"` for controlling non-blocking behavior. A 5-second timeout is
generous for an append-to-file operation that completes in <10ms. The hook
produces no stdout, so it cannot block or alter the skill's execution regardless.

---

### 3. Reflect Integration

**Path:** `skills/reflect/SKILL.md`

Surgical addition to Agent A (Conversation archaeology) in the Distill workflow.
Insert a new bullet after the existing "Available-but-unused information" bullet
(line 39 area). This is a new signal source, not a replacement.

```diff
 - **Available-but-unused information**: Files, env vars, configs, tools that existed
   and would have prevented friction, but the agent didn't know about or didn't use.
   (e.g., test credentials in `.env`, existing skills that weren't invoked)
+- **Skill invocation log**: Read `~/.claude/skill-invocations.jsonl`, filter to
+  current `session_id`. Report: which skills fired and how many times each.
+  Cross-reference against session activity -- if debugging happened but
+  /investigate never fired, or code was reviewed without /code-review, note the
+  gap. Feed these observations into the Swiss Cheese triage as layer 2 (Skills)
+  findings: "skill that didn't fire" or "skill fired but too late".
```

This is ~6 lines. It does not change the structure of /reflect -- it adds one
more data source to the archaeology agent, which already looks for "existing
skills that weren't invoked." The JSONL log makes that check concrete rather
than heuristic.

---

## File Summary

| File | Action |
|------|--------|
| `harness-kit-checks claude-hook skill-invocation-tracker` | Create hook entrypoint |
| `harnesses/claude/settings.json` | Edit (add PostToolUse entry) |
| `skills/reflect/SKILL.md` | Edit (add bullet to Agent A) |

## Acceptance Criteria

- [ ] `harness-kit-checks claude-hook skill-invocation-tracker <<< '{"tool_name":"Skill","tool_input":{"skill":"commit","args":"-m fix"},"session_id":"abc","cwd":"/tmp/myproject"}'` exits 0 with no stdout and appends one line to `~/.claude/skill-invocations.jsonl`
- [ ] The appended line is valid JSON with keys: `ts`, `skill`, `args`, `session_id`, `cwd`, `project`
- [ ] `harness-kit-checks claude-hook skill-invocation-tracker <<< '{"tool_name":"Bash","tool_input":{"command":"ls"}}'` exits 0 with no stdout and does NOT append to the log
- [ ] `echo '' | harness-kit-checks claude-hook skill-invocation-tracker` exits 0 (graceful on bad input)
- [ ] `settings.json` is valid JSON after the edit
- [ ] `bootstrap.sh` symlinks the new hook into `~/.claude/hooks/`
- [ ] `/reflect` in distill mode reads the JSONL log and reports skill usage for the session
- [ ] `dagger call check` still passes (no regressions)
- [ ] `skills/reflect/SKILL.md` stays under 500 lines

## Cross-Harness Notes

PostToolUse is a Claude Code hook event. Codex uses a different hook protocol
(`codex.toml` with `on_tool_*` events). If Codex gains Skill tool support:

- The JSONL format is harness-agnostic -- any writer can append
- The Python script reads a JSON blob from stdin; the stdin schema would need
  an adapter for Codex's event format (different field names)
- `/reflect` reads the JSONL file directly -- works regardless of which harness wrote it

No Codex changes needed now. The JSONL file is the integration seam.

## Risks

- **Log growth**: Unbounded append-only file. At ~200 bytes/entry and ~50
  skill invocations/day, this is ~10KB/day, ~3.6MB/year. Not a concern for
  the foreseeable future. If it becomes one, add a rotate-on-size check to
  the hook (not now -- YAGNI).
- **Concurrent writes**: Multiple Claude sessions can write simultaneously.
  Single-line JSONL appends are atomic on POSIX for lines under PIPE_BUF
  (4KB on macOS). Each entry is <300 bytes. Safe.
