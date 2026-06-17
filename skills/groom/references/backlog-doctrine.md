# Backlog Doctrine

## Two-Tier Backlog

| Tier | Location | Purpose | Queue health |
|------|----------|---------|-----|
| **Shaped work** | `backlog.d/` | Ready-to-build items with goal + oracle + verification system + sequence | Evidence-backed, ordered, actively maintained |
| **Icebox** | `.groom/BACKLOG.md` | Everything else worth remembering | Searchable, pruned, not a substitute for shaping |

`backlog.d/` is the canonical backlog. Files are the source of truth; there is
no parallel issue store.

Ideas flow between tiers during `/groom` sessions:
- **Shape:** raw finding → `backlog.d/` file (gets goal + oracle + verification system → ready to build)
- **Promote:** BACKLOG.md → `backlog.d/` (idea becomes active)
- **Demote:** `backlog.d/` → BACKLOG.md (item loses priority)
- **Archive:** BACKLOG.md → strikethrough (idea is done, obsolete, or absorbed)
- **Discard:** any tier → gone (no remaining value)

## What the active backlog is for

The active backlog (`backlog.d/`) is the current plan, not storage for every idea.
A good backlog is ordered, transparent, and actively maintained. It should make the next
decisions obvious.

## Harness Kit Product Lens

Harness Kit is primarily a harness product for other repositories. Its own repo is where
patterns get validated before they spread.

When shaping Harness Kit backlog items, prefer work that is one of:
- a reusable primitive, scaffold, reference, or policy other repos can adopt
- a proving-ground validation of a pattern meant to transfer outward
- debt removal that materially blocks downstream adoption or trust

If an item only improves Harness Kit's own repo and has no clear transfer value, demote it,
merge it into a broader reusable effort, or rewrite it until the downstream payoff is explicit.

## Core rules

- **Backlog size is telemetry, not policy.** Large queues demand evidence:
  duplicates, stale owners, orphaned themes, unclear sequencing, or weak
  oracles. Consolidate because outcomes overlap, not because a counter fired.
- Reduce while adding: consolidation and deletion proposals are part of the
  same strategic groom that emits new ambition.
- Ready work names the proof loop, not just the desired outcome. For M+
  tickets, use `harnesses/shared/references/verification-system-first.md` to
  state claim, falsifier, driver, grader, evidence packet, cadence, and gaps.
- **The backlog describes the best version of the software**, not the next
  safe increment. Rank by impact discounted by confidence; effort barely
  discounts when agents execute.
- Prefer one canonical item per outcome.
- Split discovery from delivery.
- Order work by user value, risk reduction, learning, and enablement.
- For Harness Kit itself, optimize for downstream leverage first and local convenience second.
- Keep active work narrow. High WIP destroys prioritization.
- Ideas that aren't execution-ready live in `.groom/BACKLOG.md`.

## Closure protocol

An active backlog item is closed when it leaves `backlog.d/`, not when someone
intends to close it later.

- `/ship` closes shipped work by moving it to `backlog.d/_done/` via
  `harness-kit-checks backlog archive` and carries a
  `Closes-backlog:` or `Ships-backlog:` trailer into the squash commit.
- `/groom`'s always-on tidy sweep scans master for those trailers and
  archives any surviving ticket files.
- `## What Was Built` is archival content; an item that already has that block
  does not belong in active backlog.

## Healthy item shapes

### Epic

The default shape for strategic groom emissions. Use for a multi-issue initiative with a clear
product outcome. The epic should explain why the theme matters, what success looks like, and
which child issues carry execution. The epic itself still needs done criteria; an umbrella with
no oracle is the omnibus smell, not an epic.

### Feature

Use for a user-visible capability or operator-facing behavior change. The feature should be
valuable on its own and not just a mechanical subtask.

### Bug

Use when the current behavior is wrong. State the failure, repro, expected behavior, and user or
business impact.

### Task / Refactor / Research

Use only when the work is not a feature or bug. Keep these issue types outcome-linked:
- `task`: enabling work with a clear downstream payoff
- `refactor`: complexity reduction with preserved behavior
- `research`: a decision-seeking investigation with a deliverable

## Ordering guidance

Move items up when they:
- unblock or de-risk other work
- fix trust, correctness, or safety failures
- improve a critical user path
- create leverage across multiple future issues
- create leverage across multiple downstream repositories

Move items down when they:
- are polish without evidence
- duplicate a broader surviving issue
- depend on undefined architecture
- represent “maybe someday” ideas with no current owner
- only improve Harness Kit's own repo without reusable payoff

## Cadence

- Triage new intake quickly into keep, merge, demote, or close.
- Re-read the active backlog regularly enough to remove stale assumptions.
- Run pruning passes, not just addition passes.
- Update the canonical issue body when the plan changes. Do not bury the truth in comments.
- Review `.groom/BACKLOG.md` every groom session — promote, archive, or leave.

## Smells

- 5 tickets that all mean the same thing
- “Polish” items that should be sub-points in a deeper item
- implementation tasks with no user or system outcome
- giant omnibus tickets with unclear done criteria
- items that require tribal knowledge to start
- “investigate” tickets with no decision target
- many open items with no ordering, theme, owner, oracle, or verification system
- stale items sitting open for weeks (ungroomed noise)
- BACKLOG.md not updated in 3+ groom sessions (icebox is rotting)

## Definition of ready

Before an issue is execution-ready, verify:
- the problem is specific
- the outcome is explicit
- dependencies are visible
- scope boundaries are present
- verification is executable
- downstream leverage or proving-ground rationale is explicit for Harness Kit items
- the issue can be completed in one coherent pass or should be split

## AI-agent adaptation

See `agent-issue-writing.md` for agent-specific issue shaping.

## Sources

- https://scrumguides.org/scrum-guide
- https://www.atlassian.com/agile/project-management/backlog-refinement-meeting
- https://www.atlassian.com/agile/project-management/product-backlog
