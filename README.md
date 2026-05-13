# Spellbook

29 catalog skills, 8 core agents, and harness infrastructure for AI-assisted
software development. One repo. All harnesses (Claude Code, Codex, Pi).

## Quick Start

```bash
# Bootstrap (one-time per machine)
# Symlinks if local checkout exists, downloads from GitHub otherwise
curl -sL https://raw.githubusercontent.com/phrazzld/spellbook/master/bootstrap.sh | bash
```

If you're running bootstrap from a temporary git worktree, it now prefers a
stable checkout like `~/Development/spellbook` automatically. To intentionally
point your harnesses at a specific checkout, set `SPELLBOOK_DIR=/path/to/spellbook`.

## Core Workflow Skills

| Skill | Purpose |
|-------|---------|
| `/deliver` | Inner-loop composer: ticket → merge-ready (shape → implement → review+ci+refactor+qa) |
| `/flywheel` | Outer-loop orchestrator: cycles of /deliver → /deploy → /monitor → /reflect |
| `/code-review` | Parallel multi-agent review, auto-fix loop |
| `/diagnose` | Investigate, triage, fix |
| `/qa` | Verify the changed surface and capture evidence |
| `/demo` | Show what changed with the right artifact for the change shape |
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
                                  /deliver → /deploy → /monitor → /reflect → next)
```

## Structure

```
spellbook/
├── skills/        # Canonical skill catalog
├── agents/        # Agent definitions
├── harnesses/     # Per-harness configs (claude/, codex/, pi/)
│   └── shared/    # Common engineering principles
├── registry.yaml  # External skill sources (for embeddings)
└── bootstrap.sh   # Discovers skills/agents from filesystem, symlinks to harness dirs
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
