# `/tailor` — per-repo harness generator

Priority: P0 (shipped)
Status: done
Estimate: L
Shipped: 2026-04-20 (tailor-harden branch squash); superseded-and-
validated-by: 045 (harness pivot to minimal globals).
Aliases: `/attune`

## What shipped

`skills/tailor/SKILL.md` (382 lines). Explores the repo + session
history, browses the catalog, synthesizes a repo brief at
`.harness-kit/repo-brief.md`, dispatches parallel rewriters (one per
workflow skill) with critic adjudication, installs shared-root
skills under `.agents/skills/` with per-harness symlink bridges
(`.claude/skills/`, `.codex/skills/`, `.pi/skills/`), writes a
router `AGENTS.md`, writes per-harness settings (merged additively).

Validated first-run on harness-kit itself (commit `bef16b2`, 2026-04-20):
14 skills (9 workflow rewrites + 5 universal) + 7 agents installed,
gate 12/12 green, self-audit (byte-identity, agent-refs-resolve,
debt-map-zero-unfiled, skip-justification) passes.

## What was cut (deliberate)

The Oracle-level criteria below (`manifest.json` with hashes, A/B
eval machinery, automatic rollback on loss, 7-day manifest refresh
guard, `tailor-lint.sh` pre-commit hook, drift detection) were
deleted in two refactors:

- `68e276b refactor(tailor): cut 683 lines — push scoring to skill
  judgment` — removed the A/B eval harness and scoring code; replaced
  with skill-judgment criteria (critic checklist, subtractive test,
  cohesion test, byte-identity test).
- `f91f1c4 refactor(harness): pivot to minimal globals — /tailor +
  /seed only` — removed the `.harness-kit.yaml` allowlist machinery,
  `HARNESS_KIT_TEST_MODE` probe, `EXTERNAL_SKILLS[]` array.

The `.harness-kit` marker file (`installed-by: tailor`, `tailor-version:
<sha>`, `category: workflow|universal|agent|external`) serves the
"what's tailor-owned" function that `manifest.json` was originally
designed for, with less ceremony.

## Follow-on work

- `046-curate-skill-triage.md` — legacy unmarked `curate` skill
  surfaced during first-run self-audit.
- `047-tailor-external-skill-install.md` — extend `/tailor` with a
  third install mode for externally-managed skills (from
  `registry.yaml` sources — vercel-*, anthropic-*, jakub-*, emil-*,
  karpathy-*).

## Original shape preserved below for reference

## Goal

Generate a project-local harness (`.claude/` tree + `AGENTS.md`) specialized
to the target repo, **gated by an A/B eval that rolls back losers.**

Ownership: project-local artifacts only. Global workflow skills are
untouchable.

## The Central Question

Off-the-shelf Claude/Codex = minimum maintenance, minimum specificity.
Hand-tuned per-task harness = max quality, unbearable maintenance. Per-repo
is the **legibility unit** — a human reads `.claude/` and understands
"here's what this repo expects of an agent." Per-task (Live-SWE-agent)
regenerates context every prompt; off-the-shelf (Windsurf SWE-1.5 MoE)
undershoots on novel repo idioms.

## Why This Isn't `/focus` Rebuilt

`/focus` was 71 commits over two months, killed March 2026 (commit 6092678).
It did **discovery + install of existing skills** via embeddings: 87
candidate skills, 41 domain auditors. Failure mode: ceremony-to-value,
choice paralysis, install ≠ usage, no killswitch.

`/tailor` is different:

| `/focus` (killed) | `/tailor` |
|---|---|
| Discovery + install of existing skills | Generation of new bespoke artifacts |
| No killswitch | **A/B eval with rollback on loss (Phase 5)** |
| Ran continuously | Runs once; refresh is manual in MVP |
| 87 skills, 41 auditors | **Cap 2 domain skills in MVP** |
| Recommended skills | Writes files, bounded + gated |
| Prose rules on scope | **Loader-level enforcement** (pre-commit lint) |

If a global skill exists, tailor is forbidden to generate a project-local
override of it. Enforced structurally, not in prose. Closes bug #014
territory.

## State Model

Single source of truth: `.claude/.tailor/manifest.json`

```json
{
  "schema_version": 1,
  "generated_at": "2026-04-14T...",
  "harness_kit_version": "<git sha>",
  "repo_sig": {
    "primary_lang": "rust",
    "frameworks": ["axum", "tokio"],
    "ci": "github-actions+dagger",
    "hotspots": ["src/ingest/...", "..."],
    "commit_count_at_gen": 12431
  },
  "owned_files": [
    {"path": ".claude/settings.local.json", "hash": "sha256:..."},
    {"path": "AGENTS.md", "hash": "sha256:...", "sections": [...]}
  ],
  "eval": {
    "task": "run test suite and report failures",
    "baseline": {"tool_calls": 47, "wall_s": 312, "passed": true},
    "tailored": {"tool_calls": 29, "wall_s": 198, "passed": true},
    "verdict": "win"
  },
  "domains_owned": ["qa"]
}
```

### Human-Edit Preservation

Any file whose on-disk hash ≠ `manifest.owned_files[].hash` is treated as
human-touched; subsequent regens preserve it. `<!-- tailor:keep -->` fences
in markdown mark permanent protected sections.

### Enforcement (structural, not prose)

`scripts/tailor-lint.sh` runs as pre-commit hook on `.claude/skills/`:
- Reject if `.claude/skills/<name>/SKILL.md` shadows a global workflow skill name (`groom`, `shape`, `deliver`, `flywheel`, `code-review`, `settle`, `reflect`, `tailor`, `harness`)
- Reject if the 2-skill MVP cap is exceeded

## Control Flow

```
/tailor [generate | refresh | drift]
    │
    ▼
┌─ Phase 0: Preflight ─┐
│ cooldown check       │ (reject if manifest <7 days, unless --force)
│ .claude backup       │
└──────────────────────┘
    │
    ▼
┌─ Phase 1: Repo analysis (parallel, 3 Explore agents) ─┐
│  lang-detector    → stack, frameworks                  │
│  ci-inspector     → build/test/lint commands, gates    │
│  existing-harness → current .claude/, AGENTS.md        │
└────────────────────────────────────────────────────────┘
    │ verdicts in ~90s, ~15k tokens
    ▼
┌─ Phase 3: Dialectic (1 round, no loop) ─┐
│  planner → proposed artifacts tree       │
│  critic  → focus-postmortem checklist    │
│  lead    → synthesize; abort if critic blocks │
└──────────────────────────────────────────┘
    │
    ▼
┌─ Phase 4: Generation (diff-and-confirm gate) ─┐
│  writes:                                       │
│   .claude/settings.local.json                  │
│   AGENTS.md (repo-specific content only)       │
│   .claude/.tailor/manifest.json                │
│  (MVP stops here — no skills/agents/hooks)     │
└────────────────────────────────────────────────┘
    │
    ▼
┌─ Phase 5: A/B eval (killswitch) ─────────────┐
│  ephemeral worktree A: CLAUDE_SKIP_PROJECT=1  │
│  ephemeral worktree B: tailored               │
│  run canned task (from ci-inspector output)   │
│  compare: tool_calls, wall_s, passed          │
│  if B not better on ≥2 of 3 → ROLLBACK        │
│     delete owned_files, restore backup        │
│     surface delta to user, exit non-zero      │
└───────────────────────────────────────────────┘
    │
    ▼
  write manifest, done
```

MVP skips: Phase 2 (external research via /research), Phase 6 (drift
detection at 200 commits / 90 days). Refresh is manual in MVP.

### Critic Checklist (`references/focus-postmortem.md`)

The critic attacks each proposed artifact against:
- Does this exist globally already? (If yes — reject.)
- Would scaffold-on-demand (`/qa scaffold`, `/demo scaffold`) cover it? (If yes — reject.)
- Is this a 1–3 workflow domain skill, or 41-auditor ceremony? (If ceremony — reject.)
- Can this file's value be proven in Phase 5 eval? (If no — reject.)

Stopping condition: critic returns zero blocking objections in round 1.
No round 2. That's where `/focus` rotted.

## Components

| Component | Type | Owns |
|---|---|---|
| `skills/tailor/SKILL.md` | skill | orchestration |
| `skills/tailor/references/focus-postmortem.md` | reference | critic checklist |
| `skills/tailor/references/eval-task.md` | reference | how to derive canned task from ci-inspector output |
| `scripts/tailor-lint.sh` | script | shadow-name + cap enforcement (pre-commit) |
| `scripts/tailor-ab.sh` | script | ephemeral worktree A/B runner |
| `agents/planner.md`, `agents/critic.md` | existing | reused for Phase 3 |

## What Specializes Per-Repo vs Global

| Layer | Owner | Examples |
|---|---|---|
| Workflow primitives | **Global (harness-kit)** — canonical reference | groom, shape, deliver, flywheel, code-review, reflect, settle, harness, tailor |
| Philosophy bench | Global — canonical reference | beck, carmack, grug, ousterhout, planner, critic |
| **Primitive derivatives** | **Per-repo (overlay on global)** — v2 | tailored `/code-review` with this repo's review rubric; tailored `/qa` with this repo's golden paths; tailored `/implement` with this repo's TDD conventions |
| Domain skills (novel) | Per-repo (generated) — v2 | `<repo>-migrations`, `<repo>-fixtures`, `<repo>-deploy` — only when no global primitive applies |
| AGENTS.md content | Per-repo — MVP | build/test commands, hot paths, gotchas |
| Hooks | Per-repo — v2 | pre-edit lint, post-test runner |
| Permissions allowlist | Per-repo — MVP | actual binaries used (`settings.local.json`) |
| Reviewer agents | Per-repo only if stack-specific beats generic — v2 | `rust-unsafe-reviewer` yes, `general-reviewer` no |

Sharp line enforced by `tailor-lint.sh`. In MVP, `.claude/skills/<name>/`
shadowing a global name is forbidden outright. In v2, overlays are the
**sanctioned** shadow mechanism — see below.

## Reference-Derivative Pattern (v2)

The working model is **harness-kit as canonical reference, per-repo as
specialization**. A global primitive (`/code-review`, `/qa`,
`/implement`) is the stable contract; the per-repo tailored version
inherits its shape and adds repo-specific content.

### Overlay Composition

```
skills/code-review/SKILL.md          ← global, canonical
.claude/.tailor/overlays/code-review.md  ← per-repo specialization
         ↓ (compose at tailor-generate time)
.claude/skills/code-review/SKILL.md  ← emitted final artifact
```

Overlay file schema (proposal, to harden in v2 shaping):

```markdown
---
extends: code-review        # global primitive name (required)
repo_sig_hash: sha256:...   # invalidates on repo signature change
---

<!-- tailor:inherit description -->
<!-- tailor:inherit trigger -->

<!-- tailor:append body -->
## Repo-specific review rubric
- Rust: flag `unsafe` blocks without SAFETY: comment
- Axum handlers must return typed errors, not anyhow
- Migrations in `migrations/`: require paired down.sql

<!-- tailor:keep -->
## Human additions (preserved across regens)
```

Compose-time rules:
- `inherit` sections are copied from global SKILL.md as-is.
- `append` sections are concatenated under the matching section header.
- `keep` fences preserve human edits through regeneration.
- If the global primitive version changes between generations, the
  overlay `repo_sig_hash` is checked and the user is warned if the
  inherited contract drifted.

### Why Overlay, Not Fork

- **Global improvements propagate.** Bug fixes or UX changes to the
  global skill flow into every repo on next tailor-refresh.
- **The diff is legible.** `.claude/.tailor/overlays/code-review.md`
  shows exactly what this repo adds beyond the global contract.
- **Cross-harness first.** Composition runs at tailor-generate time
  on the harness-kit side; the emitted SKILL.md works on Claude, Codex,
  and Pi without harness-specific runtime machinery.
- **Avoids `/focus` bloat.** The overlay is small and derivative.
  A repo can't accumulate 87 full skills — only overlays on ≤N
  canonical ones.

### Greenfield Domain Skills (also v2)

Novel per-repo skills that don't derive from a global primitive
(e.g., `<repo>-migrations`) are still allowed in v2, but require a
stronger justification in the critic pass: "no global primitive could
be overlayed to cover this." Overlay-first is the default.

### MVP Scope Impact

None. MVP still ships only `AGENTS.md` + `settings.local.json` +
`manifest.json`. The overlay system lands in v2 after MVP validates
that tailored > vanilla on A/B. This section exists to ensure MVP
doesn't foreclose the overlay design with incompatible decisions
(e.g., baking absolute shadow-name prohibition into the manifest
schema rather than the lint rule).

## Interfaces

- **Input:** target repo (pwd), optional `--force`, optional `--task "<canned eval>"`
- **Output:** `.claude/settings.local.json`, `AGENTS.md` deltas, `.claude/.tailor/manifest.json`, stdout A/B summary
- **Exit codes:** 0 = win, 1 = rollback (tailored lost), 2 = user rejected diff, 3 = preflight failed

## Failure Modes

| Failure | Recovery |
|---|---|
| A/B eval: tailored loses | Rollback: delete owned_files, restore `.claude.backup.<ts>/`, log delta, exit 1 |
| Eval task doesn't exist (no tests) | Abort Phase 5, refuse to write manifest (no generation without eval) |
| Human edited generated file between runs | Refresh detects hash mismatch; preserves human version, re-plans around it |
| Generation attempts shadowing skill | `tailor-lint.sh` rejects pre-commit |
| `ls .claude/skills/` exceeds cap | Lint rejects |

## Interaction Contract with `/flywheel`

Two touchpoints:

1. **Auto-scaffold arbitration.** `/flywheel` reads
   `.claude/.tailor/manifest.json:domains_owned` before auto-scaffolding
   `/qa` or `/deploy`. If tailor owns, flywheel uses tailored artifact.
   Single ownership file. No race.

2. **Drift counter isolation** (v2). When `/tailor drift` counts commits,
   exclude refs matching `refs/heads/cycle/*` and
   `refs/heads/harness/auto-tune`. `/flywheel` writes to those branches;
   they're not "repo evolution."

## MVP Slice (~week 1)

Ship only:
1. `skills/tailor/SKILL.md` (~200 lines, one mode: generate)
2. Phase 1 with **3 subagents only**: lang-detector, ci-inspector, existing-harness-reader
3. Skip Phase 2 entirely (no external context — pure introspection)
4. Phase 3: single planner pass + critic pass (no synthesis loop)
5. Phase 4 generates only `AGENTS.md` + `.claude/settings.local.json` + `.claude/.tailor/manifest.json` — no skills, no agents, no hooks
6. Phase 5: one canned task ("run test suite and report failures"). Compare wall time + tool-call count A vs B. **Rollback on loss active from day 1** (this is the central differentiator from /focus — non-negotiable).
7. Skip Phase 6. Refresh is manual via `/tailor generate --force`.

Validate premise: if three real repos can't show tailored > vanilla on the A/B, **kill before building Phases 2 and 6.**

## Oracle

- [ ] `/tailor generate` in a real repo produces `AGENTS.md` + `settings.local.json` + `manifest.json`
- [ ] `manifest.json` conforms to schema, includes `repo_sig`, `owned_files` with hashes, `eval` block
- [ ] A/B eval runs two ephemeral worktrees, reports tool_calls + wall_s + passed for both
- [ ] If tailored loses on ≥2 of 3 metrics, rollback happens automatically and user sees the delta
- [ ] `tailor-lint.sh` pre-commit hook rejects a test commit attempting to create `.claude/skills/shape/SKILL.md` (global shadow)
- [ ] `tailor-lint.sh` rejects a test commit exceeding 2-skill cap
- [ ] Human edit of `AGENTS.md` section outside `<!-- tailor:keep -->` fence is preserved on regen
- [ ] `/tailor` refuses to run if manifest is <7 days old without `--force`
- [ ] `/flywheel` reads `manifest.json:domains_owned` and defers to tailored artifact when present
- [ ] Validated on 3 real repos showing tailored > vanilla on the A/B

## Non-Goals (MVP)

- External research phase (exemplar-finder, framework-harness-hunter, best-practice fetcher)
- Overlay composition — v2 (reference-derivative pattern above)
- Domain skill generation (`<repo>-migrations` etc.) — v2
- Reviewer agent generation (`rust-unsafe-reviewer` etc.) — v2
- Hook generation — v2
- Drift detection at 200 commits / 90 days — v2; manual refresh only in MVP
- Continuous monitoring — no background process
- Multi-round planner/critic loop — one round max
- Touching global harness-kit skills — ever
- Merging or PRing changes — writes to working tree, human commits

## Implementation Notes (added during shaping)

These sections resolve ambiguities that would block `/implement` on the
MVP. Listed in dependency order — Gap 1 is the load-bearing spike.

### Commit 0 — Prove Phase 5 measurement exists (BLOCKING spike)

Before building any of Phases 0-4, prove that tool-call counting +
wall-time capture work headlessly on at least one target harness.
**If this spike fails, MVP redesigns around Alt C below — do not
build Phases 0-4 on unverified measurement machinery.**

Candidate path: `claude -p "<prompt>" --output-format stream-json`
emits JSONL events including tool-use entries. Count tool-use events,
measure wall time with `time`, check final `passed` from the task's
own exit semantics (e.g., `pytest` return code).

Acceptance: a 20-line shell script `scripts/tailor-ab-spike.sh` that
runs a canned task (`"list the failing tests"`) against this repo,
prints `{tool_calls, wall_s, passed}`, and exits 0. If this works on
Claude Code, the MVP ships Claude-first; Codex/Pi parity follows.

### A/B alternatives (if Commit 0 blocks)

| Alt | Mechanism | Keeps killswitch? | Cost |
|---|---|---|---|
| **A (planned)** | Headless run, count tool_calls + wall_s + passed | Yes (objective) | Depends on harness `-p` + structured output |
| **B** | Run canned shell command (e.g., `pytest -v`) in both worktrees, no Claude involvement | **No** — measures commands, not tailoring | Low |
| **C** | Side-by-side Claude runs, LLM-as-judge rates outputs on fixed rubric | Yes (subjective but gated) | Another LLM call + judge prompt eng |
| **D** | Skip A/B entirely, manual rollback | **No** — this is /focus | Trivial |

B and D are rejected: they lose the killswitch, which is the entire
premise of /tailor. C is the MVP fallback if A is infeasible on any
harness. The implementation must not commit to A's plumbing (JSONL
parsing, tool-use counting) until Commit 0 validates the approach.

### Baseline isolation mechanism

`CLAUDE_SKIP_PROJECT=1` in the original control-flow diagram is a
placeholder — no such env var exists in Claude Code. Real mechanism
for MVP:

- `scripts/tailor-ab.sh` uses `git worktree add` to create two
  ephemeral worktrees under `$(git rev-parse --git-dir)/tailor-ab/`.
- **Baseline worktree:** `rm -rf .claude/ AGENTS.md CLAUDE.md` before
  running the canned task. Global `~/.claude/` skills still load
  (that's the "vanilla" baseline — not no-harness-kit, just
  no-project-tailoring).
- **Tailored worktree:** leave `.claude/` and `AGENTS.md` in place
  as generated by Phase 4.
- Both worktrees share the codebase under test; only the harness
  layer differs. This is the right comparison — does project-level
  tailoring beat global-only?

Cleanup: `git worktree remove --force` in a `trap EXIT`. If the
worktree registry is corrupted from a prior crash, `git worktree
prune` first.

### A/B scoring — deterministic rules

Three metrics, each emits a ternary verdict:

| Metric | B-win | Tie | A-win |
|---|---|---|---|
| `tool_calls` | B < A | B == A | B > A |
| `wall_s` | B < A*0.95 (≥5% faster) | within ±5% | B > A*1.05 |
| `passed` | B=true ∧ A=false | B==A | B=false ∧ A=true |

Aggregate rule: tailored (B) ships iff **at least 2 of 3 metrics are
B-win AND no metric is A-win.** Any A-win → rollback. Two B-wins + one
tie → ship. This is stricter than "not-worse-than-A" to avoid shipping
no-op tailoring that passes by noise. Rationale: the cost of a bad
harness is high (agent drift into wrong rubrics); the cost of a
rollback is low (no-op, manifest not written).

### skills/tailor/SKILL.md skeleton

Target structure (~180-220 lines):

```
---
name: tailor
description: Generate a project-local harness specialized to this repo, A/B-evaluated with rollback on loss. ...
---

# Overview (10 lines) — what /tailor does, what it won't do

## When to use (6 bullets)

## Execution protocol
### Phase 0: Preflight — cooldown, backup (15 lines)
### Phase 1: Repo analysis — 3 parallel Explore subagents (25 lines)
### Phase 3: Dialectic — planner + critic, one round (30 lines)
### Phase 4: Generation — write AGENTS.md, settings.local.json, manifest (30 lines)
### Phase 5: A/B eval — scripts/tailor-ab.sh, rollback on loss (20 lines)

## Invariants (hard lines the skill cannot cross)
- never touch global harness-kit/~/.claude skills
- never exceed 2-skill cap in .claude/skills/ (MVP)
- never shadow global skill name (MVP; v2 overlays)
- never proceed past Phase 5 without a verdict
- never write manifest.json without a Phase 5 eval result

## Failure modes (6-entry table — same as backlog spec)

## References
- references/focus-postmortem.md
- references/eval-task.md
```

Agent-of-record reads /tailor SKILL.md; orchestration is inline, not
delegated to a script. The two scripts (`tailor-lint.sh`,
`tailor-ab.sh`) are enforcement-only, not orchestration.

### Ephemeral worktree mechanics

```bash
# scripts/tailor-ab.sh (sketch)
set -euo pipefail
AB_DIR="$(git rev-parse --git-dir)/tailor-ab"
trap 'git worktree remove --force "$AB_DIR/baseline" "$AB_DIR/tailored" 2>/dev/null || true' EXIT

# Refuse on dirty tree — tailored worktree must reflect current HEAD
[ -z "$(git status --porcelain)" ] || { echo "dirty tree, stash first"; exit 3; }

git worktree add "$AB_DIR/baseline" HEAD
git worktree add "$AB_DIR/tailored" HEAD
(cd "$AB_DIR/baseline" && rm -rf .claude AGENTS.md CLAUDE.md)
# .claude and AGENTS.md in tailored worktree are the generated ones,
# copied in after the worktree is created (they're uncommitted).

# Run canned task, capture metrics
run_task "$AB_DIR/baseline"  > "$AB_DIR/baseline.metrics"
run_task "$AB_DIR/tailored" > "$AB_DIR/tailored.metrics"
compare_metrics  # applies scoring rules above, emits verdict + diff
```

### First-commit slice

Once Commit 0 spike passes, the sequenced buildout is:

1. `scripts/tailor-ab-spike.sh` → Commit 0 validates measurement (standalone, no skill yet)
2. `scripts/tailor-lint.sh` → pre-commit shadow-name + cap rule (testable in isolation)
3. `scripts/tailor-ab.sh` → real A/B runner using the spike's measurement technique
4. `skills/tailor/references/focus-postmortem.md` → critic checklist
5. `skills/tailor/references/eval-task.md` → canned task derivation
6. `skills/tailor/SKILL.md` → the skill itself, orchestrating phases 0-5
7. Dogfood on 3 real repos (this one + 2 others), capture verdicts

Each step is a single commit with its own tests. Steps 1-5 are
build-before-integration; step 6 is where orchestration comes
together; step 7 validates the premise.

## Related

- Depends on: 028 (`/flywheel` — cycle events provide eval signal and refresh triggers in v2)
- Structurally supersedes: 014 (archived 2026-04-14 — commit `6d3944d` fixed
  the name-matching root cause; `tailor-lint.sh` prevents the broader class
  of shadowing bugs)
- Prior art (killed): `/focus` skill (commit 6092678, March 2026)

## Name Collision Notes

- `/tailor` has one niche collision (obscure `tailor` git-history rewriter, different domain — low practical risk)
- `/attune` fully clean
- `/forge` rejected (Laravel Forge, Foundry, Sourcegraph Forge)
- `/provision` rejected (Terraform/Ansible/Pulumi)
- `/init-harness` rejected (collides with native `/init` + harness-kit `/harness`)
- `/scaffold-harness` rejected (collides with `/groom`'s scaffold alias)
