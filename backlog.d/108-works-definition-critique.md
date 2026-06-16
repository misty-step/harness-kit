# Context Packet: "Works" definition critic

Priority: P1
Status: ready
Estimate: M

## PRD Summary

- User: operators reviewing agent-written changes before merge.
- Problem: agents can satisfy tests while missing the human/product dimensions
  of "works": public API shape, CLI ergonomics, UX cohesion, performance
  tradeoffs, compatibility boundaries, and roadmap fit.
- Goal: add a reusable works-critique reference and route it into `/shape`,
  `/deliver`, `/qa`, and `/code-review` when changes touch public surface,
  performance, compatibility, or user workflow.
- Why now: the hit-list quote captures a recurring failure not fully covered by
  current "tests pass is not QA" language.
- UX enabled: reviewers can attack "works" claims with a concrete checklist and
  fresh-context prompt instead of broad taste complaints.
- Deliverable type: doctrine/reference update plus critic prompt fixture.
- Success signal: a sample review prompt produces specific concerns for API,
  CLI/UX, performance, or compatibility rather than generic "run tests" advice.

## Product Requirements

- P0: Keep this as a critic lens/reference, not a standalone skill.
- P0: Attach the lens only when relevant: public API/CLI/UI surface,
  performance, compatibility, migration, or roadmap-cohesion changes.
- P0: The lens must ask for explicit tradeoffs, not universal preservation of
  compatibility or premature performance work.
- P0: Fresh-context critics see diff + oracle + works lens, not author
  reasoning.
- Non-goals: no prose regex gate, no mandatory works section on every packet, no
  copy-paste of the tweet into global doctrine.

## Repo Anchors

- `harnesses/shared/AGENTS.md` - existing "Validates is not acceptance",
  Completion Evidence, and critic routing tables.
- `skills/shape/SKILL.md` - premise challenge and acceptance oracle discipline.
- `skills/deliver/SKILL.md` - adversarial pre-ship thinking and HTML plan start.
- `skills/qa/SKILL.md` - "Tests pass is not QA" and verdict requirements.
- `skills/code-review/SKILL.md` - reviewer hunt list for plausible-but-wrong
  output.
- `skills/harness-engineering/SKILL.md` - doctrine line vs skill boundary.

## Alternatives

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Do nothing | Current QA/deliver already warns tests are insufficient. | Leaves API/CLI/UX/performance/compatibility failure modes unnamed and inconsistently reviewed. | Reject. |
| Add new `works` skill | Discoverable command. | This is a lens used inside existing workflows, not a full workflow. Adds catalog tax. | Reject. |
| Add global AGENTS section | Always available. | Every-session prose tax for a situational lens. | Reject. |
| Add reference plus routing lines | Low context tax, usable by critics, composable with existing skills. | Needs sample prompt/evidence to avoid becoming decorative prose. | Choose. |
| Add structural linter | Enforces mentions. | Regex over judgment is the known failure mode. | Reject. |

## Design

Add `harnesses/shared/references/works-critique.md` with a compact review card:

- Public surface: does the API/CLI/UI fit nearby conventions and future use?
- Human experience: can the intended user complete the workflow without hidden
  instructions, brittle ordering, or surprise state?
- Performance: what resource tradeoff matters here, and what is intentionally
  not optimized?
- Compatibility: what must remain compatible, what may break, and why?
- Operations: how would production know this "working" behavior degraded?

Then route it from shared critic tables and targeted skill references:

- `/shape`: use when a packet claims behavior is "ready" for public surface or
  compatibility-sensitive work.
- `/deliver`: use before ship for changes touching public surfaces,
  performance, or compatibility.
- `/qa`: include in verdict when tests pass but human workflow still matters.
- `/code-review`: add as an optional reviewer lens.

## Oracle

- New reference exists and is linked from the relevant skill/docs routes.
- `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`
  passes.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
- A committed sample critic prompt under `.evidence/works-critique/` reviews a
  small public-surface fixture and returns findings in at least two of:
  API/CLI/UX, performance, compatibility, operations. A response that only says
  "run tests" fails the sample.

## Premise Source

Premise Source: sha256:ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663 HIT-LIST.md

## HTML Plan

HTML Plan: `.evidence/shape-hit-list/hit-list-shape-index.html#works`

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| New boilerplate on every change | Route only for public surface/performance/compatibility/user workflow changes. |
| Critic gets vague | Provide exact output shape and example fixture. |
| Compatibility conservatism | Require intentional compatibility decision, not compatibility-at-all-costs. |
| Prose tax | Put detail in reference; shared AGENTS gets at most a routing line. |

Rollback: remove references and routing lines; existing `/qa`, `/deliver`, and
`/code-review` behavior remains intact.
