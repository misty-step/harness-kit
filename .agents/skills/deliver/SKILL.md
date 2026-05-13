---
name: deliver
description: |
  Inner-loop composer for spellbook. Takes one backlog.d/ item (or open
  git-bug bug) to merge-ready code. Composes /shape → /implement →
  {/code-review + /ci + /refactor + /qa} (clean loop) and stops. Does not
  push, does not merge, does not deploy. Communicates with callers via exit
  code plus <state-dir>/receipt.json — no stdout parsing. Every run also
  ends with a tight operator-facing delivery brief plus a full /reflect
  session.
  Use when: building a shaped ticket, "deliver this", "make it merge-ready",
  driving one backlog.d/NNN-*.md through review + CI.
  Trigger: /deliver.
argument-hint: "[backlog-item|bug-id] [--resume <ulid>] [--abandon <ulid>] [--state-dir <path>]"
---

# /deliver (spellbook)

Inner-loop composer. One `backlog.d/NNN-*.md` (or open `git-bug bug`) →
merge-ready code on a `<type>/<slug>` branch. **Delivered ≠ shipped.**
The outer loop (`/flywheel`) consumes `receipt.json` and decides whether
to deploy. Humans merge.

This is the spellbook-tailored variant. It targets this repo's gate
(`dagger call check --source=.`), this repo's backlog format
(`backlog.d/NNN-*.md` / `_done/`), this repo's base branch (`master`),
and this repo's composition lint (`check-deliver-composition`).

**`/qa` is part of the clean loop.** Spellbook has no browser UI, so QA is
not Playwright. It verifies the library/harness surfaces that users
actually consume: Dagger gate receipts, skill eval suites, generated
artifact drift, symlink bridge topology, and command-level smoke evidence.

## Invariants

- **Compose atomic phase skills via their trigger syntax.** Never inline
  phase logic. Specifically: never invoke `dagger call check` directly,
  never dispatch `critic`/`ousterhout`/`carmack`/`grug`/`beck` bench
  agents directly, never run raw linters. Those are `/ci` and
  `/code-review`'s jobs. Regression-guarded by the `check-deliver-composition`
  gate against `skills/deliver/SKILL.md`.
- **Fail loud.** A dirty phase is a dirty phase — do not mask it, do not
  retry past the cap, do not write `status: merge_ready` when anything
  is red.
- **Base branch is `master`.** Not `main`. HEAD-detection must match.
- **Never push.** Delivery ≠ shipping. `git push` is the outer loop's
  (or the human's) call.
- **Never merge.** `gh pr merge` and `scripts/land.sh` are human
  decisions.

## Closeout Contract

Every run ends with two operator-facing outputs, in this order:

1. A tight delivery brief (1–2 paragraphs or 4–6 flat bullets).
2. A full `/reflect` session.

The brief is not a file inventory, not a raw changelog, not a "green
tests" note. It answers:

- What `backlog.d/NNN-*.md` (or `git-bug` id) was worked; what changed.
- Why merge-readiness now is useful (delta to open debts in
  `backlog.d/` — e.g. does this close 023, unblock 025, reduce gate
  latency?).
- What alternatives to the implemented design existed.
- Why the implemented design is best under current constraints — or,
  if it is not clearly best, a plain admission plus why it was still
  the right delivery choice (e.g. shape locks, thinness doctrine,
  cross-harness parity).
- Value for contributors/operators (does it shrink SKILL.md, thin the
  harness, reduce gate drift, speed `dagger call check`?).
- Value that lands for users of spellbook — downstream repos that
  bootstrap from `~/.claude`, `~/.codex`, `~/.pi`.
- What was verified (which of the 13 sub-gates ran green; what
  `/code-review` synthesized) and what residual risk remains before
  merge.

`/reflect` stays mandatory. The brief explains the delivered result;
`/reflect` captures the learnings, harness changes, and backlog
mutations. When `/deliver` is invoked under `/flywheel`, keep the same
shape but let the outer loop own the final session-level shipping
brief.

## Composition

```
/deliver [backlog-item|bug-id] [--resume <ulid>] [--state-dir <path>]
    │
    ▼
  pick (if no arg) — highest-priority backlog.d/NNN-*.md, else git-bug
    │
    ▼
  /shape          → context packet (goal + oracle + sequence + anchors)
    │
    ▼
  /implement      → TDD build on feat/<slug> (or fix/chore/refactor/docs)
    │
    ▼
┌── CLEAN LOOP (max 3 iterations) ─────────────────────────┐
│  /code-review  → philosophy bench + thinktank + cross-    │
│                  harness; verdict ref under               │
│                  refs/verdicts/<branch>                   │
│  /ci           → audits Dagger module, runs the gate      │
│                  (13 parallel sub-gates), self-heals lint │
│                  drift; escalates logic failures          │
│  /refactor     → diff-aware simplification of base...HEAD │
│  /qa           → non-browser harness/library evidence     │
└──────────────────────────────────────────────────────────┘
    │ all green → merge-ready (exit 0)
    │ cap hit or hard fail → fail loud (exit 20/10)
    ▼
  receipt.json written; stop. No push, no merge, no deploy.
```

## Phase Routing

| Phase | Trigger | What it owns | Skip when |
|---|---|---|---|
| shape | `/shape` | context packet, oracle, sequence, repo anchors | packet already has executable oracle (backlog item is already shaped) |
| implement | `/implement` | TDD red→green→refactor, commits on `<type>/<slug>` | — |
| review | `/code-review` | bench + thinktank + cross-harness review, verdict ref, `.groom/review-scores.ndjson` entry | — |
| ci | `/ci` | audits `dagger.json`, runs the gate, self-heals lint gates, bounded heal via `dagger call heal` | `/ci` itself decides — do not pre-filter |
| refactor | `/refactor` | diff-aware simplify on `master...HEAD` | trivial diffs (<20 LOC, single file) |
| qa | `/qa` | verifies Spellbook's non-app surfaces and captures evidence | `/qa` itself decides — do not pre-filter |

Each skill has its own contract and receipt. `/deliver` reads those
receipts; it never re-implements the phase.

### What /qa means here

Spellbook has no runtime UI — no React app, no HTTP service, no
application binary end-users run. Artifacts are SKILL.md bodies, agent
definitions, a Dagger module (`ci/src/spellbook_ci/main.py`), shell
scripts, symlink bridges, and git hooks. `/qa` therefore runs a
library/harness verification path, not browser exploration. It checks and
records evidence for the changed surfaces:

- YAML/shell/Python syntax (`lint-yaml`, `lint-shell`, `lint-python`)
- SKILL.md frontmatter + line limits (`check-frontmatter`)
- Derived-artifact drift (`check-index-drift`, `check-vendored-copies`)
- Skill tests and eval contracts (`test-bun`, `check-skill-evals`)
- Portable-path + harness-install invariants (`check-portable-paths`,
  `check-harness-install-paths`)
- Composition lint for this very skill (`check-deliver-composition`)
- Banned primitives (`check-no-claims`, `check-exclusions`)
- Cross-harness bridges: `.claude/skills`, `.codex/skills`, `.pi/skills`
  resolve to `.agents/skills`

The Dagger gate is still the load-bearing verification. `/qa` is the
evidence and smoke layer around it: prove the changed harness surface is
exercised, name what was not exercised, and hand the record to `/ship`.

## Cross-Cutting Invariants

- **No claims.** Dropped per `backlog.d/_done/032` and enforced by the
  `check-no-claims` gate. Single local workspace. Concurrent worktrees
  coordinate via state-dir isolation (see `references/worktree.md`).
- **Never re-deliver stale backlog.** If the target backlog item
  already carries `## What Was Built`, already lives under
  `backlog.d/_done/`, or current-branch history contains a closure
  marker like `Closes backlog:<item-id>` or `Ships backlog:<item-id>`,
  stop and route to `/groom tidy`. Example footgun: 028 is marked
  "mostly shipped" in the repo brief — re-delivering it is drift, not
  work. Backlog state must be fixed first.
- **Never commit to `master`.** Feature branch only; see
  `references/branch.md` for HEAD detection and naming.
- **No `index.yaml` edits.** The pre-commit hook regenerates it when
  `skills/` or `agents/` changes. A phase that hand-edits `index.yaml`
  is a bug in that phase.
- **`.spellbook/deliver/<ulid>/{state,receipt}.json` are agent-written
  and gitignored.** The pre-commit hook refuses force-adds and prints
  the escape hatches (`/deliver --resume <ulid>` /
  `/deliver --abandon <ulid>`). If a phase skill somehow stages those
  files, that is a bug in that phase skill, not in the composer.
- **Evidence is out-of-band.** `/deliver` writes zero artifacts
  itself; per-phase skills emit; receipt records pointers only. See
  `references/evidence.md`.

## Contract (exit code + receipt)

`/deliver` communicates exclusively via its exit code and
`<state-dir>/receipt.json`. Callers — human or `/flywheel` outer loop —
do not parse stdout.

| Exit | Meaning | Receipt `status` |
|---|---|---|
| 0 | merge-ready | `merge_ready` |
| 10 | phase handler hard-failed (missing tool, Dagger engine down, etc.) | `phase_failed` |
| 20 | clean loop exhausted (3 iterations, still dirty) | `clean_loop_exhausted` |
| 30 | user/SIGINT abort | `aborted` |
| 40 | invalid args / missing dep skill | `phase_failed` |
| 41 | double-invoke on an already-delivered item | `phase_failed` |

Full receipt schema + state lifecycle: `references/receipt.md`.

## Resume & Durability

State is filesystem-backed and resumable.

- **State root:** `<worktree-root>/.spellbook/deliver/<ulid>/`
  (gitignored; pre-commit hook blocks force-adds). Override via
  `--state-dir <path>`; `/flywheel` uses this to land state under the
  cycle's evidence tree.
- **Checkpoint:** after each phase, `state.json` rewritten atomically
  (write → `fsync` → rename → `fsync` parent dir). POSIX atomic-rename
  guarantee.
- **`--resume <ulid>`:** loads `state.json`, skips completed phases,
  re-enters at `current_phase`. Phase handlers must be idempotent
  (e.g. `/ci` re-running `dagger call check --source=.` with engine
  cache is cheap; `/code-review` re-reviews the current diff).
- **`--abandon <ulid>`:** removes state-dir; leaves the
  `<type>/<slug>` branch intact.
- **Double-invoke:** `/deliver <already-delivered-item>` → exit 41,
  not silent re-run. The pre-commit hook's error message is the
  canonical source of the escape-hatch wording.

Full protocol: `references/durability.md`.

## Dagger gate self-heal (via /ci)

The gate is self-healing, but `/deliver` does not run the heal
directly. `/ci` decides when to invoke `dagger call heal` for a
failing lint-style gate (`lint-yaml`, `lint-shell`, `lint-python`,
`check-frontmatter`) and bounds it at the skill level (`--attempts=2`
per `/ci`'s policy). If `/ci` escalates — because the failure is
`check-deliver-composition`, `check-no-claims`, `check-index-drift`,
a test (`test-bun`), or anything outside the heal target set — the
clean loop treats it as dirty, dispatches a fix, and re-runs. Do not
inline a raw `dagger call heal` here.

## Gotchas (judgment, not procedure)

- **Retry vs escalate.** Dirty on iteration 1 → retry (normal). Dirty
  on iteration 3 → exit 20, write receipt, hand to human. The cap is
  load-bearing: loops without one produce slop.
- **What counts as "dirty".** `/code-review` blocking verdict (no
  `refs/verdicts/<branch>` pointing at HEAD, or one with
  `verdict: dont-ship`), `/ci` non-zero, `/refactor` non-zero. Review
  "nit" and "consider" are not blocking.
- **Verdict ref freshness.** A `refs/verdicts/<branch>` whose `sha`
  doesn't match `git rev-parse HEAD` is stale — re-trigger review.
  `/code-review` enforces this, but `/deliver` must treat a stale
  verdict as "review not done" even if the ref exists.
- **Inlining a missing phase.** `/implement` unavailable → exit 40.
  Do NOT fall back to your own TDD build — inlined fallbacks become
  permanent.
- **Inlining the gate.** Spawning `dagger call check --source=.`
  directly here trips `check-deliver-composition`. Route through
  `/ci`. Same for direct bench-agent dispatch — use `/code-review`.
- **Silent push.** A phase skill that "helpfully" runs `git push` is
  a bug in that phase skill. Surface it; do not suppress it in the
  composer.
- **Re-shaping mid-delivery.** If `/implement` reveals the shape is
  wrong (e.g. the oracle contradicts an invariant in
  `harnesses/shared/AGENTS.md`, or the packet assumes a
  harness-native feature that violates cross-harness-first), stop the
  clean loop and exit 20 with `remaining_work` pointing at re-shape.
  Do not spin.
- **Skipping shape on unshaped bugs.** `git-bug bug` issues rarely
  carry an oracle. Running `/implement` against a raw bug title yields
  plausible garbage. `/shape` runs first. Always.
- **Stale "open" item, already merged.** An item can live in
  `backlog.d/` while `master` already contains its closure commit
  (human landed it outside `/flywheel`). Refuse to treat that as new
  work — check `git log master` for `Closes backlog:<id>` /
  `Ships backlog:<id>` first; fix backlog state (move to `_done/`)
  before any phase runs.
- **Base-branch assumption.** Spellbook's default is `master`, not
  `main`. A phase skill that hardcodes `main` (e.g. bench selection
  using `origin/main`) produces wrong diffs. Treat as dirty.
- **Cross-harness violation slipping through review.** Every touch of
  `harnesses/`, `bootstrap.sh`, or a new skill must answer "what does
  this do on Codex? on Pi?" If review misses it, `/ci`'s
  `check-harness-install-paths` catches it — but fixing there is
  late. Prefer `/code-review` to surface it.
- **Merging.** Never. End-state is merge-ready, not merged.

## Hot files you are likely to touch

These are the recent-churn surfaces; expect most work here:

- `skills/<name>/SKILL.md` — skill bodies (<500 lines, frontmatter-
  gated; `check-frontmatter` is blocking).
- `skills/<name>/references/*.md` — deep content; no
  `references/<repo-name>.md` sidecars (spellbook's own anti-pattern).
- `ci/src/spellbook_ci/main.py` — Dagger module, where all 12 gates
  live. Test locally with `dagger call check --source=.` (via `/ci`,
  not directly here).
- `bootstrap.sh` — two modes (symlink / download) that both install
  only the minimal global skills (`tailor seed`) plus all agents;
  per-repo skill subsets are handled by `/tailor` / `/seed`, not by
  bootstrap.
- `scripts/*.sh`, `scripts/*.py` — `check-frontmatter.py`,
  `check-harness-agnostic-installs.sh`, `generate-index.sh`,
  `heal-commit.sh`, `land.sh`, `sync-external.sh`. `lint-shell` /
  `lint-python` gates both apply.
- `.githooks/*` — `pre-commit` (index regen + state-file protection),
  `pre-merge-commit` (verdict gate), `pre-push`, `post-commit`, etc.
- `backlog.d/NNN-*.md` — shape updates live here, not in issue
  trackers. `_done/` is moved-to, not deleted-from.
- `harnesses/shared/AGENTS.md` — symlinked to every harness; changes
  ripple everywhere; `check-harness-install-paths` guards install
  wording.

## Delegation bench (for clean-loop fixes)

When a phase surfaces a blocker that needs a builder dispatch, the
spellbook philosophy bench is available:

- `planner` — spec-level design decisions.
- `builder` — the default fix dispatcher (general-purpose).
- `critic` — adversarial review inside `/code-review`.
- `ousterhout` / `carmack` / `grug` / `beck` — structural lenses,
  pinned to `/code-review`'s bench map (see
  `skills/code-review/references/bench-map.yaml`).

A11y triad agents are **not installed** (no UI). Do not select them.

## Non-Goals

- Deploying — `/flywheel` outer loop's concern.
- Merging — humans (and `scripts/land.sh`) do this, gated by
  `refs/verdicts/<branch>` and `.githooks/pre-merge-commit`.
- Multi-ticket operation — one `backlog.d/NNN-*.md` per invocation.
- Claim-based coordination — explicitly dropped.
- Version-controlled evidence — gitignored under `.spellbook/`.
- `/qa` — no UI to exercise.

## References

- `references/clean-loop.md` — iteration cap, dirty-detection per
  phase (no `/qa` row), escalation protocol
- `references/receipt.md` — full JSON schema, exit-code table, state
  lifecycle
- `references/durability.md` — `state.json` atomic checkpoint,
  `--resume` / `--abandon` semantics, double-invoke
- `references/evidence.md` — per-phase emission paths under
  `.spellbook/deliver/<ulid>/`, gitignored
- `references/branch.md` — `master` as base, `<type>/<slug>` naming,
  HEAD-detection, no-push rule
- `references/worktree.md` — state-root resolution via
  `git rev-parse --show-toplevel`, concurrent worktrees

## Related

- Consumer: `/flywheel` — outer loop passes
  `--state-dir backlog.d/_cycles/<ulid>/evidence/deliver/` and reads
  `receipt.json`.
- Phases: `/shape`, `/implement`, `/code-review`, `/ci`, `/refactor`.
- Downstream landing: `scripts/land.sh` (human-invoked; enforces
  verdict ref + gate) — not `/deliver`'s concern.
