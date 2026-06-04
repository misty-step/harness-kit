---
name: debrief
description: |
  Lightweight evidence-backed retro and catch-up reports for a current repo,
  branch, PR, backlog slice, or recent agent session. Use when the user asks
  for a debrief, catch me up, what changed, why it matters, product
  implications, end-user implications, developer experience implications,
  current app state, backlog state, workspace state, alternatives considered,
  or context rebuild after losing the thread. Trigger: /debrief.
---

# Debrief

Produce a high-context explanation of what happened and why it matters.
Default to read-only. This is a catch-up and sensemaking skill, not a delivery
or codification skill.

Use `/reflect` instead when the main job is harness postmortem, operator
coaching, prompt debt, or writing follow-up codification. Use `/demo` when the
main job is a visual or runnable proof artifact. A debrief may point to either
as a next move.

## Evidence Stance

Start from live evidence. Prefer:

- `git status --short --branch --untracked-files=all`
- recent commits, staged diff, unstaged diff, branch/base comparison
- backlog files, PR descriptions, CI output, Dagger/test logs, trace receipts
- docs touched by the work, app routes or CLI help for changed surfaces
- screenshots or running app state only when the request needs rendered truth

Treat memory and chat history as leads, not truth. If evidence is missing,
stale, or too expensive to refresh, say that in `Sources & Gaps`.

Do not imply validation without the exact command, path, route, artifact, or
source that proves it. Separate confirmed facts, reasonable inference, and
opinion.

## Depth Control

Calibrate to the user's wording:

- "quick debrief", "where are we" -> concise report, no provider lanes unless
  scope is broad or uncertain.
- "thorough", "comprehensive", "aggressive", "deep retro" -> probe roster and
  use two or more provider lanes for independent interpretation or risk review.
- "right now" usually means prioritize useful synthesis over exhaustive
  archaeology; name gaps instead of blocking on perfect history.

When using provider lanes, give them live artifacts only: diff, backlog, commit
range, traces, or docs. Do not leak the lead's conclusions.

## Workflow

1. **Frame the scope.** Name the repo, branch, time window, backlog item, PR, or
   session being debriefed. If the scope is ambiguous, choose the current
   workspace and state that assumption.
2. **Read current state.** Inspect git status, recent commits, active backlog
   index, and the files most likely to explain the work. Re-read after
   compaction or handoff.
3. **Reconstruct the story.** Identify what changed, why it was attempted, the
   decision path, alternatives not taken, and current residual risk. Prefer
   file-backed evidence over narrative memory.
4. **Translate implications.** Explain what this means for product behavior,
   end users, operators, developers, maintainers, and the codebase shape.
5. **Describe state now.** Include application/system state, backlog state, and
   workspace/git state. Make blockers and dirty-tree facts explicit.
6. **Recommend next moves.** Give a short ordered list. Distinguish "ship now",
   "verify first", "shape next", and "defer".

## Report Shape

Use this shape by default. Compress sections when the change is small, but keep
`Sources & Gaps`, `Current State`, and `Next Moves`.

```markdown
**Sources & Gaps**
- Confirmed sources:
- Missing or stale sources:
- Assumptions:

**Short Version**
One paragraph that catches up a busy operator.

**What Changed**
- User-visible or operator-visible behavior:
- Code/docs/tests/infrastructure:
- Validation evidence:

**Why It Matters**
- Product:
- End user:
- Developer experience:
- Codebase health:

**Decision Record**
- Decisions made:
- Alternatives considered or implied:
- Why those alternatives were not taken:

**Current State**
- Application/system:
- Backlog:
- Workspace/git:

**Risks & Unknowns**
- Confirmed risks:
- Unverified paths:
- Follow-up debt:

**Next Moves**
1. Immediate:
2. Next best:
3. Later:
```

## Writing Standard

Assume the reader has lost context. Define repo-local names the first time they
matter. Do not dump raw diffs or command logs; synthesize them into an
operator-useful story with citations to concrete files, commits, commands, or
artifacts.

Make tradeoffs explicit. Good debriefs explain both "what we did" and "what we
chose not to do." If alternatives are not visible in the evidence, say so
instead of inventing a design debate.

Keep recommendations opinionated but bounded. The output should leave the user
able to decide whether to ship, continue, pause, or reshape.
