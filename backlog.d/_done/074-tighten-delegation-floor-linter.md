# Tighten delegation-floor lint from keyword scan to negative fixtures

Priority: P2
Status: merge-ready
Estimate: S

## Goal

Make `harness-kit-checks check-agent-roster` reject weak or performative delegation
floor sections that merely contain the right keywords without actually
committing the workflow to roster-backed delegation.

## Oracle

- [x] Add a fixture or test case with a deliberately weak `## Delegation Floor`
      section that mentions words like `lane`, `receipt`, `context`, and
      `lead` but does not state the full contract.
- [x] `cargo test --workspace --locked agent_roster` proves the weak fixture
      is rejected.
- [x] The accepted fixture still proves a complete floor with roster default,
      direct-work exceptions, lane responsibilities, context boundary,
      evidence/receipt contract, and lead verification.
- [x] `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .` and
      `dagger call check --source=.` pass.

## Notes

Raised from the `/reflect cycle` for shipped backlog `063`. The shipped gate is
useful, but provider audit noted that the current requirement matcher is still
keyword-shaped. A negative fixture is the smallest durable hardening step; do
not turn this into frontmatter metadata or a semantic workflow engine unless a
later failure proves that complexity is needed.

## Progress

- Added commitment-level delegation-floor checks for two-provider dispatch,
  direct-work exceptions, scoped lane handoff, and lead-owned synthesis.
- Added negative fixtures for keyword stuffing and hedged pattern-shaped prose,
  plus a positive fixture that proves complete reordered direct-work
  exceptions still pass.
- Accepted provider review feedback from `grok-build` and `claude`; `pi`
  timed out and was recorded as a failed lane.

## Verification

- `cargo test --workspace --locked agent_roster`
- `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`
- `dagger call check --source=.`
