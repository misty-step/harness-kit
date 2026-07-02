# Declare and verify the greenfield Factory template

Priority: P1 · Status: in-progress · Estimate: M

## Goal

Make the one-core-many-faces template explicitly greenfield-only and keep it
from drifting by instantiating and building it in CI.

## Oracle

- [x] `skills/harness-engineering/references/one-core-many-faces.md` and the
      template README state that the template is for greenfield products, not
      fleet retrofits.
- [x] A repo-owned fixture or command materializes
      `skills/harness-engineering/templates/one-core-many-faces/` with sample
      substitutions in a temp directory. (`harness-kit-checks check-template`,
      `crates/harness-kit-checks/src/template_check.rs`.)
- [x] The materialized template runs at least `cargo build --locked`; if API or
      MCP boot is not yet runnable, the gap is named as a child rather than
      hidden by the test. (Runs `cargo generate-lockfile && cargo build
      --locked --workspace` — a fresh instantiation has no committed
      Cargo.lock, so generate-lockfile-then-locked-build is the template's own
      documented flow, not a weakening of `--locked`. API/MCP boot and SDK
      consumer builds are child 4, explicitly not attempted here.)
- [x] The CI/local gate includes the template instantiation check or an explicit
      sub-gate that `check --repo .` runs. (`check-template` gate lane in
      `ci_check.rs`, verified both directions live: passes on the real
      template, and fails with the real `cargo` compile error when a `.tmpl`
      file was deliberately corrupted for the test, then reverted.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Verification System

- Claim: the Factory product template is an executable greenfield scaffold, not
  doctrine plus uncompiled `.tmpl` files.
- Falsifier: a placeholder substitution is missing; generated Cargo workspace
  fails to build; docs imply retrofit is expected; the gate does not exercise
  the template.
- Driver: template materialization fixture, `cargo build --locked` in the
  generated workspace, docs/reference inspection, and the aggregate repo gate.
- Grader: generated files contain no unreplaced tokens; build exits 0 or names
  a deliberate unsupported face; docs say greenfield-only; gate output is green.
- Evidence packet: materialization transcript, build output, and PR diff.
- Cadence: every template edit.

## Children

1. [x] Add the greenfield-only declaration to the template reference and README.
2. [x] Write the smallest Rust-owned materializer/check that substitutes sample
   tokens into a temp workspace. (`template_check::materialize` — walks every
   `.tmpl` file, strips the suffix, substitutes the 10 documented tokens;
   `README.md` is correctly excluded since it documents the template rather
   than being part of the generated tree.)
3. [x] Build the generated workspace and capture the first failure honestly.
   (`template_check::build_generated_workspace`; a deliberately broken
   `crates/core/src/lib.rs.tmpl` produced the real `rustc` "unclosed
   delimiter" error surfaced verbatim through the gate, exit code 1 —
   verified live, not just asserted in a unit test.)
4. [ ] Add boot/protocol checks for API, MCP, SDK, and deploy only after the build
   path is stable. **Not attempted.** `cargo build --locked --workspace`
   compiles all 5 crates (core/shell/api/cli/mcp) but nothing runs the API
   server, boots the MCP stdio server, or builds the TypeScript SDK — those
   need protocol-specific drivers (HTTP request replay, MCP `initialize`
   handshake, `npm install && tsc`) this pass didn't build.
5. [ ] Backfill store/auth pieces from a golden repo only when a consumer needs
   them and the generated check can prove them. **Not attempted** — no
   current consumer of this template exists to backfill from.

## Notes

Operator decision, 2026-07-01: "Template: declare greenfield-only;
CI-instantiate it so it can't drift."

The teardown verified that PR #137 added the deploy layer. It also found no CI
lane instantiating the template and no evidence that retrofitting existing fleet
repos is economical. This epic resolves the template's posture before adding
more faces.

**2026-07-01 — children 1-3 landed.** `check-template` runs as part of
`check --repo .` (adds ~10-12s per run: `cargo generate-lockfile` + a cold
`cargo build --locked --workspace` of a fresh 5-crate workspace with real
dependencies — axum, tokio, clap — resolved fresh each run since the template
intentionally ships unpinned version ranges, not a frozen lockfile). Children
4-5 stay open for a future pass; they need real protocol drivers, not more
`cargo build` coverage.
