Run the harness-kit bootstrap and verify skill sync.

Run from the repo root:

```
cargo run --locked -p harness-kit-checks -- bootstrap
```

This installs every first-party skill into each detected harness (Claude, Codex,
Pi, etc.). Externals are vendored at registry pins.

After bootstrap, verify:
1. `index.yaml` is regenerated (never edit by hand).
2. `harnesses/claude/settings.json` is copied to the Claude config.
3. Skills are installed in the global skill directories.

If bootstrap fails, report the exact error. Do not edit `index.yaml` manually —
fix the bootstrap command or the skill source that caused the failure.
