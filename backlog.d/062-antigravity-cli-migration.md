# Migrate Gemini support to Antigravity CLI and IDE

Priority: P1
Status: ready
Estimate: L

## Goal

Make Antigravity CLI and IDE first-class harness targets and demote Gemini CLI
to legacy migration support. Google is transitioning Gemini CLI users to
Antigravity CLI; this repo should stop designing new surfaces around Gemini CLI
as the future Google harness.

## Non-Goals

- Do not delete Gemini CLI support before existing local configs and downstream
  repos have a migration path.
- Do not guess undocumented Antigravity config semantics. Use official docs,
  the installed CLI, and observed local state.
- Do not make Antigravity the common abstraction. It is one projection target,
  not the architecture.
- Do not require the Antigravity IDE for terminal-only users. CLI support must
  stand alone.

## Oracle

- [ ] `harnesses/antigravity-cli/` exists with a README, plugin template, skill
      projection notes, rules guidance, hook guidance, and settings guidance.
- [ ] `harnesses/antigravity-ide/` exists or `harnesses/antigravity/` clearly
      documents the split between CLI and IDE paths.
- [ ] `bootstrap.sh` detects Antigravity CLI/IDE installs and reports what it
      would link or copy. Any mutation is conservative and reversible.
- [ ] `bootstrap.sh` and `/seed` know how to expose skills in Antigravity
      skill/plugin locations.
- [ ] Gemini CLI docs and config remain only as legacy import/migration support.
- [ ] `README.md`, `project.md`, `AGENTS.md`, and active harness docs name
      Antigravity instead of Gemini CLI as the Google first-class target.
- [ ] A local smoke check proves Antigravity sees at least one projected skill
      or plugin from this repo.
- [ ] `dagger call check --source=.` passes.

## Notes

### Primary-source findings

Google published a transition notice saying Gemini CLI and Gemini Code Assist
IDE extensions stop serving free, Pro, and Ultra individual requests on
June 18, 2026, and that the same use cases continue in Antigravity CLI:
https://developers.googleblog.com/en/an-important-update-transitioning-gemini-cli-to-antigravity-cli/

Antigravity CLI docs describe plugins as bundles containing skills, agents,
rules, MCP servers, and hooks under
`~/.gemini/antigravity-cli/plugins/<plugin_name>/`, and describe asynchronous
subagents with a `/agents` panel:
https://antigravity.google/docs/cli-features

Antigravity CLI overview says Gemini CLI migration supports one-time import of
existing Gemini CLI extensions, skills, and settings:
https://antigravity.google/docs/cli-overview

### Local observed paths

On this machine, Antigravity-related state exists under:

- `~/.gemini/antigravity-cli/`
- `~/.gemini/antigravity/`
- `~/.gemini/antigravity-ide/`

Observed directories include `plugins/`, `skills/`, `global_skills/`,
`global_workflows/`, `mcp_config.json`, and `settings.json`. Treat these as
implementation evidence to verify, not as a public contract by themselves.

### 2026-05-27 CLI experiment notes

Controlled local checks showed that `agy --help`, `agy plugin list`, and
`agy changelog` are stable low-risk probes. `agy --print-timeout 45s --print
"Reply with exactly: AGY_OK"` follows a sentinel prompt. `agy --add-dir
/path/to/repo ... --print "Read project.md"` can run from a neutral directory
while granting access to the target repo.

The important command-shape finding is that `--print` must be the final flag
before the prompt. With the old roster shape,
`agy --print --dangerously-skip-permissions --print-timeout 10m <prompt>`,
Antigravity treated `--dangerously-skip-permissions` / timeout text as prompt
content and returned setup/permission-status output while exiting successfully.
Harness Kit's roster now uses:

```sh
agy --dangerously-skip-permissions --print-timeout 10m --print <prompt>
```

Keep Antigravity conditional in the roster and require transcript inspection or
a sentinel prompt for smoke tests; a zero exit code alone is not sufficient
evidence that the prompt was followed.
