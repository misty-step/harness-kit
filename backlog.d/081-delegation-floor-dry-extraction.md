# Extract shared doctrine to a single source: kill the 19-skill copy-paste

Priority: P1
Status: ready
Estimate: M

## Goal

Remove the largest DRY violation in the harness. The "Delegation Floor"
paragraph is copy-pasted near-verbatim across **19 of 28 skills**; the
Completion-Gate block across 8; the clean-tree closeout across 5 files. State
each cross-cutting contract ONCE in `harnesses/shared/`, and replace every
per-skill copy with a one-line pointer. A policy change should be a one-file
edit, not a 19-file sweep.

## Non-Goals

- Do NOT change the doctrine's meaning. This is a move + dedupe, not a rewrite.
- Do NOT remove a skill's *local* "what lanes for THIS phase" guidance — only
  the generic boilerplate that is identical everywhere.
- Do NOT over-indirect. Skills keep a one-line statement of the rule + a pointer;
  a reader must still see that the floor applies without chasing three hops
  (progressive-disclosure failure is the risk to avoid).
- Do NOT touch the external-provider roster mechanics (`scripts/`, `agents.yaml`).

## Constraints / Invariants

- Cross-harness: the single source is plain markdown under `harnesses/shared/`,
  symlinked everywhere by bootstrap. No runtime-only mechanism.
- Depends on `051` landing first (the signposted shared AGENTS.md / references
  home is where the single source lives).
- The shared statement must be self-contained enough that a skill citing it
  needs no second file to know the floor applies.

## Authority Order
tests > type system > code > docs > lore

## Repo Anchors
- The 19 skills carrying the duplicated paragraph (verified by
  `grep -rl "When a provider roster is available" skills/*/SKILL.md`):
  ci, code-review, create-repo-skill, deliver, demo, design, diagnose, groom,
  hardening, flywheel, refactor, implement, monitor, shape, reflect, qa, ship,
  yeet, settle.
- `harnesses/shared/AGENTS.md` — the single source (post-`051` layering).
- Completion-Gate duplication: code-review, deliver, design, hardening,
  implement, refactor, qa, demo.
- Clean-tree closeout: deliver, yeet, ship + root `AGENTS.md` + shared AGENTS.md.

## Alternatives Considered
| Option | Shape | Strength | Failure mode | Verdict |
|---|---|---|---|---|
| Single source + 1-line pointer (this) | Doctrine in shared, skills cite it | One edit point; cross-harness; readable | Slight indirection | **choose** |
| Keep copies, add a lint that diffs them | Detect drift, don't dedupe | No indirection | Still 19 copies; lint nags forever | reject |
| Inline-everything generator | Templating skill bodies at build | DRY source-of-truth | Build complexity; opaque rendered skills | reject |

## Oracle (Definition of Done)
- [ ] `grep -rl "When a provider roster is available" skills/*/SKILL.md | wc -l`
      returns 0 (the paragraph lives once in `harnesses/shared/`).
- [ ] Each of the 19 skills retains a ≤2-line statement that the delegation
      floor applies + a pointer to the shared source; its phase-specific lane
      guidance is preserved.
- [ ] Completion-Gate and clean-tree closeout each have a single canonical
      definition referenced by the skills that used to inline them.
- [ ] A reviewer reading any one skill can still tell, without opening another
      file, that the floor/gate/closeout applies (one-line restatement + link).
- [ ] `scripts/generate-index.sh` green; no dangling references.
- [ ] `dagger call check --source=.` passes; a spot-check of 3 skills'
      behavior (groom, deliver, code-review) is unchanged.

## Implementation Sequence
1. (After `051`.) Confirm the canonical delegation-floor / completion-gate /
   clean-tree statements in `harnesses/shared/`.
2. Replace the paragraph in each of the 19 skills with the 1-line statement +
   pointer; preserve local lane guidance.
3. Same for Completion-Gate (8 skills) and clean-tree (5 files).
4. `generate-index.sh`, `dagger call check`, spot-check 3 skills.

## Risk + Rollout
- **Over-indirection** — the explicit user/lane warning. Mitigation: the
  1-line-restatement-plus-pointer rule; never a bare "see X".
- **A skill had a subtly different floor.** Mitigation: diff each copy against
  the canonical text before replacing; if a skill genuinely differs, keep the
  delta local and cite the base.
- Rollback: the copies are recoverable from git history; revert per file.

## Delegation Evidence
- All four lanes independently flagged the delegation-floor duplication; codex
  and grok cited specific file:line ranges; agy enumerated 10 skills; lead grep
  confirmed 19. Web brief reinforced "context window is your most important
  resource" + references-one-level-deep. Accepted.

## Related
- Depends on `051` (single home). Pairs with `076` (collision/health lint can
  later assert no re-duplication). Part of the MECE/DRY consolidation line with
  `080` (settle→deliver).
