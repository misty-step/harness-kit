---
name: groom
description: |
  Always-on backlog grooming. Tidy, brainstorm, interrogate, investigate,
  research, and simplify in a single loop. Tidy is not a mode — it happens
  every time. Strategic-layer work is a deep multi-perspective brainstorm —
  parallel investigation, critique, and research lanes composed for the
  repo at hand — that lands an epic-scoped, ambitious backlog.
  Use when: "groom", "what should we build", "rethink this", "biggest
  opportunity", "backlog", "prioritize", "backlog session",
  "audit skills", "skill quality audit".
  Trigger: /groom, /groom audit, /backlog, /rethink, /moonshot, /scaffold.
argument-hint: "[audit|--emphasis explore|rethink|moonshot|scaffold] [context]"
---

# /groom

Keep `backlog.d/` true: tidy what shipped, challenge what's queued, surface
what's missing, propose what to delete. A groom that only lists is not a
groom; the depth comes from repo evidence, not fixed quotas.

The backlog diff is the artifact. Prose exists to justify it.

## Tidy (mandatory, mechanical)

Tooling owns closure; consume it, don't hand-roll it:

```sh
default="$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')"
cargo run --locked -p harness-kit-checks -- backlog ids-from-range "origin/${default:-main}..${default:-main}"
cargo run --locked -p harness-kit-checks -- backlog archive "$id"   # idempotent
```

- Archive every ticket closed by `Closes-backlog:`/`Ships-backlog:` trailers
  or marked done/shipped in frontmatter. Commit as
  `chore(backlog): archive shipped tickets swept by /groom`.
- Flag stale `in-progress` (merged/deleted branch, or 30+ days untouched).
- Surface duplicates with a proposed consolidation — never merge silently.
- **Backlog size is telemetry, not policy.** If the active queue looks too
  broad, report the evidence: count, age, duplicates, stale owners, orphaned
  themes, and unfocused small items. Consolidate only when tickets genuinely
  share one outcome. Never veto an evidenced emission because of an arbitrary
  item count.

Trailer canon lives in `meta/CONTRACTS.md`. Emit trailers only via
`git interpret-trailers`; hand-formatted variants are invisible to tooling.

## Delegation Judgment

Delegate on judgment per the shared Roster contract: native subagents by
default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: independent lanes for premise challenge, technical
hotspots, product opportunity, security/privacy, and simplification when the
session's stakes warrant them; the lead keeps final prioritization.

## Ambition Floor

Calibrate scope to what frontier agents can execute, not what a human team
can staff. Execution is cheap; vision is the scarce input. If a strategic
groom lands only small, safe tickets, explain why broad exploration did not
justify a larger move; do not force epics to satisfy process.

- **Brainstorm deep, from perspectives composed for this repo.** There is
  no canonical list of layers to sweep. Pick the obvious axes this codebase
  demands, then add lenses no stock list would hand you — invert a premise,
  borrow from an adjacent domain, ask what a competitor, operator, or
  first-time user would notice. Fan the perspectives out as parallel
  fresh-context lanes; pull in `/research` when outside knowledge would
  change a verdict. The bar is genuine diversity and depth of exploration,
  judged fresh each session.
- **Describe the best version of this software,** not the next safe
  increment: elegant, easy to change, personalizable, delightful. The
  distance between that vision and the live repo is backlog material;
  close it with epics.
- **Epic-scoped by default.** Strategic emissions are epics — a product
  outcome with an ordered child sequence — never pre-shredded tasks. Small
  items exist as children of an epic or as genuine isolated fixes.
- **Ambition is not slop.** Every epic's premise survives the same vetting
  as any finding: open the file, run the command. A perspective that comes
  back with "all fine here" is making a claim — vet it like one. The floor
  raises scope, not tolerance for unevidenced claims.

## Judgment (the actual grooming)

Investigate before opining. A tidy-only pass exists, but only when the user
asks for one; any other session owes a deep brainstorm at the Ambition
Floor's bar, with genuinely independent perspectives run in parallel and
`/research` when outside context would change a verdict. Fresh-context
lanes exist to decorrelate judgment, not to fill a roster. Starter prompts
and scan recipes live in `references/investigation-bench.md` — worked
examples to adapt, not a bench to re-run.

- **Read the live code, not just ticket text.** Hotspots, debt
  concentrations, the oldest stuck ticket. Every codebase has findings;
  "everything is fine" means the investigation was shallow.
- **Challenge premises of the top items.** Symptom or root cause? A ticket's
  framing is a first draft. Reframe before re-ranking.
- **Propose deletions.** The best groom shrinks the backlog. Every deletion
  is a proposal with rationale — humans ratify removals.
- **Audit the repo's own harness.** Agent readiness is backlog work, not a
  separate ceremony: does this repo have a verification skill with its real
  routes/commands (the highest-impact skill category)? Verified build/test/
  lint commands and conventions an agent can discover cold? Runbooks for
  its deployed surfaces? A CI gate that would catch the likely failure?
  Stale AGENTS/CLAUDE prose? Product context a cold agent would need? Each
  gap is a ticket like any other.
- **Vet findings before presenting them.** Re-check each claim against the
  live repo — open the file, run the command. A plausible finding that
  doesn't survive a second look is noise that erodes trust in the whole
  groom.
- **Theme, then recommend.** Group findings by shared root cause, rank by
  impact discounted by confidence — effort barely discounts now that agents
  execute — and argue for one concrete action per theme. Synthesis stays on
  the lead.

## Ticket Standard

`backlog.d/<nnn>-<kebab-slug>.md`, bare numeric IDs.

```markdown
# <Title as imperative sentence>

Priority: P0–P3 · Status: pending|ready|blocked|in-progress|done|shipped|abandoned · Estimate: S–XL

## Goal
<one sentence — outcome, not mechanism>

## Oracle
- [ ] <mechanically verifiable; rough oracles are still oracles>

## Notes
<constraints, prior art, open questions>
```

Epics are the default shape for strategic emissions: same file, plus a
`## Children` section — ordered child outcomes that stay inline until
picked up, then graduate to their own tickets. An epic still needs a Goal
and an Oracle for the whole arc; "umbrella" files with no done criteria are
storage, not epics.

Every active ticket has Goal + Oracle; fix or demote anything that doesn't.
M+ tickets promoted to `Status: ready` follow `/shape`'s
`references/prd-ticket-quality.md`; otherwise they stay raw ideas. When
grooming Harness Kit itself, apply the product lens in
`references/backlog-doctrine.md`.

## Output

1. **Tidy diff** — archived, flipped, flagged; by ID, no padding.
2. **Themes** — recommendation first, evidence second, one at a time.
3. **Emissions** — new epic or ticket / edit / deletion candidate, each
   with a one-line `**Why:**` naming the perspective it came from.
   Strategic emissions default to epic shape; the set should show the
   brainstorm's breadth.
4. **Residual** — open questions, blocked dependencies, next pickups.

Apply non-destructive backlog edits when the user asked for grooming.
Deletions, abandonments, and silent merges stay proposals unless explicitly
approved.

## Audit Mode

`/groom audit` is a read-only harness-health report, not a grooming run:

```sh
cargo run --locked -p harness-kit-checks -- telemetry --repo .
```

It summarizes skill/prompt usage from hook logs (and staleness vs last
edit). Read it with judgment: low usage with high value-when-used is fine —
say so; low usage with no story is a deletion candidate. Present findings
ordered by severity; do not auto-fix.

## Refuse

- Never auto-delete or silently merge tickets.
- Never archive a ticket whose trailer points at an unmerged branch.
- Never let backlog size alone veto an evidenced ticket or epic.

## Gotchas

- **Menu, not grooming.** Themes without a defended recommendation are a
  report. Pick one action per theme and argue it.
- **Mundane harvest.** A strategic session that only finds small, safe work is
  suspicious. Widen the perspective set before concluding the repo simply has
  no ambitious, evidence-backed moves.
- **Stock-lens grooming.** Running the same investigator roster in every
  repo is process, not thought. The revealing perspectives are the ones
  composed for this codebase, this session.
- **Over-decomposing.** An agent-hour of work is one ticket, not three; a
  coherent multi-ticket ambition is one epic, not ten orphan tasks.
- **Backlog as graveyard.** Age is a stale signal, not an automatic verdict.
  Inspect branch, owner, and live relevance before flipping, archiving, or
  proposing deletion.
- **Accepting the ticket's framing.** Five-whys the top items before
  re-ranking them.

## Verification

`/groom audit` (above) scores skill quality; backlog mechanics are enforced
by `harness-kit-checks backlog` subcommands. A groom run ends with a clean
tree: archives committed, emissions written, deletions awaiting ratification.
