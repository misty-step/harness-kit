# Fix installed harness-kit-checks CLI self-update blind spot

Priority: P2 (shipped) · Status: done · Estimate: S · Shipped: 2026-07-02

## Goal

Make `bootstrap`'s CLI install step actually refresh the installed
`~/.harness-kit/bin/harness-kit-checks` binary when a rebuild happened,
instead of permanently freezing at whatever version first got installed.

## Oracle

- [x] After a source change + `cargo build`, running the *installed* binary
      (`~/.harness-kit/bin/harness-kit-checks check --repo .` or via
      `command -v harness-kit-checks` on `PATH`) reflects the new build —
      not just `cargo run -p harness-kit-checks`. (Verified live: reproduced
      the bug with the old installed binary — "already current" while
      `target/debug` held a genuinely different build — then proved the fix
      by running the *newly-fixed installed binary's own* `bootstrap` against
      a simulated newer `target/debug` build and watching it correctly
      refresh itself.)
- [x] `bootstrap`'s `install_cli` step distinguishes "the running binary is
      already the freshest build" from "the running binary happens to be the
      installed copy, which may itself be stale." (Option (a) from the notes:
      sha256 content comparison between the freshest `target/{release,debug}`
      build and the installed destination, never `current_exe()` identity.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

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

**2026-07-02 — fixed.** Implemented option (a): `crates/harness-kit-checks/src/cli_install.rs`
(split out of `bootstrap.rs`, see below) picks the newest-by-mtime of
`target/release/harness-kit-checks` / `target/debug/harness-kit-checks` as
the install *source* (falling back to `current_exe()` only when neither
exists), then compares its sha256 against the installed destination's —
content identity, never path/process identity. Live reproduction-then-fix
transcript: with the pre-fix installed binary, running its own `bootstrap`
against a `target/debug` build with different content still reported
"already current" (bug reproduced). After installing the fix via
`cargo run -- bootstrap`, running the *newly-installed, now-fixed* binary's
own `bootstrap` against a simulated newer `target/debug` build correctly
detected the content mismatch and refreshed itself. 3 fixture tests added
(`cli_install::tests`) covering: stale-installed-vs-fresh-debug-build,
identical-content-skip (with an mtime-untouched assertion proving it's a
true no-op, not just idempotent), and release-preferred-over-debug-when-newer.

Incidental fix: extracting `install_cli` into its own module was required
to keep `bootstrap.rs` under the repo's 800-line god-file ceiling (adding the
fix + tests in place would have pushed it to 848 lines) — a small, welcome
head start on `backlog.d/129`'s "repeat by domain" child 4, though the
roster-cluster split that ticket actually tracks is separate and unstarted.
