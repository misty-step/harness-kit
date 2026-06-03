---
name: critique
description: |
  Run one targeted, read-only architecture or quality critique through a named
  lens from the shared rubric. Use when: "critique this module",
  "run an Ousterhout pass", "lens critique", "architecture critique".
  Trigger: /critique.
argument-hint: "[--lens <name> --target <path>] [--lenses]"
---

# /critique

Single-lens signal. Not a merge gate, not `/code-review`, not a parallel bench.

## Contract

- `--lenses` lists available lens names by reading
  `harnesses/shared/references/lenses.md`; do not hardcode the list here.
- `--lens <name>` resolves the matching `## <name>` block from that rubric.
  Unknown lens means stop and print the available names.
- `--target <path>` is required for a focused critique. Resolve it against the
  invoking repo and read only the files needed to answer.
- Never reference `agents/<lens>.md`. Lenses are rubrics, not static persona
  files.
- The critic is read-only. Do not edit, stage, commit, or emit a Ship verdict.

## Dispatch

Spawn one fresh read-only critic subagent with:

- role: critic
- objective: apply `<lens>` to `<target>` and find production-relevant design,
  correctness, test, or operability risks
- scope: target path, relevant nearby contracts, and the exact lens excerpt
- boundary: no edits, no broad review, no author reasoning trail
- output: structured findings only

If the harness cannot spawn a fresh subagent, the primary adopts the exact
lens text for one read-only pass and marks the header
`lens-adopted-by-primary`. This is a fallback, not fresh-context separation.

## Output

```markdown
## Critique
- Lens: <name>
- Target: <path>
- Evidence read: <files/commands inspected>
- Fresh-context mode: <subagent|lens-adopted-by-primary>
- Findings:
  - Finding: <concise claim>
    Evidence: <file:line>
    Impact: <high|medium|low>
    Suggested next step: <smallest useful action>
- Residual uncertainty: <what was not read or could change the verdict>
```

Findings require file:line evidence. If no finding survives, say so and cite
the files that were read. Do not return `Ship`, `Conditional`, or `Don't Ship`;
that verdict belongs to `/code-review`.

## Examples

- `/critique --lenses`
- `/critique --lens ousterhout --target src/auth`
- `/critique --lens cooper --target tests/auth.spec.ts`
