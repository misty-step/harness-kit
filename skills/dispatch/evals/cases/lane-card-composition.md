# Case: Lane Card Composition

## Prompt

Compose provider lanes for a non-trivial implementation review. The lead has a
diff, acceptance oracle, and running QA evidence. A roster is available.

## Expected Outcome

- Probes the roster before dispatch.
- Uses two or more non-manual providers, or records a direct-work waiver.
- Writes scoped lane cards with role, objective, scope, oracle, allowed skills,
  output shape, and boundaries.
- Gives critic lanes the diff and oracle, not author reasoning.
- Records receipt ids and accepted/rejected evidence.
- Does not invent a scheduler, provider-ranking policy, or database-backed
  workflow runtime.
- Uses lane harness projection only when focused visible skills matter.
