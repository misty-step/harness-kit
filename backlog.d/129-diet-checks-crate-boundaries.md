# Diet the checks crate into clear maintenance boundaries

Priority: P1 · Status: pending · Estimate: L

## Goal

Preserve the green Harness Kit gate while splitting or parking the unrelated
roster, hook, site, and orchestration sprawl currently living inside
`harness-kit-checks`.

## Oracle

- [ ] A checked-in boundary plan classifies every current
      `crates/harness-kit-checks/src/*.rs` module into keep-in-checks,
      split-to-roster, split-to-hooks, split-to-site/analytics, or park/delete.
- [ ] The first mechanical split moves at least one non-gate domain behind a
      clearer crate or module boundary without changing CLI behavior.
- [ ] Existing CLI entrypoints used by hooks, bootstrap, CI, and docs either
      continue to work or emit an intentional migration message with a tested
      compatibility path.
- [ ] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
- [ ] No gate is disabled, loosened, or removed without a replacement that
      catches the same named failure.

## Verification System

- Claim: the repo-maintenance gate remains boring and reliable while unrelated
  orchestration/hook/site responsibilities stop accumulating in one crate.
- Falsifier: a hook/bootstrap/CI command breaks; a gate disappears; a split
  module creates shallow pass-through crates with no simpler interface.
- Driver: module-boundary inventory, CLI compatibility smoke tests for moved
  entrypoints, crate tests, and the aggregate repo gate.
- Grader: inventory covers all modules; old and new command surfaces match or
  intentionally redirect; tests and gate pass; diff shows a smaller/simpler
  checks interface.
- Evidence packet: boundary plan, compatibility transcript, and PR diff.
- Cadence: one domain per PR after the initial plan; never combine crate split
  with unrelated skill catalog changes.

## Children

1. Write the boundary inventory from the current crate: gates/bootstrap/index as
   the checks core; roster/delegations/lane harness as roster; Claude hooks as
   hooks; docs-site/analytics/external sync as explicit keep/split decisions.
2. Pick the smallest non-gate domain whose CLI compatibility can be proven and
   split it first.
3. Add compatibility tests around the moved entrypoints before changing their
   internals.
4. Repeat by domain until `harness-kit-checks` is a maintenance gate and install
   tool rather than a grab bag.

## Notes

Operator decision, 2026-07-01: "Checks-crate diet: keep gates; split/park
roster+hooks+site sprawl."

The teardown judged the gate set itself strong. This epic is not permission to
weaken gates; it is a boundary and ownership cleanup.
