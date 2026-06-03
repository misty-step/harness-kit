# jj-first agent VCS and portable execution contract

Priority: P1
Status: shaped
Estimate: L

## Goal

Make Harness Kit operate jj-first for agent local work while preserving Git
compatibility for remotes, evidence objects, hooks that still earn their keep,
and existing Dagger gates.

## Non-Goals

- Do not replace Git object storage, GitHub/Git remotes, or colocated `.git`
  compatibility in the first slice.
- Do not do a global `git` to `jj` command substitution across skills and
  scripts.
- Do not weaken `dagger call check --source=.` or move CI authority back into
  platform-specific workflow YAML.
- Do not make Dagger more mandatory than it already is for lightweight
  bootstrap or read-only repo exploration.
- Do not build a semantic workflow engine around provider CLIs, jj, or Dagger.
- Do not migrate verdict refs, evidence storage, or backlog trailers until a
  failing oracle proves the new contract is equivalent or better.

## Constraints / Invariants

- Cross-harness first: Codex, Claude, Pi, Antigravity, and human shell use must
  all see the same filesystem-level contracts.
- Dagger remains the canonical gate: `dagger call check --source=.`.
- Colocated jj is the default migration form: `.jj/` adds agent-safe local
  history while `.git/` remains available for Git remotes and plumbing.
- Closeout must become dual-view before it becomes jj-only: Git cleanliness and
  jj cleanliness must agree at receipt boundaries.
- Verdict, backlog, and evidence contracts must remain inspectable by local
  scripts and Dagger gates without a hosted service.
- Existing user work and untracked backlog files stay protected by shared
  Closeout doctrine.

## Authority Order

tests > executable oracles > Dagger gates > scripts > skills > external prior art > lore

## Repo Anchors

- `harnesses/shared/AGENTS.md` - shared Closeout currently names
  `git status --short --untracked-files=all`.
- `AGENTS.md` - Harness Kit root contract: base branch `master`, Dagger gate,
  roster floor, clean-tree closeout.
- `ci/src/harness_kit_ci/main.py` - Dagger check/heal spine; currently installs
  `git` in the lint and repair containers and explicitly tells repair agents not
  to use Git.
- `dagger.json` - Dagger module source and engine version.
- `.githooks/pre-commit` and `.githooks/pre-push` - Git lifecycle hooks that
  regenerate derived artifacts and run Dagger before push.
- `scripts/lib/verdicts.sh` - review verdict proof stored in Git object/ref
  plumbing.
- `scripts/lib/backlog.sh` - backlog trailer parsing and `git mv` archival.
- `scripts/lib/evidence.sh` - branch-derived evidence paths.
- `scripts/heal-commit.sh` - Dagger repair plus Git staging/commit choreography.
- `skills/yeet/SKILL.md`, `skills/ship/SKILL.md`,
  `skills/deliver/references/worktree.md`, `skills/code-review/SKILL.md`,
  `skills/refactor/SKILL.md` - user-facing Git workflow surfaces that need
  routing, not blind replacement.
- `skills/shape/references/cli-design.md` - CLI surface requirements for new
  helpers.

## Prior Art

- Jujutsu official Git compatibility docs: colocated workspaces keep `.jj/` and
  `.git/` side by side, import/export on jj commands, and allow mixing read-only
  Git tooling with jj local workflow:
  https://docs.jj-vcs.dev/latest/git-compatibility/.
- Jujutsu repository: Git-compatible VCS, latest Exa-observed release v0.41.0
  on 2026-05-07: https://github.com/jj-vcs/jj.
- Dagger LLM docs: Dagger can be an agent runtime and lets LLMs discover/use
  documented Dagger functions: https://docs.dagger.io/features/llm/.
- Dagger product docs and agentic CI posts support the repo's existing
  local-first, containerized gate posture: https://dagger.io/ and
  https://dagger.io/blog/automate-your-ci-fixes-self-healing-pipelines-with-ai-agents/.
- Agent-facing jj ecosystem:
  - `agentjj` - agent-oriented porcelain for Jujutsu:
    https://docs.rs/crate/agentjj/latest.
  - `jj-worktree` - shim translating `git worktree` operations to
    `jj workspace`: https://github.com/kawaz/jj-worktree.
  - `claude-jj-worktree` - Claude Code WorktreeCreate/Remove hooks mapped to
    jj workspaces: https://github.com/jasagiri/claude-jj-worktree.
  - `pi-jj` - Pi extension with jj checkpoints and stacked PR support:
    https://github.com/manojlds/pi-jj.
  - `dwm` - workspace manager for jj and Git with agent-status hooks:
    https://dwm.drpz.xyz/.

## Research Fanout Evidence

### Exa (neural search)

Exa surfaced official jj Git compatibility docs, the live jj repository, and
small but relevant agent/jj projects: `jj-worktree`, `claude-jj-worktree`,
`jjw`, `jj-skipper`, `agentjj`, and `pi-jj`. The strongest external signal is
that the ecosystem is solving agent worktree/workspace mismatch with shims and
porcelains rather than by abandoning Git compatibility.

### xAI / Grok (web_search)

Grok's web-search synthesis agreed that colocated jj is the low-risk path:
agents get snapshots, `jj undo`, operation logs, first-class conflicts, and no
staging area, while Git remotes/tools stay available. It also flagged real
migration cautions: detached HEAD surprises when mixing mutating Git commands,
branch/bookmark divergence, hook semantics, less mature IDE/tooling support,
and the need to keep Dagger's container overhead from becoming a universal
bootstrap blocker.

### Thinktank (complete)

Thinktank `research/quick` completed at `/tmp/hk-jj-thinktank` with `systems`
and `verification` agents. Accepted findings:

- Harness Kit is already meaningfully Dagger-first; this ticket should not
  relitigate the gate spine.
- Git coupling is deepest in user workflow, hook lifecycle, verdict staleness,
  and branch/staging choreography, not in Git object storage itself.
- Move verdict enforcement into a Dagger gate before relying on jj-native land
  paths, because Git merge hooks do not represent jj workflows.
- Sequence colocated jj support before any jj-default or jj-only posture.

Rejected/discounted finding:

- The verification lane described replacing `.git` with `.jj`; that conflicts
  with official colocated jj docs and with the repo's Git-compatible evidence
  contracts, so the first slice must retain `.git`.

### Codebase

Current repo evidence shows Dagger is already canonical (`AGENTS.md`,
`skills/ci/SKILL.md`, `dagger.json`, `ci/src/harness_kit_ci/main.py`,
`.githooks/pre-push`). Git assumptions are broad: closeout doctrine, `/yeet`,
`/ship`, `/code-review`, `/refactor`, `/deliver` worktree references, backlog
helpers, verdict refs, evidence helpers, external skill sync, and hook tests.
The high-leverage design is therefore a narrow VCS helper contract plus
skill/script routing, not a broad rewrite.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Status quo Git-first | Keep Git CLI as the only supported workflow | Maximum compatibility; no migration risk | Agents keep losing time to staging/index/branch confusion and weaker undo | Reject as end-state |
| Colocated jj operator path | Add jj-first local workflow while preserving `.git` plumbing and remotes | Captures jj safety with low blast radius | Dual-tool confusion unless routing/oracles are explicit | Choose |
| VCS helper contract | Add small scripts for root/status/head/branch/commit identity with Git and jj backends | Deep interface; lets skills stop hardcoding VCS probes | Can become shallow wrapper soup if it mirrors every Git command | Choose as first implementation slice |
| jj-only rewrite | Remove Git workflow and make `.jj` the only VCS source | Clean conceptual story for jj enthusiasts | Breaks remotes, hooks, verdict refs, users, and many agent tools | Reject |
| Git shim around jj workspaces | Put a `git` shim in front of tools that insist on worktrees | Helps Claude/Codex parallelism quickly | Hidden indirection can mask safety failures; tool-specific | Defer as optional integration |
| Dagger-everything | Run all agent commands, bootstrap, skill execution, and gates through Dagger | Hermetic and portable | Heavy daemon/container dependency for simple reads and bootstrap | Reject |
| Just/Taskfile-first execution | Make local task runners the canonical gate and Dagger optional | Fast and familiar | Regresses repo doctrine: platform/local gates drift from CI | Reject for Harness Kit |
| Nix/Flox/Devbox execution | Use declarative environments instead of Dagger containers | Strong reproducibility without Docker in some setups | New ecosystem dependency; does not replace existing Dagger gate | Defer as adjunct research |
| Worktree manager first | Adopt `dwm`, `jjw`, or similar as the primary UX | Improves parallel agent sessions | External runtime becomes product dependency before contract is shaped | Defer |

## Tradeoff Matrix

| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| Status quo Git-first | 2 | 5 | 5 | 3 | 5 | 5 | 5 |
| Colocated jj operator path | 5 | 4 | 5 | 5 | 5 | 4 | 4 |
| VCS helper contract | 5 | 3 | 5 | 5 | 5 | 5 | 4 |
| jj-only rewrite | 2 | 1 | 4 | 4 | 1 | 2 | 2 |
| Git shim around jj workspaces | 3 | 3 | 4 | 4 | 4 | 3 | 3 |
| Dagger-everything | 3 | 2 | 4 | 3 | 3 | 3 | 2 |
| Just/Taskfile-first execution | 2 | 4 | 5 | 4 | 4 | 4 | 4 |
| Nix/Flox/Devbox execution | 3 | 2 | 5 | 3 | 3 | 3 | 3 |
| Worktree manager first | 3 | 3 | 4 | 4 | 4 | 3 | 3 |

The selected shape is `colocated jj operator path` plus a bounded `VCS helper
contract`. It scores best because it moves the repo toward the requested
jj-first agent outcome while preserving the Git-compatible evidence layer that
already makes Harness Kit portable.

## Agent Readiness

- Profile source: `.harness-kit/agent-readiness.yaml` not present.
- Stack feedback strength: strong for existing Python/shell/Dagger gate
  patterns; weak for jj until explicit fixtures install or stub jj.
- ADR decision: required. This changes the repo's core VCS contract and the
  meaning of "git-native" evidence.
- Infrastructure path: CLI-managed. First slice should depend on `jj` only in
  fixture/smoke paths and keep Git fallback mandatory.
- Gate: `python3 scripts/check-agent-roster.py`, targeted VCS helper tests, then
  `dagger call check --source=.`
- Evidence storage: backlog packet, `.harness-kit/traces/delegations.jsonl`,
  provider-lane transcripts, and future `.harness-kit/examples/` VCS fixtures.
- Mock policy impact: preserved. Tests should use temporary real Git/jj repos or
  command stubs only at external CLI boundaries, never mocked internal helpers.

## CLI Surface

- Command tree: `scripts/vcs-state.sh <root|status|head|branch|is-jj|is-git>`
  and later `scripts/vcs-closeout.sh`.
- Usage: `scripts/vcs-state.sh status --format plain|json`.
- Args/flags: `--repo <path>` optional, `--format`, `--require-clean`,
  `--allow-git-only`, `--allow-jj-colocated`.
- Output contract: primary data to stdout; diagnostics to stderr; JSON includes
  `backend`, `root`, `head`, `branch_or_bookmark`, `dirty`, `untracked_count`,
  `jj_colocated`, and `git_available`.
- Error/exit code map: `0` clean/success, `1` dirty when `--require-clean`,
  `2` not a supported repo, `3` backend command missing, `4` Git/jj views
  disagree.
- Config/env precedence: flags > env (`HARNESS_KIT_VCS_BACKEND`) > repo config
  > auto-detect.
- Safety controls: helpers are read-only in the first slice; mutation helpers
  require a separate shaped ticket.
- Examples:
  - `scripts/vcs-state.sh root`
  - `scripts/vcs-state.sh status --require-clean --format json`
  - `HARNESS_KIT_VCS_BACKEND=jj scripts/vcs-state.sh head`

## Delegation Evidence

- Roster providers used:
  - `claude`, architecture critic, receipt
    `36c2b917-d2f4-4d88-9553-c34d97c259ff`, transcript
    `.harness-kit/traces/provider-lanes/20260603T181342.845485Z-claude-6d90f2a8.txt`.
  - `pi`, implementation-risk and oracle reviewer, receipt
    `91435c54-35b0-46cd-99f4-b18432b9fe75`, transcript
    `.harness-kit/traces/provider-lanes/20260603T181343.915953Z-pi-7985fed0.txt`.
  - `agy`, product premise critic, receipt
    `d4e044df-bc1b-4cec-860b-53b4cdd2f76a`, transcript
    `.harness-kit/traces/provider-lanes/20260603T181342.637505Z-agy-f4027527.txt`.
- Thinktank used: `research/quick`, output `/tmp/hk-jj-thinktank`, `systems`
  and `verification` complete.
- Accepted evidence: start with colocated jj, move verdict enforcement toward
  Dagger, add dual-view closeout, preserve Git plumbing for verdict/backlog
  evidence, and avoid a broad rewrite.
- Rejected evidence: jj-only migration, Dagger-everything, Justfile replacing
  Dagger as Harness Kit's canonical gate, and universal command substitution.
- Waivers: native named Ousterhout/Carmack/Grug/Beck subagent bench was not
  separately launched because roster lanes plus Thinktank covered architecture,
  premise, implementation risk, and verification lenses in this shaping pass.

## Exemplar Techniques

- No project-root `exemplars.md` found in this worktree.

## Oracle (Definition of Done)

- [ ] A VCS contract document exists under `skills/harness-engineering/` or a
      new `references/` file and states the supported backends: Git-only,
      colocated jj+Git, and unsupported jj-native-only.
- [ ] Read-only VCS helper(s) expose root, status, head SHA, branch/bookmark,
      and dirty state without changing repository state.
- [ ] Fixture tests create temporary Git and colocated jj repos and prove helper
      output is stable for clean, dirty, untracked, and branch/bookmark states.
- [ ] Shared Closeout doctrine and affected skills route through the helper or
      explicitly document dual Git/jj commands.
- [ ] `scripts/lib/verdicts.sh` either remains Git-plumbing-only by design with
      a colocated jj test, or a Dagger verdict gate replaces merge-hook-only
      enforcement.
- [ ] `.githooks/pre-push` or a new Dagger gate proves verdict enforcement is
      not lost when jj users do not trigger Git merge hooks.
- [ ] `/yeet`, `/ship`, `/code-review`, `/refactor`, and `/deliver` worktree
      references stop presenting Git CLI as the only path when jj is detected.
- [ ] Dagger remains the canonical gate and no platform-specific workflow YAML
      becomes authoritative.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Acceptance Evidence

- Acceptance source: temporary Git and colocated jj repo fixtures plus Dagger
  gate coverage for helper scripts and any skill prose lint.
- Evidence that proves it: helper self-test output, Dagger gate output, and
  explicit command transcripts showing Git and jj views agree or fail loudly.
- Exact command/path/route exercised:
  `scripts/vcs-state.sh status --require-clean --format json`,
  `scripts/check-agent-roster.py`, and `dagger call check --source=.`
- Oracle / acceptance artifact hash: not yet available; first implementation
  slice must add fixtures and record their `shasum -a 256` values before
  claiming acceptance.
- Contract-change acknowledgment: this packet intentionally changes
  "git-native" from "Git CLI only" to "Git-compatible durable storage with
  jj-first local workflow where supported."
- Residual risk: jj CLI availability in Dagger containers and on user machines
  is not yet proven.

## Formal Spec

- Formal Spec Required: yes. The change touches core workflow contracts,
  cross-harness behavior, merge/push safety, evidence storage, and expensive
  regressions that are hard to detect manually after merge.
- Informal spec: Harness Kit must support jj-first local agent workflows in
  colocated repos without losing Git-compatible durability, Dagger gate
  authority, verdict proof, backlog closure proof, or clean-tree closeout.
- Formal examples: temporary Git repo transcript, temporary colocated jj repo
  transcript, dirty-state transcript, verdict-staleness transcript, and
  pre-push/Dagger verdict enforcement transcript.
- Acceptance oracle: helper self-test plus Dagger gate that fails before helper
  support and passes after Git/jj parity is implemented.
- Hardening budget: bounded mutation/acceptance mutation for closeout helper
  status classification and verdict-staleness detection; cap at one focused
  hour in the first implementation slice.
- Waiver path: only the lead can waive jj fixture execution when `jj` is not
  installable in the environment; waiver must record exact missing binary or
  container failure and keep Git-only behavior unchanged.

## Observability Plan

- Changed behavior to watch: closeout, verdict validation, backlog closure, and
  provider worktree/workspace isolation under colocated jj.
- Named signal or evidence surface: VCS helper self-test output, Dagger gate
  logs, delegation receipts, and future `.harness-kit/traces/` records tagged
  with `backend=git|jj-colocated`.
- Instrumentation debt if no signal exists: provider dispatch receipts do not
  yet record VCS backend or workspace kind.

## Implementation Sequence

1. Add a narrow read-only VCS helper contract and tests for Git-only and
   colocated jj repos.
2. Add a Dagger gate or targeted script self-test for the helper without
   changing mutation paths.
3. Patch shared Closeout and the smallest set of skill references to route
   status/root/head detection through the helper or through explicit jj/Git
   dual commands.
4. Add a verdict-staleness and hook-gap spike: prove whether
   `scripts/lib/verdicts.sh` remains valid in colocated jj and whether verdict
   enforcement must move from merge hook into Dagger.
5. Only after the read-only and verdict slices are green, shape mutation
   helpers for `/yeet`, `/ship`, and `scripts/heal-commit.sh`.

## Risk + Rollout

- Silent safety regression: jj workflows may skip Git merge hooks. Move
  merge-only guarantees into Dagger or pre-push before advertising jj as
  default.
- Tooling mismatch: provider CLIs still expect Git worktrees. Keep colocated
  `.git` and evaluate jj workspace shims as optional integrations.
- Agent command drift: skills may still hallucinate Git commands. Add routing
  prose and lint only after helper commands are stable.
- Docker/Dagger friction: Dagger is already canonical, but simple read-only
  helpers should remain local shell scripts so bootstrap and orientation do not
  require a daemon.
- Rollback: remove VCS helper references and keep Git-only paths; colocated jj
  fixtures should not alter source repo state.
