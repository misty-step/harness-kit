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
      --locked --workspace`, then boots the API and hits it, then builds the
      SDK — see child 4. MCP boot stays a named, explicit gap: the crate is a
      one-line placeholder with no protocol implementation to boot, see child
      4's notes.)
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
4. [x] Add boot/protocol checks for API, MCP, SDK, and deploy only after the build
   path is stable. **API and SDK done; MCP explicitly still not attempted.**
   `template_check::check_api_boots_and_serves` spawns the built
   `{{binary}}-server` on a free port, polls `/healthz` to readiness, then
   replays `/readyz`, `GET /items`, a valid `POST /items`, and a malformed
   `POST /items` asserting the 4xx validation-error path — a `ChildGuard`
   kills the process on any exit path so a failing assertion never leaks a
   bound port. `template_check::check_sdk_builds` runs `bun install` +
   `bun run build` (reusing the `bun` already provisioned in CI via
   `oven-sh/setup-bun`, no new Node.js toolchain needed) then a throwaway
   consumer script that imports the built `dist/index.js` and asserts the
   client class's public method exists — the template README's own "SDK
   face: throwaway consumer build" requirement. Both checks are proven to
   have teeth, not just green-by-default: deliberately reverting the SDK
   fix below and re-running `check-template` reproduced the real `tsc`
   type error through the gate (exit 1), confirmed live, then reverted.
   Found and fixed two real template bugs along the way (see Notes): a
   missing `tsconfig.json.tmpl` and a strict-mode type error in the SDK's
   `listItems()`. **MCP still not attempted** — `crates/mcp/src/main.rs.tmpl`
   is a one-line placeholder (`eprintln!("...replace with SDK-backed stdio
   or streamable HTTP server")`) with zero protocol implementation. There is
   nothing to boot or handshake with yet; writing a fake "MCP check" against
   a stub that implements no protocol would be structural theater, not
   proof (the harness-engineering doctrine's own warning: "structural eval
   trees are not semantic proof"). Making this real requires picking and
   wiring an actual MCP SDK into the template first — a product-shaping
   decision for the flagship template's MCP face, not a test-writing task;
   left for an operator-directed pass.
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

**2026-07-02 — child 4 landed for API + SDK; MCP scoped out explicitly.**
Real bugs the new checks caught immediately: the SDK template had no
`tsconfig.json.tmpl` at all (the `package.json`'s `build` script referenced
one that didn't exist) and, once added, a strict-mode type error
(`response.json()` typed as `unknown` under `@types/node`, not `any`) in
`listItems()`. Both fixed
(`sdk/typescript/tsconfig.json.tmpl`,
`sdk/typescript/package.json.tmpl` — added `@types/node`,
`sdk/typescript/src/index.ts.tmpl` — explicit cast). This epic stays
`in-progress`: child 5 (store/auth backfill) still has no consumer to
backfill from, and MCP within child 4 needs an operator call on which SDK to
adopt before it can get a real boot check.
