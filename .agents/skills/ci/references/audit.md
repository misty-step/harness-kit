# CI Audit Rubric — Spellbook

Spellbook's pipeline exists and is healthy: `dagger call check --source=.`
is defined by `ci/src/spellbook_ci/main.py` with 14 parallel gates. An
audit here does **not** ask "should we have CI" — it asks "does the gate
set still cover what's being added to the repo?"

## What the Audit Is Scoped To

Four questions, in order:

1. **Is any new signal escaping the gate set?** A new failure mode
   (hardcoded path, generator drift, new exclusion pattern, new
   inlined-phase call) that no existing gate catches.
2. **Is any existing gate weak?** Its allow/deny list has grown stale,
   its thresholds lag the code, or its error reporting is opaque.
3. **Is heal coverage aligned with gate set?** A new lint-style gate
   that should be healable needs entry in `HEALABLE_GATES`.
4. **Are the hooks still firing?** Pre-commit, pre-merge-commit,
   pre-push — each must actually execute what it claims.

Everything outside those four questions is out of scope for `/ci`.
"Should coverage be gated?" is a design question for `/shape`.

## Gate-by-Gate Audit Checks

### `lint-yaml`

- New `*.yaml`/`*.yml` files at depth >2? The current gate only
  scans `find . -maxdepth 2`. Deep YAML (e.g. under `skills/<name>/`)
  isn't covered. If the repo grows deep YAML, the gate needs depth
  raised or a separate gate.
- Any YAML generated via templating with `$VAR` unescaped? `safe_load`
  won't catch that — consider a schema validator.

### `lint-shell`

- Scripts under `ci/` are ignored by design. If a shell script is
  added there, it must be self-hosted (Dagger container).
- `shellcheck --severity=error` skips warnings. Strengthening = lower
  severity, but only if the repo is ready.

### `lint-python`

- `py_compile` catches syntax only, not type / import-resolution. If
  the repo grows meaningful Python surface (beyond `scripts/` one-
  offs), introduce `ruff check` or `mypy` as a separate gate rather
  than overloading `lint-python`.

### `check-frontmatter`

- `scripts/check-frontmatter.py` enforces: required fields, description
  length limits, line count caps. If a new frontmatter field becomes
  load-bearing (e.g. `installed-by` for tailor-owned markers), extend
  the script.

### `check-index-drift`

- Diffs committed `index.yaml` vs regenerated, minus `# Generated:`
  timestamp line. If the generator output format changes (new field,
  reordering), the gate catches it — but the fix is regenerating, not
  altering the gate.

### `check-vendored-copies`

- Today covers the vendored copies in `scripts/check-vendored-copies.sh`.
  When a new canonical/vendored pair is introduced (e.g. a skill
  vendored into `harnesses/claude/`), the script must learn about it
  or the drift is invisible.

### `test-bun`

- Only `skills/research/` runs bun tests. If another skill adopts
  TypeScript, decide: expand `test-bun` to a glob, or add a separate
  gate per skill. Keep gates atomic when possible.

### `check-exclusions`

Denies `@ts-ignore`, `@ts-expect-error`, `as any`, `: any`,
`eslint-disable` (without `--` justification), `.skip()`, `xit(`,
`xdescribe(`. Audit questions:

- Is a new suppression idiom creeping in? (`as unknown as`, `# type: ignore`,
  `pytest.mark.skip`). Add to denylist if so.
- Are new file globs in scope? Currently: `.ts/.tsx/.js/.jsx/.py`.
  Add `.mjs`/`.cjs` if the repo adopts them.
- Is the `SKIP` set stale? `hooks/`, `coverage/`, `dist/`, `.next/`,
  `node_modules/` — drop any that no longer apply.

### `check-portable-paths`

Denies `/Users/<name>/` and `C:\Users\<name>\` outside
`.claude/hooks/`, `coverage/`, `.next/`, `dist/`, `harnesses/claude/`.

- New generated file with embedded paths? Add to `ALLOW` **only** if
  the file must be machine-local; otherwise rewrite to portable form
  and fail the gate.
- Glob covers `.sh`, `.bash`, `.zsh`, `Makefile`, `.env*`. Add `.fish`
  / `.ps1` / `.toml` if relevant to the repo.

### `check-harness-install-paths`

`scripts/check-harness-agnostic-installs.sh` rejects Claude-only
install instructions in `/seed` / `/tailor` copy. If a new workflow
skill starts emitting install instructions, the script's scope likely
needs extending.

### `check-deliver-composition`

Denies inside `skills/deliver/SKILL.md`:

- `source scripts/lib/claims.sh`, `claim_acquire`, `claim_release`
- Raw `dagger call check`
- Raw `bunx playwright` / `npx playwright`
- Direct bench-agent dispatch (`Agent('critic' | 'ousterhout' |
  'carmack' | 'grug' | 'beck')`, `subagent_type='...'` variants).

Audit: when a new phase skill is added to the composer contract,
decide whether inlining its internals should be denied here. Usually
yes.

### `check-no-claims`

Regression guard from `backlog.d/032`. If the backlog item re-opens
(claim primitives re-introduced for a different reason), this gate
must be removed deliberately, not bypassed. Until then, any hit is a
real regression.

## Hook Health

- `.githooks/pre-commit` — regenerates `index.yaml`, runs
  `check-harness-agnostic-installs.sh`, blocks `/deliver` state
  force-adds. Audit: check `git config core.hooksPath` points at
  `.githooks`. If developers disabled hooks, the gate still catches
  issues at CI, but local signal is gone.
- `.githooks/pre-merge-commit` — verdict gate for non-FF merges. If
  `/code-review` isn't producing verdicts consistently, this hook
  becomes a drag. Escape via `SPELLBOOK_NO_REVIEW=1`.
- `.githooks/pre-push` — runs `dagger call check` before push; skips
  cleanly when Docker/Dagger absent. Silent skip is a signal; surface
  that the pre-push isn't actually gating in those environments.
- `.githooks/post-commit` / `post-merge` / `post-rewrite` — re-run
  `./bootstrap.sh` when skill/agent content changed.

## Severity Matrix

| Severity | Meaning                                                    | Action              |
|----------|------------------------------------------------------------|---------------------|
| high     | A new failure mode is uncovered by any gate                | Add gate inline     |
| med      | A gate's allowlist / denylist is stale                     | Patch gate inline   |
| low      | Error message clarity, speed, polish                       | Defer to backlog    |

Mechanical strengthenings apply directly. A new-gate proposal that
changes a behavioral contract (e.g. raising `--severity` on
shellcheck from `error` to `warning`) is a product decision — surface
and wait for ratification.

## Anti-Findings

Things that look like gaps in a generic repo but aren't here:

- **"No unit tests"** — this repo is skills + agents + configs; the
  `test-bun` gate covers the only code that needs unit testing
  (`skills/research/`). "Tests for SKILL.md" is a category error.
- **"No coverage gating"** — there's no meaningful coverage floor for
  judgment-encoding markdown. Don't add one.
- **"Pipeline doesn't run on Windows"** — Dagger containers do, but
  this repo targets macOS/Linux developers and `check-portable-paths`
  already flags Windows-style hardcoded paths.
- **"No GitHub Actions deploy"** — by design. Local-first CI is the
  product. Proposing Actions is a scope change, not a gap.
