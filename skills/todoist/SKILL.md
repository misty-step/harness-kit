---
name: todoist
description: |
  Manage the operator's Todoist as the system of record for tasks, reminders,
  and follow-ups — capture, triage, complete, and organize. Agent-native via
  the Todoist MCP; scriptable via the `td` CLI. Use when: "add this to my
  todoist", "add a task", "remind me to", "capture this", "what's on my
  todoist", "what's due today", "what's in my inbox", "mark this done",
  "create a project/label", "log a follow-up". Trigger: /todoist, /task, /todo.
argument-hint: "[add <nl task> | today | inbox | upcoming | done <id>]"
---

# /todoist

Todoist is the operator's task system of record (migrated off Google Tasks for
agent-friendliness). Two surfaces, one account (`phraznikov@gmail.com`):

| Surface | Use it for | How |
|---|---|---|
| **Todoist MCP** (`mcp__todoist__*`) | interactive agent task ops inside a session — preferred | OAuth, in-session tool calls |
| **`td` CLI** | scripts, batch, hooks, deterministic JSON output, cron | `td <cmd>`, OAuth-authed once |

Prefer the **MCP** when reasoning conversationally; prefer the **CLI** when you
need scripted, parseable, or repeatable output (`td ... --progress-jsonl`, `-q`
prints just the created ID).

## Preflight (cheap, do it once per session if writing)

- MCP: if a `mcp__todoist__*` call returns an auth error, tell the operator to
  run `/mcp` → `todoist` → authenticate (browser OAuth). You cannot complete
  OAuth for them.
- CLI: `td auth status`. If "No API token found", the operator runs
  `td auth login` (browser OAuth) or `td auth token <token>`. Do not paste a
  token you read from disk into a command line; use `TODOIST_API_TOKEN` by ref.
- `td doctor` diagnoses CLI/env problems.

## Core operations (CLI; MCP has equivalent tools)

```bash
td add "Email Sarah about Q3 deck tomorrow 9am p1 #Work @waiting"  # NL quick-add
td today                 # due today + overdue
td upcoming              # next 7 days  (td upcoming --days N)
td inbox                 # untriaged capture
td task list --project "Work"
td task view <id|url>
td task close <id>       # complete   (td task reopen <id> to undo)
td project list          # td project add "Name"
td label list
```

Natural-language quick-add grammar (same in MCP and CLI):
`tomorrow 9am`, `every weekday`, `p1`–`p4` (priority), `#Project`, `@label`,
`+Person` (assignee, shared projects). Default destination is **Inbox** unless
a `#Project` is given.

## Conventions

- **Capture fast, triage later.** Ambiguous asks → drop in Inbox with a clear
  title; don't invent projects/labels. Surface them for triage with `td inbox`.
- **Read back after writing.** After an add/close, echo the resulting task
  (id + title + due) so the operator can verify; `-q` gives just the ID for
  scripts.
- **Priorities:** p1 = today/urgent, p2 = this week, p3 = soon, p4 = someday
  (Todoist's default). Don't assign p1 unless the operator signals urgency.

## Boundaries

- Confirm before **bulk** changes, deletes, or anything irreversible. Closing a
  single task is fine; deleting a project is not without a yes.
- This skill owns **Todoist only**. The operator also runs a separate
  topydo/todo.txt habit system in `~/Documents/daybook/` (aliases `t`, `ta`,
  streak scripts). Do **not** cross-write between them or touch daybook here.
- ⚠️ Shell note: in the operator's interactive shell the alias `td='t do'`
  (topydo) currently shadows the Todoist `td` binary at `~/.npm-global/bin/td`.
  Until the operator resolves the alias, invoke the CLI by full path in scripts:
  `~/.npm-global/bin/td`.
