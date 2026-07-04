---
name: code-review
description: |
  Dispatch-shaped code review: fan the diff out to fresh-context reviewers
  across diverse providers and model families, synthesize, fix blockers,
  re-review until clean. Use when: "review this", "code review", "is this
  ready to ship", "second-model review". Trigger: /code-review, /review.
argument-hint: "[branch|diff|files]"
---

# /code-review

You are the marshal. Read the diff, dispatch diverse fresh-context
reviewers, synthesize, fix blockers, loop until clean. The authoring agent
never ships on its own review — that is the one hard rule.

## Dispatch

1. **Scope = the diff.** `git diff <base>...HEAD` (default base: the repo's
   default branch). Classify what changed: API, UI, security surface, data
   model, infra, docs.
2. **Learnings audit.** Grep matched learnings before dispatch:
   `rg -n --glob '*.md' '^(title|tags|applies_when):|<module>|<failure-mode>' docs/solutions`.
   For each applicable match, the synthesis includes
   `followed|violated <learning title> <file:line> <why>`; no file:line means
   the verdict is not anchored.
3. **Choose the risk tier.** Use
   `harnesses/shared/references/quality-system.md` to decide whether the diff
   needs a tiny, substantive, high-stakes, or Mode B review topology. Scale to
   the failure cost, not to habit.
4. **Fan out in parallel, decorrelated.** Native subagents for focused
   lenses (pick 2–4 that fit the diff: correctness, security, simplicity,
   tests); peer harness CLIs (`/roster`) for cross-model judgment — a
   different model family has decorrelated failure modes. If the harness can run a
   large-scale background orchestration where reviewers adversarially
   cross-check each other's findings before reporting, a substantive diff
   is a natural fit — that scale costs tokens, so routine diffs don't get
   it. Reviewers get the diff, the acceptance oracle, and a risk lens —
   **never the author's reasoning trail.** When the delivery logged
   deviations, hand reviewers the deviation *sites* — where the plan bent is
   where plausible-but-wrong concentrates — but never the author's
   justifications for them.
   Add `harnesses/shared/references/works-critique.md` when the diff touches
   public API, CLI, UI, performance, compatibility, migration, or operator
   workflow. Add `harnesses/shared/references/delete-first.md` when the diff
   adds abstraction, automation, dependencies, modes, or optimization; pair it
   with the synced Ponytail skill
   (`skills/.external/dietrich-ponytail/SKILL.md`) when the main risk is bloat,
   boilerplate, or speculative engineering.
   Add the synced Thermo-Nuclear skill
   (`skills/.external/cursor-thermo-nuclear-code-quality-review/SKILL.md`)
   whenever the diff changes meaningful implementation structure, grows large
   files, adds wrappers, or risks spaghetti branching; this is the default
   harsh maintainability lens, not a last resort.
   Add `harnesses/shared/references/verification-system-first.md` when the
   diff's proof story is missing, weak, eval/benchmark-shaped, or depends on
   QA/manual judgment.
5. **Aim reviewers at production embarrassment, not nitpicks.** Tell each
   one what to ignore (style, naming, speculative "consider…") as
   explicitly as what to find.

## What reviewers hunt

Plausible-but-wrong is the failure mode of model-written code:

- Stub or specification-shaped implementations that pass tests but don't work
- Wrong complexity (O(n²) hiding behind a clean interface)
- Tests that never invoke the changed entrypoint (adjacent green lanes)
- Missing verification system: no claim, falsifier, driver, grader, evidence
  packet, or cadence for a substantive change
- Missing invariant checks that only matter at scale or under concurrency
- Unnecessary abstraction — wrappers, modes, layers that don't earn their keep
- Swallowed errors, magic fallbacks, internal mocks

If the diff adds or changes an executable path (CLI, script, migration, job),
someone must run it once or cite the gate that does — otherwise it's an
**unverified runtime path** and blocks Ship. If the diff touches a visual or
user-facing surface, at least one reviewer exercises it live.
If the diff claims eval, benchmark, QA, or agent-behavior improvement, reviewers
must inspect the driver and grader, not just the report prose.

## Synthesize and verdict

Dedupe across reviewers; rank **blocking** (correctness, security, unverified
runtime path) > **important** (architecture, test strength) > **advisory**
(everything else). Blocking findings get fixed and the fix re-reviewed —
full pass, not a spot-check. Max 3 fix-review iterations, then escalate to
the operator with the open findings. Ship / Don't-ship is the lead's call on
the reviewers' evidence; advisory findings never block.
When the receipt pile is large, use `julius-caveman` compression for the
synthesis only. Findings must stay precise; PR comments and code suggestions
stay normal English unless the operator explicitly asks otherwise.

## Gotchas

- **Monoculture.** Same-model subagents alone are groupthink with extra
  steps. Substantive diffs get at least one other model family.
- **Reviewing the repo instead of the diff.** Scope discipline keeps
  findings actionable.
- **Treating all findings equally.** Severity ranking is the marshal's job;
  a wall of undifferentiated comments is review theater.
- **Skipping re-review after fixes.** A fix can introduce the next bug;
  blockers get a fresh pass.
