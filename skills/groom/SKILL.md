---
name: groom
description: |
  Always-on backlog grooming. Tidy, brainstorm, interrogate, investigate,
  research, and simplify in a single loop. Tidy is not a mode — it happens
  every time. Strategic-layer work fans out parallel interrogation,
  design-critique, technical-review, and research lanes.
  Use when: "groom", "what should we build", "rethink this", "biggest
  opportunity", "backlog", "prioritize", "backlog session",
  "audit skills", "skill quality audit".
  Trigger: /groom, /groom audit, /backlog, /rethink, /moonshot, /scaffold.
argument-hint: "[audit|--emphasis explore|rethink|moonshot|scaffold] [context]"
---

# /groom

Keep `backlog.d/` true: tidy what shipped, challenge what's queued, surface
what's missing, propose what to delete. Every run does all four — tidy is
the price of admission, and a groom that only lists is not a groom.

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
- **Cap:** more than 30 open items is storage, not strategy. Declare a
  reduction session; no new items until under cap.

Trailer canon lives in `skills/ship/SKILL.md`. Emit trailers only via
`git interpret-trailers`; hand-formatted variants are invisible to tooling.

## Delegation Judgment

Delegate on judgment per the shared Roster contract: native subagents by
default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: independent lanes for premise challenge, technical
hotspots, product opportunity, security/privacy, and simplification when the
session's stakes warrant them; the lead keeps final prioritization.

## Judgment (the actual grooming)

Investigate before opining, and let the evidence — not a quota — set the
depth. A routine sweep needs less; a "what should we build" session needs
genuinely independent perspectives run in parallel (premise challenge,
product opportunity, technical debt, security/privacy, simplification),
plus `/research` when outside context would change a verdict. Fresh-context
lanes exist to decorrelate judgment, not to fill a roster. Lane prompts and
scan recipes live in `references/investigation-bench.md`.

- **Read the live code, not just ticket text.** Hotspots, debt
  concentrations, the oldest stuck ticket. Every codebase has findings;
  "everything is fine" means the investigation was shallow.
- **Challenge premises of the top items.** Symptom or root cause? A ticket's
  framing is a first draft. Reframe before re-ranking.
- **Open the aperture.** What should the product become, separately from
  what the next ticket is? What's missing from the backlog entirely?
- **Propose deletions.** The best groom shrinks the backlog. Every deletion
  is a proposal with rationale — humans ratify removals.
- **Theme, then recommend.** Group findings by shared root cause, rank by
  (impact on product vision) × (feasibility) / (effort), and argue for one
  concrete action per theme. Synthesis stays on the lead.

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

Every active ticket has Goal + Oracle; fix or demote anything that doesn't.
M+ tickets promoted to `Status: ready` follow `/shape`'s
`references/prd-ticket-quality.md`; otherwise they stay raw ideas. When
grooming Harness Kit itself, apply the product lens in
`references/backlog-doctrine.md`.

## Output

1. **Tidy diff** — archived, flipped, flagged; by ID, no padding.
2. **Themes** — recommendation first, evidence second, one at a time.
3. **Emissions** — new ticket / edit / deletion candidate, each with a
   one-line `**Why:**` naming the perspective it came from.
4. **Residual** — open questions, blocked dependencies, next pickups.

User ratifies per theme. Silence is not consent.

## Audit Mode

`/groom audit` is a read-only skill-quality report, not a grooming run:

```sh
cargo run --locked -p harness-kit-checks -- audit-skills --repo .
```

Present the report ordered by severity. Do not auto-fix from findings.

## Refuse

- Never auto-delete or silently merge tickets.
- Never archive a ticket whose trailer points at an unmerged branch.
- Never add items past the cap without a reduction session.

## Gotchas

- **Menu, not grooming.** Themes without a defended recommendation are a
  report. Pick one action per theme and argue it.
- **Over-decomposing.** An agent-hour of work is one ticket, not three.
- **Backlog as graveyard.** 30+ days with no progress is dead — archive or
  propose deletion.
- **Accepting the ticket's framing.** Five-whys the top items before
  re-ranking them.

## Verification

`/groom audit` (above) scores skill quality; backlog mechanics are enforced
by `harness-kit-checks backlog` subcommands. A groom run ends with a clean
tree: archives committed, emissions written, deletions awaiting ratification.
