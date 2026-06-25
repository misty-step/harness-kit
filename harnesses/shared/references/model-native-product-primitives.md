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

## Structure Is a Cost — Especially on Model-Facing Interfaces

The bitter lesson cuts both ways. The Boundary Rule says deterministic code
*may* own typed contracts — but "may" is not "must," and reaching for rigid
structure by reflex is its own failure mode. When the consumer of a value is an
LLM agent, it reads fuzzy, unstructured, natural-language context directly. It
needs the *information*, not a schema. A required field, fixed taxonomy, or enum
engineered so "the next system can parse it" is cost without payoff when the next
system is a model: it constrains what can be expressed, rots when reality shifts,
and buys nothing the model needed.

Default to the simplest representation that carries the needed information.
Impose structure only where deterministic code must actually branch on the value:

- a value a gate, policy, budget, or release check compares against
- a persistence / idempotency / dedupe / replay key
- an eval grader's expected field
- a contract two deterministic components parse across a boundary
- a field a UI renders in a fixed slot

If the only downstream consumer is a model — a remediation agent, a summarizer,
a planner, a judge, the next step in an agent loop — prefer rich prose or
loosely-shaped data over a tight schema. Carry the facts; let the model do the
interpreting.

This is a recurring failure mode when building products that are AI-powered or
sit upstream of one: falling back on deterministic heuristics and
over-constrained structure where a model's general capability already handles
the fuzziness. Sometimes structure still earns its place (the bullets above);
often it is more trouble than it is worth. Ask which it is before adding it —
the default is less.

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
