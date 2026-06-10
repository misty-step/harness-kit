# CLAUDE.md

## What This Repo Is

**Harness Kit** — the ad-hoc operator harness: ~12 judgment skills, a few
saved prompts, vendored external skills, and harness configs, installed to
every harness (~/.claude, ~/.codex, ~/.pi, antigravity) by the Rust
bootstrap. Event-driven automation (CI-native review, incident response,
outer loops) is Mode B and lives in bitterblossom, not here; the boundary
contract is `meta/CONTRACTS.md`.

## Structure

```
harness-kit/
├── skills/         # Judgment skills (deliver, groom, qa, code-review, …)
│   └── .external/  # Vendored third-party skills, pinned via registry.yaml
├── prompts/        # Saved invocations (yeet, ship, orient, critique, reflect)
├── harnesses/      # Per-harness configs, hooks, shared AGENTS.md doctrine
├── meta/           # Cross-repo contracts (Mode B boundary, trailers)
├── registry.yaml   # External source provenance: repo, pin, license notes
├── crates/harness-kit-checks/  # Bootstrap, gates, hooks, sync, telemetry
└── bootstrap.sh    # curl-compatible launcher for the Rust bootstrap
```

Primitive test (full version in `skills/harness-engineering/SKILL.md`):
prompt = "what I'd retype"; skill = "changes what a frontier model does";
doctrine line = "worth paying for every session"; event-triggered = Mode B,
not here.

## Issue Tracking

**`backlog.d/` is the single source of truth.** Open work lives in
`backlog.d/NNN-<slug>.md`; closed work moves to `backlog.d/_done/`.
Closure is trailer-driven (`Closes-backlog:` on the squash commit; canon in
`meta/CONTRACTS.md`). `/ship` injects trailers and archives; `/groom`
sweeps for drift.

## Workflow

```
backlog.d/ → /groom → /shape (when the idea needs it) → /deliver → /ship
```

`/deliver` is the spine: context-first, docs→tests→code, live QA,
three-altitude refactor, diverse-provider review, adversarial pre-ship
thinking. It stops at merge-ready unless asked to ship.

## Principles

See `harnesses/shared/AGENTS.md` — one file, symlinked to every harness.

- **Thin harness, strong models** — judgment and context, not process
  machinery. Phase prose the model already knows is railroading.
- **Cross-harness first** — filesystem + SKILL.md is the primary layer;
  harness-native features are optimizations. Codex and Claude are
  first-class; Pi/antigravity ride the same format.
- **Gotchas > instructions**; **description is the trigger**; **map, not
  manual** — AGENTS/CLAUDE point at skills, never contain them.
- **Telemetry before catalog changes** — `harness-kit-checks telemetry`;
  usage evidence beats vibes in both directions.

## Gotchas for Contributing to This Repo

- Run the primitive test before adding anything. Most "new skills" are
  prompts or doctrine lines.
- Skills encode judgment, not procedures. If the model already knows how,
  delete it.
- The pre-commit hook regenerates `index.yaml` and `docs/site` — never edit
  them manually.
- The gate is `cargo run --locked -p harness-kit-checks -- check --repo .`.
  Every gate must be able to name a real failure it catches; otherwise
  delete it.
- `harnesses/claude/settings.json` is COPIED by bootstrap (Claude mutates it
  at runtime); changes need a re-bootstrap.
- Bootstrap from a stable checkout, not a disposable worktree — worktree
  symlinks make global skills vanish when the worktree dies.
- External skills are vendored at pins; edit one and it's a fork — mark it
  in `registry.yaml` and stop syncing it.
