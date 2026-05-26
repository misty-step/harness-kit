---
name: reflect
description: |
  Session retrospective, operator coaching, harness postmortem, codification,
  and outer-loop cycle critique. Distills learnings into hooks/rules/skills,
  mutates the backlog from evidence, and emits harness-tuning suggestions to
  a branch. Learning engine of the outer loop.
  Use when: "done", "wrap up", "what did we learn", "retro", "reflect",
  "calibrate", "how could I have asked better", "prompt better",
  "teach me from this session", "what should I learn from this",
  "reflect on cycle", "cycle postmortem", post-/flywheel critique.
  Trigger: /reflect, /retro, /calibrate, /reflect cycle <cycle-ulid>.
argument-hint: "[distill|calibrate|coach|tune-repo|append|cycle] [context]"
---

# /reflect

Structured reflection that improves both the harness and the operator.

If `.spellbook/agents.yaml` exists, include `.spellbook/traces/` delegation
receipts in the evidence set when they are relevant. Reflection should convert
provider-lane results and failure modes into backlog, harness, or coaching
outputs without inventing hidden rankings.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/reflect` uses two or more roster
members for substantive postmortem, coaching, or harness-tuning analysis.
Use lanes to surface independent failure interpretations and improvement
proposals; the lead owns synthesis, codification choices, and receipts.
Direct lead-only reflection is limited to mechanical summary formatting,
emergency unblocks, explicit user-forbidden delegation, or an explicit waiver
when fewer than two roster members are available.

Every finding becomes one of three things:
- a codified artifact
- a concrete coaching note
- an explicit justification for not codifying

## Routing

| Mode | Intent | Reference |
|------|--------|-----------|
| **distill** (default) | End-of-session retrospective -> codified artifacts + operator coaching | `references/distill.md` |
| **calibrate** | Mid-session harness postmortem — fix the harness before the code | `references/calibrate.md` |
| **coach** | Deep dive on prompt quality, technical specificity, and concept building | `references/coach.md` |
| **tune-repo** | Refresh context artifacts, detect drift, update repo guidance | `references/tune-repo.md` |
| **append** | Append issue-scoped retro notes for `/groom` to consume later | `references/retro-format.md` |
| **cycle** | Outer-loop cycle critique — read `backlog.d/_cycles/<ulid>/`, emit `reflect.signals.json`, mutate `backlog.d/`, push harness suggestions to `reflect/<cycle-id>` branch | `references/cycle.md` |

If the first argument matches a mode name, route to that reference.
If no mode is provided, run `distill`.

Interpret natural-language requests as:
- "how could I have asked better", "teach me from this", "help me prompt better"
  -> `coach`
- "why did you do that", "you made the wrong call", "fix your instructions"
  -> `calibrate`
- "tune this repo", "refresh AGENTS", "context drift"
  -> `tune-repo`
- "reflect on cycle <ulid>", "postmortem this cycle", invocation from `/flywheel`
  -> `cycle`

## Responsibility Split

Reflection must separate three classes of failure:

1. **Harness failure** — the instructions, skills, tools, or codebase should
   have prevented the problem
2. **Shared ambiguity** — both sides left important constraints implicit
3. **Operator-spec gap** — the decisive information lived only in the user's
   head, so a tighter prompt would have reduced search space

Do not dump harness failures onto the user. If the repo, docs, or available
context already contained the answer, that is not a prompt-quality critique.

## Default Deliverables

Even in `distill`, inspect both lanes:
- **System lane** — instructions, skills, hooks, tests, CI, AGENTS.md, docs
- **Operator lane** — prompt rewrites, vocabulary, stack concepts, next-session moves

System codification is mandatory.
Operator coaching is mandatory to assess, but only mandatory to emit when there
is concrete, high-leverage feedback. Otherwise say so explicitly instead of
manufacturing generic advice.

Use `coach` when the user wants the operator lane expanded into a deeper lesson.

## Codification Hierarchy

When encoding knowledge, always target the highest-leverage mechanism:

```
Type system > Lint rule > Hook > Test > CI > Skill/reference > AGENTS.md > Memory
```

## Cycle Mode Authority (outer-loop only)

When invoked as `cycle`, reflect gains two privileges the other modes lack:

1. **Backlog mutation** — may create, edit, consolidate, or delete items in
   `backlog.d/` (never `backlog.d/_done/`). Every mutation must cite an
   evidence ref from the cycle (event line, artifact path, diff hunk).
2. **Harness suggestion branch** — may push skill/agent/hook/AGENTS.md edits
   to a branch named `reflect/<cycle-id>`, never to the current feature
   branch and never to main/master. Humans review and merge.

All other modes are read-only against `backlog.d/` and the harness. If
`cycle` cannot cite evidence for a mutation, downgrade it to a finding and
let a human decide.

See `references/cycle.md` for the full contract, `reflect.signals.json`
schema, and judgment rules (consolidate vs split, branch vs memory note).

## Gotchas

- **Blaming the user for missing repo context**: If the agent could have found
  it, it is a harness or retrieval failure.
- **Giving generic prompt advice**: "Be more specific" is not feedback. Name
  the missing constraint, example, acceptance test, or boundary.
- **Skipping the operator lane**: A retro that only mutates harness artifacts
  leaves user leverage on the table.
- **Turning coaching into scolding**: Start with what the prompt achieved, then
  show the stronger version and why it improves the search space.
- **Teaching concepts without anchoring them to the session**: Vocabulary only
  sticks when tied to a concrete decision, bug, or design tradeoff.
