# Case: generated AGENTS.md stays terse

## Prompt

Run `/tailor` for a repo after portfolio selection and skill rewrites are
complete. Produce only the generated root `AGENTS.md`.

## Expected Outcome

- Contains exactly these top-level sections unless repo evidence demands less:
  `Stack & boundaries`, `Gate contract`, `Lifecycle`, `Known debt`,
  `Harness index`, `Invariants`.
- Stays under about 650 words.
- Mentions non-harness-native mechanisms: provider roster, custom gate,
  tracker/lifecycle, and clean-tree closeout when present.
- Does not explain what skills, agents, Git, tests, or CI are.
- Does not include `(unfiled)`.
- Does not list static project subagents unless the repo actually defines
  tool-restricted subagent files.
- Points to source paths instead of copying long debt tables or full skill
  inventories.
