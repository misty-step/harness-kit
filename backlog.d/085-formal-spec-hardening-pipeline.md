# Formal spec-to-hardening pipeline

Priority: P1
Status: ready
Estimate: M

## Goal

Fold the informal-spec -> formal-spec -> acceptance-test -> unit-test -> code
-> refactor -> property-test -> mutation-test -> full-suite pattern into the
core Harness Kit workflow for high-risk changes. The outcome is not "Gherkin for
everything"; it is a clear escalation path that `/shape`, `/implement`,
`/hardening`, `/deliver`, and `/ship` can use when ordinary TDD is not enough
evidence for the blast radius.

## Non-Goals

- Do not require Gherkin, property testing, mutation testing, or CRAP/SCRAP
  analysis for every ticket.
- Do not add slow mutation testing to the default fast Dagger path.
- Do not add language-specific hardening tools to the global harness contract.
- Do not replace human spot checks at the ambiguous/specification boundary.
- Do not duplicate the mechanical evidence checks in `073`.

## Constraints / Invariants

- Start informal, then harden only when the risk justifies it.
- Acceptance behavior comes before implementation. Unit tests drill down after
  an acceptance oracle exists.
- Refactoring is a first-class phase, not cleanup by vibes. Complexity,
  duplication, and shallow tests must be made visible.
- Mutation and acceptance-mutation are evidence tools. Survivors either get
  killed, documented as equivalent, or explicitly waived with rationale.
- Fresh-context critics review the artifact and oracle, not the author's
  reasoning trail.

## Repo Anchors

- `skills/shape/SKILL.md` - context packet should be able to mark a work item as
  "formal-spec required" and include examples/Gherkin where useful.
- `skills/implement/SKILL.md` and `skills/implement/references/tdd-loop.md` -
  ordinary red/green/refactor owner.
- `skills/hardening/SKILL.md` - already covers property, mutation, acceptance
  mutation, CRAP/SCRAP, and DRY modes.
- `skills/deliver/SKILL.md` - phase router and merge-readiness receipt.
- `backlog.d/073-mechanical-hardening-evidence-gates.md` - mechanical evidence
  gate this ticket depends on.
- `backlog.d/065-repo-grounded-acceptance-contract.md` - live-repo acceptance
  evidence contract.

## Pipeline Shape

Use this ladder only for high-risk or ambiguity-heavy changes:

1. `/shape` captures the informal specification and states why formalization is
   required.
2. A specifier lane converts it into concrete tasks plus examples or Gherkin.
3. The lead spot-checks the formal spec before coding starts.
4. `/implement` writes failing acceptance tests from the examples first, then
   unit tests, then code.
5. `/implement` refactors after green, preserving observable behavior.
6. `/hardening risk` identifies complex or under-tested changed surfaces.
7. `/hardening property` adds invariants where the domain supports them.
8. `/hardening mutation` kills meaningful language-level survivors.
9. `/hardening acceptance` mutates Gherkin/examples/contracts to prove the
   user-facing oracle is connected.
10. `/deliver` runs the full suite, records hardening evidence, and dispatches a
    fresh critic against the diff and oracle.

## Oracle

- [ ] `/shape` defines when a packet must include `Formal Spec Required: yes`
      and which fields are required when set: informal spec, formal examples,
      acceptance oracle, hardening budget, and waiver path.
- [ ] `/implement` recognizes formal-spec packets and starts with acceptance
      tests before unit tests or production code.
- [ ] `/hardening` adds a "formal-spec ladder" reference that composes existing
      risk/property/mutation/acceptance modes without restating all of them in
      the main skill.
- [ ] `/deliver` records formal-spec ladder evidence in its receipt when the
      packet required it, including commands run, survivor disposition, and
      critic/verifier result.
- [ ] `073`'s evidence checker recognizes the formal-spec evidence fields once
      that ticket lands.
- [ ] A sample packet or fixture demonstrates a high-risk change that triggers
      the ladder and a low-risk change that does not.
- [ ] `dagger call check --source=.` passes.

## Implementation Sequence

1. Add a reference file under `skills/hardening/references/` describing the
   formal-spec ladder and waiver policy.
2. Patch `/shape` packet output with the optional formal-spec fields and trigger
   criteria.
3. Patch `/implement` to route formal-spec packets through acceptance-first TDD.
4. Patch `/deliver` receipt guidance to carry hardening evidence when required.
5. Add tests/fixtures for packet parsing or lint checks if the repo has a stable
   parser by then; otherwise land prose first and make `073` enforce the fields.
6. Run focused checks plus `dagger call check --source=.`

## Trigger Criteria

Require the ladder when two or more are true:

- The change rewrites core business rules, money/security/auth behavior, data
  migrations, permissions, or cross-service contracts.
- The user-facing behavior is best expressed as examples, scenarios, CLI
  transcripts, API fixtures, or golden files.
- A regression would be expensive to detect manually after merge.
- The changed code has high complexity, low coverage, or a known weak oracle.
- The implementation needs multiple agents or long-running milestones where
  context drift is likely.

## Alternatives Considered

- Always run full ladder: maximum evidence, but too slow. Agents will skip or
  fake it. Reject.
- Keep hardening as ad-hoc user request only: simple, but misses the highest-risk
  changes. Reject.
- Put every detail in `/implement`: easy discovery, but bloats the atomic build
  skill. Reject.
- Compose existing skills with a reference and receipt fields: keeps deep
  modules, but needs `073` to enforce evidence. Choose.

## Research Notes

- The user-provided Uncle Bob flow is accepted as a useful high-rigor pattern:
  informal specification, formal scenarios, acceptance tests, unit tests, code,
  refactor, property tests, mutation, acceptance mutation, full suite, then
  human spot check.
- Factory Missions uses milestone validation and fresh workers for long-running
  work; this ticket adapts that idea to high-risk Harness Kit delivery without
  adding a mission-control engine.
- Anthropic's Claude Code best practices emphasize verification, exploration
  before coding, aggressive context management, subagents for investigation,
  parallel sessions, and adversarial review; the ladder makes those behaviors
  explicit only where the risk warrants them.
- OpenAI's harness-engineering writeup frames the engineer's role as designing
  environments, intent, and feedback loops. This ticket adds a heavier feedback
  loop for changes where normal tests are insufficient.

## Delegation Evidence

- claude receipt `36409cc3-9916-4abb-be5a-a29a2c61b73a` put `065` in the top
  five after doctrine/runtime cleanups; accepted as evidence that this should
  compose with repo-grounded acceptance rather than sit as a standalone idea.
- pi receipt `1745ac96-93ea-46b7-ba4e-a8bea0ecd7d0` ranked `073` second;
  accepted as evidence that formal-spec workflow prose needs mechanical evidence
  fields before it can be trusted at scale.

## Related

- Depends on: `073`.
- Composes with: `065`, `084`.
