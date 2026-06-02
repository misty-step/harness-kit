# Prompt-debt reflection loop

Priority: P2
Status: merge-ready
Estimate: S

## Goal

Teach `/reflect` and `/monitor` to turn repeated user corrections and repeated
prompt patterns into shaped harness work instead of leaving them as chat-only
advice.

The first-class output is a small prompt-debt brief plus one concrete
codification proposal: update a skill, AGENTS.md, hook, eval, or backlog item.
The operator should not have to keep restating the same default once the system
has seen it recur.

## Non-Goals

- Do not store raw Chronicle details, private messages, credentials, or
  sensitive personal context.
- Do not build a new transcript database. Use existing local memory,
  session-index, trace, or reflect artifacts when available.
- Do not auto-edit skills from an automation run. Emit a shaped proposal or
  backlog item unless the user explicitly asks to apply the change.
- Do not turn every one-off complaint into doctrine. The loop only promotes
  repeated or high-severity corrections.

## Oracle

- [x] `skills/reflect/SKILL.md` gains a prompt-debt mode or subsection that
      detects repeated corrections, repeated requests, and repeated decision
      patterns from available local surfaces.
- [x] `skills/reflect/references/` gains a short reference defining promotion
      thresholds:
      repeated at least twice across sessions, or once if it prevented a
      shipped regression, runaway spend, data loss, or client-facing artifact
      error.
- [x] `/reflect cycle` output can include a `prompt_debt` proposal with:
      pattern name, safe evidence snippets or counts, recommended codification
      target, acceptance criteria, and residual risk.
- [x] `skills/monitor/SKILL.md` names prompt-debt checks as a valid local
      monitoring path for repeated workflows: when the same workflow requires
      repeated human correction, escalate to `/reflect` rather than continuing
      to watch passively.
- [x] The reference explicitly applies the codification hierarchy:
      type/lint/hook/test/CI before skill/AGENTS/memory, with memory as the
      fallback for preference-level defaults.
- [x] A sample brief is added under a reference or eval fixture using sanitized
      counts only, not sensitive personal Chronicle details.
- [x] `dagger call check --source=.` passes.

## Notes

### Why this belongs in Harness Kit

This is the learning loop for the harness itself. The repeated-prompt reducer
automation found useful defaults, but the durable fix should live where
workflow corrections can become skills, AGENTS rules, evals, or backlog items.

Harness Kit already has `/reflect`, `/monitor`, `/groom`, and `backlog.d/`.
Adding a tiny prompt-debt contract to those primitives is cheaper and more
portable than creating a new tool.

### Input surfaces

Use whatever exists locally, in this order:

1. repo-local reflect, review scores, delegation receipts, trace, and
   work-ledger artifacts;
2. harness session index or history summaries;
3. durable memory notes;
4. Chronicle summaries only as workflow context, never as exposed personal
   detail.

If a surface is missing, say so and continue with the available evidence.

### Output shape

The brief should stay decision-ready:

```markdown
## Prompt Debt

- Pattern:
- Evidence count:
- Recommended target:
- Acceptance criteria:
- Risk if ignored:
```

One run should nominate one highest-leverage codification target by default.
If there are multiple unrelated patterns, emit backlog proposals rather than a
long advisory memo.

### Relationship to other tickets

- Pairs with `056-agent-session-trace-lifecycle.md` and
  `058-work-ledger-mission-control.md`; those provide better raw material, but
  this ticket should work with today's session-index and memory surfaces.
- Feeds `053-skill-quality-audit-mode.md` by identifying skills that repeatedly
  fail in practice despite satisfying static quality checks.
- Feeds `065-repo-grounded-acceptance-contract.md` when the recurring prompt
  debt is "you claimed done without live evidence."

## Progress

- Added `/reflect prompt-debt` routing and a prompt-debt subsection to
  `skills/reflect/SKILL.md`.
- Added optional `prompt_debt` output to `/reflect cycle` with pattern,
  sanitized evidence count/snippets, recommended target, acceptance criteria,
  and residual risk.
- Added `skills/reflect/references/prompt-debt.md` with sources, thresholds,
  codification hierarchy, redaction rules, and a sanitized sample brief.
- Taught `/monitor` to treat repeated human correction loops as local monitor
  findings that escalate to `/reflect prompt-debt`.

## Delegation Evidence

- `grok-build` and `claude` planning lanes both identified the same minimal
  additive implementation shape. Accepted.
- Final `claude` critic reported `BLOCKING: no`; accepted.
- Final `grok-build` critic reported blockers because the diff packet omitted
  the new untracked reference file. Rejected as packet error, not a code gap.

## Verification

- `python3 scripts/check-frontmatter.py`
- `git diff --check`
- `rg -n 'Cycle summary|prompt-debt|prompt_debt|category 3|Output Contract|argument-hint' skills scripts ci harnesses docs README.md AGENTS.md`
- `bash scripts/check-docs-site.sh`
- `dagger call check --source=.`
