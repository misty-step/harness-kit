# Repo-grounded acceptance contract for workflow skills

Priority: P1
Status: ready
Estimate: M

## Goal

Make every review, delivery, repo-local vendoring, and QA workflow prove two things before
it can claim success:

1. the workflow used live repo evidence rather than generic harness knowledge;
2. the result is actually repo-fit, not merely structurally valid.

This captures the recurring prompt-debt pattern where agents stop at "the gate
passed" or "the scaffold validates" even though the generated surface is still
generic, stale, or untested against the changed executable path.

## Non-Goals

- Do not add a new orchestration layer or semantic workflow database.
- Do not lower any gate or replace `dagger call check --source=.`.
- Do not require every small edit to run every phase. The contract applies when
  the workflow claims review, delivery, repo-local vendoring, or QA success.
- Do not encode repo-specific stack doctrine in Harness Kit. Harness Kit defines
  the evidence shape; repo-local config or vendored skills supply the concrete
  commands, files, and acceptance gates.

## Oracle

- [ ] `skills/code-review/SKILL.md` requires every ship verdict to include:
      base/ref reviewed, files inspected, acceptance source, exact gates or
      executable paths exercised, and residual unverified runtime paths.
- [ ] `skills/qa/SKILL.md` names "repo-fit evidence" explicitly: app shape,
      live path chosen, concrete command/URL/tool call, artifact location, and
      why adjacent tests were or were not enough.
- [ ] `skills/ship/SKILL.md` refuses to land merge-ready work that lacks
      acceptance evidence for the exact HEAD, and carries accepted evidence
      refs into the final report and `/reflect` packet.
- [ ] `skills/seed/SKILL.md` adds a post-install acceptance section that
      compares vendored harness state against live repo language, commands,
      docs, shared skill root, bridge topology, and known user corrections from
      session history.
- [ ] `skills/deliver/SKILL.md` treats "valid but not repo-fit" as dirty:
      if review, QA, or repo-local vendoring evidence says the output is
      generic, stale, or unexercised, `/deliver` exits with remaining work instead of
      merge-ready.
- [ ] `harnesses/shared/AGENTS.md` gets a compact always-on rule:
      "validates is not acceptance; name the live repo evidence and the
      repo-fit check before claiming done."
- [ ] A regression case exists under the most appropriate skill eval suite
      where a synthetic harness passes structural checks but fails repo-fit
      evidence because commands, language, or skill-root adoption are wrong.
- [ ] `dagger call check --source=.` passes.

## Notes

### Why this belongs in Harness Kit

This is not a Curb, Canary, ThinkTank, or Brandt rule. It is the repeated
failure mode of the harness layer itself: the agent knows how to produce a
plausible artifact, but the operator has to ask again for live repo evidence,
actual command paths, authenticated QA, or a before/after harness diff.

Harness Kit should own the generic acceptance contract because it owns the
workflow primitives. Downstream repos should only own the concrete evidence:
their commands, routes, skill roots, deployment surfaces, and invariants.

### Prompt-debt evidence

Recent sessions showed the same pattern across several domains:

- `gradient init` and related harness dogfood could pass validation while
  producing generic skill surfaces that missed language, commands, and bridge
  topology.
- QA receipts could stop at signed-out or adjacent evidence when the changed
  behavior needed authenticated browser/upload smoke or direct executable-path
  verification.
- Delivery and review loops repeatedly needed the instruction that a green gate
  is necessary but not sufficient when a newly added path was never exercised.

### Contract shape

Each relevant skill should produce a short acceptance block:

```markdown
## Acceptance Evidence

- Live repo evidence read:
- Acceptance source:
- Exact command/path exercised:
- Repo-fit check:
- Structural gate:
- Unverified paths / residual risk:
```

This block is deliberately boring. It should be easy to paste into receipts,
review verdicts, QA notes, and final human briefs.

### Relationship to other tickets

- Complements `063-dynamic-delegation-skill-contract.md`: delegation says who
  investigates; this ticket says what evidence proves the workflow is done.
- Complements `053-skill-quality-audit-mode.md`: audit finds missing skill
  quality coverage; this ticket changes the workflow success contract.
- Generalizes part of `054-clean-context-philosophy-bench.md`: reviewers still
  need clean context, but final verdicts must cite the artifact and acceptance
  evidence they reviewed.
