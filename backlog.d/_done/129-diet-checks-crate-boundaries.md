# Diet the checks crate into clear maintenance boundaries

Priority: P1 (shipped) · Status: done · Estimate: L · Shipped: 2026-07-02

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
4. [x] Repeat by domain until `harness-kit-checks` is a maintenance gate and install
   tool rather than a grab bag. `harness-kit-roster` split landed:
   `agent_roster` + `lane_harness` + `summarize_delegations` +
   `source_refs` (~4200 LOC) moved to `crates/harness-kit-roster/`. The
   mutual `agent_roster` ↔ `lane_harness` dependency resolved cleanly by
   moving both together as internal modules of the new crate — Rust
   forbids cycles *between* crates, not *within* one, so pairing them was
   never the actual obstacle. `source_refs.rs` was re-classified from
   "core" to "roster" after checking the real dependency graph directly
   (`grep -rln "source_refs::"`) instead of trusting `BOUNDARIES.md`'s
   original guess — nothing in core actually used it, so keeping it in
   core would have created a real two-way crate cycle
   (`harness-kit-checks` ↔ `harness-kit-roster`) once `main.rs` started
   depending on the roster crate for CLI dispatch. `harness-kit-checks`
   depends on `harness-kit-roster` as a library exactly like it already
   depends on `harness-kit-hooks`; the reverse dependency is zero.
   Verified live post-split: `probe-agent-roster --validate-only` and a
   full `record-delegation` → `summarize-delegations` round-trip both
   produce correct output through the new crate boundary;
   `cargo test --workspace` carries all 34 roster-cluster tests over
   unmodified (184 total workspace tests, 0 failed). God-file baseline
   updated to the new paths at unchanged ceilings (no gate loosened).

## Notes

Operator decision, 2026-07-01: "Checks-crate diet: keep gates; split/park
roster+hooks+site sprawl."

The teardown judged the gate set itself strong. This epic is not permission to
weaken gates; it is a boundary and ownership cleanup.

**2026-07-02 — closing.** Both split candidates `BOUNDARIES.md` identified
(`harness-kit-hooks`, `harness-kit-roster`) are landed. The remaining
"keep in core" modules are gates, bootstrap/install, and catalog-integrity
tooling — the "true checks identity" per the inventory's own definition —
and `BOUNDARIES.md`'s "Park / delete" section identifies no further
candidates. `harness-kit-checks` is now 12.3k LOC/24 files, down from
18.1k/26 at the epic's start; the two extractions (~2.6k LOC hooks, ~4.2k
LOC roster) moved dispatch/receipt/hook-runtime concerns out into their own
crates with their own audiences, leaving gates+bootstrap+install+catalog as
the core identity `ci_check.rs` orchestrates directly.
