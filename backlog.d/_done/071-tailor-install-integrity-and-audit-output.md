# Tighten tailor install integrity and audit output

Priority: P1
Status: retired
Estimate: M

## Outcome

Retired with Tailor. The generator no longer exists; the durable fix is to avoid
generated repo-local harness artifacts unless a future shaped ticket proves they
earn their complexity.

## Goal

Make `/tailor` prove that generated harness references only installed local
assets and that audit artifacts explain placeholder evidence clearly.

The Conviction tailor PR review found several generated-artifact defects:

- `code-review` bench-map references `a11y-auditor`, but the tailored repo did
  not install `.claude/agents/a11y-auditor.md`.
- Bench-map documentation still pointed at `spellbook/agents/` even though the
  generated repo's callable agent root is `.claude/agents/`.
- `deliver/references/branch.md` documented `<type>/<slug>` even when the
  tailored repo contract requires `<type>/<id>-<slug>`.
- Historical gate evidence showed public placeholder Convex and Clerk env vars
  without an explicit note that they were non-secret build placeholders.
- Tailor readlinks are hard to review because generated sections lack stable
  headers.

## Non-Goals

- Do not make every Spellbook agent globally installed in every target repo.
- Do not hand-edit historical audit artifacts as the durable fix.
- Do not add target-repo-specific doctrine to Spellbook source files.

## Oracle

- [ ] `/tailor` validates every generated bench-map agent reference against the
      generated repo's callable agent root.
- [ ] If a generated bench-map rule references `a11y-auditor`, the target repo
      has a callable `a11y-auditor` agent or the rule is omitted.
- [ ] Generated bench-map documentation names the target repo's callable agent
      root, not Spellbook's source `agents/` directory.
- [ ] Generated deliver branch documentation uses the target repo's branch
      contract, including item id plus slug when that is the repo convention.
- [ ] Generated gate evidence labels public placeholder env values as
      non-secret build placeholders whenever placeholders are used.
- [ ] Tailor readlink output includes stable section headers so reviewers can
      navigate generated evidence quickly.
- [ ] Regression coverage installs into a target repo with a web surface and
      proves the bench map references only installed agents.
- [ ] `dagger call check --source=.` passes.

## Notes

This ticket captures the generator-side fix. A downstream PR can still patch
its generated files to address review feedback, but the next `/tailor` run
should not reproduce these defects.
