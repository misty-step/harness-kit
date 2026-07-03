---
name: groom
description: |
  Always-on backlog grooming. Tidy, brainstorm, interrogate, investigate,
  research, and simplify in a single loop. Tidy is not a mode — it happens
  every time. Strategic-layer work is a mega-sweep: swarm investigation,
  external research, critique, synthesis, and backlog shaping across product,
  codebase, docs, infrastructure, ops, architecture, system design, value prop,
  and agent readiness.
  Use when: "groom", "what should we build", "rethink this", "biggest
  opportunity", "backlog", "prioritize", "backlog session",
  "audit skills", "skill quality audit".
  Trigger: /groom, /groom audit, /backlog, /rethink, /moonshot, /scaffold.
argument-hint: "[audit|--emphasis explore|rethink|moonshot|scaffold] [context]"
---

# /groom

Keep `backlog.d/` true and make the project more ambitious. Normal strategic
`/groom` is a mega-sweep, not triage: boil the ocean on purpose, then distill
it into a world-class plan and backlog diff. A groom that only lists, lightly
prioritizes, or emits a tiny handful of issues is incomplete unless the user
explicitly scoped the run down.

The backlog diff is the artifact. Prose exists to justify it.
Groom is a conversation at the seams where the operator's judgment is the scarce
input. At intake, interrogate the operator on ambition, vision deltas, and
hidden priors so the sweep aims true — the `grill-me`-style posture in
`harnesses/shared/references/interrogate-first.md` is the default here, not a
fallback gated on whether they arrived with a backlog item. The investigation
sweep itself stays autonomous — explore the codebase for what it can answer;
reserve interrogation for the operator's judgment, not for facts you can read. If the groom produces a giant synthesis, use `julius-caveman`
compression for interim summaries, not for the committed backlog text.

## Project Vision

Every strategic groom creates, refreshes, or explicitly validates a durable
project vision before ranking work. Vision is the north star that makes
brainstorming and backlog movement coherent; without it, grooming collapses
into local cleanup.

- If a project already has a canonical vision or strategy artifact, read it
  early and use it to judge every theme, deletion, emission, and next pickup.
- If none exists, draft the smallest durable vision artifact the repo can own.
  Prefer root `VISION.md` as a first-class repository artifact; use
  `docs/product.md`, a roadmap doc, or a clearly named backlog epic only when
  repo evidence says root vision is the wrong fit. Treat creating it as a
  first-class groom emission.
- Use `/vision` when creating or materially revising root `VISION.md`; groom
  consumes the north star instead of hand-rolling a parallel strategy doc.
- Keep the vision concrete: audience, job-to-be-done, category, standards,
  non-goals, strategic bets, and what excellent looks like in 6-12 months.
- Revise the vision when live evidence contradicts it. Do not bury direction
  changes in chat, issue comments, or the final report only.
- Use the vision to backchain: themes and tickets should say which part of the
  vision they advance, de-risk, simplify, or deliberately reject.
- A vision or moonshot artifact can carry a one-glance **system map** when the
  relationships, scope, or sequencing are too complex for prose alone. Generate
  it only when it earns that role: `harnesses/shared/references/image-generation.md`.

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

Local lane guidance: strategic grooming uses swarms by default. Launch
independent lanes for product/value, operator experience, runtime reliability,
architecture, simplification/deletion, security/privacy, docs/onboarding,
ops/infra, testing/verification, agent readiness, and external exemplars.
The lead keeps final prioritization.

## Mega Sweep

For any strategic groom, load `references/mega-groom.md`. It is the contract
for swarm size, coverage map, source matrix, output shape, and the
"world-class plan" bar. Use `references/investigation-bench.md` only for lane
prompt shapes; it is a template library, not the whole run.

## Ambition Floor

Calibrate scope to what frontier agents can execute, not what a human team
can staff. Execution is cheap; vision is the scarce input. Strategic grooming
must describe the best version of the whole project, then backchain from that
standard into epics, deletions, sequencing, and first pickups.

- **Brainstorm deep, from perspectives composed for this repo.** There is
  no canonical list of layers to sweep. Pick the obvious axes this codebase
  demands, then add lenses no stock list would hand you — invert a premise,
  borrow from an adjacent domain, ask what a competitor, operator, or
  first-time user would notice. Fan the perspectives out as parallel
  fresh-context lanes; pull in `/research` when outside knowledge would
  change a verdict. When the sweep keeps returning the obvious (the *mundane
  harvest* failure), route the divergence through `nous-creative-ideation` — a
  routed library of named methods (lateral provocations, analogy/biomimicry,
  premortem-and-inversion) that manufactures non-obvious lenses on purpose. The
  bar is genuine diversity and depth of exploration, judged fresh each session.
- **Describe the best version of this software,** not the next safe
  increment: elegant, easy to change, personalizable, delightful,
  operationally boring, and valuable enough to matter. The distance between
  that vision and the live repo is backlog material; close it with epics.
- **Epic-scoped by default.** Strategic emissions are epics — a product
  outcome with an ordered child sequence — never pre-shredded tasks. Small
  items exist as children of an epic or as genuine isolated fixes.
- **Ambition is not slop.** Every epic's premise survives the same vetting
  as any finding: open the file, run the command. A perspective that comes
  back with "all fine here" is making a claim — vet it like one. The floor
  raises scope, not tolerance for unevidenced claims.

## Judgment (the actual grooming)

Investigate before opining. A tidy-only pass exists, but only when the user
asks for one; any other session owes the `references/mega-groom.md` sweep,
with genuinely independent perspectives run in parallel and `/research` when
outside context would change a verdict. Fresh-context lanes exist to
decorrelate judgment, not to fill a roster.

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
  A meaningful, enforced quality floor that gates the diff and ratchets legacy
  debt, or only advisory, arbitrary gates
  (`harnesses/shared/references/quality-gates.md`)? Security gates that catch
  secret leaks in files and Git/PR metadata before publication? Stale
  AGENTS/CLAUDE prose? Product context a cold agent would need? Each gap is a
  ticket like any other.
- **Vet findings before presenting them.** Re-check each claim against the
  live repo — open the file, run the command. A plausible finding that
  doesn't survive a second look is noise that erodes trust in the whole
  groom.
- **Theme, then recommend.** Group findings by shared root cause, rank by
  impact discounted by confidence — effort barely discounts now that agents
  execute — and argue for one concrete action per theme. Synthesis stays on
  the lead; when the plan is contestable, land it with the operator the same
  `grill-me` way — walk the decision tree (sequencing, deletions, the next
  pickup) one branch at a time, recommending each — instead of dropping the
  full plan for a rubber-stamp.

## Ticket Standard

`backlog.d/<nnn>-<kebab-slug>.md`, bare numeric IDs.

```markdown
# <Title as imperative sentence>

Priority: P0–P3 · Status: pending|ready|blocked|in-progress|done|shipped|abandoned · Estimate: S–XL

## Goal
<one sentence — outcome, not mechanism>

## Oracle
- [ ] <mechanically verifiable; rough oracles are still oracles>

## Verification System
- Claim:
- Falsifier:
- Driver:
- Grader:
- Evidence packet:
- Cadence:

## Notes
<constraints, prior art, open questions>
```

Epics are the default shape for strategic emissions: same file, plus a
`## Children` section — ordered child outcomes that stay inline until
picked up, then graduate to their own tickets. An epic still needs a Goal
and an Oracle for the whole arc; "umbrella" files with no done criteria are
storage, not epics.

Every active ticket has Goal + Oracle; fix or demote anything that doesn't.
For M+ work or anything promoted to `Status: ready`, add the verification
system from `harnesses/shared/references/verification-system-first.md`.
M+ tickets promoted to `Status: ready` follow `/shape`'s
`references/prd-ticket-quality.md`; otherwise they stay raw ideas. When
grooming Harness Kit itself, apply the product lens in
`references/backlog-doctrine.md`.

## Output

1. **Tidy diff** — archived, flipped, flagged; by ID, no padding.
2. **Source matrix** — swarm lanes, local commands, external research,
   skipped/failed lanes, and what each contributed.
3. **World-class plan** — project vision, gaps, themes, sequencing,
   deletion/consolidation candidates, and the one best next pickup.
4. **Emissions** — epic/ticket edits with `**Why:**` naming the evidence lane.
   Strategic emissions should show breadth across the domain map, not just
   the easiest implementation slice.
5. **Residual** — open questions, blocked dependencies, unverified areas,
   and what would make the sweep stronger.

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
- Never call a strategic groom complete after a tiny issue set unless the user
  explicitly asked for a narrow pass.
- Never skip the swarm when subagent, peer CLI, or sprite lanes are available;
  if all delegation is blocked, report degraded mode and do the local matrix.

## Gotchas

- **Menu, not grooming.** Themes without a defended recommendation are a
  report. Pick one action per theme and argue it.
- **Mundane harvest.** A strategic session that only finds small, safe work is
  not done. Widen the swarm, add external research, invert the premise, and
  inspect neglected surfaces before concluding the repo has no ambitious,
  evidence-backed moves.
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
