# Spellbook Codebase Map

Spellbook is a portable primitive library for coding-agent workflows. It
ships markdown-first skills, specialized agent personas, harness-specific
runtime config, and the local gates that keep those artifacts portable across
Claude Code, Codex, and Pi.

The shortest accurate mental model is:

```text
skills + agents + shared doctrine
        |
        v
bootstrap installs minimal globals
        |
        v
/tailor or /seed installs repo-local subsets
        |
        v
backlog.d work flows through /groom -> /shape -> /deliver -> /settle -> /ship
        |
        v
dagger call check --source=. guards the catalog and harness contracts
```

## Source Of Truth

Spellbook has five primary source surfaces.

| Surface | Path | Owns |
|---|---|---|
| Skills | `skills/<name>/SKILL.md` with optional `references/`, `scripts/`, `evals/` | Workflow judgment and triggerable agent guidance. |
| Agents | `agents/<name>.md` | Reusable personas such as planner, builder, critic, and the review bench. |
| Repo-local harness | `.agents/skills/` | The canonical, version-controlled harness used to build Spellbook itself. |
| Harness configs | `harnesses/{claude,codex,pi,shared}/` | Per-runtime settings plus shared doctrine. |
| Work queue | `backlog.d/NNN-*.md` and `backlog.d/_done/` | Shaped work, status, oracle, and closure history. |

Generated or runtime surfaces are deliberately secondary:

- `index.yaml` is generated from skills and agents by `scripts/generate-index.sh`.
  Do not edit it by hand.
- `.agents/skills/` is source, not scratch. It is hidden only by Unix naming
  convention; audits must use hidden-aware searches such as
  `rg --hidden -g '!.git/**'`.
- `.claude/`, `.codex/`, and `.pi/` are runtime bridge directories.
- `.spellbook/deliver/<ulid>/` is runtime state for `/deliver`; it is
  agent-written, gitignored, and blocked from forced commits.
- `.spellbook/repo-brief.md` is `/tailor`'s durable rewrite spine for this
  repo.
- `skills/.external/` is a local cache populated from `registry.yaml`; it is
  not redistributable source.

## How It Works

### Skills

A skill is a small markdown module with frontmatter. The frontmatter
description is the trigger surface; the body encodes judgment, invariants, and
gotchas; references hold details that should load only when needed.

The codebase assumes skills stay thin. `scripts/check-frontmatter.py` enforces
required frontmatter and a 500-line cap for first-party `SKILL.md` files. The
design pressure is intentional: if a skill has many modes, the root
`SKILL.md` becomes a router and detailed mode bodies move under
`references/mode-*.md`.

### Agents

Agents are scoped personas rather than workflow engines. The core set is:

- `planner`: decomposes a problem into a buildable spec.
- `builder`: implements against a spec.
- `critic`: evaluates output against criteria.
- `ousterhout`, `carmack`, `grug`, `beck`, `cooper`: review lenses for module
  depth, scope discipline, simplicity, TDD, and testing style.

The repo treats planner -> builder -> critic as the core execution triad, with
the philosophy bench used for review rather than implementation.

### Harness Layer

Spellbook's primary portability layer is the filesystem: the same `SKILL.md`
and agent markdown files can be discovered by multiple harnesses. Runtime
features are wrappers around that layer, not the architecture.

`bootstrap.sh` installs only a minimal global set:

- global skills: `/tailor` and `/seed`
- global agents: every `agents/*.md`

Every other skill is installed per target repo by `/tailor` or `/seed`.
This prevents a machine-wide skill catalog from pretending every repo needs
every primitive.

Local bootstrap prefers symlinks to a stable checkout. Remote bootstrap
downloads the same minimal global set from GitHub. Claude settings are copied,
not symlinked, because Claude mutates `settings.json` at runtime.

### Registry And Externals

`registry.yaml` declares external skill sources. It is not the runtime catalog.
It feeds:

- `scripts/sync-external.sh`, which syncs selected upstream skills into
  `skills/.external/<alias>/`
- embedding/index tooling that can search first-party and external skill
  material

The registry encodes one strong assumption: external namespaces must be
explicit. Non-default sources need `alias_prefix` so upstream skill names do
not silently collide.

### CI And Hooks

The load-bearing check is:

```bash
dagger call check --source=.
```

The Dagger module in `ci/src/spellbook_ci/main.py` runs 13 gates in parallel:
YAML, shell, Python, frontmatter, index drift, vendored copies, Bun tests for
`skills/research`, exclusion-pattern scans, portable paths, harness-agnostic
install wording, `/deliver` composition, dropped claim primitives, and skill
eval-suite structure.

`dagger call heal --source=. --model=gpt-4.1 --attempts=2` can repair one
lint-style failure class: YAML, shell, Python, or frontmatter. Other failures
are meant to be diagnosed, not papered over.

Git hooks reinforce the same contracts:

- `pre-commit` regenerates `index.yaml`, blocks `.spellbook/deliver/` state
  commits, and checks harness-agnostic install wording.
- `pre-push` runs the Dagger check when Dagger and Docker are available.
- `pre-merge-commit` enforces review verdicts for non-fast-forward merges,
  with `SPELLBOOK_NO_REVIEW=1` as the explicit escape hatch.
- post-commit/post-merge/post-rewrite hooks rerun `bootstrap.sh` when local
  skill or agent changes should propagate.

## Operating Loop

The repo is designed around an agentic software delivery loop:

```text
backlog.d/NNN
  -> /groom      problem framing and backlog hygiene
  -> /shape      solution framing and executable oracle
  -> /deliver    /implement plus clean loop of /code-review, /ci, /refactor, /qa
  -> /settle     merge-readiness and git-native review/verification
  -> /ship       backlog archive, trailers, merge, and bounded /reflect
  -> _done
```

Use `/flywheel` when the desired behavior is to keep cycling through that loop
across the queue. Use `/deliver <ticket>` when the target item is already
chosen. Use `/harness` when the change touches Spellbook primitives
themselves: skills, agents, harness configs, registry entries, or gates.

## Encoded Assumptions

### Cross-Harness First

Every new mechanism must have a credible story for Claude Code, Codex, and Pi.
Filesystem-level skill discovery is the primary layer. Harness-specific config
is an optimization or projection.

Concrete consequences:

- Do not build a feature that only works because Claude has a unique runtime
  toggle.
- If runtime toggling is needed, generate per-harness artifacts from one
  source.
- If you cannot answer "what does this do on Codex?", the design is incomplete.

### Thin Harness, Strong Models

Spellbook resists semantic workflow engines. Skills should name judgment and
boundaries; agents do the reasoning. Shallow pass-through wrappers, elaborate
phase DSLs, and coordination primitives are treated as regressions.

This is why `/deliver` is composition-lint-gated: it must invoke phase skills
instead of inlining their internals or directly dispatching review benches.

### Local-First, Git-Native Work

The backlog is markdown in Git. Verdicts, evidence, and traces are moving
toward Git-native storage. GitHub PRs can exist, but they are not supposed to
be the only source of truth for the workflow.

### Minimal Global Install

Machine-wide install is intentionally small. `/tailor` and `/seed` are the
entry points; repo-local installation decides which other primitives belong.
This encodes the belief that harness quality is repo-specific, not a global
dump of every available skill.

### Tests Prove Executed Paths

The repo distinguishes adjacent evidence from runtime proof. The Dagger gate
is necessary but not automatically sufficient for a changed executable path:
new scripts, CLIs, runners, or Dagger functions need a command or artifact that
exercised that exact path.

## Effective Use

For a new machine:

```bash
curl -sL https://raw.githubusercontent.com/phrazzld/spellbook/master/bootstrap.sh | bash
```

Set `SPELLBOOK_DIR=/path/to/spellbook` when you want global harness symlinks to
point at a specific checkout instead of the stable default search path.

For a new or existing repo that should consume Spellbook:

1. Run `/tailor` when you want repo-specific skill selection and rewriting.
2. Run `/seed` when you want a fast default shared skill layer.
3. Treat `.agents/skills/` as the canonical repo-local shared skill root, with
   `.claude/skills/`, `.codex/skills/`, and Pi config as bridges.

For Spellbook development:

1. Read `AGENTS.md` and `harnesses/shared/AGENTS.md` before changing design
   surfaces.
2. Choose the workflow skill that matches the job: `/harness` for primitives,
   `/groom` for backlog, `/shape` for a new ticket, `/deliver` for a ticket,
   `/ci` for gate failures, `/diagnose` for unknown failures.
3. Never hand-edit `index.yaml`.
4. Run `dagger call check --source=.` before merge or ship.
5. If a skill or agent changes, rerun `./bootstrap.sh` or rely on the repo hook
   to propagate changes after commit.

## Remaining Work

The active backlog clusters into five themes.

### Workflow And Lifecycle

- `backlog.d/051-agents-md-three-layer-restructure.md`: restructure shared
  AGENTS guidance and routing tables.
- `backlog.d/052-spellbook-config-contract.md`: define `.spellbook/*.yaml`
  config contracts for lifecycle skills.
- `backlog.d/056-agent-session-trace-lifecycle.md`: preserve agent sessions as
  durable work artifacts.
- `backlog.d/058-work-ledger-mission-control.md`: add a local-first work ledger
  tying branch, phase, evidence, agents, blockers, and next action together.

### Evidence, Review, And Gate Strength

- `backlog.d/023-review-score-feedback-loop.md`: make review score data drive
  skill evolution.
- `backlog.d/024-offline-evidence-storage.md`: replace PR/release evidence
  coupling with Git-native `.evidence/` storage.
- `backlog.d/025-dagger-merge-gate.md`: make Dagger the merge gate rather than
  only a local pre-push check.
- `backlog.d/027-end-to-end-offline-validation.md`: prove the offline workflow
  after evidence storage exists.
- `backlog.d/054-clean-context-philosophy-bench.md`: ensure review agents see
  only diff plus criteria, not the author's session context.

### Skill Quality And Review Doctrine

- `backlog.d/048-code-review-pattern-catalog-convention.md`: standardize
  per-repo review pattern catalogs.
- `backlog.d/049-bounded-payload-discipline-reference.md`: add a reusable
  bounded-payload review reference.
- `backlog.d/053-skill-quality-audit-mode.md`: add `/groom audit` coverage for
  skill health.
- `backlog.d/055-mcp-first-integration-doctrine.md`: document when external
  integrations should start with MCP.

### Sync, Legacy, And Future Automation

- `backlog.d/026-multi-machine-sync.md`: sync verdict refs across machines.
- `backlog.d/031-harness-auto-tune-gepa.md`: parked until enough real
  `/flywheel` cycle data exists.
- `backlog.d/046-curate-skill-triage.md`: decide the fate of legacy unmarked
  `/curate` skill copies.

### Stale Or Reconciled State

- `backlog.d/059-tailor-durable-brief-and-ownership-conflicts.md` is marked
  fixed but still lives in the active backlog. It should be archived or
  otherwise reconciled by the normal closure workflow.

## Known Sharp Edges

- `project.md` is older than the current harness pivot. It still references
  `/focus`, collections, and manifest-pull as active architecture. Treat
  `AGENTS.md`, `README.md`, `bootstrap.sh`, and the Dagger module as more
  current.
- Some gates are intentionally prose/regex based, especially harness install
  wording and `/deliver` composition. They encode valuable doctrine but can
  produce false positives when wording changes.
- `pre-push` can only enforce Dagger when the local machine has Dagger and
  Docker available.
- `pre-merge-commit` has an explicit escape hatch. That is useful operationally
  but means merge discipline is still partly cultural.
- Remote bootstrap is a convenience path. Symlink mode against a stable local
  checkout is the development path because edits propagate immediately.
