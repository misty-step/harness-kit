---
name: orient
description: |
  Fast session-start repository orientation from live local evidence. Use when:
  "orient yourself", "start of session", "new session", "where are we",
  "catch me up before acting", after compaction,
  after switching worktrees, or before choosing a Harness Kit workflow. Trigger:
  /orient, /ground, /session-start.
argument-hint: "[scope|--deep]"
---

# /orient

Start a session from repo truth, not memory. Read-only. Fast by default.

## Contract

Produce a short orientation report that prevents a wrong first move. Do not
deliver, groom, debrief, assess readiness, reflect, mine transcripts, or mutate
state.

Default budget: under 2 minutes, under 12 report lines. Use `--deep` only when
the user asks for more archaeology.

## Sources

Read the smallest live set that explains the current workspace:

- scoped `AGENTS.md` and repo `AGENTS.md`
- `project.md`, then `README.md` only if project focus is still unclear
- `git status --short --branch --untracked-files=all`
- current branch and recent commits
- active `backlog.d/*.md` count and titles
- newest relevant `backlog.d/_done/*.md` items or closure trailers
- `.harness-kit/agents.yaml` or roster probe output when next action depends
  on delegation
- `.harness-kit/agent-readiness.yaml` only to name the profile state, not to
  score the repo

Optional when requested or obviously relevant: work ledger, delegation receipt
summary, skill invocation analytics, review-score trends, or docs site state.

## Report Shape

```markdown
**Orientation**
- Repo / branch:
- Workspace state:
- Current focus:
- Backlog signal:
- Recent closure signal:
- Roster state:
- Applicable constraints:
- Blockers / gaps:
- Likely next skill:
- Residual uncertainty:
```

Keep each line evidence-backed. Include exact paths or commands for claims that
would change the next action.

## Routing Judgment

| Signal | Recommend |
|---|---|
| Active shaped ticket and clean branch | `/deliver <ticket>` |
| Dirty existing branch with intended changes | `/deliver --polish-only <branch>` |
| Empty or stale backlog | `/groom` |
| User asks what happened or why it matters | `/orient --deep` or `/shape` by context |
| Failure, broken gate, or unclear root cause | `/diagnose` |
| Finished work needing closeout | `/ship`, `/yeet`, or `/reflect` by context |
| Readiness/profile question | `/qa` or repo-specific readiness surface |
| Skill/harness primitive change | `/harness-engineering` |

If the next action is unclear, say what evidence is missing instead of choosing
a workflow from vibes.

## Gotchas

- Do not turn orientation into a mandatory ceremony. If the user already gave a
  precise command and the workspace state is obvious, keep it tiny.
- Do not summarize all history. Name the latest useful closure signal and stop.
- Do not store session memory. Durable state belongs in backlog, receipts,
  traces, commits, or explicit profile files.
- Do not call provider lanes unless scope is broad, stale, or contested.
- Do not label the repo "ready" or "validated" from orientation. Route that to
  the owning skill.
- Do not replace scoped `AGENTS.md`; respect it as the governing instruction
  source.

## Verification

When editing this skill in the Harness Kit source repo, validate trigger/catalog
shape with:

```sh
cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .
```

When invoking `/orient` in another repo, this source-repo command is not
required. Acceptance is the report's cited live evidence and useful next-skill
recommendation.
