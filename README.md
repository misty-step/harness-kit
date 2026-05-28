# Harness Kit

28 catalog skills, 11 core agents, and harness infrastructure for AI-assisted
software development. One repo. All harnesses (Claude Code, Codex, Pi).

Harness Kit is an operator-facing harness primitive library, not a buyer-facing
governed workflow package or admin-control plane. Read
[`docs/positioning.md`](docs/positioning.md) before framing this repo for
clients, departments, procurement, security reviewers, or executives.

## Quick Start

```bash
# Bootstrap (one-time per machine)
# Installs all first-party skills and the provider roster system-wide; symlinks
# if a local checkout exists, downloads from GitHub otherwise
curl -sL https://raw.githubusercontent.com/misty-step/harness-kit/master/bootstrap.sh | bash
```

If you're running bootstrap from a temporary git worktree, it now prefers a
stable checkout like `~/Development/harness-kit` automatically. To intentionally
point your harnesses at a specific checkout, set
`HARNESS_KIT_DIR=/path/to/harness-kit`.

## Core Workflow Skills

| Skill | Purpose |
|-------|---------|
| `/deliver` | Inner-loop composer: ticket → merge-ready (shape → implement → review+ci+refactor+qa) |
| `/flywheel` | Outer-loop orchestrator: cycles of /deliver → /monitor → /reflect |
| `/code-review` | Parallel multi-agent review, auto-fix loop |
| `/diagnose` | Investigate, triage, fix |
| `/qa` | Verify the changed surface and capture evidence |
| `/hardening` | Property tests, mutation testing, CRAP/SCRAP, DRY, and acceptance mutation |
| `/demo` | Show what changed with the right artifact for the change shape |
| `/design` | Artifact-backed critique and polish for hierarchy, typography, layout, and taste |
| `/monitor` | Watch post-change signals and escalate regressions |
| `/groom` | Backlog management, brainstorming, rethink, scaffold |
| `/harness` | Skill engineering, primitive management, context lifecycle |
| `/reflect` | Session retrospective, harness postmortem, operator coaching |
| `/research` | Multi-source web research, delegation, think tank |
| `/shape` | Spec/design → context packet output |

## The 8 Core Agents

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

## Static Docs Companion

Harness Kit's public static docs companion is generated from live repo sources:

```bash
scripts/build-docs-site.sh
scripts/check-docs-site.sh --self-test
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
├── harnesses/     # Per-harness configs (claude/, codex/, pi/)
│   └── shared/    # Common engineering principles
├── registry.yaml  # External skill sources (for embeddings)
└── bootstrap.sh   # Discovers skills/agents/roster, symlinks to system harness dirs
```

## Adding a Skill

1. Create `skills/{name}/SKILL.md` with frontmatter
2. Keep it < 500 lines. Encode judgment, not procedures.
3. Run `/harness lint` to validate quality gates
4. Run `bootstrap.sh` — it discovers skills from the filesystem automatically

## Principles

- **Thin skills, strong agents** — resist ceremony
- **Gotchas > instructions** — enumerate what goes wrong
- **Strip non-load-bearing scaffold** — stress-test after model upgrades
- **Symlink, not copy** — bootstrap.sh links to local checkout when available
- **Progressive disclosure** — description → SKILL.md → references

## License

MIT
