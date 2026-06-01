# CLI Design Contract

Use this when a context packet creates or changes a command-line interface.
Inspired by Peter Steinberger's `create-cli` skill and the public CLI
guidelines at https://clig.dev/.

## Lock The Interface

- Command name and one-sentence purpose.
- Primary user: human, script, or both.
- Inputs: args, stdin, files, URLs, config, env.
- Outputs: human text, `--json`, `--plain`, files, artifacts.
- Interactivity: prompts allowed, `--no-input`, confirmations.
- Safety: `--dry-run`, `--force`, `--confirm`, destructive operations.
- Config precedence: flags > env > project config > user config > defaults.
- Platform/runtime: macOS/Linux/Windows, single binary vs runtime.

## Defaults

- `-h` and `--help` always show help and ignore other args.
- `--version` prints the version to stdout.
- Primary data goes to stdout; diagnostics and progress go to stderr.
- Machine output uses `--json`; stable line output uses `--plain`.
- Prompts only when stdin is a TTY; `--no-input` disables prompts.
- Destructive non-interactive runs require `--force` or explicit confirmation.
- Respect `NO_COLOR` and `TERM=dumb`; provide `--no-color` when colored output
  is otherwise default.
- Ctrl-C exits quickly with bounded cleanup.

## Packet Requirements

For CLI work, the context packet should include:

```markdown
## CLI Surface
- Command tree:
- Usage:
- Args/flags:
- Output contract:
- Error/exit code map:
- Config/env precedence:
- Safety controls:
- Examples:
```

If the command is script-facing, include golden examples for stdout/stderr and
exit codes. If it is human-facing, include at least one common happy path and
one failure example.
