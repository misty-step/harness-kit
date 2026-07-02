# Fix installed harness-kit-checks CLI self-update blind spot

Priority: P2 · Status: pending · Estimate: S

## Goal

Make `bootstrap`'s CLI install step actually refresh the installed
`~/.harness-kit/bin/harness-kit-checks` binary when a rebuild happened,
instead of permanently freezing at whatever version first got installed.

## Oracle

- [ ] After a source change + `cargo build`, running the *installed* binary
      (`~/.harness-kit/bin/harness-kit-checks check --repo .` or via
      `command -v harness-kit-checks` on `PATH`) reflects the new build —
      not just `cargo run -p harness-kit-checks`.
- [ ] `bootstrap`'s `install_cli` step distinguishes "the running binary is
      already the freshest build" from "the running binary happens to be the
      installed copy, which may itself be stale."
- [ ] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Verification System

- Claim: git hooks (pre-commit, pre-push) that invoke `harness-kit-checks`
  via `PATH` always validate against current source, not a frozen snapshot.
- Falsifier: edit a gate, `cargo build`, then run the installed binary via
  `PATH` — it still reports the pre-edit gate list.
- Driver: a fixture that simulates two builds (older content installed,
  newer content in `target/debug`) and asserts `install_cli` updates the
  installed copy; a live before/after `check --repo .` diff.
- Grader: the fixture fails on current code and passes after the fix; the
  live before/after diff shows the installed binary's gate list changing.
- Evidence packet: fixture test output, live before/after transcript.
- Cadence: whenever `install_cli`/bootstrap's CLI-sync logic changes.

## Notes

Found 2026-07-01 while landing `check-template`
(backlog.d/132-greenfield-template-ci.md): `~/.harness-kit/bin/harness-kit-checks`
was stale by an entire session's worth of commits (missing
`check-eval-coverage` from backlog 128 and `check-template` from backlog 132),
even though every commit's pre-commit hook ran `bootstrap` and reported
"Installing Rust CLI... bin/harness-kit-checks" (implying success).

**Root cause** (`crates/harness-kit-checks/src/bootstrap.rs:install_cli`):
once `command -v harness-kit-checks` resolves to the *installed* copy (which
`.githooks/pre-push` and presumably pre-commit prefer over `cargo run` when
available), every subsequent `bootstrap` invocation runs *as* that installed
binary. `install_cli` compares `env::current_exe()` (the installed binary,
since that's what's running) against the install destination — they're
canonically the same path, so it always concludes "already current" and
skips the copy. There is no code path left that ever copies a freshly built
`target/debug/harness-kit-checks` over the installed one, because nothing
still on `PATH` ever runs `cargo run -- bootstrap` again once the installed
binary takes over. This is a self-perpetuating staleness trap, not a one-off
glitch — it will recur on every future session once the installed binary
existed once.

**Immediate hygiene fix applied same-day** (not a code fix): manually rebuilt
and force-copied `target/debug/harness-kit-checks` over the installed copy so
this session's hooks validate current code going forward. That is a
workaround, not the fix this ticket tracks — the underlying logic still has
the blind spot and will refreeze at the next post-install `bootstrap` run.

A real fix likely needs either: (a) comparing file hash/mtime/build
timestamp instead of path identity, (b) always building fresh
(`cargo build --locked -p harness-kit-checks`) before the self-check rather
than trusting `current_exe()`, or (c) a version/build-id stamp compiled into
the binary that `install_cli` can compare without relying on which copy
happens to be running.
