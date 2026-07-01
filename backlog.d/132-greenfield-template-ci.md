# Declare and verify the greenfield Factory template

Priority: P1 · Status: pending · Estimate: M

## Goal

Make the one-core-many-faces template explicitly greenfield-only and keep it
from drifting by instantiating and building it in CI.

## Oracle

- [ ] `skills/harness-engineering/references/one-core-many-faces.md` and the
      template README state that the template is for greenfield products, not
      fleet retrofits.
- [ ] A repo-owned fixture or command materializes
      `skills/harness-engineering/templates/one-core-many-faces/` with sample
      substitutions in a temp directory.
- [ ] The materialized template runs at least `cargo build --locked`; if API or
      MCP boot is not yet runnable, the gap is named as a child rather than
      hidden by the test.
- [ ] The CI/local gate includes the template instantiation check or an explicit
      sub-gate that `check --repo .` runs.
- [ ] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

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

1. Add the greenfield-only declaration to the template reference and README.
2. Write the smallest Rust-owned materializer/check that substitutes sample
   tokens into a temp workspace.
3. Build the generated workspace and capture the first failure honestly.
4. Add boot/protocol checks for API, MCP, SDK, and deploy only after the build
   path is stable.
5. Backfill store/auth pieces from a golden repo only when a consumer needs
   them and the generated check can prove them.

## Notes

Operator decision, 2026-07-01: "Template: declare greenfield-only;
CI-instantiate it so it can't drift."

The teardown verified that PR #137 added the deploy layer. It also found no CI
lane instantiating the template and no evidence that retrofitting existing fleet
repos is economical. This epic resolves the template's posture before adding
more faces.
