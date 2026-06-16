# Context Packet: Delete-first doctrine lens

Priority: P2
Status: ready
Estimate: S

## PRD Summary

- User: agents and operators shaping or refactoring changes.
- Problem: Harness Kit already says "delete before adding," but agents still
  tend to optimize, automate, or abstract work before questioning whether the
  requirement or process should exist.
- Goal: add a compact delete-first lens that makes the order explicit:
  question requirements, delete, simplify, speed up, automate.
- Why now: the hit list captured the common engineering trap: optimizing a
  thing that should not exist.
- UX enabled: shape/refactor packets can show deletion and simplification were
  considered before automation or optimization.
- Deliverable type: doctrine/reference update.
- Success signal: a sample shape packet rejects an unnecessary automation by
  naming the deletion/simplification path first.

## Product Requirements

- P0: Add this as a lens/reference, not a new standalone skill.
- P0: Keep always-loaded doctrine terse; detailed checklist belongs in a
  reference file.
- P0: Route the lens into `/shape` alternatives and `/refactor` stop conditions.
- P0: The lens must preserve explicit user requirements and safety/security
  basics; "delete" is not permission to ignore the task.
- Non-goals: no Elon quote dump, no personality framing, no structural linter
  that scans for the words "delete" or "automate".

## Repo Anchors

- `harnesses/shared/AGENTS.md` - Layer 1 already has "Delete before adding" and
  premise challenge.
- `skills/shape/SKILL.md` - alternatives must include a boring/manual path and
  inverted assumption.
- `skills/refactor/SKILL.md` - refactor stop rules and architecture-theater
  gotchas.
- `skills/harness-engineering/SKILL.md` - primitive test and "prefer deletion"
  contract.
- `skills/shape/references/critique-personas.md` - grug/carmack/jobs lenses
  already exist for simplicity/shippability.

## Alternatives

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Do nothing | Existing doctrine already says delete before adding. | It lacks the explicit order that prevents premature automation/optimization. | Reject. |
| Add standalone `delete-first` skill | Discoverable command. | This is a lens applied inside shape/refactor, not a workflow. | Reject. |
| Expand AGENTS with full algorithm | Always visible. | Adds every-session prose tax and quote bulk. | Reject. |
| Add terse doctrine line plus reference | Captures the ordering with low context tax and reusable detail. | Needs sample proof to avoid decorative prose. | Choose. |
| Add Rust linter | Enforceable. | Regex over judgment is brittle and likely theater. | Reject. |

## Design

Add `harnesses/shared/references/delete-first.md`:

1. Question the requirement: what user outcome would disappear if this were not
   built?
2. Delete: can the feature/process/file/dependency be removed?
3. Simplify: can stdlib/native/existing capability cover it?
4. Speed up: only after the thing has survived deletion/simplification.
5. Automate: only for repeated, verified, bounded work.

Then link it from:

- `/shape` gotchas or alternatives guidance.
- `/refactor` working loop or gotchas.
- `/harness-engineering` primitive test/contract.

Shared AGENTS gets at most one sentence if needed:
"Delete, simplify, speed up, automate - in that order."

## Oracle

- New reference exists and is linked from `/shape`, `/refactor`, and
  `/harness-engineering` or their relevant references.
- A sample shape fixture under `.evidence/delete-first/` considers a requested
  automation and chooses deletion/simplification first.
- `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`
  passes.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Premise Source

Premise Source: sha256:ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663 HIT-LIST.md

## HTML Plan

HTML Plan: `.evidence/shape-hit-list/hit-list-shape-index.html#delete-first`

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| Becomes generic advice | Keep detail in one reference and require sample use. |
| Used to dodge explicit work | State that explicit user requirements and safety/security basics survive. |
| Duplicates Ponytail | Route Ponytail as an opt-in coding skill; delete-first is a shaping/refactor lens. |
| Prose tax | Add minimal shared doctrine only if reference routing is insufficient. |

Rollback: remove reference and links; existing delete-before-add doctrine remains.
