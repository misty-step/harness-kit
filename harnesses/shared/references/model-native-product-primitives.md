# Model-Native Product Primitives

Use this when the product outcome depends on semantic judgment, agent work,
realtime voice, speech, vision, or other model-native capability.

## Premise Test

Before implementing, answer:

- What user outcome requires a model rather than deterministic code?
- Which current model/provider surface actually provides that capability?
- What deterministic boundary keeps the model from owning authority it should
  not own?
- What eval, fixture, live smoke, or QA path proves the model behavior is good
  enough and catches likely false positives / false negatives?

If those answers are missing, the first deliverable is research + shaping, not
a phrase list.

## Boundary Rule

Deterministic code may own:

- schema validation and typed contracts
- persistence, event logs, projections, and replay
- user approvals, policy gates, budgets, and sandboxing
- dedupe, rate limits, and deterministic fallbacks
- eval drivers, graders, fixtures, and evidence packets

Deterministic code must not silently replace the product's semantic brain with:

- keyword lists for open-ended intent detection
- static confidence scores for judgment claims
- prompt templates that masquerade as model output
- rules that only work for the operator's exact phrasing

Those shortcuts are acceptable only when the ticket explicitly scopes a
fallback, fixture, seed path, or safety guard.

## Realtime / Speech Bias

For realtime voice or meeting products, verify current primary docs before
choosing the boundary. The likely shape is a model-native agent/classifier path
plus deterministic approval and execution policy, not STT followed by brittle
string matching.

Record the chosen provider facts in the context packet or backlog ticket. If
the roster index is stale or lacks the relevant modality, update it before
claiming the design is grounded.

## Verification

Model-native behavior needs a model-behavior proof loop:

- held-out transcripts/audio/images/tasks
- expected accept/reject decisions or rubric
- grounding checks against cited evidence
- adversarial paraphrases and negations
- provider failure and fallback behavior
- evidence packet another agent can inspect

Unit tests over Rust/TypeScript prove the boundary; they do not prove semantic
quality without an eval or live artifact.
