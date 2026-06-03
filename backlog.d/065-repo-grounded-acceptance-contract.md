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

- [x] `skills/code-review/SKILL.md` requires every ship verdict to include:
      base/ref reviewed, files inspected, acceptance source, exact gates or
      executable paths exercised, and residual unverified runtime paths.
- [x] `skills/qa/SKILL.md` names "repo-fit evidence" explicitly: app shape,
      live path chosen, concrete command/URL/tool call, artifact location, and
      why adjacent tests were or were not enough.
- [x] `skills/ship/SKILL.md` refuses to land merge-ready work that lacks
      acceptance evidence for the exact HEAD, and carries accepted evidence
      refs into the final report and `/reflect` packet.
- [x] `skills/create-repo-skill/SKILL.md` adds a post-generate acceptance section that
      compares vendored harness state against live repo language, commands,
      docs, shared skill root, bridge topology, and known user corrections from
      session history.
- [x] `skills/deliver/SKILL.md` treats "valid but not repo-fit" as dirty:
      if review, QA, or repo-local vendoring evidence says the output is
      generic, stale, or unexercised, `/deliver` exits with remaining work instead of
      merge-ready.
- [x] `harnesses/shared/AGENTS.md` gets a compact always-on rule:
      "validates is not acceptance; name the live repo evidence and the
      repo-fit check before claiming done."
- [x] A regression case exists under the most appropriate skill eval suite
      where a synthetic harness passes structural checks but fails repo-fit
      evidence because commands, language, or skill-root adoption are wrong.
- [x] `dagger call check --source=.` passes.

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

### Implementation Notes

- `skills/seed/SKILL.md` is retired/not present on current `master`; the live
  repo-local vendoring/generation owner is `/create-repo-skill`, so the oracle
  was satisfied there rather than reviving `/seed`.
- `/code-review`, `/qa`, `/deliver`, and `/ship` now name live repo evidence,
  acceptance source, exact exercised command/path/route, repo-fit check, and
  residual risk in their existing `Completion Gate` flow.
- `/create-repo-skill` and `/harness-engineering` now include schema-valid
  post-generate/post-sync `Acceptance Evidence` templates for repo-fit proof.
- New eval case: `skills/code-review/evals/cases/repo-fit-evidence.md`; grader:
  `skills/code-review/evals/graders/check-repo-fit.sh`.

### Delegation Evidence

- `claude` (`3019a7c9-b794-40ec-a270-e5d7044e6607`) identified the exact
  missing fields in code-review, QA, ship, deliver, and create-repo-skill, and
  caught the dead `/seed` path plus the `Acceptance Evidence` checker trap.
- `grok-build` (`f99ea110-90ab-4eb1-b711-a481ec6c2f67`) converged on the
  shared "validates is not acceptance" rule, repo-fit fields, and code-review
  eval placement; its narrower patch recommendation was rejected where the
  backlog oracle explicitly named QA, deliver, and ship.

### Verification

- `bash scripts/build-docs-site.sh`
- `bash scripts/generate-index.sh`
- `git diff --check`
- `python3 scripts/check-evidence-blocks.py skills`
- `python3 scripts/check-agent-roster.py`
- `python3 skills/harness-engineering/scripts/validate-evals.py`
- `bash scripts/check-docs-site.sh`
- `skills/code-review/evals/graders/check-repo-fit.sh <sample-output>`
- `dagger call check --source=.` -> 16 passed, 0 failed

### Relationship to other tickets

- Complements `063-dynamic-delegation-skill-contract.md`: delegation says who
  investigates; this ticket says what evidence proves the workflow is done.
- Complements `053-skill-quality-audit-mode.md`: audit finds missing skill
  quality coverage; this ticket changes the workflow success contract.
- Generalizes part of `054-clean-context-philosophy-bench.md`: reviewers still
  need clean context, but final verdicts must cite the artifact and acceptance
  evidence they reviewed.
