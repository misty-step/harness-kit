---
name: next
description: |
  Recommend the best next move from live thread and repo state. Use when:
  "what's next", "what next", "now what", "what should I do next",
  "what should we do next", "anything else to do", "where are we now,
  what's next", "what next in the backlog". Trigger: /next, /what-next,
  /now-what.
argument-hint: "[repo|thread|backlog|branch|topic] [--act-if-safe]"
---

# /next

Answer the operator's recurring "what's next?" with a concrete recommendation,
not a strategy essay. Read-only by default.

## Contract

Ground first, then choose one next move. The output must separate agent-owned
work from user-owned external actions.

Use `--act-if-safe` only for reversible evidence gathering or mechanical checks:
status, branch, recent commits, backlog listing, open PR lookup, or reading the
current plan. Do not edit, commit, merge, push, message people, spend money, or
change external systems under `--act-if-safe`.

## Grounding

Read the smallest useful surface for the scope:

- current user ask and visible thread context
- `git status --short --branch --untracked-files=all` when in a repo
- active branch, upstream, dirty/unpushed state, and recent commits
- open `backlog.d/*.md` titles when backlog priority is in scope
- root `VISION.md` when the next move depends on product direction,
  positioning, project identity, or long-term sequencing
- live tracker queues across delivery states when an external tracker is in
  scope: at minimum `ready_for_dev`, `verification`, `in_progress`, and
  `blocked`/prerequisite-bearing items
- current plan, goal, PR, issue, or acceptance oracle when present
- recent closeout signal: shipped commit, archived backlog, PR status, or
  blocker note

Prefer live files and commands over remembered thread state. If external facts
would determine the next move, say which fact is missing rather than guessing.
Do not treat one empty scoped filter as absence of work. If a module/repo
filter returns nothing but the operator asked for that product area, broaden by
status, prefix, module id/name, open PRs, and verification/residual items before
recommending work outside that area.

## Report Shape

```markdown
**Next**
- State:
- Best next:
- Why this wins:
- My action:
- Your action:
- First move:
- Alternatives:
- Stop condition:
```

Keep it compact. The default answer should fit on one screen.

## Routing Judgment

| Signal | Best next |
|---|---|
| Dirty worktree with unclear ownership | Classify paths, then `/yeet` or ask one scope question |
| Clean feature branch, work complete | `/ship` or PR closeout |
| Unshaped idea, unclear acceptance | `/shape` |
| Open prioritized backlog, no active branch | `/groom` summary, then one pickup |
| Item already in verification | `/qa` or closeout before new delivery |
| Broken gate, failing app, unclear root cause | `/diagnose` |
| Running surface needs proof | `/qa` |
| User asks "what do I need to do?" | Separate user external action from agent action |
| Human external blocker | Say the agent is blocked; do not invent process work |
| Harness primitive/invocation problem | `/harness-engineering` |

## Gotchas

- Do not return only a menu. Recommend one path and defend it.
- Do not collapse user-owned and agent-owned work. "Call this person" is not
  an agent task unless the tool can actually do it.
- Do not redefine success around what is easy to do locally. Name the real
  acceptance or blocker.
- Do not recommend a new repo just because the active repo has no
  `ready_for_dev` items. Check verification, in-progress, recently shipped PRs,
  and blocked/prerequisite items first; finishing or proving existing work
  usually beats starting a new ticket.
- Treat title/description blockers as real even when status says ready. Name the
  prerequisite instead of presenting the item as immediately deliverable.
- Do not start a delivery workflow unless the user asked you to act or passed
  `--act-if-safe` and the first step is reversible.
- Do not answer from stale memory when a live repo/status read is cheap.
- Do not over-explore. If the next move is obvious after status/backlog/branch,
  stop and report.

## Verification

When editing this skill in Harness Kit:

```sh
cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .
```

Semantic acceptance: a useful `/next` answer names current state, one best next
move, why it wins, my action, your action, the first move, alternatives, and a
stop condition.
