# `/flywheel` — outer delivery loop (formerly `/iterate`)

Priority: P0 (shipped)
Status: done
Estimate: L
Shipped: 2026-04-15 (Phase 1 + rename); 2026-04-16 (thin-harness pivot)

## What shipped

`/flywheel` exists at `skills/flywheel/SKILL.md` (42 lines) — the
"thin-harness reference form" cited in `.harness-kit/repo-brief.md`. It
composes `/deliver` → land → `/deploy` → `/monitor` → `/reflect cycle`
and loops.

## What was cut (deliberate)

The Phase 2+ scaffold originally shaped below (`skills/flywheel/scripts/
flywheel.sh` helper lib, ULID cycle IDs, NDJSON event streams, lock file
coordination, budget/predicate flag machinery, GEPA feedback hooks) was
deleted in commit `7ccd00d refactor(flywheel): thin-harness — scripts
out, judgment in` (977 + 738 LOC → 42-line SKILL.md).

The pivot aligns with the doctrine in `harnesses/shared/AGENTS.md`:
*"Prefer thin harnesses over semantic orchestration. Launch, bound, and
record agents; do not pre-solve their work in harness code."* The model
runs the cycle; the skill encodes the invariants that aren't inferable
from the leaf names.

GEPA-style auto-tune lives on as a separate ticket (`031-harness-auto-
tune-gepa.md`, parked until ≥20 cycles of signal exist).

## Original shape preserved below for reference

## Progress

- 2026-04-15: Phase 1 MVP shipped as `/iterate` (commit e2db08e).
- 2026-04-15: Rename `/iterate` → `/flywheel` shipped (commit 5d5358a).
  Directory, scripts, lock file, helper libs, triggers, and all cross-refs
  renamed. 48/48 tests green.
- Remaining: Phase 2+ (deploy/monitor/triage/reflect wiring, budgets,
  predicate stop conditions, GEPA feedback hooks). TODOs flagged in
  `skills/flywheel/SKILL.md` and `skills/flywheel/scripts/flywheel.sh`.


## Rename

Formerly `/iterate`. The outer loop is the *actually* autonomous skill —
multi-cycle, unattended, budgeted, cross-cycle learning. The name
`/flywheel` belongs to this, not to the inner single-ticket pipeline.

- Old `/flywheel` (single-shot ticket delivery) → renamed to `/deliver` (see 032)
- Old `/iterate` (this skill) → renamed to `/flywheel`
- Naming swap tracked in 032. This ticket refers to the new meaning throughout.

## Goal

Close the delivery loop. `/deliver` ships one item to merge-ready code and
exits. `/flywheel` picks items, delivers them, deploys, monitors, triages,
reflects, updates the backlog + harness, and picks the next. It composes
existing skills as phase handlers — it does not reimplement phases.

OpenHands inner-loop vs outer-loop distinction is load-bearing. `/deliver`
is inner (single-shot, interactive). `/flywheel` is outer (continuous,
unattended).

## Why Not Grow `/deliver`

Conflating single-shot delivery with continuous operation forces `/deliver`
to grow deploy + monitor + retro + bucket-rewrite + budget logic it
shouldn't own. Two skills, two clear stop conditions, one composition
contract.

## Composition Contract

```
/flywheel [--max-cycles N] [--budget $X] [--until <pred>] [--unattended]
    │
    ▼
  acquire <worktree-root>/.harness-kit/flywheel.lock
    │
    ▼
┌── CYCLE START ───────────────────────────────┐
│  1. pick        → deterministic selector     │  cycle.opened
│  2. deliver     → /deliver (full inner loop) │  deliver.done (≡ merge-ready)
│  3. deploy      → /deploy                    │  deploy.done
│  4. monitor     → /monitor                   │  monitor.done | monitor.alert
│  5. triage      → /diagnose (if alert)    │  triage.done
│  6. reflect     → /reflect (session+harness) │  reflect.done
│  7. update-bucket → backlog mutation         │  bucket.updated
│  8. update-harness → branch-only suggestion  │  harness.suggested
└── CYCLE CLOSED ──────────────────────────────┘
    │
    ▼
  stop? (predicate / max-cycles / budget / SIGINT) → next cycle or exit
```

`/deliver` itself loops shape → implement → code-review → ci → refactor
→ qa → evidence internally (see 032). `/flywheel` treats `/deliver` as a
black-box merge-readiness step and consumes its receipt (see `/deliver`
Exit Contract below).

### `pick` — eligible-item selection

Source: `backlog.d/*.md` only. `git-bug` is out of scope for MVP
(no cross-store reconciliation). `_done/` is excluded by glob.

Eligibility filter (applied in order):
  1. Path matches `backlog.d/[0-9][0-9][0-9]-*.md` (no `_done/`, no `_cycles/`)
  2. `Status:` field is NOT `done`, `shipped`, `abandoned`, `blocked`
  3. Not referenced as `item_id` in any OPEN manifest under
     `backlog.d/_cycles/*/manifest.json` (`status != closed`)
  4. No `Blocked-by:` line whose target is still eligible-or-open

Scoring (deterministic, no LLM in hot path):
```
score = 1000*priority_rank + 100*readiness + age_days
  priority_rank: P0=4, P1=3, P2=2, P3=1, unset=0
  readiness:    has `## Oracle` section=2, has `Estimate:`=1, else=0
```
Highest score wins; ties break by lowest filename (NNN prefix).

Resume interaction: if `--resume <ulid>` is passed, `pick` is skipped
entirely — the ulid's manifest supplies `item_id`. An in-progress manifest
without `--resume` is NOT auto-picked; the operator must explicitly resume
or abandon it.

Empty-eligible-set behavior: emit `cycle.opened` with payload
`{"item_id": null, "reason": "backlog_empty"}` immediately followed by
`cycle.closed` with `status: "noop"`, release lock, exit 0. This is
load-bearing — `--until "backlog empty"` checks for this exact closed-reason.

### `update-bucket` — post-cycle backlog mutation

Runs after `reflect.done`, before `update-harness`. All mutations are keyed
by `cycle_id` so re-running on crash is idempotent (see Durability Guarantees).

Mutations on success (`status: shipped`):
  1. Move `backlog.d/NNN-<slug>.md` → `backlog.d/_done/NNN-<slug>.md`
  2. Append to the moved file:
     ```
     ## What Was Built (cycle <ulid>)
     - Branch: <branch>
     - Commits: <sha>..<sha>
     - Evidence: backlog.d/_cycles/<ulid>/evidence/
     - Reflections: <one-paragraph summary from /reflect>
     ```
  3. If `/reflect` emitted follow-up items (payload
     `reflect.done.new_items: [{title, priority, rationale}, ...]`), write
     each as `backlog.d/NNN-<slug>.md` with the next free NNN and a
     `Parent-cycle: <ulid>` header line.

Mutations on failure (`status: failed|abandoned`):
  1. Source file stays in place (not moved to `_done/`)
  2. Append a `## Cycle <ulid> Attempt (<status>)` section with last phase,
     outcome, and evidence pointer.
  3. If priority was P0/P1, bump `Retry-count:` header (cap at 3;
     auto-demote priority one step at cap with `Auto-demoted: true`).

Idempotence rule: every mutation is guarded by `grep -q "cycle <ulid>"` on
the target file. Re-running is a no-op if the stamped ulid is already
present. The `bucket.updated` event payload carries the list of touched paths.

### `update-harness` — suggestion-only harness delta

`/reflect` may surface harness-level findings. `/flywheel` NEVER mutates
the harness in-place. Every finding lands on the `harness/auto-tune`
branch as a suggestion for human review.

Event payload (`harness.suggested`):
```json
{
  "suggestion_id": "<ulid>",
  "kind": "skill_edit | hook_edit | claude_md_edit | new_skill",
  "target_path": "skills/foo/SKILL.md",
  "rationale": "<free-text, cycle-linked>",
  "evidence_refs": ["backlog.d/_cycles/<ulid>/..."],
  "patch_ref": "<git sha on harness/auto-tune>"
}
```

Body format: BOTH a unified diff (applied as a commit on the branch) AND a
natural-language ADR in the commit message body (`Why:`, `Alternatives:`,
`Rollback:`). Diff is machine-readable; ADR is auditable.

Branch mechanics:
  1. If `harness/auto-tune` doesn't exist, create from default branch HEAD
     (NOT cycle's feature branch — keeps harness drift decoupled).
  2. If it exists, rebase onto default branch before appending. Conflict →
     emit `phase.failed` with `{"reason": "harness_branch_conflict"}`; do
     not force-push.
  3. Each suggestion is one commit: `harness: <kind> — <target> (cycle <ulid>)`.
  4. Push to `origin` only if `FLYWHEEL_PUSH_HARNESS=1`; default off.

Never-auto-merge: `harness/auto-tune` is marked with a CODEOWNERS entry
requiring human review. The spec forbids any `/flywheel` code path from
opening, approving, or merging a PR from this branch.

## State Model

One cycle = one bucket item worked end-to-end. Each cycle gets a ULID:

```
backlog.d/_cycles/<ulid>/
├── cycle.jsonl        # typed event log (see events.sh)
├── evidence/          # QA artifacts, review transcripts, deliver receipt
│   └── deliver/       # /deliver state dir when invoked by /flywheel
└── manifest.json      # {item_id, branch, started, closed, status, budget}
```

### Event Schema

Typed envelope in `scripts/lib/events.sh`. Closed enum of kinds; writes
with unknown kinds fail. JSONL corruption breaks `/reflect`, so writes are
`flock`'d and `fsync`'d.

Kinds: `cycle.opened`, `deliver.done`, `deploy.done`, `monitor.done`,
`monitor.alert`, `triage.done`, `reflect.done`, `bucket.updated`,
`harness.suggested`, `phase.failed`, `budget.exhausted`, `cycle.closed`.

**Note:** the per-phase kinds from the old iterate spec (`shape.done`,
`build.done`, `review.iter`, `ci.done`, `qa.done`) move inside `/deliver`
and are no longer emitted at the `/flywheel` level — `/flywheel` sees
one `deliver.done` event. Drops cross-cycle noise.

### Stopping Predicates

- `--until "backlog empty"` — no eligible items (see `pick`)
- `--until "P0 closed"` — highest-priority item shipped this run
- `--max-cycles N` — hard count of completed cycles (ulid count)
- `--budget <dollars>` — cumulative USD model spend, **primary** accounting

Budget accounting — primary: dollars, secondary: wall-clock:
```json
"budget": {
  "cap_usd": 20.00,
  "spent_usd": 3.42,
  "spent_by_phase": {"deliver": 2.80, "monitor": 0.12},
  "wall_seconds": 742,
  "wall_cap_seconds": null
}
```

USD is summed per phase from the billing signal returned by each subagent
invocation. Wall-clock is always recorded; optionally capped via
`--wall-budget <seconds>`. Wall-clock is the failsafe when token telemetry
misfires (hung subprocess, tool loop).

Check point: after EACH phase (not mid-phase), compare
`spent_usd >= cap_usd * 0.95`. If true, finish the current cycle's current
phase, then emit `budget.exhausted` and stop. The 5% headroom absorbs
metering lag.

Interactive vs unattended:
- Interactive (stdin is a TTY): `--budget` optional. Default cap $5 USD
  with prompt on breach ("continue? y/N"); operator can raise.
- Unattended (no TTY, or `--unattended` flag): `--budget` **required**.
  Exits 2 with "flywheel: unattended mode requires --budget <usd>".

### Resume & Abandon Semantics

`--resume <ulid>` continues an interrupted cycle. `--abandon <ulid>` marks
it closed with `status: abandoned` and releases locks.

Phase resolution (resume):
  1. Read `backlog.d/_cycles/<ulid>/cycle.jsonl` line-by-line; drop any
     trailing line that fails JSON-parse (crash-truncation).
  2. The LAST valid event's `kind` maps to the last completed phase:

| Last event         | Next phase        |
|--------------------|-------------------|
| `cycle.opened`     | `deliver`         |
| `deliver.done`     | `deploy`          |
| `deploy.done`      | `monitor`         |
| `monitor.done`     | `reflect`         |
| `monitor.alert`    | `triage`          |
| `triage.done`      | `reflect`         |
| `reflect.done`     | `update-bucket`   |
| `bucket.updated`   | `update-harness`  |
| `harness.suggested`| `cycle.closed`    |
| `phase.failed`     | refuse; require `--abandon` |
| `cycle.closed`     | refuse; already done |
| `budget.exhausted` | refuse unless new `--budget` supplied |

  3. `manifest.json` is re-read for `item_id`, `branch`, budget state.
     `pick` is skipped.

**Re-runnable-phase rule (non-negotiable):** every phase handler is
re-runnable from its phase-start with no duplicate side-effects.
- `deliver`: resumes via `/deliver --resume` (032 owns that contract).
  `/flywheel` re-invokes with same branch; `/deliver` no-ops already-done
  sub-phases.
- `deploy`: idempotent deploy targets (035); duplicate-deploy detection
  lives in `/deploy`.
- `monitor`: OBSERVATIONAL — re-runs from scratch. No preserved partial
  observations across crashes.
- `triage`, `reflect`: pure reads over event log + diff; harmless to re-run.
- `update-bucket`, `update-harness`: ulid-stamped idempotence (sections above).

Crash-in-middle-of-phase policy: the phase is the atomic unit. On resume,
the phase with no `*.done` event is re-run from its start. A handler that
is not safe to re-run is a spec violation.

`--abandon <ulid>` writes `cycle.closed` with `{"status": "abandoned",
"reason": "<operator|budget|phase-fail>"}`, releases lock, does NOT move
the backlog item to `_done/`, stamps the source file with a
`## Cycle <ulid> Attempt (abandoned)` section.

## Durability Guarantees

Interruption is the common case. SIGINT, crashes, context resets, lid close,
OOM kill — the loop must be boringly resumable, not heroic.

Invariants:
- **D1.** Cycle directory is created BEFORE any phase runs and BEFORE
  `cycle.opened` is emitted. A cycle without a dir is invalid — resume refuses.
- **D2.** Every event is written via `emit_event`: `O_APPEND` +
  `flock(LOCK_EX)` + `fsync(fd)` before fd close. Partial lines are never
  visible to same-flock readers.
- **D3.** `manifest.json` is rewritten atomically (temp + rename) at two
  moments: cycle open, and after every `*.done` event. Always reflects
  state at-or-before the last event.
- **D4.** Lock file (`.harness-kit/flywheel.lock`) records
  `{pid, cycle_id, started_at}`. Stale-pid steal via `O_CREAT|O_EXCL`
  already implemented in Phase 1 (`scripts/lib/flywheel_lock.sh`).
- **D5.** Worst-case loss on SIGKILL: the phase in progress is re-run from
  its start on `--resume`. No in-flight events silently dropped.

Non-guarantees:
- Subagent-internal state is the subagent's problem. `/flywheel` only
  guarantees phase-boundary durability.
- Remote state (pushed commits, opened PRs) is best-effort — underlying
  skills own their own rollback.

## Worktree Behavior

`/flywheel` state is worktree-local, not machine-global. Two worktrees of
the same repo can run `/flywheel` concurrently without interference.

Path resolution: all state paths anchor to `git rev-parse --show-toplevel`
(the worktree root for linked worktrees, main checkout otherwise). NOT
`$HOME`, NOT `git rev-parse --git-common-dir`.

Worktree-local paths:
```
<worktree-root>/.harness-kit/flywheel.lock
<worktree-root>/backlog.d/_cycles/<ulid>/...
<worktree-root>/.harness-kit/flywheel-state.json
```

Rationale: git worktrees share `.git/` but have independent working trees.
Two `/flywheel` instances writing the same `backlog.d/_cycles/` would
corrupt each other's event logs. Anchoring to the worktree root gives each
a private state space.

Non-goals (per design principle: no claim coordination):
- No cross-worktree coordination. If worktree-A and worktree-B both pick
  item 028, they race on branch names; human merge resolves it. Out of
  scope for MVP.
- No global "active ulid" registry.

Backlog file conflicts: `backlog.d/*.md` is shared across worktrees (under
VCS). `update-bucket` relies on git to surface conflicts at merge time —
operator resolves with normal git tooling.

## Components

| Component | Status | Owns |
|---|---|---|
| `skills/flywheel/SKILL.md` | rename from iterate | Orchestration, event writing, lock, budget, stop predicates |
| `scripts/lib/events.sh` | ✓ shipped (was daybook.sh) | `emit_event` — atomic JSONL append with fsync |
| `scripts/lib/flywheel_lock.sh` | ✓ shipped (rename from iterate_lock.sh) | Single-instance per-worktree lock |
| `/deliver` | 032 — rename flywheel + compose | Full inner pipeline to merge-ready |
| `/deploy` | 035 — new | Ship to environment |
| `/monitor` | 036 — new | Post-deploy signal watch + escalate |
| `/diagnose` | ✓ exists | Triage on monitor.alert |
| `/reflect` | 037 — upgrade | Session + bucket + harness critique |

## Failure Modes

| Failure | Recovery |
|---|---|
| Phase handler fails | `phase.failed` event; stop cycle; lock released; cycle closable via `--resume` or `--abandon` |
| Monitor flags anomaly | `monitor.alert` → triage → remediation or `phase.failed` |
| Budget exceeded mid-cycle | Finish current phase, `budget.exhausted`, stop |
| Event log write fails | Fatal — fsync every event; corrupted JSONL breaks reflect |
| Two `/flywheel` attempts in same worktree | Second exits on lock |
| `/deliver` internal fail | See Exit Contract below |

### `/deliver` Exit Contract

`/deliver` returns via exit code AND a receipt file. Both are load-bearing;
`/flywheel` consults both and treats disagreement as a bug.

Exit codes:
```
0    — merge-ready (receipt.status == "merge_ready")
10   — phase handler hard-failed
20   — clean loop exhausted
30   — SIGINT abort
40/41— refused (bad arg, missing dep, double-invoke); may skip receipt
130  — SIGKILL/SIGTERM; partial receipt may exist
```

Receipt path (when called by `/flywheel`):
`backlog.d/_cycles/<ulid>/evidence/deliver/receipt.json`

`/flywheel` invokes `/deliver --state-dir backlog.d/_cycles/<ulid>/evidence/deliver/`
so `/deliver`'s state lands under the cycle's evidence tree directly. See
032 for the receipt schema.

`/flywheel` behavior on each status:
- `merge_ready` → emit `deliver.done`, proceed to `deploy`
- `clean_loop_exhausted` / `failed` / `partial` → emit `phase.failed`
  with inline receipt; stop cycle
- Missing receipt on exit 0 → `phase.failed` with
  `{"reason": "deliver_missing_receipt"}`; stop. No auto-retry.

## Phase Plan

**Phase 1 — shipped on `feat/iterate-mvp-phase1` (current name `/iterate`):**
- Dry-run walk of all phases
- Typed event log (`scripts/lib/events.sh`, formerly `daybook.sh`)
- Single-instance lock with stale-pid steal (`scripts/lib/iterate_lock.sh`)
- Single-cycle guard (`--max-cycles > 1` exits 2)
- 27 regression tests

**Phase 2 — rename + real handlers (~3-4 dev-days):**

### SKILL.md Shape (harness lint compliance)

Eight phases × detailed semantics will trip the harness `>4 modes with
inline content` lint. Pre-empt by extracting:

```
skills/flywheel/
├── SKILL.md                       # ≤300 lines: routing table + cross-phase invariants
└── references/
    ├── phase-pick.md              # eligibility filter, scoring formula
    ├── phase-deliver.md           # /deliver invocation, --state-dir, receipt consumption
    ├── phase-deploy.md            # /deploy contract
    ├── phase-monitor.md           # /monitor + alert handoff to /diagnose
    ├── phase-reflect.md           # /reflect invocation + new-item payload
    ├── phase-update-bucket.md     # mutation grammar, idempotence rules
    ├── phase-update-harness.md    # branch mechanics, suggestion format
    ├── events.md                  # full event schema + JSON examples
    ├── budget.md                  # USD accounting, interactive vs unattended
    ├── resume.md                  # phase-resolution table, re-runnable rule
    └── durability.md              # D1–D5 invariants, worktree behavior
```

SKILL.md content (what stays inline):
- Composition contract diagram
- Routing table (phase → reference)
- Cross-cutting invariants the model must hold across phases (no claims,
  worktree-local state, never-auto-merge, fail-loud)
- Top-level gotchas (≤10 bullets) — judgment, not procedure

What stays out of SKILL.md: state-machine tables, JSON schemas, exit-code
tables, event schemas, rename history. These are reference material.

Token budget: SKILL.md ≤3K tokens (target), ≤5K (ceiling).

### Harness Enforcement (lands with the rename)

Structural prevention beats prose. Add alongside the rename commit:

- [ ] `.gitignore` includes `.harness-kit/` (state dir must never be committed)
- [ ] `.gitignore` includes `backlog.d/_cycles/*/evidence/deliver/`
      (gitignored — cycles are committed, but `/deliver` state inside is not)
- [ ] CODEOWNERS entry: `harness/auto-tune` requires human review before merge
- [ ] Pre-push hook: reject pushes from a branch named `harness/auto-tune`
      unless `FLYWHEEL_PUSH_HARNESS=1` is set in env
- [ ] Pre-commit hook: reject manual edits to `backlog.d/_cycles/**/*.jsonl`
      and `manifest.json` (machine-written; human edits = corruption)
- [ ] Lint rule (added to harness/lint or skills/harness eval): no skill
      other than `/flywheel` may write to `harness/auto-tune` branch —
      grep-based check on `harness/auto-tune` string in skill files

These are the burner-stove redesigns. Without them, the never-auto-merge
and worktree-local rules are just CLAUDE.md prose.

### Rename mechanics (grep-verified, exhaustive)

Files to move:
- `skills/iterate/` → `skills/flywheel/`
- `skills/iterate/scripts/iterate.sh` → `skills/flywheel/scripts/flywheel.sh`
- `skills/iterate/scripts/iterate_test.sh` → `skills/flywheel/scripts/flywheel_test.sh`
- `scripts/lib/iterate_lock.sh` → `scripts/lib/flywheel_lock.sh`
- `scripts/lib/iterate_lock_test.sh` → `scripts/lib/flywheel_lock_test.sh`

Symbols to rename:
- `iterate_acquire()` → `flywheel_acquire()`
- `iterate_release()` → `flywheel_release()`
- `ITERATE_LOCK_PATH` → `FLYWHEEL_LOCK_PATH`
- `ITERATE_LOCK_FILE` → `AUTOPILOT_LOCK_FILE`
- `ITERATE_LOCK_CYCLE_ID` → `AUTOPILOT_LOCK_CYCLE_ID`
- `ITERATE_LOCK_PID` → `AUTOPILOT_LOCK_PID`

Path strings:
- `.harness-kit/iterate.lock` → `.harness-kit/flywheel.lock`
- `backlog.d/_cycles/` unchanged — keep historical cycles readable

Env var: `AUTOPILOT_MODE=1` (currently no `ITERATE_MODE` in code —
introduce during Phase 2 if/when needed).

External references:
- Rename `backlog.d/028-flywheel-outer-loop-orchestrator.md` → `028-flywheel-outer-loop-orchestrator.md`
- Update `backlog.d/031-harness-auto-tune-gepa.md`, `037-reflect-upgrade.md`
- `skills/flywheel/SKILL.md` header, trigger, argument-hint
- `CLAUDE.md` and `AGENTS.md` if any trigger mention (grep first)

Composition work:
1. Rename mechanics (above) — mechanical pass, own commit
2. Drop inner-phase event kinds from `EVENT_KINDS`: remove `shape.done`,
   `build.done`, `review.iter`, `ci.done`, `qa.done`. Add `deliver.done`,
   `monitor.done`, `monitor.alert`, `triage.done`, `bucket.updated`.
   Update tests.
3. Wire real handlers: `/deliver` (032), `/deploy` (035), `/monitor` (036),
   `/diagnose` (existing), `/reflect` (037).
4. Multi-cycle control flow with stop predicates (`--until`, `--budget`,
   `--max-cycles`).
5. Budget tracking in `manifest.json`.
6. `pick`, `update-bucket`, `update-harness` implementations.

**Phase 3 — unattended ops (~2 dev-days):**
- `--resume <ulid>` / `--abandon <ulid>`
- `harness.suggested` → `harness/auto-tune` branch
- Worktree-local path anchoring (`git rev-parse --show-toplevel`)
- Durability oracle items (SIGKILL mid-phase, two-worktree concurrency)
- Harness Kit dogfoods on itself

## Oracle

Functional:
- [ ] `/flywheel --max-cycles 1` runs pick → deliver → deploy → monitor →
      reflect → update-bucket → update-harness on a real backlog item
- [ ] Cycle event log contains ≥6 typed events, all valid against
      closed-enum schema
- [ ] `/flywheel` refuses unattended (no-TTY or `--unattended`) without
      `--budget`; exit 2 with clear message
- [ ] Second `/flywheel` invocation while first holds lock exits non-zero
      within the same worktree
- [ ] `/deliver` runs its internal loop and returns merge-ready via exit 0
      + receipt.json with `status == "merge_ready"`
- [ ] `monitor.alert` triggers `/diagnose` automatically, emitting
      `triage.done` before `reflect`
- [ ] `harness.suggested` commits land on `harness/auto-tune` only, never
      on default branch; branch rebased-clean before each append;
      CODEOWNERS blocks auto-merge

Durability:
- [ ] SIGKILL mid-phase followed by `--resume <ulid>` completes the cycle
      with no duplicate side-effects
- [ ] SIGINT releases lock; `--resume <ulid>` continues from phase after
      last `*.done` event
- [ ] Crash-truncated final jsonl line is dropped by resume parser without
      wedging the cycle
- [ ] `--abandon <ulid>` closes cycle, releases lock, stamps source file

Worktree:
- [ ] `/flywheel` run from a linked worktree uses
      `<worktree-root>/.harness-kit/flywheel.lock`, not main checkout's
- [ ] worktree-A and worktree-B can both run `/flywheel` concurrently on
      the same repo; each has its own lock, cycle dir, budget state

Integration:
- [ ] Harness Kit dogfoods on itself (one real cycle, one real item shipped
      end-to-end)

## Non-Goals

- Dynamic philosophy bench classifier — static globs first (see 030)
- GEPA auto-tuner — defer until ≥20 cycles of signal (031)
- Model Council at reflect — too expensive for MVP; shape only
- MAR-style multi-reflector — single reflect pass in MVP
- Merging PRs automatically — humans merge; loop suggests only
- Unattended mode without explicit `--budget`
- Tailored per-repo skills — separate initiative (029 needs rework)
- Cross-worktree coordination or claim-based locking — explicitly dropped;
  assume single local isolated workspace per cycle

## Related

- Depends on: 022 (swarm review default), 025 (dagger merge gate), 032
  (`/deliver` rename + recompose), 035 (`/deploy`), 036 (`/monitor`), 037
  (`/reflect` upgrade)
- Sibling: 030 (static bench-map), 024 (evidence storage — see 032 for fate)
- Unlocks: 031 (harness auto-tune — parked until cycles produce signal)
- Supersedes: old `/flywheel` continuous-mode speculation; claim-based
  multi-agent coordination (explicitly dropped)
