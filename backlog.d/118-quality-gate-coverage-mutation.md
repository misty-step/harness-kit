# Add diff-coverage, mutation, and advisory gates to the quality floor

Priority: P2 · Status: in-progress · Estimate: M

## Goal
Extend the standing quality floor
(`harnesses/shared/references/quality-gates.md`) from structural ratchets to the
behavioral coverage-quality and full supply-chain tiers: diff-scoped coverage,
mutation score on core logic, and networked vulnerability advisories — the
tool-integrated gates deferred from the initial dogfood swing.

## Context
The first swing (shipped) added three native, offline gates to
`harness-kit-checks`: `check-godfiles` (ratchet), `check-source-markers`, and
`check-supply-chain` (cargo-deny bans/licenses/sources, offline, graceful-skip
when cargo-deny is absent). The tool-integrated and networked gates were
deferred here because they need external installs and/or network, which the
fast-local Rust gate deliberately avoids.

## Oracle
- [ ] Diff-coverage gate: `cargo-llvm-cov` emits LCOV; `diff-cover` enforces a
      patch-coverage floor on changed lines (not a global %), green on a no-op
      diff. **Deferred — see child 1.**
- [ ] Mutation gate: `cargo-mutants --in-diff` runs on changed Rust; surviving
      mutants in core modules fail; runtime budget documented. **Deferred —
      see child 1.**
- [x] Advisory gate: `cargo-deny check advisories` (RustSec) runs at the network
      tier, separate from the offline `check-supply-chain`; passes on the current
      tree or files the triage. (`check_supply_chain_advisories` in
      `quality_gates.rs`, CLI verb `check-supply-chain-advisories`. Found and
      fixed a real, live finding while landing this: RUSTSEC-2026-0190,
      unsoundness in `anyhow::Error::downcast_mut()`, fixed upstream in
      `>=1.0.103` — this repo was on `1.0.102`; bumped via
      `cargo update -p anyhow`, no code changes needed since nothing here
      calls `downcast_mut`.)
- [x] A CI workflow (HK's first real *networked-tier* gate workflow, separate
      from `ci.yml`'s fast local gate) runs the networked/expensive tier so
      the fast-local gate stays offline and fast.
      (`.github/workflows/advisories.yml` — PR, push-to-master, and a daily
      cron, since a newly published RustSec advisory can turn an unchanged,
      already-merged `Cargo.lock` vulnerable overnight with no code change to
      trigger a PR check.)

## Verification System
- Claim: the coverage-quality and advisory tiers are enforced, not advisory.
- Falsifier: a PR that drops patch coverage, adds a surviving mutant, or pulls a
  RustSec-flagged dependency merges green.
- Driver: `cargo-llvm-cov` + `diff-cover`; `cargo-mutants --in-diff`;
  `cargo-deny check advisories` — in a CI workflow.
- Grader: nonzero exit gates the merge; reports archived under `.evidence/`.
- Evidence packet: CI run links + sample coverage/mutation reports.
- Cadence: per-PR (diff-scoped); full mutation nightly if per-PR is too slow.

## Children

1. [ ] Diff-coverage + mutation tiers. **Deliberately not attempted in the
   advisories pass.** Both need real threshold calibration a single
   unattended session can't responsibly do: `cargo-mutants --in-diff`'s
   runtime scales with diff size and is known to run long even scoped to
   changed lines — the wrong runtime budget either makes the gate useless
   (skips on any real diff) or makes every future PR wait tens of minutes;
   a diff-coverage floor set from a first guess rather than this repo's real
   patch-coverage distribution risks blocking legitimate PRs on day one, and
   there's no branch protection today to make either non-blocking while
   they're tuned (i.e. once added, they read as load-bearing/likely enforced
   immediately, so they need to be right before they land, not tuned after
   the fact against real PR traffic).

## Notes
- Free/OSS only (no Codecov/SonarCloud): `cargo-llvm-cov`, `diff-cover`,
  `cargo-mutants`, `cargo-deny`.
- Correction: this ticket's original claim "HK has no test CI workflow today
  (only `deploy-docs-site.yml`)" was already stale when written —
  `.github/workflows/ci.yml` (running `check --repo .`) predates this ticket.
  What HK genuinely lacked, and what this pass adds, is a *networked-tier*
  workflow separate from the fast local gate.
- Two-tier discipline (`/ci`): keep these out of the fast-local pre-push gate;
  they belong at PR/ship time.
- Follow-on candidates once the floor is proven: duplication (jscpd), dead-code
  (cargo-machete/knip), and a complexity *report* tier.

**2026-07-02 — advisory tier shipped, scoped down from the full epic.** Only
the advisory/RustSec slice landed (child not applicable to that item; see
Oracle checkboxes). Diff-coverage and mutation stay open as child 1 — real
work, not blocked on anything except calibration against live PR traffic,
which this session couldn't safely fabricate. Epic stays `in-progress`.
