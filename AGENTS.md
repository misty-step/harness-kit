# AGENTS.md — Spellbook

Router for AI agents working inside the spellbook repo. See
`harnesses/shared/AGENTS.md` for the universal principles file (symlinked
to every harness); this file is the **spellbook-specific** index.

## Stack & boundaries

| Layer | Lives at | Owns |
|---|---|---|
| **Skills** | `skills/<name>/SKILL.md` + `references/` + `scripts/` | Judgment. <500-line SKILL.md; frontmatter-triggered. |
| **Agents** | `agents/<name>.md` | Scoped personas with tool restrictions and model pins. |
| **Harness configs** | `harnesses/{claude,codex,pi,factory,gemini,shared}/` | Per-harness hooks, settings, principles. `harnesses/shared/AGENTS.md` symlinks into every harness. |
| **CI module** | `ci/src/spellbook_ci/main.py` | 13 Dagger gates + heal loop. Python 3.12. |
| **Bootstrap** | `bootstrap.sh` | Installs minimal globals (`GLOBAL_SKILLS=(tailor seed)` + all agents) via symlink OR download mode. Per-repo subsets are `/tailor` / `/seed`'s job. |
| **Scripts** | `scripts/` | Shell + Python utilities: frontmatter check, index regen, embeddings, external sync, harness lint. |
| **Backlog** | `backlog.d/NNN-*.md` (open), `backlog.d/_done/` (closed), `.spellbook/deliver/<ulid>/` (runtime state, gitignored) | Shaped work ready to build. Single source of truth; closure via `Closes-backlog:` trailers on squash-merge commits (handled by `/ship`). |

## Ground-truth pointers

Stale training data lies about these — always read the file:

- **`ci/src/spellbook_ci/main.py`** — exact set of gates, what each
  enforces, which are healable.
- **`harnesses/shared/AGENTS.md`** — the principles file. Red flags,
  doctrine, anti-patterns. Cited verbatim by `/code-review`.
- **`bootstrap.sh:271`** — `GLOBAL_SKILLS=(tailor seed)`. Anything you
  thought was globally symlinked but isn't in this list: it isn't.
- **`.githooks/pre-commit`** — what runs automatically on every commit
  (index regen, harness-agnostic install wording, `.spellbook/deliver/`
  force-add block).
- **`.githooks/pre-merge-commit`** — verdict gate for non-FF merges.
  Escape: `SPELLBOOK_NO_REVIEW=1`.
- **`index.yaml`** — derived; never edit manually (pre-commit hook
  regenerates from `scripts/generate-index.sh`).
- **`registry.yaml`** — external-skill source registry with
  `alias_prefix` doctrine.
- **`.spellbook/repo-brief.md`** — `/tailor`'s shared spine for every
  rewriter in this repo. Regenerated each `/tailor` run.

## Invariants

- **Cross-harness first (Red Line).** Every new mechanism — skill, hook,
  setting, lint — must work on Claude Code, Codex, AND Pi. Anchoring a
  design on one harness's unique feature is a bug. Prior art:
  `harnesses/pi/settings.json:skills[]` globs.
- **Thin harness, strong models.** Don't compensate for weak models with
  scaffold. `skills/flywheel/SKILL.md` (43 lines) is the reference.
- **Skills are self-contained.** No `../..`, no `$REPO_ROOT/…` sourcing.
  Libs resolve via `readlink -f` + `$SCRIPT_DIR/lib/…`. State roots
  anchor to the *invoking* project's `git rev-parse --show-toplevel`,
  not the skill's install dir. Symlink-install + invoke from a foreign
  project is the canonical self-containment test.
- **No `index.yaml` edits.** Pre-commit regenerates it.
- **No `references/<repo-name>.md` sidecar files.** Spellbook-specific
  content belongs in SKILL.md body. Stack-specific references under their
  own topic (e.g. `references/cross-harness.md`) are fine.
- **`.spellbook/deliver/<ulid>/state.json` + `receipt.json` are agent-
  written, never human-edited, never committed.** Gitignored; pre-commit
  hook blocks force-adds.
- **Base branch: `master`.** Topic branches: `feat/*`, `fix/*`, `chore/*`,
  `docs/*`, `refactor/*`, `backlog/*`, `doctrine/*`.
- **No claim-coordination primitives under `skills/`.** `claims.sh`,
  `claim_acquire`, `claim_release` were dropped per `backlog.d/_done/
  032-deliver-inner-composer.md`; regression guarded by
  `check-no-claims`.
- **`/deliver` must compose atomic phase skills via trigger syntax.**
  Raw `dagger call check`, direct bench-agent dispatch, or inlined
  phase internals inside `skills/deliver/SKILL.md` fail
  `check-deliver-composition`.
- **`harnesses/claude/settings.json` is COPIED by bootstrap**, not
  symlinked (Claude mutates it at runtime). Changes require re-bootstrap.

## Gate contract

**The load-bearing gate is `dagger call check --source=.`** — 13 parallel
sub-gates, all must pass to ship:

| Gate | What it enforces |
|---|---|
| `lint-yaml` | YAML parseability |
| `lint-shell` | `shellcheck --severity=error` on non-`ci/` shell scripts |
| `lint-python` | `py_compile` on non-`ci/` Python |
| `check-frontmatter` | Required fields + line limits (`scripts/check-frontmatter.py`) |
| `check-index-drift` | `index.yaml` matches `scripts/generate-index.sh` output |
| `check-vendored-copies` | Vendored copies match canonical sources |
| `test-bun` | `bun test` under `skills/research/` |
| `check-exclusions` | No `@ts-ignore`, `.skip()`, `eslint-disable`, `as any` |
| `check-portable-paths` | No hardcoded `/Users/<name>/` or `C:\Users\` outside `harnesses/claude/` + `.claude/hooks` |
| `check-harness-install-paths` | No Claude-only install wording for `/seed` or `/tailor` |
| `check-deliver-composition` | `skills/deliver/SKILL.md` composes atomic phase skills, never inlines |
| `check-no-claims` | No `claims.sh` / `claim_acquire` / `claim_release` under `skills/` |
| `check-skill-evals` | Existing `skills/<name>/evals/` suites have README, case, and grader files |

**Self-heal:** `dagger call heal --source=. --model=gpt-4.1 --attempts=2`
repairs one failing lint-style gate (yaml / shell / python / frontmatter).
Non-heal-eligible failures escalate to human review.

**Enforced where:**
- Pre-commit hook: index regen, harness-agnostic wording,
  `.spellbook/deliver/` force-add block.
- Pre-merge-commit hook: verdict gate for non-FF merges.
- Pre-push hook: last chance for gate run locally.
- Post-commit / post-merge / post-rewrite: auto re-run `./bootstrap.sh`
  when skills/agents change.
- Humans / agents: must run `dagger call check --source=.` before
  merging. `/ci` owns the gate; other skills cite but don't re-implement.

## Known-debt map

Tracked shapes (open, `backlog.d/NNN-*.md`):

| Issue | Concern | Tracker |
|---|---|---|
| Review-score feedback loop | `.groom/review-scores.ndjson` wired but operationally empty | `backlog.d/023-review-score-feedback-loop.md` |
| Offline evidence storage | evidence files need durable local storage | `backlog.d/024-offline-evidence-storage.md` |
| Dagger merge gate | the gate should block merges directly, not just pre-commit | `backlog.d/025-dagger-merge-gate.md` |
| Multi-machine sync | harness state across machines | `backlog.d/026-multi-machine-sync.md` |
| End-to-end offline validation | full offline run coverage | `backlog.d/027-end-to-end-offline-validation.md` |
| Harness auto-tune (GEPA) | adaptive harness tuning — parked until ≥20 flywheel cycles produce signal | `backlog.d/031-harness-auto-tune-gepa.md` |
| Legacy `curate` skill triage | `.agents/skills/curate/` + `.claude/skills/curate/` predate `.spellbook` markers | `backlog.d/046-curate-skill-triage.md` |
| `/tailor` external-skill install | externals declared in `registry.yaml` aren't on any harness skill-discovery path; three-mode install (copy / rewrite / symlink) | `backlog.d/047-tailor-external-skill-install.md` |

Hot files (recent churn — check `git log` before editing):
- `skills/tailor/SKILL.md` — actively evolving on `feat/tailor-harden`.
- `skills/deliver/SKILL.md` — composition-lint-gated.
- `ci/src/spellbook_ci/main.py` — add gates here.
- `bootstrap.sh` — symlink/download modes; changes require re-bootstrap.
- `harnesses/shared/AGENTS.md` — principles; `doctrine(harness): …`
  commit scope.

Backlog closure: `/groom tidy` sweeps recent master commit trailers
(`Closes-backlog:` / `Ships-backlog:`) against active `backlog.d/` to
detect stale tickets; `/ship` archives to `backlog.d/_done/` on merge.

## Harness index — skills

Installed in this repo-local harness at `.agents/skills/<name>/` with
`.claude/skills/`, `.codex/skills/`, `.pi/skills/` symlink bridges:

| Skill | Role here |
|---|---|
| `/research` | Web research, multi-AI delegation. Universal — use for any ecosystem investigation. |
| `/groom` | Backlog management + problem-diamond divergence. Input surface: `backlog.d/` + recent master commit trailers (`Closes-backlog:` / `Ships-backlog:`). |
| `/office-hours` | YC-style interrogation of a raw idea before it enters the backlog. |
| `/ceo-review` | Dialectical premise-and-alternatives audit of a shape or plan. |
| `/reflect` | Session retrospective; distills learnings into hook / rule / skill mutations. |
| `/shape` | Solution-diamond divergence → `backlog.d/NNN-<slug>.md`. Output: shaped ticket (Priority / Estimate / Goal / Design / Oracle / Non-Goals). Cross-harness Red Line is mandatory. |
| `/implement` | TDD atomic build. Green signal is `dagger call check --source=.`. Concrete test surfaces: `ci/tests/` (pytest), `skills/research/` (bun). |
| `/qa` | Non-browser verification for this library: Dagger gate, skill eval suites, generated index drift, symlink bridge topology, and command-level smoke evidence. |
| `/demo` | Evidence capture for library/harness changes: terminal transcripts, diff summaries, gate excerpts, and release-note blurbs rather than screenshots or videos. |
| `/code-review` | Marshal protocol with philosophy bench (ousterhout / carmack / grug / beck / critic). Tier 0 is Dagger gates — don't duplicate them. Cites `harnesses/shared/AGENTS.md` red flags verbatim. |
| `/ci` | Owns the gate. Names all 13 sub-gates; knows heal semantics. Only skill permitted to invoke `dagger call check` directly. |
| `/refactor` | Deletion-first. Past exemplars: `68e276b` (tailor −683 lines), `7ccd00d` (flywheel → 43 lines), `f91f1c4` (80 globals → 2). |
| `/diagnose` | Investigate gate failures, hook behavior, generated artifact drift, symlink topology, and harness regressions before editing. |
| `/monitor` | Watch Spellbook signals: Dagger/CI regressions, stale backlog closure, skill eval drift, external registry drift, and config-schema breakage. |
| `/settle` | Git-native merge. GitHub PRs are optional. Verdict gate (`.githooks/pre-merge-commit`) on non-FF merges; escape `SPELLBOOK_NO_REVIEW=1`. Closes by `git mv backlog.d/NNN-*.md _done/`. |
| `/ship` | Final mile: archive backlog files into `_done/`, preserve `Closes-backlog:` / `Ships-backlog:` trailers, land the merge, and invoke bounded `/reflect`. |
| `/yeet` | Conventional commits split. Observed types: `feat|fix|refactor|docs|chore|test|backlog|doctrine`. Special: `backlog(NNN): …`, `doctrine(harness): …`. Index regen folds into the skill commit; never a standalone `chore: regen`. |
| `/deliver` | Inner-loop composer: `/shape` → `/implement` → clean loop of (`/code-review` + `/ci` + `/refactor` + `/qa`). Composition-lint-gated; no raw phase internals. |
| `/flywheel` | Outer-loop orchestrator for Spellbook maintenance: pick work, run deliver/yeet/settle/ship/monitor, then continue from the shipped lifecycle result. |
| `/harness` | Create / eval / lint / convert / sync / engineer / audit on the catalog. Names `scripts/check-frontmatter.py`, `scripts/generate-index.sh`, `scripts/check-harness-agnostic-installs.sh`, `scripts/lint-external-skills.sh`, `scripts/sync-external.sh`. |

**Skipped skills** (concrete absence):

| Skipped | Reason |
|---|---|
| `/deploy` | No `vercel.json`, no `fly.toml`, no `.github/workflows/*deploy*`, no `Dockerfile` for deploy. `bootstrap.sh` is library install, not deploy. |
| `/a11y` | No UI / web surface. |
| `/agent-readiness` | One-off audit, not a workflow skill. |
| `/deps` | Minimal dep surface (`dagger-io` only). |
| `/model-research` | No in-repo LLM model selection work. |
| `/tailor` + `/seed` | Canonical source lives here (`skills/tailor/`, `skills/seed/`). Installed globally via `bootstrap.sh`. A repo-local copy would create symlink ambiguity — do not install. |

**Legacy unmarked skill** (preserved; see `backlog.d/046`):
- `/curate` at `.agents/skills/curate/SKILL.md` + `.claude/skills/curate/
  SKILL.md` — predates `.spellbook` marker era. Triage pending.

## Harness index — agents

Installed at `.claude/agents/<name>.md`:

| Agent | Role | Use for |
|---|---|---|
| `planner` | Decomposes work into specs. Writes context packets; does not implement. | Architecture / design problems. |
| `builder` | Implements specs via TDD. Heads-down execution. | Spec → green tests on feature branch. |
| `critic` | Evaluates output against grading criteria. Skeptical by default. | `/code-review` core; `/tailor` rewrite adjudication. |
| `ousterhout` | Deep modules + information hiding. | Lens on API design, hidden coupling, shallow wrappers. |
| `carmack` | Direct implementation + shippability. "Focus is deciding what NOT to do." | Lens on scope discipline, over-engineered shapes. |
| `grug` | Complexity demon hunter. | Lens on unnecessary scaffolding, semantic workflow DSLs. |
| `beck` | TDD + simple design. "Red. Green. Refactor." | Lens on test-first discipline (limited utility for markdown-only changes). |
| `cooper` | Classicist testing discipline. | Lens on internal mocks, fake-vs-mock boundaries, and behavior-level test design. |

**Skipped agents** (no UI surface): `a11y-auditor`, `a11y-fixer`, `a11y-critic`.

## Operating loop

```
backlog.d/NNN → /groom → /shape → /deliver → /settle → /ship → _done/
                 └─ problem diamond   └─ solution diamond   └─ inner loop   └─ merge-ready   └─ close + reflect
```

Outer-loop orchestration (`/flywheel`) is installed here because Spellbook
is the source repo for the harness loop. `/deploy` remains absent because
there is no deploy target. To shape a new feature end-to-end: use `/shape`
to produce `backlog.d/NNN-*.md`, `/deliver` to reach merge-ready,
`/settle` to validate, and `/ship` to archive backlog state and reflect.
