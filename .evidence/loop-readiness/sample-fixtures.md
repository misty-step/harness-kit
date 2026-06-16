# Loop Readiness Sample Fixtures

Reference: `harnesses/shared/references/loop-readiness.md`

## Dependency Bump Loop

Verdict: accepted only if the handoff names a package scope, state file,
consumer build, changelog/report artifact, max iterations, no-progress rule,
token/dollar budget, and human review before merge.

## Architecture Rewrite Loop

Verdict: rejected. The goal is not recurring, the verifier would be subjective,
and broad self-improvement would invite the worker to grade its own design.

## CI Triage Loop With No Automated Verifier

Verdict: rejected until a verifier exists. A loop that cannot distinguish
progress from repeated failure is a billing incident, not automation.
