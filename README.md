# Harness Kit

Harness infrastructure for AI-assisted software development. One repo. All
harnesses (Claude Code, Codex, Pi, Antigravity).

Harness Kit is an operator-facing harness primitive library, not a buyer-facing
governed workflow package or admin-control plane. Read
[`docs/positioning.md`](docs/positioning.md) before framing this repo for
clients, departments, procurement, security reviewers, or executives.

## Quick Start

```bash
# Bootstrap (one-time per machine)
# Installs first-party skills, synced external skills, and the provider roster
# system-wide; symlinks if a local checkout exists, downloads Rust source otherwise
curl -sL https://raw.githubusercontent.com/misty-step/harness-kit/master/bootstrap.sh | bash
```

If you're running bootstrap from a temporary git worktree, it now prefers a
stable checkout like `~/Development/harness-kit` automatically. To intentionally
point your harnesses at a specific checkout, set
`HARNESS_KIT_DIR=/path/to/harness-kit`.

Fresh-machine bootstrap requires a Rust toolchain unless `harness-kit-checks`
is already installed on `PATH`.

## Core Workflow Skills

| Skill | Purpose |
|-------|---------|
| `/deliver` | Inner-loop composer: ticket → merge-ready (shape → implement → review+ci+refactor+qa) |
| `/dispatch` | Compose roster-backed specialist lanes with prompt-native lane cards |
| `/flywheel` | Outer-loop orchestrator: cycles of /deliver → /monitor → /reflect |
| `/code-review` | Parallel multi-agent review, auto-fix loop |
| `/diagnose` | Investigate, triage, fix |
| `/qa` | Verify the changed surface and capture evidence |
| `/hardening` | Property tests, mutation testing, CRAP/SCRAP, DRY, and acceptance mutation |
| `/demo` | Show what changed with the right artifact for the change shape |
| `/design` | Artifact-backed critique and polish for hierarchy, typography, layout, and taste |
| `/monitor` | Watch post-change signals and escalate regressions |
| `/groom` | Backlog management, brainstorming, rethink, scaffold |
| `/harness-engineering` | Skill engineering, primitive management, context lifecycle |
| `/create-repo-skill` | Generate repo-local QA, persona acceptance, and bespoke workflow skills |
| `/reflect` | Session retrospective, harness postmortem, operator coaching |
| `/research` | Multi-source web research, delegation, think tank |
| `/shape` | Spec/design → context packet output |

## Core Agent Patterns

**GAN triad:** planner (spec) → builder (implement) → critic (evaluate)

**Design review bench:** ousterhout (depth), carmack (ship), grug (simplicity), beck (TDD), cooper (classicist testing)

## Workflow

```
backlog.d/ → /groom → /shape → /deliver → ship
                              └─ /flywheel (outer loop cycles:
                                  /deliver → /monitor → /reflect → next)
```

For a deeper map of the repository architecture, encoded assumptions, operating
loop, and active backlog, read [`CODEBASE.md`](CODEBASE.md).

## Focused Lane Harnesses

`/dispatch` composes prompt-native lane cards. `dispatch-agent` can run a
roster lane with a projected harness root so the child provider sees only the
skills named by a `lane_harness.v1` manifest instead of the full system-wide
skill install.

```bash
cargo run --locked -p harness-kit-checks -- materialize-lane-harness \
  --manifest .harness-kit/examples/lane-harness.yaml

cargo run --locked -p harness-kit-checks -- dispatch-agent \
  --provider-target codex \
  --objective "review the CI lane only" \
  --input-ref backlog.d/_done/101-focused-lane-harness-projection.md \
  --prompt-file /tmp/review.md \
  --lane-harness .harness-kit/examples/lane-harness.yaml
```

Projection roots are ignored runtime artifacts under
`.harness-kit/tmp/lane-harness/`. The dispatcher sets harness-specific config
environment variables (`CODEX_HOME`, `CLAUDE_CONFIG_DIR`, `PI_HOME`,
`GEMINI_CONFIG_DIR`, and `HOME`) to that projected root for the child process,
then removes the root unless `--keep-lane-root` is supplied.

Manifests are deliberately small: role, provider target, optional roster model
override, local skill allowlist, pinned external aliases, tool labels, oracle,
evidence expectations, and fallback policy. Projection failure is recorded as a
receipt instead of exploding the whole composition. Provider failures such as
auth, credits, missing binaries, timeouts, nonzero exits, and sentinel
mismatches are summarized through `failure_kind`; lane receipts also include
`lane_harness_ref`, `lane_harness_sha256`, `projection_status`, and
`output_check`.

## Static Docs Companion

Harness Kit's public static docs companion is generated from live repo sources:

```bash
cargo run --locked -p harness-kit-checks -- build-docs-site
cargo run --locked -p harness-kit-checks -- check-docs-site --self-test
```

Open [`docs/site/index.html`](docs/site/index.html) for the rendered site.
Public-facing copy lives in [`docs/copy/site.json`](docs/copy/site.json).
Source changes auto-regenerate the site via pre-commit; CI fails on drift.
The generated catalog includes every local skill and agent, CI gate map,
workflow walkthroughs, governance notes, and an agent-readable
[`docs/site/llms.txt`](docs/site/llms.txt).

Deploys publish [`docs/site`](docs/site) to GitHub Pages from `master` after
the generated-site check passes.

## Structure

```
harness-kit/
├── skills/        # Canonical skill catalog
├── agents/        # Agent definitions
├── harnesses/     # Per-harness configs (claude/, codex/, pi/, antigravity-cli/, antigravity-ide/)
│   └── shared/    # Common engineering principles
├── registry.yaml  # Pinned external skill sources for sync/search
└── bootstrap.sh   # Curl-compatible launcher for the Rust bootstrap command
```

## Adding a Skill

1. Create `skills/{name}/SKILL.md` with frontmatter
2. Keep it < 500 lines. Encode judgment, not procedures.
3. Run `/harness-engineering lint` to validate quality gates
4. Run `cargo run --locked -p harness-kit-checks -- bootstrap` or
   `./bootstrap.sh` — bootstrap discovers skills from the filesystem
   automatically

## Principles

- **Thin skills, strong agents** — resist ceremony
- **Gotchas > instructions** — enumerate what goes wrong
- **Strip non-load-bearing scaffold** — stress-test after model upgrades
- **Symlink, not copy** — the Rust bootstrap links to local checkout when
  available
- **Progressive disclosure** — description → SKILL.md → references

## License

MIT
