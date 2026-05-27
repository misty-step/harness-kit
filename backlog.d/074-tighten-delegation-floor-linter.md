# Tighten delegation-floor lint from keyword scan to negative fixtures

Priority: P2
Status: ready
Estimate: S

## Goal

Make `scripts/check-agent-roster.py` reject weak or performative delegation
floor sections that merely contain the right keywords without actually
committing the workflow to roster-backed delegation.

## Oracle

- [ ] Add a fixture or test case with a deliberately weak `## Delegation Floor`
      section that mentions words like `lane`, `receipt`, `context`, and
      `lead` but does not state the full contract.
- [ ] `python3 -m unittest ci.tests.test_agent_roster` proves the weak fixture
      is rejected.
- [ ] The accepted fixture still proves a complete floor with roster default,
      direct-work exceptions, lane responsibilities, context boundary,
      evidence/receipt contract, and lead verification.
- [ ] `python3 scripts/check-agent-roster.py` and
      `dagger call check --source=.` pass.

## Notes

Raised from the `/reflect cycle` for shipped backlog `063`. The shipped gate is
useful, but provider audit noted that the current requirement matcher is still
keyword-shaped. A negative fixture is the smallest durable hardening step; do
not turn this into frontmatter metadata or a semantic workflow engine unless a
later failure proves that complexity is needed.
