# Diet the checks crate into clear maintenance boundaries

Priority: P1 · Status: in-progress · Estimate: L

## Goal

Preserve the green Harness Kit gate while splitting or parking the unrelated
roster, hook, site, and orchestration sprawl currently living inside
`harness-kit-checks`.

## Oracle

- [x] A checked-in boundary plan classifies every current
      `crates/harness-kit-checks/src/*.rs` module into keep-in-checks,
      split-to-roster, split-to-hooks, split-to-site/analytics, or park/delete.
      (`crates/harness-kit-checks/BOUNDARIES.md`.)
- [x] The first mechanical split moves at least one non-gate domain behind a
      clearer crate or module boundary without changing CLI behavior.
      (`harness-kit-hooks` — PR #147.)
- [x] Existing CLI entrypoints used by hooks, bootstrap, CI, and docs either
      continue to work or emit an intentional migration message with a tested
      compatibility path. (Verified live post-split: `claude-hook
      permission-auto-approve`, `claude-hook time-context`, `claude-hook
      skill-invocation-tracker` all produce identical output — PR #147.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
- [x] No gate is disabled, loosened, or removed without a replacement that
      catches the same named failure. (God-file baseline entry for
      `claude_hooks.rs` moved to its new path, still enforced at the same
      ceiling — not loosened.)

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

1. [x] Write the boundary inventory from the current crate: gates/bootstrap/index as
   the checks core; roster/delegations/lane harness as roster; Claude hooks as
   hooks; docs-site/analytics/external sync as explicit keep/split decisions.
   (`BOUNDARIES.md`, based on real `grep -n "^use crate::"` dependency
   evidence, not vibes — surfaced that `claude_hooks.rs`+`invocation_kind.rs`
   have zero code coupling to the rest of the crate, the cleanest possible
   split candidate.)
2. [x] Pick the smallest non-gate domain whose CLI compatibility can be proven and
   split it first. (`harness-kit-hooks` — PR #147: zero code coupling to the
   rest of the crate, confirmed before moving.)
3. [x] Add compatibility tests around the moved entrypoints before changing their
   internals. (All 39 existing tests moved and pass unmodified; live CLI
   smoke-check for 3 representative hooks — PR #147.)
4. [ ] Repeat by domain until `harness-kit-checks` is a maintenance gate and install
   tool rather than a grab bag. **Not yet done.** `harness-kit-roster`
   (`agent_roster` + `summarize_delegations` + `lane_harness`, ~3941 LOC,
   mutually coupled per `BOUNDARIES.md`) is the next candidate — materially
   riskier than the hooks split since it requires untangling a real mutual
   dependency, not just moving a self-contained pair. Leave this epic
   `in-progress` until that split lands or is explicitly deferred.

## Notes

Operator decision, 2026-07-01: "Checks-crate diet: keep gates; split/park
roster+hooks+site sprawl."

The teardown judged the gate set itself strong. This epic is not permission to
weaken gates; it is a boundary and ownership cleanup.
