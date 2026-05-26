---
name: harness
description: |
  Build, maintain, evaluate, and audit spellbook's own primitives — skills
  under skills/<name>/SKILL.md, agents under agents/<name>.md, per-harness
  configs under harnesses/{claude,codex,pi,factory,gemini,shared}/, registry
  entries, and the 12 Dagger gates that enforce them.
  Use when: "create a skill", "add skill", "update skill",
  "eval skill", "lint skill", "validate this skill", "check this skill",
  "convert agent to skill", "sync externals", "update registry",
  "audit skills", "skill health", "dead skills", "harness hygiene",
  "improve the harness".
  Trigger: /harness, /skill, /primitive.
argument-hint: "[create|eval|lint|convert|sync|engineer|audit] [target]"
---

# /harness (spellbook self-edit)

This is spellbook — the harness library itself. Every operation here
mutates the source of truth: `skills/<name>/`, `agents/<name>.md`,
`harnesses/*/`, `registry.yaml`. The Dagger gate (`dagger call check
--source=.`) is the wall. Pre-commit regenerates `index.yaml`. Do not
hand-edit derived artifacts.

## Routing

| Intent | Reference |
|--------|-----------|
| Create a new skill or agent | `references/mode-create.md` |
| Eval a skill vs baseline | `references/mode-eval.md` |
| Lint a skill (14-gate preflight) | `references/mode-lint.md` |
| Convert agent ↔ skill (across harnesses) | `references/mode-convert.md` |
| Sync external sources via `registry.yaml` | `references/mode-sync.md` |
| Design harness improvements (hooks, gates, codification) | `references/mode-engineer.md` |
| Audit skill health and consolidation | `references/mode-audit.md` |
| Choose open-model roster defaults | `references/open-model-roster.md` |

If no argument, ask: "What do you want to do? (create, eval, lint, convert,
sync, engineer, audit)"

## Spellbook-specific load-bearing facts

- **Source of truth lives here.** Skills are `skills/<name>/SKILL.md`
  (plus optional `references/`, `scripts/`). Agents are
  `agents/<name>.md`. `harnesses/shared/AGENTS.md` is THE principles
  file and is symlinked (not copied) to every harness dir by
  `bootstrap.sh`.
- **`dagger call check --source=.` is the gate.** Fourteen parallel
  sub-gates (named by function in `ci/src/spellbook_ci/main.py`):
  `lint-yaml`, `lint-shell`, `lint-python`, `check-frontmatter`,
  `check-index-drift`, `check-vendored-copies`, `test-bun`,
  `check-exclusions`, `check-portable-paths`,
  `check-harness-install-paths`, `check-deliver-composition`,
  `check-no-claims`, `check-skill-evals`, `check-agent-roster`. Any new
  mechanism you add must survive all fourteen.
- **`scripts/check-frontmatter.py`** enforces SKILL.md ≤ 500 lines and
  required `name` + `description` fields (agents require the same two;
  no line cap). Missing or malformed frontmatter fails the gate.
- **`scripts/generate-index.sh`** regenerates `index.yaml` from
  `skills/*/SKILL.md` and `agents/*.md`. The pre-commit hook
  (`.githooks/pre-commit`) re-runs it whenever those paths change. Do
  not hand-edit `index.yaml`; `check-index-drift` will fail.
- **`scripts/check-harness-agnostic-installs.sh`** blocks
  Claude-specific install wording from `skills/seed/SKILL.md`,
  `skills/tailor/SKILL.md`, and `index.yaml`. Required phrases on both
  skills: `shared skill root` (or `shared repo-local skill layer` /
  `shared skill layer`) and a description of `.claude/skills/` as a
  `bridge`. Forbidden: `into .claude/ with no filtering or tailoring`,
  `per-repo set of skills and agents in .claude/`, and `Copy every
  skill in ... into .claude/skills/`.
- **`scripts/sync-external.sh`** fetches external skill repos listed in
  `registry.yaml` into `skills/.external/<alias>/` (gitignored,
  per-machine state). Refuses floating refs (`main`/`HEAD`/branch)
  unless the source declares `allow_floating: true` or the operator
  passes `--allow-floating`. Every non-default source MUST set
  `alias_prefix:` — silent collisions are load-bearing bugs.
- **`scripts/lint-external-skills.sh`** flags hardcoded `/Users/`,
  `$HOME/.claude`, or `../../../` tree-escapes in installed externals.
  Use `--strict` in CI; advisory by default.
- **`bootstrap.sh` has two modes.** Symlink mode (local checkout found
  via `resolve_spellbook_dir` — honors `$SPELLBOOK_DIR`, then known
  canonical locations; worktrees fall through unless explicitly selected)
  symlinks `skills/tailor` and `skills/seed` into each detected
  harness's skills dir, plus all `agents/*.md` into the harness agents
  dir. Download mode (`curl | bash`) pulls the same minimal set from
  GitHub. `harnesses/claude/settings.json` is COPIED and sanitized
  (dead-hook-path pruning), NOT symlinked — Claude mutates it at
  runtime. Changes require re-bootstrap.
- **Global install is minimal on purpose.** Only `/tailor` and `/seed`
  are globally symlinked; every other skill is installed per-repo by
  `/tailor` into `.agents/skills/` (canonical) with `.claude/skills/`
  and Codex/Pi mirrors as bridges.

## Cross-harness first (the Red Line)

Every skill, agent, hook, setting, or lint rule MUST work on Claude,
Codex, AND Pi. The filesystem + SKILL.md is the primary layer — every
harness scans its skills dir at startup. Anchoring a design on Claude
`enabledPlugins`, Codex `/plugins`, or Pi's unique features alone is a
bug. Prior art living in this repo: `harnesses/pi/settings.json`'s
`skills[]` glob ruleset (`+skills/...`, `!skills/**`) — filesystem-
level allow/deny that works by construction.

When a mechanism legitimately needs runtime toggling, emit per-harness
artifacts from one source (one manifest in-repo → Claude `plugin.json`
+ Codex `plugin.json` + Pi glob, rendered deterministically). If you
cannot answer "what does this do on Codex?", the design is incomplete.

## Agent-format translation

Claude and Codex-classic agents use YAML frontmatter (`name:`,
`description:`, `tools:`, optional `model:`, `disallowedTools:`) with
markdown body. Codex TOML profiles (`harnesses/codex/config.toml`) are
execution config, not personas — don't conflate. When converting, see
`references/mode-convert.md`; frontmatter field-mapping differs across
harnesses and silent drops (e.g. `disallowedTools`) lose behavior.

## Skill design quality standard

These govern every `/harness create|lint|eval`. They are the same
standard enforced by the Dagger gate where a gate exists, by prose
review otherwise.

0. **Roster floor workflows.** Core workflow skills name the two-or-more
   roster-member floor for substantive work when `.spellbook/agents.yaml`
   exists, while keeping schema details and receipt mechanics centralized in
   the roster scripts and shared AGENTS doctrine.
1. **One skill = one domain, 1–3 workflows.** Five workflows is a
   refactor signal. 26 skills is the current steady state — justify
   additions against description tax.
2. **Token budget: 3k target, 5k ceiling.** `scripts/check-frontmatter.py`
   enforces the 500-line SKILL.md cap. Exemplars:
   `skills/flywheel/SKILL.md` (~43 lines — composer, states invariants),
   `skills/shape/SKILL.md` (~117 lines — invariant-first at larger scale).
3. **Mode content in `references/mode-*.md`.** Mandatory when a skill
   has >3 modes. Thin SKILL.md with a routing table; mode bodies under
   `references/`. This skill is the reference form.
4. **Encode judgment, not procedures.** If the model can derive the
   steps, delete the skill. Gotcha lists outperform happy-path prose.
   Exemplars: `skills/diagnose/SKILL.md` (routing table IS the skill),
   `skills/settle/SKILL.md` (mode detection IS the skill).
5. **Self-contained.** Scripts source only paths under `skills/<name>/`.
   Resolve `SCRIPT_DIR` via `readlink -f`; source libs from
   `$SCRIPT_DIR/lib/`; anchor state roots (cycles, locks, backlog) to
   the *invoking project's* `git rev-parse --show-toplevel`, never the
   skill's install dir. The canonical test is symlink-install into a
   foreign project and invoke from there.
6. **Cross-harness first.** See Red Line above. A Claude-only
   mechanism is broken by construction.
7. **Description is the trigger.** Frontmatter `description:` is
   assertive (`"Use when: '...'"` with concrete phrases), not
   aspirational. If a skill doesn't fire, the description is wrong.
8. **Prose for an intelligent reader.** You are writing for an agent,
   not a Python interpreter. Phase-0/Phase-N flowcharts, exit-code
   routing tables, and deterministic scoring scripts inside SKILL.md
   are smells. Strip to invariants + a paragraph of "shape of the

   work." The agent fills sequencing.
9. **No `references/<repo-name>.md` sidecars.** Repo-specific body
   content belongs in SKILL.md itself. Stack-topic references
   (`references/convex-patterns.md`) are fine.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/harness` starts substantive catalog,
doctrine, lint, eval, conversion, or sync work by dispatching two or more
available roster members. Use lanes for cross-harness parity, simplicity,
gate impact, and regression risk. The lead owns the final harness mutation,
verification, and receipts. Direct lead-only harness work is limited to
mechanical command execution, emergency unblocks, explicit user-forbidden
delegation, or an explicit waiver when fewer than two roster members are
available.

## Composition discipline (the `/deliver` rule)

`check-deliver-composition` forbids `skills/deliver/SKILL.md` from
inlining phase-skill internals. `/deliver` composes atomic phase skills
via their trigger syntax — `/code-review`, `/ci`, `/qa`, `/implement`,
`/refactor`, `/shape`. Forbidden inline: `dagger call check`, `bunx
playwright`, direct bench-agent dispatch (`Agent("critic"|"ousterhout"|...)`).
When designing new composers, follow the layering in
`references/mode-engineer.md`: leaf → composer → outer-loop.

## Dropped primitives (regression guard)

`check-no-claims` forbids `claims.sh`, `claim_acquire`, and
`claim_release` anywhere under `skills/`. Dropped per `backlog.d/032`.
Do not reintroduce.

## Bootstrap + install invariants

- Minimal global set: only `/tailor` and `/seed` are symlinked into
  `~/.{claude,codex,pi}/skills/`. Everything else is per-repo.
- `/tailor` writes into `.agents/skills/<name>/` (canonical shared
  root); `.claude/skills/`, `.codex/skills/`, and Pi's `skills[]`
  configuration point at it (bridge layer). The `check-harness-install-paths`
  gate enforces this wording on `skills/seed` and `skills/tailor`.
- `harnesses/claude/settings.json` is COPIED (not symlinked) because
  Claude mutates it at runtime. Changes require re-bootstrap.
- Broken symlinks to `$SPELLBOOK/` are surfaced by
  `verify_no_broken_spellbook_symlinks` in `bootstrap.sh` — fix them at
  source, don't paper over.

## Registry discipline

`registry.yaml` declares external skill sources. Load-bearing rules
(enforced by `scripts/sync-external.sh`):

- Every non-default source declares `alias_prefix:`. Same prefix is
  allowed only when skill-name sets are provably disjoint (the two
  `vercel-labs/*` entries). Same prefix + same name = fatal.
- `pin: <full-sha>` is preferred. `ref: main/HEAD/<branch>` is
  refused unless `allow_floating: true`. TODOs in the file that say
  "pin after first sync" mean exactly that — finish the TODO.
- `skills/.external/` is gitignored. Reproducibility lives in
  `registry.yaml`. Never commit sync output.
- Sources may need multiple entries when upstream scatters skills
  across paths (see `vercel-labs/agent-browser` at both `skills/` and
  `skill-data/`).

## Gotchas

- Editing `index.yaml` by hand fails `check-index-drift`. Let
  pre-commit regenerate it via `scripts/generate-index.sh`.
- Writing a SKILL.md > 500 lines fails `check-frontmatter`. Extract
  to `references/mode-*.md` before the gate catches it.
- Adding a hardcoded `/Users/<name>/` outside `harnesses/claude/` or
  `.claude/hooks` fails `check-portable-paths`. Use
  `$HOME`/`git rev-parse --show-toplevel` instead.
- Describing seed/tailor installs as "copy into `.claude/skills/`"
  fails `check-harness-install-paths`. Use "shared skill root" + a
  bridge clause.
- Inlining `dagger call check` or direct bench-agent dispatch in
  `skills/deliver/SKILL.md` fails `check-deliver-composition`. Compose
  via `/ci`, `/code-review`, `/qa`, etc.
- Writing `claim_acquire` / `claim_release` / `claims.sh` under
  `skills/` fails `check-no-claims`. These were dropped per 032.
- Scripts that `source $REPO_ROOT/…` or `../../…` break the moment the
  skill is symlink-installed into a foreign repo. Use `$SCRIPT_DIR/lib/`.
- Anchoring a new mechanism on one harness's runtime feature is the
  Red Line violation. Primary layer is filesystem + SKILL.md.
- Adding `disable-model-invocation: true` to a skill makes it
  user-only. The two global skills (`tailor`, `seed`) rely on model
  invocation in consuming repos — don't break them accidentally.
- `harnesses/claude/settings.json` changes only take effect after
  `./bootstrap.sh` re-runs. Symlinked configs propagate instantly;
  this one doesn't.
- `skills/.external/` is per-machine state. Don't write skills that
  depend on a specific external being present — `registry.yaml` is
  the contract, not a runtime guarantee.
- Writing `references/spellbook.md` or any `references/<repo>.md`
  sidecar is the forbidden sewn-on-sleeve pattern. Put repo-specific
  content in SKILL.md body.
- Regexes over agent prose and semantic workflow DSLs wrapped around
  general-purpose agents are strong smells. If the harness is parsing
  free-form agent output with regex, the boundary is wrong.

## Evaluation loop

`/harness eval` spawns matched baseline + with-skill sub-agents on a
representative prompt, then a critic sub-agent compares. Marginal
improvement means delete the skill — the description tax isn't free
(~100 tokens per skill × every conversation). Eval prompts live at
`skills/<name>/evals/` in the skill dir and rerun after changes. See
`references/mode-eval.md`.
