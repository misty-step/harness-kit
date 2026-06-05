# Reflect teach-back checkpoints

Priority: P2
Status: shaped
Estimate: M

## Goal

Add an opt-in `/reflect` checkpoint that proves the human operator understands
the load-bearing decision, failure mode, and next action before a complex
session advances or closes.

## Source Evidence

- Thariq/Suzanne gist and attached screenshot:
  `sha256:c52adc36e5ab7b5941addb62051b5b5cd3633f75bcafcd1c0d5ef819ca046bd5`
  `/Users/phaedrus/Desktop/HJwSCWMa4AAWI5Q.jpg`.
- Gist source: https://gist.github.com/ThariqS/1389dcdff9eba4789887a2211370f06b.
- The prompt asks the agent to maintain a checklist of what the human should
  understand, have them restate understanding, fill gaps, quiz them, and end
  only after demonstrated understanding.

## Non-Goals

- Do not make every Harness Kit run quiz the operator.
- Do not block mechanical work, small fixes, or emergency state preservation.
- Do not create a general teaching app, spaced-repetition system, or hosted
  session UI.
- Do not store personal/private learning transcripts in repo by default.
- Do not let the agent invent understanding claims without a recorded
  restatement and verdict.

## Constraints / Invariants

- Checkpoints are opt-in from a packet, skill, or explicit operator request.
- The artifact records evidence refs and a short restatement, not raw session
  transcripts.
- The checkpoint must classify `pass`, `partial`, or `fail`; no vague
  "seems understood" status.
- The gate must fail closed only when a packet explicitly requires it.
- `/reflect` remains retrospective/codification owner; `/deliver` and
  `/shape` may reference checkpoints but do not own the teaching loop.

## Authority Order

checkpoint artifact > evidence refs > operator restatement > agent summary > lore

## Repo Anchors

- `skills/reflect/SKILL.md` - reflect modes, coaching, prompt debt, and cycle
  critique.
- `skills/reflect/references/coach.md` - existing operator coaching surface.
- `skills/reflect/references/prompt-debt.md` - codification from repeated
  corrections.
- `skills/deliver/references/clean-loop.md` - milestone and closeout behavior.
- `skills/trace/SKILL.md` - refs rather than raw transcripts.
- `harnesses/shared/AGENTS.md` - completion evidence and no validated claims
  without exact evidence.

## Prior Art

- Suzanne/Thariq prompt: incremental teaching, checklist, restatement, quiz,
  debugger/code review when needed, and no done claim until understanding is
  demonstrated.
- `/reflect coach`: already supports operator coaching, but not an opt-in
  pass/fail checkpoint artifact.
- `/shape` and `/deliver`: already have milestone concepts where a
  comprehension checkpoint could be useful for high-risk decisions.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Do nothing | Keep explanations ad hoc | No new process | Operator loses thread during long runs | Reject |
| Global teaching mode | Every session teaches and quizzes | Strong learning loop | Obnoxious, slow, and likely ignored | Reject |
| `/reflect checkpoint` | Opt-in artifact with restatement, verdict, gaps | Fits existing reflection owner | Needs strict scoping to avoid ceremony | Choose |
| `/deliver` milestone quiz | Composer blocks until operator passes | Close to execution | Puts teaching logic in wrong skill | Reject |
| External note app | Store learning checklist outside repo | Personal workflow fit | Not cross-harness or auditable | Reject |
| Raw transcript review | Agent summarizes entire session to teach | Complete context | Privacy and context bloat | Reject |
| Test-only explanation gate | Agent writes an explanation and self-grades | Easy automation | No evidence the human understood | Reject |

## Tradeoff Matrix

| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| Do nothing | 2 | 5 | 5 | 2 | 5 | 1 | 5 |
| Global teaching mode | 2 | 2 | 3 | 3 | 3 | 3 | 1 |
| `/reflect checkpoint` | 5 | 3 | 4 | 5 | 5 | 5 | 4 |
| `/deliver` milestone quiz | 3 | 3 | 4 | 4 | 4 | 4 | 3 |
| External note app | 2 | 3 | 3 | 2 | 3 | 2 | 3 |
| Raw transcript review | 3 | 2 | 1 | 3 | 2 | 3 | 1 |
| Self-graded explanation | 2 | 4 | 5 | 4 | 5 | 2 | 4 |

The opt-in `/reflect checkpoint` shape keeps the learning loop available where
it matters without turning every run into pedagogy.

## Agent Readiness

- Profile source: not applicable.
- Stack feedback strength: medium; this is mostly a local artifact/schema and
  checker.
- ADR decision: not required.
- Infrastructure path: local JSON/Markdown artifact and checker command.
- Gate: checkpoint self-test, `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`, then
  `dagger call check --source=.`
- Evidence storage: `.harness-kit/reflect/checkpoints/` or fixture examples
  under `skills/reflect/evals/`.
- Mock policy impact: preserved; tests use fixture checkpoint artifacts.

## Delegation Evidence

- Roster providers used:
  - `claude` repo investigator, receipt
    `c5a1708e-e046-4590-8141-1d08412317a5`.
  - `pi` premise critic, receipt `fe9a2a9a-c48e-41ce-a4a2-9feea8338884`.
  - `codex` oracle critic, receipt `2920ae5b-d21c-46a6-9202-0861490134fa`.
- Accepted evidence: Claude identified `/reflect checkpoint` as the narrowest
  owner; Pi warned against a generic comprehension workflow engine; the ticket
  is opt-in and artifact-backed.
- Rejected evidence: global teaching mode and raw transcript storage.
- Waivers: no live interaction with the gist author or source post; gist and
  screenshot were treated as inspiration, not acceptance oracle.

## Oracle

- [ ] `/reflect checkpoint <topic>` or equivalent reference contract defines a
      checkpoint artifact schema.
- [ ] A fixture checkpoint with no operator restatement fails validation.
- [ ] A fixture checkpoint with `lead_verdict: pass` and empty `gaps` passes.
- [ ] A fixture checkpoint with `lead_verdict: partial` or `fail` causes
      `--gate <topic>` to exit non-zero when that topic is required.
- [ ] A packet opt-in marker such as `Comprehension-required: <topic>` is
      documented; absent marker means no gate.
- [ ] The artifact records `topic`, `source_refs`, `question`, short
      `operator_restatement`, `lead_verdict`, `gaps`, `next_action`, and
      timestamp.
- [ ] `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .` passes.
- [ ] `dagger call check --source=.` passes.

## Acceptance Evidence

- Acceptance source: checkpoint artifact fixtures.
- Evidence that proves it: validator accepts a passing checkpoint and rejects
  missing/partial/failing checkpoint cases under gate mode.
- Exact command/path/route exercised: implementation should add a deterministic
  self-test command, such as `cargo run --quiet --locked -p harness-kit-checks -- reflect-checkpoint --self-test`.
- Oracle / acceptance artifact hash:
  `sha256:c52adc36e5ab7b5941addb62051b5b5cd3633f75bcafcd1c0d5ef819ca046bd5`
  `/Users/phaedrus/Desktop/HJwSCWMa4AAWI5Q.jpg`.
- Contract-change acknowledgment: this intentionally adds an opt-in
  comprehension artifact to `/reflect`.
- Residual risk: human restatement can be perfunctory; the checker can verify
  artifact structure, not actual cognition.

## Observability Plan

- Changed behavior to watch: complex sessions can record whether the operator
  understood the load-bearing decision before moving on.
- Named signal or evidence surface: checkpoint artifact and gate result.
- Instrumentation debt: no longitudinal comprehension analytics; avoid that
  until real checkpoint usage exists.

## Implementation Sequence

1. Add a `/reflect checkpoint` reference or mode contract with schema and
   opt-in rules.
2. Add a small validator/self-test with pass, missing-restatement, partial, and
   fail fixtures.
3. Teach relevant composer/shape prose only to reference the opt-in marker;
   keep checkpoint ownership in `/reflect`.
4. Run the checkpoint self-test, roster check, and Dagger gate.

## Risk + Rollout

- Risk: annoying operator friction. Mitigate by opt-in only and M+/high-risk
  guidance, not default gating.
- Risk: storing private learning context. Mitigate by storing refs and short
  restatement only.
- Rollback: remove checkpoint mode/reference and fixture validator; no existing
  delivery contract should depend on it by default.
