# Works Critique

Use when a change claims "works" and touches public API, CLI, UI, performance,
compatibility, migration, operations, or user workflow. Do not use for tiny
internal-only edits.

## Review Card

1. **Public surface:** Does the API, CLI, UI, or config fit nearby conventions
   and leave a coherent path for likely next use?
2. **Human workflow:** Can the intended user complete the job without hidden
   ordering, chat-only instructions, brittle setup, or surprise state?
3. **Performance:** Which resource matters here: latency, CPU, memory, IO,
   tokens, dollars, or none? Name what is intentionally not optimized.
4. **Compatibility:** What must remain compatible, what may break, and why is
   that boundary acceptable?
5. **Operations:** How would production or a future operator know this working
   behavior degraded?

## Critic Prompt

Give the critic only the diff, acceptance oracle, and this card. Ask for:

- `BLOCKING:` yes/no.
- Findings in public surface, human workflow, performance, compatibility, or
  operations.
- Ignore style nits unless they hide one of those failures.

## Failure Mode

A response that only says "run tests" did not apply the lens. Tests can pass
while the public surface, workflow, or operational signal is wrong.
