---
name: refactor
description: |
  Spellbook simplification. On a feature branch, shrink the diff before merge;
  on master, find the single highest-leverage cut in the catalog. "Refactor"
  here overwhelmingly means DELETE — scaffold, pass-throughs, regex-over-prose,
  semantic workflow DSLs, procedural bloat that a strong model doesn't need.
  The gate is `dagger call check --source=.`. A refactor that breaks any of
  the 14 sub-gates is a regression.
  Use when: "refactor this", "simplify this diff", "cut this down", "reduce
  complexity", "thin harness", "pay down debt".
  Trigger: /refactor.
argument-hint: "[--base <branch>] [--scope <path>] [--report-only] [--apply]"
---

# /refactor

Spellbook refactors are subtractive by default. The lineage:

- `68e276b refactor(tailor): cut 683 lines — push scoring to skill judgment`
- `7ccd00d refactor(flywheel): thin-harness — scripts out, judgment in` (977+738 LOC deleted, 43-line replacement)
- `f91f1c4 refactor(harness): pivot to minimal globals — /tailor + /seed only` (80 globals → 2)
- `53fe623 feat: radical simplification — 8 skills, 7 agents, harness consolidation`
- `8e1ed4c docs(skills): delete generic Execution Stance blocks (bitter lesson)`

Every one: net deletion. Judgment pushed into the model, scaffold torn out.
The question is never "how do we abstract this?" — it's "why does this exist
at all?"

## The gate is load-bearing

`dagger call check --source=.` runs 14 parallel sub-gates (see repo brief).
A refactor must leave them all green:

- `check-frontmatter` — line limits on SKILL.md, required fields
- `check-deliver-composition` — `skills/deliver/SKILL.md` composes phase
  skills via trigger syntax, never inlines `dagger call check`, `bunx
  playwright`, or direct bench-agent dispatch
- `check-portable-paths` — no hardcoded `/Users/<name>/` outside
  `harnesses/claude/` + `.claude/hooks`
- `check-harness-install-paths` — seed/tailor copy must not be Claude-only
- `check-no-claims` — no `claims.sh` / `claim_acquire` / `claim_release`
  under `skills/` (regression guard from backlog.d/032)
- `check-vendored-copies` — vendored copies match canonical
- `lint-{yaml,shell,python}`, `test-bun`, `check-index-drift`,
  `check-exclusions`

If a "simplification" breaks one of these, you deleted load-bearing wall.
Revert, don't patch.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/refactor` starts by probing the
roster and dispatching two or more available roster members before choosing
or applying a simplification. Use one lane to map complexity and another to
challenge behavior-preservation risk; add a builder lane for the selected
change. The lead agent owns the final cut, proof, and receipts. Direct
lead-only refactoring is limited to mechanical command execution, emergency
unblocks, explicit user-forbidden delegation, or an explicit waiver when
fewer than two roster members are available.

Fallback for failing lint-style gates only: `dagger call heal
--source=. --model=gpt-4.1 --attempts=2`. Heal fixes yaml/shell/python/
frontmatter. It does **not** cover composition, portable-paths,
install-paths, claims, or vendored-copies — those are judgment gates.

## Branch-aware routing

- Current: `git rev-parse --abbrev-ref HEAD`
- Primary: `git symbolic-ref --short refs/remotes/origin/HEAD | sed 's#^origin/##'`
  (fallback `master` — the repo brief says so)

`current != primary` → **Feature mode**.
`current == primary` → **Master mode**.

If `HEAD` is detached or base is ambiguous, stop and require `--base`.
Computing the wrong diff silently is worse than failing.

Flags:
- `--base <branch>` override base
- `--scope <path>` restrict to one subtree
- `--report-only` no edits (default on master)
- `--apply` allow edits on master

## Feature mode

Goal: land the smallest surface change that carries the invariant. Active
branch context: `feat/tailor-harden` is the reference — five commits, each
named after *one* concrete invariant (repo brief, iterative rewriters,
reconciliation, cross-harness install, force depth on loop-core skills).

### 1. Map the delta

`git diff --stat <base>...HEAD` and `git diff <base>...HEAD` against the
files the spec touched. Look for the named red flags from `harnesses/shared/AGENTS.md`:

> Shallow modules, pass-through layers, hidden coupling, large diffs,
> untested branches, speculative abstractions, stale context,
> responding to agent errors with prose instead of structural fixes,
> regexes over agent prose, semantic workflow DSLs around general agents.

Concrete forms each takes in this repo:

- **Shallow modules / pass-through layers** — A `scripts/` shim whose
  body is one `dagger call X` line. A skill SKILL.md that says "run
  `./scripts/foo.sh`." If the skill adds no judgment, it is shallow.
  `f91f1c4`'s deletion of `scripts/tailor-ab-spike.sh`,
  `scripts/tailor-lint.sh`, `scripts/test-*.sh` is the canonical purge.
- **Regexes over agent prose** — A pre-commit check that greps SKILL.md
  for forbidden phrases. `/harness lint` exists; `check-frontmatter`
  exists. Adding a fourteenth regex check that scolds the model for
  writing "consider" is the failure mode.
- **Semantic workflow DSLs around general agents** — State-machine
  YAML, phase enums, custom orchestration grammars layered on top of
  a model that can already sequence steps. `7ccd00d` killed 977 lines
  of `/flywheel` orchestrator — ULID primitives, lock helpers,
  `events.sh`, manifest schema — because the model picks cycles from
  `backlog.d/` with judgment.
- **Speculative abstraction** — Config keys with one caller, hook
  points with no subscriber, "extensibility" for a future that's not
  on the backlog. Torvalds-test it: actionable, scoped, time-bound,
  or delete.
- **Large diffs** — If this PR already touches >15 files, the refactor
  target is the PR's scope, not the codebase.

### 2. Rank

`(lines removed × confidence) / blast radius`. Prefer, in order:

1. **Deletion.** Dead path, unused export, shim for a consumer that
   no longer exists, a whole skill no one invokes. `f91f1c4
   refactor(harness): pivot to minimal globals` is the exemplar —
   removed bootstrap.sh allowlist parser + `SPELLBOOK_TEST_MODE`
   probe + `EXTERNAL_SKILLS[]` machinery, all load-bearing for a
   feature that no longer existed.
2. **Consolidation.** Two near-duplicates become one. `d049cad refactor:
   unify search scripts — extract shared embedding module` and `cddc40d
   feat(autopilot): add code-level pattern checks to review/refactor
   pipeline` are exemplars.
3. **State reduction.** Mode flags collapse; two-branch conditional
   becomes one path.
4. **Naming.** Only when it unblocks deletion or clarifies an
   invariant. Rename-as-theater fails the diminishing-returns test.
5. **Mechanical refactor.** Last resort — do this with `sed`/`rg`
   directly; don't spawn a subagent.

### 3. Execute (unless `--report-only`)

One bounded cut per commit. Preserve or improve test/contract coverage.
Update contracts that actually changed (not stylistic touch-ups).

### 4. Verify

- `dagger call check --source=.` green
- Affected lint-style gate passes
- `check-deliver-composition` if `skills/deliver/` was touched
- `check-frontmatter` if any SKILL.md line count shifted

## Master mode

Goal: one highest-impact cut for the catalog. Default `--report-only` —
shape into a `backlog.d/NNN-*.md` ticket with oracle. `--apply` only for
low-risk bounded cuts.

### 1. Hotspot map

From `git log --since='60 days ago' --name-only -- skills/ agents/ ci/
scripts/ bootstrap.sh | sort | uniq -c | sort -rn` plus the repo brief's
named hot files:

- `skills/tailor/SKILL.md` — the /tailor skill, recently churned 10×,
  just underwent a 683-line cut in `68e276b`. Still the first place to
  look for the next cut.
- `skills/deliver/SKILL.md` — inner-loop composer. Drifts into
  re-implementing phase-skill internals on every pass — hence the
  `check-deliver-composition` gate exists *because this file keeps
  breaking the contract*. If deliver has grown bash, regex, or direct
  bench-agent dispatch, this is the refactor.
- `bootstrap.sh` — 19 commits recently. Has two modes (symlink /
  download), both installing the minimal globals hardcoded near the
  bottom (`GLOBAL_SKILLS=(tailor seed)` plus all agents). Watch for
  duplicate path-resolution blocks and mode-specific branches that
  can collapse.
- `ci/src/spellbook_ci/main.py` — 12 gates. New gate = new Python
  function. Repeated shell-out boilerplate across gates is the
  consolidation target.
- `harnesses/shared/AGENTS.md` — symlinked to every harness.
  Principles doc, not code. Scope-check before cutting.
- `skills/settle/SKILL.md`, `skills/reflect/SKILL.md`, `skills/groom/SKILL.md`,
  `skills/harness/SKILL.md`, `skills/focus/SKILL.md` — next-hottest.

### 2. Known debts (ratified targets)

From the repo brief's "Known debts" section:

- `backlog.d/023-review-score-feedback-loop.md` —
  `.groom/review-scores.ndjson` wired but operationally empty.
  Refactor target: delete the wiring *or* close the loop. Code that's
  written but never read fails the Chesterton's-fence completion test.
- `.agents/skills/curate/` — pre-`.spellbook`-marker era, unmarked.
  `/tailor` preserves it by default, but it references
  `scripts/generate-embeddings.py` patterns that may have drifted.
  Audit-before-preserve target.
- `skills/deliver/` composition drift — the check-deliver-composition
  gate exists because this file keeps re-inlining phase-skill logic.
  Every re-run of /deliver leaves detritus; each /refactor pass on
  master should reassert the invariant that deliver is a composer.

### 3. Subagents — only when the territory is new

Launch subagents when you cannot already name the cut. If you already
know the cut is "delete `scripts/foo.sh` because no skill references
it" — that's `rg foo.sh skills/` + `git rm`. A subagent here is pure
overhead (executive protocol: "If the prompt to the subagent would be
mostly 'do this exact sed command,' don't spawn the subagent — run the
sed command.").

When the territory *is* new, parallel dispatch:

- **Deletion Hunter (Explore)** — grep for shim scripts, orphan
  references, SKILL.md files not reachable from any trigger, skills
  lacking `.spellbook` markers that `/tailor` doesn't own.
- **Bitter-lesson Auditor (Explore)** — scan SKILL.md bodies for
  generic Execution Stance blocks, "Phase 1/2/3" scaffolds, decision
  matrices a strong model infers, regex-over-agent-prose checks. This
  is the `8e1ed4c` persona: delete what the model already knows.
- **Rebuild Strategist (Plan)** — for one hotspot, sketch the
  from-scratch 50-line version. Measure gap.

### 4. Produce outcome

`--report-only` (default):

- One winning candidate, shape into `backlog.d/NNN-<slug>.md` with
  oracle (name a measurable metric — LOC, gate count, file count,
  cyclomatic — and target delta).
- Up to two runners-up as appendix.
- Do **not** sprout a `references/<repo-name>.md` sidecar. Repo brief
  invariant: "repo-specific content belongs in SKILL.md body."

`--apply`:

- One bounded cut. Verify `dagger call check --source=.`. Record
  residual risk.

## Required output

```markdown
## Refactor Report
Mode: feature | master
Target: <branch or hotspot>
Base: <branch> (feature only)

### Selected cut
<file/scope>  — <lines removed>  — <red flag cited>

### Runners-up (optional, max 2)
1. ...
2. ...

### Action
[commit sha + one-liner, or backlog.d/NNN file created]

### Verification
[dagger call check result; any gate-specific assessment]

### Residual risks
[what remains, why it waited]
```

## Spellbook-specific gotchas

- **The 683-line cut is the reference, not the ceiling.** If the
  cleanest version of a skill is 40 lines (`skills/flywheel/SKILL.md` is
  43), write the 40-line version. Do not compromise.
- **Do not touch `index.yaml`.** The pre-commit hook regenerates it
  from `skills/` and `agents/`. Manual edits are churn; the gate
  (`check-index-drift`) will catch you.
- **Do not touch vendored copies by hand.** `check-vendored-copies`
  enforces canonical sources. Edit the canonical file; let the mirror
  sync.
- **`harnesses/claude/settings.json` is COPIED, not symlinked.**
  Claude mutates it at runtime. If you cut from it, re-bootstrap
  after. Same for any other per-harness file the repo brief marks
  "copied."
- **No claim primitives.** `claims.sh`, `claim_acquire`, `claim_release`
  under `skills/` are banned by `check-no-claims`. If your
  "simplification" reintroduces coordination primitives, reverse.
- **Cross-harness first is a Red Line.** A cut that removes Pi support
  from a skill to slim it is a regression. Every skill must still work
  on Claude, Codex, AND Pi post-refactor.
- **Chesterton's fence, spellbook flavor.** Before deleting, complete:
  "I want to remove X. X was added in <sha>. The invariant it guarded
  was Y. Y is no longer true because Z." If you cannot cite the sha,
  `git log --all -S'<snippet>'` first.
- **Complexity moved ≠ removed.** Splitting `bootstrap.sh` into two
  files with the same total logic is not simplification. Same for
  extracting a helper that's called once.
- **Diminishing returns.** After two passes with only cosmetic
  deltas, stop. Next cut lives on a different hotspot.
- **Deliver composition gate.** If you're refactoring
  `skills/deliver/SKILL.md`, the post-state must still route
  `/code-review`, `/ci`, `/qa`, `/implement`, `/refactor`, `/shape`
  via trigger syntax only. Inlining = regression.

Deep methodology (survey-imagine-simplify, deletion-first hierarchy,
complexity metrics, Chesterton's fence, one-at-a-time rule) lives in
`references/simplify.md`.
