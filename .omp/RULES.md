# Harness Kit — sticky invariants

These are re-injected every turn. Violating one breaks the gate or the repo
contract. Full contract: read `AGENTS.md` (loaded as context) and
`harnesses/shared/AGENTS.md`.

- **`index.yaml` is generated.** Never edit it by hand. Run `cargo run --locked
  -p harness-kit-checks -- bootstrap` to regenerate.
- **Never lower a gate** to go green. No skipped tests, loosened lint, weakened
  thresholds. Diagnose the root cause instead.
- **Skills are self-contained.** No `$REPO_ROOT` sourcing, no `../..` escapes.
  Scripts, libs, and references live under the skill directory.
- **Base branch is `master`.** Gate is
  `cargo run --locked -p harness-kit-checks -- check --repo .`. Green means the
  Rust-owned local gate passed.
- **No source-repo skill bridges.** Do not commit `.agents/skills/`,
  `.codex/skills/`, `.claude/skills/`, `.pi/skills/`, or
  `.antigravitycli/skills/` — they duplicate the global install in `skills/`.
- **Durable tooling is Rust.** The only allowed non-Rust surface is `bootstrap.sh`
  as the curl-compatible launcher. Every gate must name a real failure it catches.
- **`harnesses/claude/settings.json` is copied by bootstrap.** Changes require
  re-bootstrap.
- **Backlog closure** = move to `backlog.d/_done/` with `Status: done` and a
  `Closes-backlog:` / `Ships-backlog:` trailer, or an explicit backlog move
  committed with the work.
