# Extract shared doctrine to a single source: kill the 19-skill copy-paste

Priority: P1
Status: merge-ready
Estimate: M

## Goal

Remove the largest DRY violation in the harness. The "Delegation Floor"
paragraph is copy-pasted near-verbatim across **19 of 28 skills**; the
Completion-Gate block across 8; the clean-tree closeout across 5 files. State
each cross-cutting contract ONCE in `harnesses/shared/`, and replace every
per-skill copy with a one-line pointer. A policy change should be a one-file
edit, not a 19-file sweep.

## Blocker found (2026-05-29) — NOT a mechanical sweep

`scripts/check-agent-roster.py` (`delegation_floor_section()` +
`DELEGATION_FLOOR_REQUIREMENTS`, ~line 37/85) **enforces** that every
`CORE_WORKFLOW_SKILLS` entry contains a full `## Delegation Floor` section with
the phrases `two or more`, `mechanical`, `emergency`, `user-forbidden`, `fewer
than two`, `lane`, context (`give`/`scope`), `receipt`/`evidence`, `lead`. That
is exactly the duplication 081 wants to delete — so replacing each paragraph
with a one-line pointer makes the roster gate go RED. 081 must therefore
**redesign that gate first**: accept either (a) a full section OR (b) a
one-line pointer to the shared single source, and validate the canonical phrases
ONCE against `harnesses/shared/AGENTS.md` (## Roster, the single source 051
established). Live count is now **20** skills (grep `provider roster is
available`), completion-gate **8**, clean-tree **2** (settle redirect dropped
its copies). 051 dependency is satisfied. codex design lane dispatched for the
gate redesign + pointer pattern (transcript in `.harness-kit/traces`).

## Progress (2026-05-29)

- **Step 1 DONE** (`edbf9b5`): `check-agent-roster.py` redesigned — accepts a
  full `## Delegation Floor` section OR a one-line pointer; validates the
  canonical contract once against `harnesses/shared/AGENTS.md ## Roster`.
  Helpers `markdown_section`/`has_delegation_floor_pointer`/
  `delegation_contract_gaps`. A bare "see X" is rejected (over-indirection
  guard). Unit-verified; dagger 15/15.
- **Step 2 REMAINING — safe & resumable** (the gate accepts both forms, so a
  partial sweep is still green). Convert each of the 20 skills' delegation-floor
  paragraph to this exact pointer + its preserved local lane guidance:

  > ## Delegation Floor
  >
  > Delegation floor applies: probe the roster first; dispatch two or more
  > providers for substantive work; direct solo only for mechanical, emergency,
  > user-forbidden, or fewer-than-two-providers cases. See
  > `harnesses/shared/AGENTS.md` (Roster).
  >
  > Local lane guidance: <keep this skill's phase-specific "use lanes for X"
  > sentence here, verbatim from the old paragraph>.

  Skills (grep `provider roster is available` in `skills/*/SKILL.md`): ci,
  code-review, create-repo-skill, deliver, demo, design, diagnose, flywheel,
  groom, hardening, harness-engineering, implement, monitor, qa, refactor,
  reflect, research, shape, ship, yeet.
- **Step 3 REMAINING**: completion-gate (8 skills) → pointer to a shared
  completion-evidence statement; clean-tree (2 files) → pointer to
  `harnesses/shared/AGENTS.md ## Closeout`. (codex: add a shared `## Completion
  Evidence` section; grok: point at deliver's block — decide at implementation.)
- Verify each batch: `python3 scripts/check-agent-roster.py` + `dagger call
  check`. Design lanes: codex (`49190d45`) + grok-build (convergent).

## Progress (2026-06-01)

- **Step 2 DONE** (`deliver/081-delegation-floor-dry`): converted the live
  root-skill delegation-floor boilerplate to the short
  `Delegation floor applies...` pointer plus `Local lane guidance:` for 20
  skills, including `design` after live grep showed it carried the old paragraph.
- Tightened `scripts/check-agent-roster.py` so pointer-mode sections still
  require non-empty local lane guidance and cannot bypass the native-subagent
  boundary check for `shape`, `research`, `harness-engineering`, or
  `create-repo-skill`.
- **Step 3 still remaining**: Completion Gate and clean-tree/Closeout dedupe.
  Fresh critic lanes (`cc23f2fd-c781-40cf-aa16-98ebc3ba650a`,
  `9fae8c24-35c7-463d-a5e3-b5b8c8e59d4c`) both argued to split Step 3 until
  a shared `## Completion Evidence` anchor is shaped and gated.

## Progress (2026-06-01, Step 3)

- **Step 3 DONE** (`deliver/081-delegation-floor-dry`): added shared
  `harnesses/shared/AGENTS.md ## Completion Evidence` and kept phase-specific
  completion fields local instead of flattening QA, design, hardening, and demo.
- Added `scripts/check-agent-roster.py` validation for the shared Completion
  Evidence anchor, four core completion-evidence pointers, four local domain
  gates, and four shared Closeout pointers.
- Dedupe kept the local clean-tree semantics that fresh critics found
  load-bearing (`deliver` merge-ready, `ship` precondition, `yeet` operational
  checks), while pointing their universal closeout rule at shared Closeout.

## What Was Built

- `harnesses/shared/AGENTS.md` now owns the universal completion-evidence core
  and explicitly marks shared Closeout as the single source for clean-tree
  closeout.
- 20 workflow skills point at the shared Roster source with preserved
  `Local lane guidance`.
- `code-review`, `deliver`, `implement`, and `refactor` point at shared
  Completion Evidence and keep only local fields inline.
- `demo`, `design`, `hardening`, and `qa` keep local domain gates and cite shared
  Completion Evidence for the common evidence principle.
- Root `AGENTS.md`, `deliver`, `ship`, and `yeet` point at shared Closeout for
  the universal clean-tree rule.

## Verification

- `python3 scripts/check-agent-roster.py`
- `python3 scripts/check-frontmatter.py`
- `scripts/build-docs-site.sh && scripts/check-docs-site.sh`
- `dagger call check --source=.`

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
- [x] `grep -rl "When a provider roster is available" skills/*/SKILL.md | wc -l`
      returns 0 (the paragraph lives once in `harnesses/shared/`).
- [x] Each of the 19 skills retains a ≤2-line statement that the delegation
      floor applies + a pointer to the shared source; its phase-specific lane
      guidance is preserved.
- [x] Completion-Gate and clean-tree closeout each have a single canonical
      definition referenced by the skills that used to inline them.
- [x] A reviewer reading any one skill can still tell, without opening another
      file, that the floor/gate/closeout applies (one-line restatement + link).
- [x] `scripts/generate-index.sh` green; no dangling references.
- [x] `dagger call check --source=.` passes; a spot-check of 3 skills'
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
