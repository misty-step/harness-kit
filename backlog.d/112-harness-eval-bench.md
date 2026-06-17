# Context Packet: Harness eval bench against raw runs and alternative primitives

Priority: P1
Status: ready
Estimate: L

## Goal

Create a repeatable eval protocol and first evidence run that tells whether a
Harness Kit primitive improves outcomes against raw agent runs and credible
alternative primitives.

## Premise Challenged

The current hit-list scout can identify promising external skills, but it does
not prove Harness Kit is better, worse, or even different in the moments that
matter. Installed skills and confident doctrine are not evidence. The eval must
compare outcomes under matched tasks.

The user outcome is not "add a benchmark tree." The outcome is: when Harness Kit
keeps, imports, rewrites, or deletes a primitive, the decision can cite task
evidence rather than taste or repo ownership.

## Non-Goals

- Do not build a large benchmark platform inside Harness Kit.
- Do not import top external skills automatically from star count.
- Do not run paid or high-volume model sweeps in CI.
- Do not store raw private transcripts, secrets, customer data, or unredacted
  provider logs.
- Do not let an eval worker self-merge source edits.
- Do not claim global superiority from one task family.

## Constraints

- Use the existing eval contract in
  `skills/harness-engineering/references/mode-eval.md`: task, transcript,
  outcome, graders.
- Compare at least three conditions:
  - raw run: same task without the relevant Harness Kit skill or primitive;
  - Harness Kit run: current first-party primitive;
  - alternative primitive: external skill, marketplace skill, or competing
    local reference.
- Grade blind where possible: graders see artifacts and oracles, not condition
  labels.
- Prefer objective graders first: files, commands, evidence paths, forbidden
  edits absent, gate output. Use rubric/model judges only for judgment-heavy
  dimensions.
- Keep serious repeated arena work eligible to graduate to Daedalus instead of
  expanding Harness Kit into a benchmark product.

## Repo Anchors

- `skills/harness-engineering/references/mode-eval.md` - current eval protocol.
- `.evidence/skill-scout/hit-list-scout.md` - external primitive candidates.
- `crates/harness-kit-checks/src/scout_skills.rs` - current candidate scanner,
  not an outcome eval.
- `skills/shape/SKILL.md` - candidate task family for shaping comparisons.
- `skills/refactor/SKILL.md` - candidate task family for architecture and
  simplicity comparisons.
- `skills/code-review/SKILL.md` - candidate task family for review comparisons.
- `harnesses/shared/references/quality-system.md` - quality-system review
  methodology.
- `.harness-kit/traces/delegations.jsonl` - sanitized delegation receipt shape.

## Alternatives

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Telemetry only | Cheap, real usage, no artificial tasks. | Correlational; cannot compare raw vs Harness Kit vs alternatives under matched conditions. | Use as context, not the eval. |
| One-off human taste review of top skills | Fast and high-signal for obvious imports. | Repeats the current problem: plausible judgment without outcome evidence. | Reject. |
| Full Daedalus benchmark arena first | Strongest repeated-eval substrate. | Too much surface before we know the harness-task families and grading rubrics. | Defer until repeated runs prove need. |
| Minimal Harness Kit eval bench with matched tasks and blinded grading | Creates direct evidence, stays close to current skills, and can graduate later. | Requires careful task design and sanitized receipts. | Choose. |

## Design

Deliver a minimal eval bench as evidence plus reusable protocol, not as a new
semantic workflow engine.

1. Add a short Harness Kit eval reference under
   `skills/harness-engineering/references/` or extend `mode-eval.md` if the
   addition stays compact.
2. Define three task families:
   - `shape`: raw idea to context packet and HTML plan;
   - `refactor`: simplify a contained subsystem without behavior change;
   - `review`: find blocking production risks in a diff.
3. For each family, define 2-3 fixtures with:
   - prompt/task;
   - allowed repo context;
   - forbidden edits;
   - objective oracle;
   - judgment rubric.
4. Run at least one pilot comparison across raw, Harness Kit, and one
   alternative primitive. Good first alternatives:
   - Ponytail for shape/refactor simplicity pressure;
   - `obra/superpowers`, `garrytan/gstack`, or `addyosmani/agent-skills` after
     scout/manual review confirms license and runnable shape;
   - an explicit "no skill, same model" raw condition.
5. Store only sanitized receipts and final artifacts under
   `.evidence/harness-evals/<date-or-run>/`.
6. Produce a decision report that can say for each primitive:
   `keep`, `adapt`, `import`, `delete`, `needs more tasks`, or `graduate to
   Daedalus`.

If a runner becomes necessary, implement the smallest Rust-owned helper in
`crates/harness-kit-checks`; otherwise keep the first delivery as protocol plus
evidence artifacts.

## Oracle

- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
- The delivery creates a harness eval protocol/reference that names:
  - raw, Harness Kit, and alternative primitive conditions;
  - task, transcript, outcome, and grader fields;
  - blind grading expectations;
  - transcript redaction rules;
  - decision labels.
- The delivery creates at least one pilot eval report under
  `.evidence/harness-evals/` comparing all three conditions on at least one task
  family.
- The report includes at least one objective grader result and one judgment
  rubric result, with the grader condition labels blinded or an explicit waiver.
- The report makes at least one concrete recommendation about a Harness Kit
  primitive or external skill candidate.
- A fresh critic reviews the eval design and pilot report from artifacts only
  and returns no blocking methodological flaw.

## Premise Source

Premise Source:
`sha256:c231607c89cd19c04660a8f2615b3ed7f58be52a1716f7bb5103ca615451bf00 .evidence/shape-112/premise.md`

## HTML Plan

HTML Plan: `.evidence/shape-112/harness-eval-bench.html`

Opened for rendered review before delivery. The hero states the chosen design:
matched outcome evals over raw, Harness Kit, and alternative primitive
conditions.

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| Benchmark theater | Require task/outcome/grader fields and at least one pilot run. |
| Harness self-bias | Use raw baseline, alternative primitive, and blinded grading. |
| Private transcript leakage | Store sanitized receipts and artifacts only. |
| Overbuilding an arena | Start with protocol plus evidence; add Rust helper only if repetition proves it. |
| False confidence from one run | Decision labels include `needs more tasks`; report scope limits. |

## Adversarial Review Focus

Ask the critic:

- Does the eval actually compare outcomes, or only prose quality?
- Are raw and alternative conditions fair enough to teach us something?
- Can the grader be fooled by knowing which condition is Harness Kit?
- Is any "keep/import/delete" recommendation stronger than the evidence?
- Did the design avoid turning Harness Kit into a benchmark platform?
