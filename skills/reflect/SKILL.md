---
name: reflect
description: |
  Session retrospective, operator coaching, harness postmortem, codification,
  and outer-loop cycle critique. Turns evidence into hooks, rules, skills,
  backlog mutations, or explicit non-actions. Use when: "done", "wrap up",
  "what did we learn", "retro", "calibrate", "prompt better",
  "teach me from this session", "reflect on cycle", post-/flywheel critique.
  Trigger: /reflect, /retro, /calibrate, /reflect checkpoint <topic>,
  /reflect cycle <cycle-ulid>.
argument-hint: "[distill|calibrate|coach|checkpoint|prompt-debt|tune-repo|append|cycle] [context]"
---

# /reflect

Structured reflection that improves both the harness and the operator.

When roster receipts exist, include `.harness-kit/traces/` delegation receipts
in the evidence set when they are relevant. Reflection should convert
provider-lane results and failure modes into backlog, harness, or coaching
outputs without inventing hidden rankings.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use lanes to surface independent failure interpretations and improvement proposals; the lead owns synthesis and codification choices.

Every finding becomes one of three things:
- a codified artifact
- a concrete coaching note
- an explicit justification for not codifying

## Work Ledger

When `.harness-kit/work/ledger.jsonl` is available, `/reflect` consumes the
latest events for the active backlog/branch plus trace refs and delegation
receipts. It calls `scripts/work-ledger.py append` with `phase_started` at
retro start, `next_action_changed` when follow-up backlog or harness proposals
are emitted, and `phase_completed` when the reflection packet is complete.
Follow-up proposals are evidence refs, not hidden chat-only state.

## Routing

| Mode | Intent | Reference |
|------|--------|-----------|
| **distill** (default) | End-of-session retrospective -> codified artifacts + operator coaching | `references/distill.md` |
| **calibrate** | Mid-session harness postmortem — fix the harness before the code | `references/calibrate.md` |
| **coach** | Deep dive on prompt quality, technical specificity, and concept building | `references/coach.md` |
| **checkpoint** | Opt-in teach-back checkpoint with restatement, verdict, gaps, and gate artifact | `references/checkpoint.md` |
| **prompt-debt** | Promote repeated corrections and repeated workflow patterns into one codification proposal | `references/prompt-debt.md` |
| **tune-repo** | Refresh context artifacts, detect drift, update repo guidance | `references/tune-repo.md` |
| **append** | Append issue-scoped retro notes for `/groom` to consume later | `references/retro-format.md` |
| **cycle** | Bounded end-of-ship retrospective invoked by `/ship` — emit backlog mutations, harness-tuning proposals, and a cycle summary for the caller to apply | `references/cycle.md` |

If the first argument matches a mode name, route to that reference.
If no mode is provided, run `distill`.

Interpret natural-language requests as:
- "checkpoint this", "teach-back", "make sure I understand", or packet marker
  `Comprehension-required: <topic>` -> `checkpoint`
- "how could I have asked better", "teach me from this", "help me prompt better"
  -> `coach`
- "same correction again", "why do I keep repeating this", "codify this
  pattern", "prompt debt" -> `prompt-debt`
- "why did you do that", "you made the wrong call", "fix your instructions"
  -> `calibrate`
- "tune this repo", "refresh AGENTS", "context drift"
  -> `tune-repo`
- "reflect on cycle <cycle-id>", "postmortem this cycle", invocation from
  `/ship` (or transitively from `/flywheel`, which composes `/ship`)
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

## Prompt Debt

Prompt debt is a repeated human correction, repeated request, or repeated
decision pattern that should become a durable harness artifact instead of
remaining chat-only advice. Use available local surfaces only: repo-local
reflect notes, review scores, delegation receipts, traces, session summaries,
and durable memory notes. Chronicle-derived context may inform the pattern, but
do not quote private personal detail.

When `.groom/review-scores.ndjson` exists and
`scripts/review-score-trends.py` is available, run the analyzer before proposing
skill changes. Treat 5+ score entries as enough for a trend; below that, report
the count and avoid a tuning claim. If the analyzer names a dimension regression
or high false-positive rate, propose a concrete skill/reference edit using the
codification hierarchy rather than a generic observation.

Promote a pattern when it appears at least twice across sessions, or once when
it prevented a shipped regression, runaway spend, data loss, or client-facing
artifact error. Emit one highest-leverage proposal by default:

```markdown
## Prompt Debt

- Pattern:
- Evidence count:
- Safe evidence snippets:
- Recommended target:
- Acceptance criteria:
- Residual risk:
```

Apply the codification hierarchy above. Prefer type, lint, hook, test, or CI
coverage before skill prose; use AGENTS.md for always-on routing; use memory
only for preference-level defaults that cannot be enforced.

## Cycle Mode Authority (outer-loop only)

`cycle` is a **bounded invocation**: `/ship` calls it at the end of the
final-mile pipeline to capture learnings from the just-shipped ticket.
`/flywheel` triggers it transitively by composing `/ship`. When invoked as
`cycle`, reflect gains two privileges the other modes lack:

1. **Backlog mutation proposals** — may propose create, edit, consolidate,
   reprioritize, or delete on items in `backlog.d/` (never
   `backlog.d/_done/`). Every proposal must cite an evidence ref from the
   cycle (commit, diff hunk, receipt path, log line).
2. **Harness-tuning proposals** — may propose skill/agent/hook/AGENTS.md/
   CLAUDE.md edits. Reflect **emits**; it does not apply. The caller
   routes these to a harness branch for human review.

All other modes are read-only against `backlog.d/` and the harness. If
`cycle` cannot cite evidence for a mutation, downgrade it to a finding and
let a human decide.

### Invocation Contract

Triggered as `/reflect cycle` (aliases: `/reflect --cycle <cycle-id>`).
The caller — normally `/ship` — passes this input packet:

- `branch`: name of the just-shipped feature branch (pre-merge).
- `merged_sha`: squash commit SHA now on master/main.
- `closed_backlog_ids`: list of IDs closed in this cycle (the closing set
  from `/ship`'s trailer scan).
- `referenced_backlog_ids` (optional): `Refs-backlog` IDs noted but not
  closed.

A `cycle-id` identifies the retro artifact; derive it from `merged_sha`
short form when the caller does not supply one.

### Output Contract

Three required categories plus one optional prompt-debt category. The
structured categories must be cleanly separable so the caller can apply them
under different policies.

1. **Backlog mutations** (structured, machine-consumable). For each:
   - action: `create` | `edit` | `reprioritize` | `delete`
   - path: concrete `backlog.d/<id>-*.md` target
   - body: full file content for `create`, unified diff for `edit`, new
     priority for `reprioritize`, justification for `delete`
   - evidence: cycle ref justifying the mutation
   These are **proposals**. The caller (`/ship`) applies them to master
   via a follow-up commit and owns commit hygiene. Reflect does not stage,
   commit, or push these files itself.

2. **Harness-tuning proposals** (structured, machine-consumable). For each:
   - path: concrete file under `skills/`, `agents/`, `harnesses/`,
     `AGENTS.md`, `CLAUDE.md`, or a hook script
   - body: unified diff or full new-file content
   - evidence: cycle ref and codification-hierarchy justification
   Reflect **must not** write these to master. The caller routes them to
   a harness branch (`/ship` uses `harness/reflect-outputs`). A `cycle`
   run that mutates harness files on the current branch is a bug.

3. **Prompt-debt proposal** (optional, structured). Include at most one by
   default, only when repeated corrections or high-severity prompt patterns
   are visible in cycle evidence:
   - pattern: short name
   - evidence_count: sanitized count, not raw transcript text
   - safe_evidence_snippets: redacted commands, file paths, or short examples
   - recommended_target: codification target from the hierarchy
   - acceptance_criteria: how the future run proves the debt is paid down
   - residual_risk: what remains ambiguous or intentionally manual

4. **Cycle summary** (human-readable narrative). What shipped, what was
   learned, what went well, what went poorly. Also written to the
   standard retro location (`.groom/retro/<primary-id>.md` or the
   `.harness-kit/reflect/<cycle-id>/` receipts dir, matching whatever
   convention the invoking repo already uses).

### Invariants

- **Harness mutations never land on master directly.** Reflect emits;
  the caller routes to a harness branch. This is a hard cross-skill
  invariant also asserted in `ship/SKILL.md`.
- **Backlog mutations are proposals, not auto-applied.** Reflect does
  not `git add` / `git commit` on behalf of the caller.
- **Session-retrospective mode still works standalone.** `distill`,
  `calibrate`, `coach`, `tune-repo`, and `append` remain usable without
  cycle context — `cycle` is additive, not a replacement.

See `references/cycle.md` for judgment rules (consolidate vs split,
when to escalate to a harness branch, evidence standards).

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

## Verification

Run `python3 skills/reflect/scripts/checkpoint.py --self-test` to prove the
checkpoint validator rejects missing restatements, invalid verdicts, and raw
private content.
