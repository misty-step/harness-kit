# Antigravity CLI Harness Notes

Antigravity CLI is the Google-family provider lane for Harness Kit. The local
binary is `agy`.

## Dispatch Shape

Use print mode with every option before `--print`:

```sh
agy --dangerously-skip-permissions --print-timeout 10m --print "Read AGENTS.md and summarize the gate."
```

`--print` consumes the next argument as the prompt. If `--print-timeout` appears
after `--print`, Antigravity treats the timeout flag text as the prompt and the
provider lane can exit successfully without doing the requested work. Keep
`--print` last in roster dispatch commands so `harness-kit-checks dispatch-agent` can
append the prompt safely.

Useful local checks:

```sh
agy --help
agy --print-timeout 45s --print "Reply with exactly: AGY_OK"
agy --print-timeout 90s --dangerously-skip-permissions --print "Read AGENTS.md and report the gate."
agy --add-dir /path/to/repo --print-timeout 90s --dangerously-skip-permissions --print "Read project.md."
```

`--add-dir` lets a lane run from a neutral working directory while granting
workspace access to the target repo.

## Observed Behavior

- `agy --help` is the safest availability probe.
- `agy plugin list` currently works but may report `No imported plugins`.
- `agy changelog` is useful for CLI behavior changes; version 1.0.2 fixed
  print-timeout and fallback skill-discovery bugs.
- `--dangerously-skip-permissions` maps to broad auto-approval. Use it only for
  bounded provider lanes with explicit scope, output shape, and timeout.
- A successful process exit is not enough. Inspect the transcript or ask for a
  constrained sentinel response when smoke-testing, because command-order errors
  can produce a successful but irrelevant setup/status answer.

## Projection Paths

Harness Kit projects first-party skills directly into the Antigravity CLI skill
path. On this machine `~/.gemini/antigravity-cli/skills` resolves to
`~/.gemini/config/skills`; `bootstrap.sh` populates that directory with
per-skill symlinks and links shared `AGENTS.md` into
`~/.gemini/antigravity-cli/AGENTS.md`.

Antigravity plugins are a separate packaging surface. The documented plugin
shape is:

```text
~/.gemini/antigravity-cli/plugins/<plugin_name>/
├── plugin.json
├── skills/
├── agents/
├── rules/
├── hooks.json
└── mcp_config.json
```

Harness Kit does not emit a plugin bundle by default. Use the direct skill
projection for first-party skills; create a plugin only when a repo needs a
single deployable unit that combines skills with MCP servers, rules, or hooks.

## Rules, Hooks, and Settings

Rules are Antigravity-owned policy files. Harness Kit's shared doctrine lives
in `AGENTS.md`; do not duplicate it into Antigravity rules unless a repo has an
Antigravity-only constraint that cannot live cross-harness.

Hooks belong in Antigravity `hooks.json` files. Harness Kit currently installs
Claude hooks only; do not translate them blindly. Add Antigravity hooks only
after a shaped ticket proves the hook event, matcher, and JSON stdin/stdout
contract are stable.

Settings live in `~/.gemini/antigravity-cli/settings.json` and are user-owned.
Bootstrap may link shared guidance and skills, but it must not overwrite
Antigravity settings or permission policy.

## Legacy Gemini CLI Migration

Treat Gemini CLI as migration/import support, not the future Google harness.
Official Antigravity migration docs describe one-time import of Gemini CLI
extensions, skills, commands, MCP servers, and hooks into Antigravity plugins.
Harness Kit should preserve enough notes to help an operator migrate, while
new provider lanes and bootstrap projections target `agy` and Antigravity
paths.

## Harness Kit Rule

Roster entries should keep Antigravity conditional until a local smoke proves
the prompt was followed. Receipts and final synthesis should treat Antigravity
output as evidence, not authority, like every other provider lane.

## Dynamic Delegation Notes

- Use Antigravity for a bounded Google-family perspective, especially design,
  critique, docs, and cross-check lanes.
- Keep `--print` last so `harness-kit-checks dispatch-agent` can append the scoped
  commission safely.
- Give the lane role, objective, scope, output shape, and boundaries; do not
  rely on project-global chat context.
- Record receipts for followed, failed, or irrelevant outputs. A zero exit is
  not enough evidence that the prompt was obeyed.
- The lead agent owns final synthesis and verification after reading the
  Antigravity evidence.
