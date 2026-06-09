# Context Packet: Require Projected Lane Harnesses For Substantive Dispatch

Priority: P0
Status: ready
Type: harness primitive

## PRD Summary
- User: lead agents composing roster-backed lanes in Harness Kit.
- Problem: a lane can be "bespoke" only in its prompt while still inheriting
  the full ambient harness. That makes dispatch summaries overstate isolation,
  weakens debriefs, and makes cross-provider evals harder to trust.
- Goal: every substantive provider lane must either run with a projected
  `lane_harness.v1` boundary, or carry an explicit waiver that explains why a
  prompt-native lane was acceptable.
- Why now: the projection primitive exists, but `/dispatch` still treats it as
  optional context hygiene and the default lane-card template says
  `Lane harness: none`.
- Success signal: dispatch receipts and summaries can prove
  `projection_status: projected` for required lanes, or show a typed waiver
  before the lane is counted as valid evidence.

## Product Requirements
- P0: add an explicit dispatch policy for lane harness isolation:
  `required`, `optional`, or `waived`.
- P0: fail before provider launch when policy is `required` and no
  `--lane-harness` manifest is supplied.
- P0: require `lane_harness_ref`, `lane_harness_sha256`, and
  `projection_status: projected` in receipts for required projected lanes.
- P0: support a typed waiver with a short reason and scope when projection is
  intentionally skipped for a substantive lane.
- P0: make delegation summaries surface each lane as
  `projected`, `waived`, `not_requested`, or failed projection/provider.
- P0: update `/dispatch` docs, lane-card references, lane-card template, and
  model/provider harness reference docs so the default for substantive lanes is
  projected-or-waived rather than `Lane harness: none`.
- P0: remove stale examples or commands that imply unsupported dispatch flags.
- P1: add a small scaffold path that turns a lane card into a validated
  `lane_harness.v1` manifest without creating a semantic workflow engine.
- P1: add a receipt audit command that can fail a run card or backlog slice
  when substantive provider receipts are prompt-only without waiver.

## Non-Goals
- No provider ranking, fallback tree, or semantic scheduler.
- No global skill installation mutation during a dispatch.
- No claim that projection is a permission or security sandbox; it is a
  context-hygiene and evidence boundary.
- No replacement of prompt-native lane cards with a large new workflow object.
- No requirement that mechanical one-off commands use projection.

## Lead Repo Read
- `skills/dispatch/SKILL.md` currently says to use `lane_harness.v1` projection
  only when context hygiene matters, and calls projection optional.
- `skills/dispatch/references/lane-cards.md` makes `Lane harness` optional and
  uses `none` in common examples.
- `skills/dispatch/templates/lane-card.md` defaults to `Lane harness: none`.
- `crates/harness-kit-checks/src/agent_roster.rs` materializes projection only
  when `DispatchOptions.lane_harness` is present.
- `crates/harness-kit-checks/src/lane_harness.rs` already validates
  `lane_harness.v1`, materializes projected roots, and defines
  `projected`, `failed`, and `not_requested` statuses.
- `crates/harness-kit-checks/src/check_agent_roster.rs` validates the example
  lane-harness fixture, but does not require projection for actual dispatch
  receipts.
- `.harness-kit/examples/lane-harness.yaml` proves the schema is concrete
  enough to dogfood.
- `skills/harness-engineering/references/model-provider-harness-index.md`
  documents projection receipts but should be checked for stale command flags.

## Recommended Shape
Build policy-enforced projection at the dispatch boundary, backed by receipt
audit. Keep the primitive thin:

1. The lead composes a lane card as today.
2. Substantive lanes default to `harness_policy: required`.
3. The lead supplies a `lane_harness.v1` manifest, or records
   `harness_policy: waived` with a reason.
4. `dispatch-agent` validates the policy before launching the provider.
5. Receipts persist the policy, projection fields, waiver fields, and failure
   kind.
6. `summarize-delegations` and the closeout report display the isolation state
   for every provider attempt.
7. A check command fails prompt-only substantive lanes without waiver.

This keeps provider CLIs thin: launch one lane, apply an environment overlay,
record one receipt. The lead still decides team composition and replacement
providers.

## Alternatives Considered
| Alternative | Upside | Downside | Verdict |
|---|---|---|---|
| Docs-only reminders | Lowest implementation cost | We just proved reminders are easy to miss | Reject |
| Always project every lane | Simple invariant | Adds ceremony to mechanical commands and broad exploration | Reject |
| Dispatch-time required/waived policy | Fails early and makes evidence honest | Needs CLI and receipt schema work | Accept |
| Receipt-audit only | Backward compatible | Detects the problem after wasted provider runs | Partial |
| LaneSpec object replacing lane cards | Could unify prompt and manifest | Too large; drifts toward workflow engine | Defer |
| Global skill filtering | Strong ambient effect | Mutates installed harness state and risks pollution | Reject |
| Current optional projection plus better reporting | Small | Still leaves prompt-only lanes as the default path | Reject |

## Technical Design
- Extend dispatch options with a policy enum and optional waiver fields.
- Add CLI flags such as:
  - `--lane-harness-policy required|optional|waived`
  - `--lane-harness-waiver "<reason>"`
- Default policy should be conservative in code and explicit in skills:
  mechanical/direct dispatch may be `optional`; substantive `/dispatch`,
  `/shape`, `/code-review`, `/qa`, `/diagnose`, `/research`, and harness work
  should use `required` unless waived.
- Add receipt fields:
  - `lane_harness_policy`
  - `lane_harness_waiver`
  - `lane_harness_ref`
  - `lane_harness_sha256`
  - `projection_status`
- Update `summarize-delegations` so `not_requested` is visible and cannot be
  mistaken for projected.
- Add a receipt-audit command or check mode that takes a backlog/run ref and
  requires every non-manual substantive lane to be projected or waived.
- Update dispatch skill prose and templates so lane cards have a first-class
  harness-policy line, not an optional afterthought.
- Keep existing receipts readable; older receipts without policy are treated as
  legacy/unknown, not retroactively invalid.

## Acceptance Oracle
- `dispatch-agent --lane-harness-policy required` without `--lane-harness`
  exits nonzero before launching the provider and records no successful lane.
- `dispatch-agent --lane-harness-policy required --lane-harness <fixture>`
  produces a receipt with `projection_status: projected`,
  `lane_harness_ref`, and `lane_harness_sha256`.
- `dispatch-agent --lane-harness-policy waived --lane-harness-waiver <reason>`
  records the waiver and is counted separately from projected evidence.
- `summarize-delegations --backlog-ref <ref> --format text` shows
  projected/waived/not-requested counts and provider failures separately.
- A new or extended check fails a backlog/run ref containing prompt-only
  substantive provider receipts without waiver.
- Updated `/dispatch` references and templates no longer normalize
  `Lane harness: none` for substantive work.
- `cargo test --workspace --locked lane_harness agent_roster summarize_delegations`
  passes.
- `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`
  passes.
- `dagger call check --source=.` passes before shipping.

## Rollout Sequence
1. Add the policy and receipt fields behind backward-compatible parsing.
2. Add tests for required, optional, waived, projected, and legacy receipts.
3. Update `summarize-delegations` and add a focused receipt-audit check.
4. Update `/dispatch` skill docs, references, templates, and harness reference
   docs.
5. Dogfood the new policy on a shape or code-review run before making the
   doctrine stronger.

## Alignment Questions
- Should policy default to `required` in the CLI? Recommended answer: no.
  Keep the CLI conservative; make skills select `required` for substantive
  work so mechanical calls stay low-ceremony.
- Should waivers be free text? Recommended answer: short free text plus scope
  is enough for the first slice.
- Should failed provider attempts with projected roots count toward the roster
  floor? Recommended answer: they count as attempts and failure evidence, not
  successful lane evidence.
- Should `not_requested` remain valid? Recommended answer: yes for legacy and
  mechanical paths, but summaries must make it visible.
- Is this an ADR? Recommended answer: no for this scoped primitive; require an
  ADR only if replacing lane cards or changing global harness installation.

## Delegation Evidence
- Roster probe:
  `cargo run --locked -p harness-kit-checks -- probe-agent-roster --validate-only`
  reported the repo roster valid.
- Grok repo-investigator lane succeeded with a projected harness; receipt
  `34cb1bcc-c529-4762-89ea-131ce8df401b`; accepted for concrete repo gaps and
  acceptance checks.
- Cursor policy-critic lane projected successfully but failed at provider auth;
  receipt `840a6ed0-0361-44a6-be15-5bf6416fbfd7`; rejected as substantive
  critic output, accepted as proof projection receipts are independent from
  provider success.
- Antigravity replacement critic projected successfully but failed auth/sentinel
  evidence; receipt `b6eb7925-c64f-4e04-8869-381c77dde1c9`; rejected as
  substantive critic output, accepted as failure-path evidence.

## Premise Source
Operator prompt in the live Codex thread requested a `/shape` run for robust,
well-designed bespoke lane harness dispatch and asked for the result to become
the highest-priority backlog issue. No durable external work-source artifact
was provided; this packet is the durable repo-local source for the issue.

## Residual Risk
- Provider auth drift can still block critic lanes; the policy must classify
  those as provider failures, not projection failures.
- If waivers are too easy, they will become the default. The first receipt
  audit should make waived lanes visible in summaries.
- Projection remains context hygiene, not security isolation. Do not make
  permission guarantees from projected skill roots alone.
