# Backlog: CI DX Fast Path And Dagger Graph Coarsening

Priority: P0
Status: ready
Type: CI / developer experience

## Problem
Shipping Harness Kit currently pays the full CI cost too many times. In the
observed final-mile run, `dagger call check --source=.` passed on the shipping
ref, passed again on the resolved master squash result, then the pre-push hook
ran the full Dagger check a third time. That third run took roughly 17 minutes
and failed with a Dagger transport/session error rather than a test failure:

```text
Post "http://dagger/query": unexpected EOF
shutdown: do shutdown: Post "http://dagger/shutdown": http2: frame too large
```

The result is bad operator experience and bad release reliability: the work was
already validated, but the push was blocked by repeated orchestration overhead
and Dagger engine fragility.

## Live Evidence
- `.githooks/pre-push` delegates to `harness-kit-checks git-hook pre-push`.
- `crates/harness-kit-checks/src/git_hooks.rs` runs `dagger call check` on
  every push when Dagger and Docker are present; it has no same-tree green
  receipt fast path.
- `ci/src/harness_kit_ci/main.py` runs 31 gates through Dagger with concurrency
  capped at 4.
- Most Rust-backed gates construct a fresh `_rust_container`, then run
  `apt-get`, `rustup component add`, `cargo fetch`, and a small `cargo run`
  command.
- The pre-push Dagger failure did not report a failing gate. It reported a
  Dagger query/cleanup transport failure after the check had already run for
  minutes.
- Projected research lanes for this investigation:
  - Grok conservative CI lane: `0f32d623-8df3-43da-bd93-d3f037d277a6`.
  - Pi radical CI architecture lane:
    `2d7bad6f-8d35-4b4a-820d-fb64b1628a97`.

## Goals
- Preserve `dagger call check --source=.` as the canonical full gate.
- Stop re-running the full gate on the same source tree during ship/push.
- Reduce Dagger engine RPC/container churn without weakening the gate matrix.
- Make CI runtime visible with per-gate timing and cache-hit evidence.
- Keep local DX fast while remote CI remains exhaustive.

## Non-Goals
- No lowering thresholds, deleting checks, or bypassing pre-push by default.
- No making GitHub Actions or another hosted provider the source of truth.
- No semantic workflow engine around CI.
- No host-only green claim that replaces the hermetic Dagger gate.

## Recommended Implementation
### Slice 1: Same-Tree Green Receipt
- Add a local receipt store under `.harness-kit/ci/`.
- Key receipts by source tree hash plus hashes of Dagger/module inputs such as
  `dagger.json`, `ci/src/harness_kit_ci/main.py`, `Cargo.lock`, and relevant
  hook/check source files.
- After a successful full `dagger call check --source=.`, record the receipt.
- Teach `git-hook pre-push` to skip the Dagger rerun when an identical current
  tree has a current green receipt.
- Remote CI ignores local receipts and still runs the full gate.

### Slice 2: Coarsen The Dagger Graph
- Collapse many one-command Rust gates into a smaller number of Dagger execs.
- Prefer a Rust-owned `harness-kit-checks ci check-all --format json` matrix
  that runs individual gates internally and emits structured per-gate results.
- Keep individual subcommands available for focused local debugging.
- Dagger should see fewer long-lived execs with bounded output, reducing RPC
  pressure and transport failure risk.

### Slice 3: Build Once, Run Many
- Build `harness-kit-checks` once inside the Dagger Rust environment.
- Invoke the built binary for small gates instead of repeatedly using
  `cargo run`.
- Preserve `cargo test`, `cargo clippy`, and `cargo fmt` as dedicated checks
  where needed.

### Slice 4: Cache Package Managers Explicitly
- Add Dagger cache volumes for Cargo registry/git/target cache, Bun cache, and
  pip/PyYAML installation where appropriate.
- Keep cache use hermetic: cache dependencies, not generated source truth.

### Slice 5: CI Profiling
- Add a `ci profile` or `check --profile` output mode.
- Record wall time, command time, cache status, and failure detail per gate.
- Include the profile summary in `/ci` reports and failed pre-push output.

## Outside-The-Box Options
- Split local landability from remote certification: local pre-push requires a
  current full-gate receipt or runs a small safety subset; remote CI always
  certifies the complete gate from scratch.
- Add gate input manifests and diff-aware scheduling for local-only checks.
  A gate whose declared inputs are untouched can be skipped locally, while full
  Dagger remains the canonical merge/remote gate.
- Move almost all CI scheduling into Rust and make Dagger a thin hermetic shell
  around one structured executable.
- Add an explicit `ship` handoff receipt so `/ship` can pass same-HEAD gate
  evidence to pre-push instead of triggering a third full run.

## Acceptance Oracle
- A fresh tree with no valid receipt still runs the full Dagger check.
- A tree with an identical current green receipt skips pre-push Dagger and
  reports the receipt hash, tree hash, and gate command that produced it.
- Remote CI and manual `dagger call check --source=.` still run all gates.
- The check matrix still reports every existing gate by name.
- Dagger full gate passes after the coarsening.
- A repeated local ship/push path no longer runs full Dagger three times for
  the same source state.
- CI profile output identifies the slowest setup and gate phases.

## Risks
- Receipt staleness if gate logic changes without invalidating the key. Include
  check-source hashes and binary/module hashes in the receipt key.
- Diff-aware scheduling can miss transitive dependencies. Keep it local-only
  and conservative until measured.
- Coarsening can hide individual gate failures. Require structured JSON output
  and preserve per-gate names in summaries.
- Host-native fast paths can drift from Dagger. Treat them as local convenience
  only, never canonical green.
