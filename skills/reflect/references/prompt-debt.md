# Prompt Debt

Turn repeated operator corrections into one codification proposal.

## Sources

Use local, already-available surfaces:

- repo-local reflect notes, review scores, delegation receipts, traces, and
  work-ledger artifacts;
- harness session summaries or indexes;
- durable memory notes;
- Chronicle summaries only as private workflow context, never as quoted
  personal detail.

If a surface is missing, say so and continue.

## Promotion Threshold

Promote a pattern when either condition holds:

- repeated at least twice across sessions or workflow runs;
- repeated once with high severity: it prevented a shipped regression, runaway
  spend, data loss, or client-facing artifact error.

Do not promote one-off preferences, ambiguous vibes, or corrections that are
better handled by asking a normal clarifying question.

## Codification Target

Choose the highest enforceable target:

```text
Type system > Lint rule > Hook > Test > CI > Skill/reference > AGENTS.md > Memory
```

Use memory only for preference-level defaults that cannot be encoded in a gate
or workflow artifact. Use backlog when the fix needs shaping before mutation.

## Output

Emit one highest-leverage proposal by default.

```markdown
## Prompt Debt

- Pattern:
- Evidence count:
- Safe evidence snippets:
- Recommended target:
- Acceptance criteria:
- Residual risk:
```

Safe evidence snippets are counts, command names, file paths, or short
redacted examples. Do not include raw Chronicle text, private messages,
credentials, secrets, or sensitive personal context.

## Sample Brief

```markdown
## Prompt Debt

- Pattern: Done claims without live repo evidence.
- Evidence count: 3 corrections across 2 delivery sessions.
- Safe evidence snippets: "gate passed but route not exercised" (redacted);
  `dagger call check --source=.` was cited without a matching command/path
  smoke.
- Recommended target: skill/reference update for `/deliver` completion gate.
- Acceptance criteria: future merge-ready briefs include live repo evidence,
  exact command/path exercised, repo-fit check, and residual unverified paths.
- Residual risk: Some pure prose changes may only have docs/check evidence;
  the skill should allow an explicit repo-fit waiver.
```
