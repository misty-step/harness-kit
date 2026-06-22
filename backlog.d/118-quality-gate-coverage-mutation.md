# Add diff-coverage, mutation, and advisory gates to the quality floor

Priority: P2 · Status: ready · Estimate: M

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
      diff.
- [ ] Mutation gate: `cargo-mutants --in-diff` runs on changed Rust; surviving
      mutants in core modules fail; runtime budget documented.
- [ ] Advisory gate: `cargo-deny check advisories` (RustSec) runs at the network
      tier, separate from the offline `check-supply-chain`; passes on the current
      tree or files the triage.
- [ ] A CI workflow (HK's first real test/gate workflow) runs the
      networked/expensive tier so the fast-local gate stays offline and fast.

## Verification System
- Claim: the coverage-quality and advisory tiers are enforced, not advisory.
- Falsifier: a PR that drops patch coverage, adds a surviving mutant, or pulls a
  RustSec-flagged dependency merges green.
- Driver: `cargo-llvm-cov` + `diff-cover`; `cargo-mutants --in-diff`;
  `cargo-deny check advisories` — in a CI workflow.
- Grader: nonzero exit gates the merge; reports archived under `.evidence/`.
- Evidence packet: CI run links + sample coverage/mutation reports.
- Cadence: per-PR (diff-scoped); full mutation nightly if per-PR is too slow.

## Notes
- Free/OSS only (no Codecov/SonarCloud): `cargo-llvm-cov`, `diff-cover`,
  `cargo-mutants`, `cargo-deny`.
- HK has no test CI workflow today (only `deploy-docs-site.yml`); this epic adds
  the first, which also lets `check-supply-chain`'s offline checks run in CI.
- Two-tier discipline (`/ci`): keep these out of the fast-local pre-push gate;
  they belong at PR/ship time.
- Follow-on candidates once the floor is proven: duplication (jscpd), dead-code
  (cargo-machete/knip), and a complexity *report* tier.
