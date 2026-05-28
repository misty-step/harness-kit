# `/deliver` — inner composer (rename of `/autopilot`)

Priority: high
Status: pending (hard-blocked on 033 and 034)
Estimate: L (~3 dev-days)

## Rename

Current `/autopilot` → `/deliver`. The skill takes one backlog item and
produces merge-ready code. It does not ship, does not deploy — it delivers
a reviewed, CI-green, QA-passed, refactored diff. "Delivered" ≡ ready for
human merge + downstream `/autopilot` (outer loop) to deploy.

The `/autopilot` name moves to the outer loop (see 028). One swap, two
skills with honest names.

## Goal

- One skill takes a ticket all the way to merge-ready code
- Composed from atomic phase skills; no inlined phase logic
- Stop condition: diff is clean (review + ci + qa all green) OR fails loudly
- No deploy, no monitor, no reflect — those belong to `/autopilot` (outer)

## Composition

```
/deliver [backlog-item|issue-id] [--resume <ulid>] [--state-dir <path>]
    │
    ▼
  pick (if no arg)
    │
    ▼
  /shape            → context packet
    │
    ▼
  /implement        → TDD build (see 033)
    │
    ▼
┌── CLEAN LOOP ────────────────────────────────┐
│  /code-review    → critic + bench            │
│  /ci             → dagger audit + run (034)  │
│  /refactor       → diff-aware simplify       │
│  /qa             → browser-driven exploratory│
│  capture evidence → see Evidence Handling    │
└──────────────────────────────────────────────┘
    │ loop until all green, max 3 iterations
    ▼
  merge-ready (stops here — no deploy, no merge)
```

Each phase is its own skill. `/deliver` is a thin composer: dispatch,
synthesize, make proceed/fix/escalate decisions.

### Clean-Loop Termination

Max 3 iterations of the clean loop. "Still dirty" after iteration 3 means
any of:
- `/code-review` verdict contains blocking findings
- `/ci` returns non-zero (any dagger check red)
- `/qa` reports P0 or P1 findings

On cap-hit:
- Exit code **20** (`clean_loop_exhausted`; see Receipt Contract)
- Receipt records: last verdict, last CI tail, last QA findings, iteration count
- Diff is **left on the feature branch for human inspection** — no reset, no revert
- Branch is **NOT pushed** — `/deliver` never ships
- `phase.failed` marker written to state file; re-invocation without
  `--resume` refuses to clobber

Human handoff: operator reads receipt, either fixes manually and re-runs
`/deliver --resume <ulid>` (restarts clean loop on same branch), or abandons
with `/deliver --abandon <ulid>` (clears state, keeps branch).

## Receipt Contract

`/deliver` communicates with its caller (human or `/autopilot` outer)
exclusively via exit code + receipt file. No stdout parsing, no heuristic
output sniffing.

**Receipt path:** `<state-dir>/receipt.json`. Default state-dir:
`<worktree-root>/.harness-kit/deliver/<ulid>/` (gitignored). When invoked by
`/autopilot`, caller passes `--state-dir backlog.d/_cycles/<ulid>/evidence/deliver/`
so state lands under the cycle's evidence tree.

**Schema:**
```json
{
  "schema_version": 1,
  "ulid": "01JXXX...",
  "item_id": "032-deliver-inner-composer",
  "branch": "feat/deliver-inner-composer",
  "base_sha": "abc123...",
  "head_sha": "def456...",
  "status": "merge_ready | clean_loop_exhausted | phase_failed | aborted",
  "phases": [
    {"name": "shape",     "status": "ok"},
    {"name": "implement", "status": "ok"},
    {"name": "review",    "status": "dirty", "iteration": 3, "blocking_count": 2},
    {"name": "ci",        "status": "ok"},
    {"name": "qa",        "status": "p1", "findings": 1}
  ],
  "evidence": {
    "review_dir": "<state-dir>/review/",
    "ci_dir": "<state-dir>/ci/",
    "qa_dir": "/tmp/qa-<slug>/",
    "demo_release": "https://github.com/org/repo/releases/tag/qa-evidence-..."
  },
  "remaining_work": ["review: 2 blocking findings in auth.py"],
  "recommended_next": "fix-and-resume | abandon | human-review",
  "cost_usd": 2.80
}
```

**Exit codes:**

| Code | Meaning | Receipt `status` |
|---|---|---|
| 0 | Merge-ready | `merge_ready` |
| 10 | Phase handler hard-failed (tool error, not dirty output) | `phase_failed` |
| 20 | Clean loop exhausted (3 iterations, still dirty) | `clean_loop_exhausted` |
| 30 | User/SIGINT abort | `aborted` |
| 40 | Invalid args / missing dependency skill | `phase_failed` |
| 41 | Double-invoke: item already delivered (state exists, merge_ready) | `phase_failed` |

`/autopilot` outer consumes `receipt.json` and emits one `deliver.done`
event (see 028). It treats exit 0 as proceed-to-deploy; any non-zero halts
the cycle with `phase.failed`.

## What Moves Out

Today's `/autopilot` inlines a lot of build logic. Extract:

| Inlined today | Becomes |
|---|---|
| TDD build loop | `/implement` (033) |
| CI invocation | `/ci` (034 — redesigns `/settle`) |
| Ad-hoc evidence capture | Per-phase skills own emission; see Evidence Handling |
| Ship/merge logic | Removed — humans merge; `/autopilot` (outer) deploys |
| `source scripts/lib/claims.sh` | Removed — no claim coordination |

## Atomic Phase Skills Required

| Skill | Status | Ticket |
|---|---|---|
| `/shape` | ✓ exists | — |
| `/implement` | ❌ new | 033 (hard-block) |
| `/code-review` | ✓ exists | static bench-map in 030 |
| `/ci` | ❌ new (from `/settle` redesign) | 034 (hard-block) |
| `/refactor` | ✓ exists | — |
| `/qa` | ✓ exists | tailoring deferred |

## Evidence Handling

**Decision (2026-04-15):** Evidence is out-of-band and NOT version-controlled.
Per-phase skills own their own emission; `/deliver` never writes evidence
itself, only records pointers in the receipt.

| Phase | Emits | Where |
|---|---|---|
| `/code-review` | review synthesis, verdict, bench transcripts | `<state-dir>/review/` (gitignored, worktree-local) |
| `/ci` | dagger logs, failing-check tails | `<state-dir>/ci/` (gitignored) |
| `/qa` | screenshots, walkthroughs, findings | Its own scaffolded output dir; receipt records pointer |
| `/demo` | GIFs, launch videos | GitHub draft release via `/demo upload` (already works) |
| `/refactor`, `/implement` | None durable — test output transient | — |

Not in git (explicit): no `.evidence/` directory, no LFS pointers, no
committed screenshots. Review transcripts and CI logs live under
`.harness-kit/` which is gitignored wholesale. Demo artifacts live on GitHub
releases as they do today.

**Fate of ticket 024:** Superseded for `/deliver`'s purposes. If 024 stays
alive it is for a different consumer (auditable cross-cycle corpus for
`/reflect` or harness tuning). Mark 024 "reopen only with a concrete
consumer"; do not block 032 on it.

## Branch & Workspace Ownership

`/deliver` operates on its own feature branch. Policy:

- If HEAD is already on a non-default branch, `/deliver` uses it
- If HEAD is on default (`main`/`master`), `/deliver` creates
  `<type>/<slug>` where `<type>` is derived from backlog-item kind (`feat`,
  `fix`, `chore`, `refactor`) and `<slug>` is the item id
- `/deliver` NEVER commits to default
- `/deliver` NEVER pushes (delivery ≠ shipping)

**No claim coordination.** Dropped per operating principle. Single local
workspace assumption. Two concurrent `/deliver` invocations in different
worktrees on different branches are supported (see Worktree Behavior).

**When invoked by `/autopilot` outer:** the outer loop is responsible for
returning to a clean base (`git switch main && git pull`) before starting
the next cycle. `/deliver` does not clean up after itself.

## Durability & Resume

State is filesystem-backed, worktree-local, resumable.

**State root:** `<state-dir>`, default `<worktree-root>/.harness-kit/deliver/<ulid>/`.

```
<state-dir>/
├── state.json      # checkpoint: current_phase, completed_phases, item_id, branch
├── receipt.json    # written at exit (see Receipt Contract)
├── review/         # /code-review transcripts
└── ci/             # /ci logs
```

**Checkpoint protocol:** after each phase completes, `/deliver` atomically
rewrites `state.json` (write to `state.json.tmp`, fsync, rename). Phases
are re-runnable: `/implement` re-runs tests, `/code-review` re-reviews the
current diff, `/ci` re-runs dagger, `/qa` re-drives the app.

**`--resume <ulid>`:** loads `state.json`, skips completed phases, re-enters
at `current_phase`. Phase handlers are expected to be idempotent on a
partial run (requirement on 033 and 034's contracts; call out explicitly
there).

**`--abandon <ulid>`:** removes `<state-dir>`. Leaves branch as-is.

**Double-invocation:** `/deliver <same-item-id>` on an item whose state
file exists with `status: merge_ready` refuses with exit 41 ("already
delivered; use --resume or --abandon or switch to a fresh branch"). Not a
no-op — surfaces ambiguity.

**Interruption guarantees:** SIGINT, SIGKILL, power loss → on next run,
`--resume <ulid>` completes delivery. `state.json` is always either the
previous consistent state or the new one, never a torn write. Same
guarantees as `/autopilot` outer (028).

## Worktree Behavior

State paths rooted at the worktree root's `.harness-kit/deliver/` — NOT at
`$HOME` or the primary repo's `.git/`. Two worktrees of the same repository
can each run `/deliver` on different branches concurrently with zero
interference: separate `<ulid>`s, separate state directories, separate
receipts. No global locks.

Implementation: `/deliver` resolves its state root as
`$(git rev-parse --show-toplevel)/.harness-kit/deliver/`, which in a linked
worktree returns the worktree path (not the primary clone's). Verified by
an oracle check.

## `/deliver` vs current `/autopilot`

| Concern | `/autopilot` today | `/deliver` proposed |
|---|---|---|
| Scope | Pick → ship | Pick → merge-ready |
| Phases inlined | shape, build, review, ship | None — composes atomic skills |
| Stop condition | Shipped PR | Diff is clean (exit 0 + receipt) |
| Evidence handling | Ad-hoc | Per-phase, out-of-band, gitignored |
| Claims | `scripts/lib/claims.sh` | Dropped |
| Callers | Human | Human OR `/autopilot` (outer) via `--state-dir` |

## Phase Plan

1. **Hard-block on 033 and 034.** `/deliver` does NOT land until `/implement`
   (033) and `/ci` (034) exist as standalone skills with their own receipts.
   No inlined-fallback mode. Rationale: inlined fallbacks silently become
   permanent, and a composer-with-inlined-phases is the exact thing we're
   exiting. Sequence: 033 → 034 → 032. Each lands on its own branch, each
   green on CI, each with its own smoke-test ticket done via it. Only then
   does the rename+recompose happen as a single atomic commit series.

2. **Pin evidence storage** — already resolved (see Evidence Handling).
   Update 024 to "reopen only with a concrete consumer."

3. **Rename `/autopilot` → `/deliver`** — mechanical, grep-verified. Change list:

   Skill directory:
   - [ ] `git mv skills/autopilot/ skills/deliver/`
   - [ ] `skills/deliver/SKILL.md` frontmatter: `name:`, `description:`,
         `trigger:` (drop `/autopilot`, `/build`, `/ship`; add `/deliver`)
   - [ ] `skills/deliver/references/` unchanged (build.md, commit.md, etc.)

   Index / config:
   - [ ] `index.yaml` — autopilot entry renamed to deliver
   - [ ] `index.yaml` — iterate description mentioning `/autopilot` updated
         to `/deliver` (coordinate with 028's iterate → autopilot rename)
   - [ ] `.harness-kit.yaml` global-skills comment list

   Root docs:
   - [ ] `CLAUDE.md` orchestrator comment list and pipeline diagram
   - [ ] `AGENTS.md` pipeline diagram
   - [ ] `README.md` skill table and pipeline diagram
   - [ ] `project.md` pipeline row

   Cross-references in other skills (grep-verified):
   - [ ] `skills/shape/SKILL.md`
   - [ ] `skills/qa/SKILL.md`
   - [ ] `skills/demo/SKILL.md`
   - [ ] `skills/reflect/references/tune-repo.md`
   - [ ] `skills/investigate/references/triage.md`
   - [ ] `skills/groom/references/git-bug-conventions.md`
   - [x] `skills/autopilot/SKILL.md` (renamed from `skills/iterate/` under
         028; 032 flipped "old `/autopilot`" refs to `/deliver`, then 028
         flipped iterate refs to `/autopilot`)
   - [x] `skills/autopilot/scripts/autopilot.sh` (comment strings)

   Drop entirely (no claims):
   - [ ] Remove `source scripts/lib/claims.sh` calls from new `skills/deliver/SKILL.md`
   - [ ] Audit other callers (`skills/groom/SKILL.md`,
         `backlog.d/026-multi-machine-sync.md`); update groom to stop
         calling claim_acquire; keep 026 as design note; then
         `git rm scripts/lib/claims.sh`

   Harness / bootstrap:
   - [ ] `bootstrap.sh` — verify symlink walk of `skills/*` picks up rename
         automatically (expected: yes)
   - [ ] `harnesses/*/settings.json`, `harnesses/shared/AGENTS.md` — grep clean

   Backlog references (docs-only, can lag):
   - [ ] `backlog.d/022`, `024`, `029`, `033`, `035`, `036`, `037`, `028`
         — update `/autopilot` meanings where ambiguous

   Verification: after rename,
   `grep -rn "/autopilot" --exclude-dir=.git --exclude-dir=_done .` should
   return only references that mean the NEW outer loop (028). Zero hits
   point at the old inner skill.

4. **Rewrite SKILL.md as composer** — strip inlined phase logic; delegate
   to atomic skills. Target: ≤300 lines / ≤3K tokens (from ~700 today);
   ≤5K hard ceiling. Extract progressive disclosure to references/:

   ```
   skills/deliver/
   ├── SKILL.md              # routing + composition contract + top-level gotchas
   └── references/
       ├── clean-loop.md     # iteration cap, dirty-detection, escalation
       ├── receipt.md        # full schema, exit-code table
       ├── durability.md     # state.json protocol, --resume / --abandon
       ├── evidence.md       # per-phase emission, gitignored paths
       ├── branch.md         # branch-naming, no-push rule
       └── worktree.md       # state-root resolution
   ```

   SKILL.md keeps: composition diagram, phase routing table, cross-cutting
   invariants (no claims, no push, never deploy, fail loud), top-level
   gotchas (judgment, not procedure). Schemas, exit-code tables,
   state-machine details all move to references/.

5. **Harness Enforcement (lands with the rename)** — structural prevention,
   not prose:

   - [ ] `.gitignore` includes `.harness-kit/deliver/` (state dir worktree-local, never committed)
   - [ ] Pre-commit hook: reject manual edits to `.harness-kit/deliver/**/state.json` and `receipt.json`
   - [ ] Pre-push hook: warn (not block) if pushing a branch with a live
         `.harness-kit/deliver/<ulid>/state.json` whose status ≠ `merge_ready`
         (catches "I forgot to finish /deliver before pushing" footgun)
   - [ ] Lint rule: no inlined-phase calls in `skills/deliver/SKILL.md` —
         grep for known phase-skill internals (e.g., direct `/code-review`
         agent dispatch syntax) and fail if `/deliver` isn't using the
         phase-skill triggers
   - [ ] After `git rm scripts/lib/claims.sh`, add a lint rule rejecting
         re-introduction of `claims.sh` or `claim_acquire` strings anywhere
         in `skills/`

6. **Quality check (lightweight, not full eval harness)** — Run `/deliver`
   on one fixed, already-done backlog item (e.g.,
   `backlog.d/_done/013-strengthen-skill-instructions.md` — small, recent,
   well-defined oracle). Compare against the archived `/autopilot` run in
   its "What Was Built" section.

   Regression signals (any one fails the check):
   - `/code-review` misses any finding the archived run caught
   - `/ci` goes red where archived run was green
   - Demo artifact missing where archived run had one
   - Wall-clock >2x archived run (ceiling; single-sample timing noise is large)

   Explicitly NOT a formal harness eval — that's 031's territory. This is
   one smoke test to confirm composition didn't silently drop a capability.

## Oracle

- [ ] `skills/deliver/SKILL.md` exists; `skills/autopilot/` renamed (no
      stragglers from rename inventory)
- [ ] `/deliver` runs on a real ticket and produces merge-ready code without
      inlining phase logic
- [ ] All phase handlers (`/shape`, `/implement`, `/code-review`, `/ci`,
      `/refactor`, `/qa`) invoked via skill composition
- [ ] `/deliver` stops at merge-ready — does not push, does not merge,
      does not deploy
- [ ] Clean-loop termination: max 3 iterations, exit 20, receipt populated,
      diff left on branch unpushed
- [ ] Receipt file at `<state-dir>/receipt.json` validates against schema
- [ ] Exit codes match the Receipt Contract table
- [ ] **Durability:** SIGKILL mid-`/code-review` phase then `/deliver --resume <ulid>`
      completes delivery correctly
- [ ] **Durability:** torn-write simulation (kill during `state.json` rewrite)
      → resume still works
- [ ] **Double-invoke:** `/deliver <done-item>` exits 41 rather than silently
      re-running or no-op
- [ ] **Worktree:** worktree-A and worktree-B each run `/deliver` on different
      branches concurrently; both produce independent merge-ready receipts
- [ ] No references to `scripts/lib/claims.sh` anywhere in `skills/deliver/`
- [ ] Smoke test (step 5): `/deliver` on archived ticket matches archived
      quality signals
- [ ] `--state-dir <path>` override works: `/autopilot` invocation places
      state under `backlog.d/_cycles/<ulid>/evidence/deliver/`

## Non-Goals

- Deploying code — `/autopilot` outer loop owns that
- Multi-ticket operation — one ticket per invocation
- Unattended mode — interactive by default; outer loop runs `/deliver`
  unattended via its own contract
- Repo-specific tailoring — deferred (tailor rework pending)
- Claim-based coordination — explicitly dropped; single local workspace
  assumption
- Version-controlled evidence — see Evidence Handling decision

## Related

- Blocks: 028 (`/autopilot` outer loop needs `/deliver` as a black-box step)
- Depends on (hard-block): 033 (`/implement`), 034 (`/ci`)
- Supersedes for this skill: 024 (offline evidence storage) — kept alive
  only if a different consumer emerges
- Sibling: 030 (static bench-map — improves `/code-review` subphase)
- Supersedes: parts of `/settle` that overlap with `/ci`
