# Harness Kit

The ad-hoc operator harness for AI-assisted software development. One repo,
all harnesses (Claude Code, Codex, Pi, Antigravity): judgment skills,
vendored external skills at pinned versions, shared doctrine, and per-harness
configs, installed system-wide by a Rust bootstrap.

For the project north star - what Harness Kit is trying to become and what it
should refuse - read [`VISION.md`](VISION.md).

Event-driven automation (CI-native code review, incident response, outer
loops) is a separate plane — see [`meta/CONTRACTS.md`](meta/CONTRACTS.md)
for the boundary. Harness Kit is operator-facing implementation substrate,
not a buyer-facing governed workflow package; see
[`docs/positioning.md`](docs/positioning.md) before framing it for clients
or procurement.

## Quick Start

```bash
# Bootstrap (one-time per machine)
# Installs skills, vendored externals, configs, and the provider roster
# system-wide; symlinks when a local checkout exists.
curl -sL https://raw.githubusercontent.com/misty-step/harness-kit/master/bootstrap.sh | bash
```

Fresh-machine bootstrap requires a Rust toolchain unless `harness-kit-checks`
is already on `PATH`. Bootstrap from a stable checkout (set
`HARNESS_KIT_DIR=/path/to/harness-kit` to pin one), never a disposable
worktree.

## Skills

| Skill | Purpose |
|-------|---------|
| `/deliver` | One ticket end to end: context-first, docs→tests→code, live QA, three-altitude refactor, diverse review, adversarial pre-ship |
| `/groom` | Backlog truth: tidy, challenge, surface gaps (including the repo's own harness gaps) |
| `/shape` | Raw idea → context packet with an acceptance oracle |
| `/code-review` | Dispatch-shaped review across diverse providers and model families |
| `/qa` | Verify the running thing, shaped to the app (browser/API/CLI/library/MCP) |
| `/ci` | Audit and strengthen the repo gate, then drive it green |
| `/diagnose` | Feedback-loop-first debugging and incident investigation |
| `/design` | Artifact-backed visual critique and polish, accessibility included |
| `/showcase` | Demoability, product polish, marketing assets, and evidence-backed consulting proof |
| `/research` | Multi-source research, delegation, model selection |
| `/sprites` | Run lane cards on Fly Sprites for heavy/parallel/isolated work |
| `/harness-engineering` | Engineer this harness: skills, doctrine, gates, sync, telemetry |
| `/orient` | Fast read-only repo/session orientation from live evidence |
| `/next` | Recommend the best next move, with user-vs-agent action split |

## Workflow

```
backlog.d/ → /groom → /shape (when it needs shaping) → /deliver → /ship
```

For non-trivial `/shape` packets or execution plans, author and open a local
HTML plan before building:

```bash
cp skills/shape/templates/html-plan.html /tmp/<slug>-plan.html
open /tmp/<slug>-plan.html   # macOS; use xdg-open on Linux or a browser tab in other harnesses
```

Use the rendered page for hierarchy, comparison, risk, and proof. Headless
environments still write the `.html` artifact and report its path.

For the deeper architecture map, read [`CODEBASE.md`](CODEBASE.md).

## Static Docs Companion

```bash
cargo run --locked -p harness-kit-checks -- build-docs-site
```

Generated from live sources into [`docs/site`](docs/site); pre-commit
regenerates, CI fails on drift, and master deploys to GitHub Pages.

## Structure

```
harness-kit/
├── skills/         # Judgment skills
│   └── .external/  # Vendored third-party skills (pins in registry.yaml)
├── harnesses/      # Per-harness configs + shared AGENTS.md doctrine
├── meta/           # Cross-repo contracts
├── registry.yaml   # External source provenance (repo, pin, license)
├── crates/harness-kit-checks/  # Bootstrap, gates, hooks, sync, telemetry
└── bootstrap.sh    # Curl-compatible launcher
```

## Adding a Primitive

1. Run the primitive test (`skills/harness-engineering/SKILL.md`): most
   ideas are local task prompts or doctrine lines, not skills, unless
   invocation reality requires app-visible skill discovery.
2. Skills: `skills/{name}/SKILL.md`, < 500 lines, judgment not procedure.
   One-off prompts stay outside the source repo unless repeated use earns a
   skill, reference, or template.
3. Check telemetry before and after: `harness-kit-checks telemetry`.
4. Re-bootstrap; discovery is filesystem-based.

## Principles

- **Thin harness, strong models** — resist ceremony
- **Gotchas > instructions** — enumerate what goes wrong
- **Strip non-load-bearing scaffold** — stress-test after model upgrades
- **Symlink, not copy** — edits in the checkout propagate instantly
- **Progressive disclosure** — description → SKILL.md → references

## License

MIT
