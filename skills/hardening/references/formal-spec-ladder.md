# Formal-Spec Ladder

Use this ladder only when ordinary TDD is not enough evidence for the blast
radius. It composes existing `/shape`, `/implement`, `/hardening`, and
`/deliver` contracts; it is not a new hardening mode.

## Trigger

Require `Formal Spec Required: yes` when two or more are true:

- The change rewrites core business rules, money/security/auth behavior, data
  migrations, permissions, or cross-service contracts.
- User-facing behavior is best expressed as examples, scenarios, CLI
  transcripts, API fixtures, or golden files.
- A regression would be expensive to detect manually after merge.
- The changed code has high complexity, low coverage, or a known weak oracle.
- The implementation needs multiple agents or long-running milestones where
  context drift is likely.

Low-risk work stays on the normal packet and TDD path.

## Packet Fields

When triggered, `/shape` emits:

```markdown
## Formal Spec
- Formal Spec Required: yes
- Informal spec: ...
- Formal examples: ...
- Acceptance oracle: ...
- Hardening budget: ...
- Waiver path: ...
```

`Formal examples` may be Gherkin, fixtures, transcripts, API examples, golden
files, or another concrete acceptance artifact. The `Acceptance oracle` should
be executable and must name the command, route, or tool call that proves the
examples are connected.

## Ladder

1. `/shape` records the informal spec, formal examples, acceptance oracle,
   hardening budget, and waiver path.
2. `/implement` writes a failing acceptance test from the formal examples
   before unit tests or production code.
3. `/implement` writes unit tests, production code, and local refactors after
   the acceptance intent is executable.
4. `/hardening risk` chooses changed surfaces that justify deeper evidence.
5. `/hardening property` adds invariants when the domain supports them.
6. `/hardening mutation` runs bounded mutants and dispositions survivors.
7. `/hardening acceptance` mutates examples, fixtures, contracts, or golden
   values to prove the user-facing oracle notices meaningful change.
8. `/deliver` records commands run, survivor disposition, critic/verifier
   result, and any waiver in the receipt.

## Waiver Policy

Waivers are allowed only when the packet names the waiver path and the delivery
receipt records:

- skipped ladder step;
- reason the step is not useful or not currently affordable;
- replacement evidence, if any;
- residual risk;
- fresh critic or verifier verdict.

Surviving mutants or acceptance mutations are not silently ignored. Kill the
survivor, document it as equivalent, or record it as residual risk with the
waiver.
