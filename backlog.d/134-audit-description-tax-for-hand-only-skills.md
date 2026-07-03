# Audit description tax: flag hand-only skills for user-invoked projection

Priority: P2 · Status: pending · Estimate: S

## Goal

Stop paying always-loaded description context for first-party skills that only
ever fire by explicit operator trigger.

## Oracle

- [ ] Telemetry run lists every first-party skill with zero model-initiated
      invocations over the sample window (or documents that telemetry cannot
      split invocation source, as a finding)
- [ ] Each candidate gets a verdict: keep model-invoked (with the story) or
      project as user-invoked where the harness supports it
- [ ] At least one harness projection (Claude Code
      `disable-model-invocation`) applied, or explicitly rejected with reason

## Verification System

- Claim: some first-party skill descriptions pay context load with no
  autonomous-invocation payoff.
- Falsifier: telemetry shows model-initiated invocations for every skill, or
  a user-invoked projection breaks a real invocation path (skill stops firing
  when it should).
- Driver: `cargo run --locked -p harness-kit-checks -- telemetry --repo .`
- Grader: operator review of the per-skill invocation-source split.
- Evidence packet: telemetry output + per-skill verdict table in the groom
  report or this ticket.
- Cadence: once now; fold into `/groom audit` if it pays.

## Notes

Source: Matt Pocock, "Writing Great Skills" — model-invoked skills pay
context load (description in the window every turn); user-invoked skills pay
cognitive load (the operator is the index). Cross-harness-first: user
invocation is a per-harness projection concern, never a source-skill change.
Open question: whether current telemetry distinguishes model-initiated from
operator-typed invocations per harness — if not, that gap is the first
finding. If hand-only skills multiply past memory, Pocock's router-skill
cure applies (one user-invoked skill that names the others).
