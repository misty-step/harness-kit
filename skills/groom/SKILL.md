---
name: groom
description: |
  Always-on backlog grooming. Tidy, brainstorm, interrogate, investigate,
  research, and simplify in a single loop. Tidy is not a mode — it happens
  every time. Strategic-layer work fans out parallel interrogation,
  design-critique, technical-review, and research lanes.
  Use when: "groom", "what should we build", "rethink this", "biggest
  opportunity", "backlog", "prioritize", "backlog session".
  Trigger: /groom, /backlog, /rethink, /moonshot, /scaffold.
argument-hint: "[--emphasis explore|rethink|moonshot|scaffold] [context]"
---

# /groom

Keep the backlog healthy, organized, tidy, and strategically aligned. One
loop, always-on. You cannot groom a backlog without tidying it.

## Stance

Grooming is the single operation that keeps `backlog.d/` useful. Every
invocation runs the full loop:

1. **Always tidies** — closes shipped tickets, reorders by priority, flags stale items.
2. **Always brainstorms** — opens the aperture on what could be on the backlog that isn't.
3. **Always interrogates** — challenges the premise of top items with ad-hoc
   roster lanes and the technical-review bench.
4. **Always investigates** — dispatches Explore subagents against code hotspots.
5. **Always researches** — delegates to `/research` for outside context on unfamiliar territory.
6. **Always simplifies** — favors deletion and consolidation over addition.

Emphasis flags (`--emphasis explore|rethink|moonshot|scaffold`) weight the
loop toward a direction. They do not turn steps off. There is no `tidy`
subcommand; tidy is the price of admission.

You are the executive orchestrator. Keep synthesis, prioritization, and
decision authority on the lead model. Delegate investigation and evidence
gathering to focused subagents in parallel.

## Delegation Floor

When a provider roster is available (repo `.spellbook/agents.yaml` or system `~/.spellbook/agents.yaml`), `/groom` starts by probing the roster
and dispatching two or more available roster members for investigation before
the lead mutates backlog state or recommends priorities. Use lanes for
backlog drift, premise challenge, technical hotspots, product opportunity,
or external context. The lead agent keeps synthesis and prioritization; do
not delegate final backlog judgment. Direct lead-only grooming is limited to
mechanical ticket moves, emergency unblocks, explicit user-forbidden
delegation, or an explicit waiver when fewer than two roster members are available.

## The Always-On Loop

Phase-gated. Each phase completes before the next begins.

### 1. CONTEXT — load the baseline (<2 minutes)

- Read `project.md` / `CLAUDE.md` / `AGENTS.md` for product lens.
- Read `backlog.d/` — every active ticket, by ID.
- Read `.groom/retro/` if present — effort calibration, blocker patterns.
- Read `.groom/review-scores.ndjson` if present — review-quality trend.
- Read `exemplars.md` if present — existing reference implementations.
- **Cap check:** >30 open items → declare a reduction session. No new items until under cap.
- Ask the user: "Anything on your mind? Bugs, friction, missing features?"

Do not block on missing artifacts. Note absence and proceed.

### 2. TIDY — mandatory janitorial sweep (see Tidy Mechanics)

Gate: every shipped ticket archived, every stale `in-progress` flagged, duplicates called out.

### 3. INVESTIGATE — parallel fanout (see Strategic Layer)

Launch investigation bench, premise-challenge, CEO review, technical-review
bench, codebase hotspot scans, and `/research` delegations **in a single
message** so they run in parallel. A groom run that ran one subagent has
failed the fanout goal.

Gate: every dispatched subagent returned a structured report.

### 4. SYNTHESIZE — theme, rank, recommend

- **Premise audit.** For each top-priority ticket, five-whys the stated
  goal. Symptom or root cause? If symptom, reframe.
- **Cross-reference.** Findings that appeared from 2+ perspectives are
  highest signal.
- **Theme extraction.** Group into 2-4 strategic themes. A theme shares a
  root cause or a shared solution. Not a laundry list.
- **Dependency map.** Which themes unblock which?
- **Rank.** (impact on product vision) × (feasibility) / (effort).

Do NOT delegate synthesis — it requires product judgment.

### 5. SIMPLIFY — propose deletions

Ask each perspective: "what on this backlog should we just delete?" Every
top-3 candidate for deletion gets surfaced to the user with rationale.

Deletions are proposed, never executed silently. See Refuse Conditions.

### 6. EMIT — write diffs, present, ratify

One theme at a time. For each:
- Evidence from investigators.
- Recommended action (one concrete thing; not a list).
- Rough effort (S / M / L / XL).
- New ticket files, edits to existing tickets, or deletion candidates.
- Every emission carries a one-line `**Why:**` justification tying back to a concrete perspective.

User decides per theme: write / edit / delete / skip. Silence is not consent.

## Tidy Mechanics

The always-on tidy step. These steps are MANDATORY every run. Source the
helper lib once at the start:

```sh
source "$(git rev-parse --show-toplevel)/scripts/lib/backlog.sh"
```

### 1. Sweep main for closure trailers

Find every `Closes-backlog:` / `Ships-backlog:` trailer that landed since
the last archive. The merge base depends on the repo's default branch
(`main` or `master`):

```sh
default="$(git symbolic-ref refs/remotes/origin/HEAD 2>/dev/null | sed 's@^refs/remotes/origin/@@')"
default="${default:-main}"
backlog_ids_from_range "origin/${default}..${default}"
```

### 2. Archive every closed ticket still in `backlog.d/`

For each ID returned above:

```sh
backlog_archive "$id"
```

`backlog_archive` is idempotent — re-archiving an already-archived ID
exits 0 silently. Stage the moves and commit:

```
chore(backlog): archive shipped tickets swept by /groom
```

### 3. Sweep by Status field

Some tickets get marked `Status: done` / `Status: shipped` in frontmatter
without a trailer (legacy or hand-edited). Scan `backlog.d/*.md` and move
any such ticket to `_done/` via `backlog_archive` using its numeric ID.

### 4. Flag stale `in-progress`

For each ticket with `Status: in-progress`:
- If the associated branch is deleted or merged, transition Status based
  on commit evidence (`done` if the closing trailer landed, `ready`
  otherwise).
- If no branch evidence exists and the ticket hasn't been touched in 30+
  days, flag for the user — it is probably abandoned.

### 5. Dedupe

If two tickets describe the same work, flag for consolidation. Do not
merge silently — surface the pair with a proposed consolidated shape and
let the user ratify.

## Strategic Layer

The interesting work. Dispatched in parallel for fresh-context judgment.
A groom run that ran fewer than three of these perspectives has failed
the fanout goal.

### Premise challenge — raw items

For the top 3 **unshaped** items (fuzzy goal, missing oracle):
- Dispatch an ad-hoc interrogation lane with the ticket text as context.
- Capture the six forcing-question answers.
- If demand reality is absent, propose demotion to `.groom/BACKLOG.md`.

### Design challenge — shaped packets

For the top 1-2 **shaped** packets (clear goal + oracle + sequence):
- Dispatch an ad-hoc design-critique lane with the ticket text and context
  packet.
- Capture the premise verdict, mandatory alternatives, outside voice.
- If premise is reframed, rewrite the ticket or split into alternatives.

### Technical-review bench — parallel agent fanout

Dispatch in parallel, single message, one subagent per persona:
- `subagent_type: ousterhout` — module depth, information hiding,
  shallow-wrapper smells.
- `subagent_type: carmack` — scope discipline, shippability, YAGNI.
- `subagent_type: grug` — unnecessary complexity, abstraction theater.
- `subagent_type: beck` — TDD gaps, test-quality issues.

Scope each prompt to a concrete slice of recent code (see Codebase
investigation below). Ask each: "what technical-debt ticket does the
current backlog miss?" Surface every emission as a proposed new ticket
with the reviewer's name in the `**Why:**`.

### Research — `/research` on unfamiliar territory

Identify 1-2 themes where the backlog is reaching for domains we lack
depth in (e.g. "SOTA for rate-limiting a multi-tenant queue"). Invoke
`/research` with a focused query. Pipe results into the synthesis step —
use them to pressure-test or to enrich a proposed ticket.

### Codebase investigation — Explore subagents

Dispatch in parallel, single message:

- **Hotspots.** Files changed most in the last 30 days:
  ```sh
  git log --since=30.days.ago --name-only --pretty=format: \
    | sort | uniq -c | sort -rn | head -20
  ```
  Subagent prompt: "what simplification or consolidation opportunity is
  the current backlog missing for these files?"
- **Debt concentration.** Files with the highest TODO/FIXME/HACK count:
  ```sh
  grep -rn -E 'TODO|FIXME|HACK|XXX' --include='*.ts' --include='*.py' \
    --include='*.sh' --include='*.md' . 2>/dev/null \
    | awk -F: '{print $1}' | sort | uniq -c | sort -rn | head -10
  ```
  Subagent prompt: "read the TODOs in these files; which translate to
  tickets the backlog lacks?"
- **Stuck work.** The oldest ticket with `Status: in-progress`. Subagent
  prompt: "read the ticket and the branch commits; what is it actually
  stuck on? What ticket unblocks it?"

Every subagent returns a structured report:

```markdown
## [Subagent Name] Report

### Top 3 Findings
1. [finding] — Evidence: file:line / commit / metric. Impact: high/med/low.
2. ...
3. ...

### Strategic Theme
[One sentence tying findings together.]

### Single Recommendation
[One concrete ticket to add, edit, or delete. Not a list.]
```

### Simplification pass

Ask every perspective already dispatched: "what on this backlog should we
just delete?" Collect candidates. Present top 3 to the user with:
- Ticket ID and title.
- Rationale for deletion (stale, duplicate, low leverage, out of scope).
- Confirmation required before removal.

## Backlog Schema

File naming: `backlog.d/<nnn>-<kebab-slug>.md` (e.g. `029-adaptive-backoff.md`).
IDs are bare numeric strings (`029`, not `BACKLOG-029`).

```markdown
# <Title as imperative sentence>

Priority: P0 | P1 | P2 | P3
Status: pending | ready | blocked | in-progress | done | shipped | abandoned
Estimate: S | M | L | XL

## Goal
<1 sentence — the outcome, not the mechanism.>

## Non-Goals
- <what this ticket will NOT do>

## Oracle
- [ ] <mechanically verifiable criterion — prefer executable commands>
- [ ] <"how will we know this is done?" — rough oracles are still oracles>

## Notes
<constraints, prior art, open questions, linked tickets>
```

Every active ticket MUST have Goal + Oracle. A ticket without an oracle
is not ready — `/groom` either fixes it or demotes it to the icebox.

When grooming spellbook itself, prefer items that create reusable
primitives, scaffolds, references, or policies; validate proving-ground
patterns meant to transfer outward; or remove debt that blocks downstream
adoption. See `references/backlog-doctrine.md` under "Spellbook Product
Lens."

## Trailer Conventions

Closure flows through git trailers, not prose markers. Canonical keys
(recognized by `scripts/lib/backlog.sh`):

- `Closes-backlog: <id>` — closes the ticket (archival intent).
- `Ships-backlog: <id>` — synonym for Closes-backlog.
- `Refs-backlog: <id>` — references without closing.

`/ship` owns trailer injection on the squash merge commit. `/groom`'s
tidy step consumes those trailers to archive. For back-compat only: the
older prose markers `Closes backlog:<id>` / `Ships backlog:<id>` are
tolerated when scanned from old commits, but NEVER emitted by current
tooling.

Full trailer reference lives in `skills/ship/SKILL.md` under "Trailer
Conventions" — do not duplicate it here.

## Interactions

- **Invoked by:** `/flywheel` at the start of each cycle to pick the next
  item. `/flywheel` reads `/groom`'s emitted top-of-backlog and proceeds.
- **Invokes:**
  - Ad-hoc interrogation lane — premise challenge on raw items.
  - Ad-hoc design-critique lane — dialectical audit on shaped packets.
  - `/research` — external context for unfamiliar domains.
  - Technical-review bench agents (`ousterhout`, `carmack`, `grug`, `beck`).
  - Explore subagents — codebase hotspot scans.
- **Consumes:** git trailers from master commits (via `backlog.sh`).
- **Produces:** archived tickets in `backlog.d/_done/`, new tickets in
  `backlog.d/`, edits to existing tickets, proposed deletions.

## Output

The operator sees, in order:

1. **Tidy diff.** Tickets archived, statuses flipped, duplicates flagged.
   Named by ID. No prose padding.
2. **Investigation synthesis.** Themes with evidence, ranked. One theme
   at a time; recommendation first, rationale second.
3. **Emissions.** For each proposed change:
   - New ticket → path + Goal + Oracle + **Why:** one-liner.
   - Edit → ticket ID + diff summary + **Why:** one-liner.
   - Deletion candidate → ticket ID + rationale, awaiting ratification.
4. **Residual.** Open questions, blocked dependencies, next-session
   pickups.

Terse. No marketing voice. The backlog diff is the artifact; the prose
exists to justify it.

## Refuse Conditions

Stop and surface to the user instead of proceeding:

- **Never auto-delete items.** Every deletion requires explicit
  ratification. Silent deletion is data loss.
- **Never silently merge duplicates.** Propose the consolidated ticket
  and let the user approve.
- **Never archive a ticket whose trailer points to an unmerged branch.**
  The trailer exists but the work isn't on main yet — flag it.
- **Never proceed past the cap without a reduction session.** >30 open
  items means the backlog is storage, not strategy. Reduce before adding.
- **Never skip fanout.** A groom run that dispatched fewer than three
  strategic-layer perspectives has not groomed — it has listed.
- **Never accept "everything is fine" from an investigator.** Every
  codebase has findings. Push harder.

## Gotchas

- **Accepting the ticket's framing as given.** A `/groom X` request is a
  first-draft articulation, not a locked problem. Five-whys before
  theming.
- **Synthesis without themes.** That's a report, not synthesis. Group
  before presenting.
- **Themes without recommendations.** That's a menu, not grooming. Pick
  one action per theme and argue for it.
- **Items without oracles.** If you can't write a checkable done, the
  item isn't scoped. Fix or demote.
- **Over-decomposing.** An agent-hour of work is one item, not three.
- **Backlog as graveyard.** Items >30 days old with no progress are
  dead. Archive or propose deletion during the tidy sweep.
- **Trailer drift.** Hand-formatted `closes-backlog: 29` (wrong case /
  wrong key / trailing whitespace) is invisible to `backlog.sh`. Always
  emit via `git interpret-trailers --trailer`, never by hand.

## Principles

- **Tidy is always.** No subcommand, no flag, no skip.
- **Investigate before opining.** Parallel fanout first, opinions after evidence.
- **Theme, don't itemize.** Strategic themes beat feature laundry lists.
- **Recommend, don't list.** Opinion with a defense beats a menu.
- **One theme at a time.** Don't overwhelm during discussion.
- **Product vision is the ranking function.** User impact beats technical elegance.
- **Every item needs an oracle.** Unverifiable done is not ready.
- **File-driven.** `backlog.d/` is the source of truth.
- **Deletion is a proposal, not an action.** Humans ratify removals.
