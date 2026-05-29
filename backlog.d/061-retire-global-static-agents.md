# Subagent roles, not files: ad-hoc dispatch doctrine + milestone critic gate

Priority: P0
Status: ready
Estimate: L

## Goal

Replace the global static-agent catalog with a positive doctrine: the primary
agent can and should dispatch focused, ad-hoc subagents, and the harness names
the *roles* (executor, critic, planner) and *perspectives* (lenses) those
subagents embody — without shipping a persona file per role. Frontier primaries
instantiate the right subagent at runtime; the durable primitive is the
instruction to do so, plus a strong default shape and the lens content worth
reusing.

## Non-Goals

- Do NOT ban subagents or weaken delegation. The opposite: make subagent use
  more frequent, more task-specific, and better-scoped.
- Do NOT build an orchestration DSL or a subagent scheduler.
- Do NOT remove runtime-native or project-local named subagents when a target
  repo genuinely needs a persistent one.
- Do NOT discard the philosophy/security/perf content. It survives as compact
  lens rubrics the primary reads, not as installed subagent files.
- Do NOT touch the external-provider roster (`.harness-kit/agents.yaml`). That
  is a different delegation axis (cross-provider lanes) and stays as-is.

## Constraints / Invariants

- Cross-harness first. The doctrine is prose in `harnesses/shared/AGENTS.md` +
  SKILL.md, so it works on Claude, Codex, Pi, Antigravity, Grok, Cursor,
  OpenCode identically. No Claude-only `subagent_type:` dependency may remain
  load-bearing.
- The executor+critic default is a STRONG default, waivable only on
  mechanical/trivial work — mirroring the existing delegation-floor exception
  structure (`harnesses/shared/AGENTS.md` "Direct solo work only"). Same waiver
  shape; do not invent a second one.
- One carve-out survives as defined subagents: the **a11y trio**
  (`a11y-auditor`, `a11y-fixer`, `a11y-critic`) — a read-only → write → verify
  protocol where the permission split is load-bearing, not persona. Per global
  doctrine, "static project subagents are for tool/permission isolation only."

## Authority Order
tests > type system > code > docs > lore

## Repo Anchors
- `harnesses/shared/AGENTS.md` — "Roster" + "Prompts" sections; home for the
  new dispatch doctrine. Composes with `051` (three-layer restructure): the
  doctrine lands in the agent-gotchas / routing layer.
- `bootstrap.sh:361-398` — the `GLOBAL_AGENTS` install loop to retire (stops
  installing `agents/*.md` globally; a11y trio is the allowlisted exception).
- `skills/groom/SKILL.md:194-198` — the only hard `subagent_type:` references
  (ousterhout/carmack/grug/beck); convert to ad-hoc critics reading lens rubrics.
- `agents/*.md` — source content to triage (role files → doctrine; philosophy
  files → lens rubric).
- `skills/code-review/SKILL.md`, `skills/refactor/SKILL.md`,
  `skills/diagnose/SKILL.md` — name when to dispatch which role/lens.

## Prior Art
- `skills/shape/SKILL.md:91-100` — already spawns an ad-hoc design bench by
  perspective. The pattern to generalize: name the lens, give scope + evidence
  contract, synthesize. No persona file needed.
- Global CLAUDE.md "Prompts" — "Commission agents; do not chat at them" +
  "Prefer ad-hoc roster lanes over static named subagents."

## Alternatives Considered
| Option | Shape | Strength | Failure mode | Verdict |
|---|---|---|---|---|
| Roles-not-files (this) | Doctrine + lens rubric + a11y carve-out | Durable across harnesses; primary owns instantiation; least surface | Primary under-dispatches if doctrine is weak prose | **choose** |
| Grow static catalog | Keep + add security/perf/api/threat `.md` | Explicit, discoverable | Stale personas; Claude-only `subagent_type`; contradicts frontier direction | reject |
| Delete with no doctrine | Just remove files | Minimal | Loses the executor+critic default and the reusable lens content | reject |

## Oracle (Definition of Done)
- [ ] `harnesses/shared/AGENTS.md` contains a compact, always-on **dispatch
      doctrine**: (a) the primary can and should dispatch focused subagents;
      (b) strong default ≥2 — **executor + critic**; (c) **planner + executor +
      critic** triad when planning is non-trivial; (d) the waiver is the
      existing mechanical/trivial exception, not a new one; (e) name the role +
      scope + evidence contract, or leave the persona to primary discretion.
- [ ] `bootstrap.sh` no longer installs `agents/*.md` globally EXCEPT the a11y
      trio; a fresh `bash bootstrap.sh` then `ls ~/.claude/agents` shows only
      the allowlisted a11y agents (and any user-owned files), not ousterhout/
      carmack/grug/beck/cooper/planner/builder/critic.
- [ ] Philosophy/security lens content lives in ONE rubric file (e.g.
      `harnesses/shared/references/lenses.md`): each lens = name + one-line
      essence + "what it looks for" + "the failure it would catch." Covers at
      least ousterhout, carmack, grug, beck, cooper.
- [ ] `skills/groom/SKILL.md` no longer uses `subagent_type: <persona>`; it
      dispatches ad-hoc critics that embody the named lens from the rubric.
      `grep -rn "subagent_type: \(ousterhout\|carmack\|grug\|beck\|cooper\)" skills/`
      returns nothing.
- [ ] `skills/code-review`, `skills/refactor`, `skills/diagnose` each state when
      to dispatch which role/lens and the evidence it must return.
- [ ] **Milestone critic gate**: `skills/implement/SKILL.md` and
      `skills/deliver/SKILL.md` require, at each implementation milestone (chunk
      in the sequence), a fresh read-only critic that sees ONLY the diff + the
      packet oracle + the todo, and returns gaps; the loop does not advance to
      the next milestone until the critic returns no blocking gap or the gap is
      explicitly waived in the receipt. Threshold: skip for trivial diffs
      (<20 LOC, single file) to avoid over-engineering.
- [ ] `agents/*.md` disposition recorded in the commit: role files
      (planner/builder/critic) → doctrine; philosophy files → lens rubric;
      a11y trio → kept. No orphaned reference to a deleted agent in any SKILL.md
      (`scripts/generate-index.sh` green; no dangling links).
- [ ] `dagger call check --source=.` passes all gates.

## Implementation Sequence
1. Write the dispatch doctrine in `harnesses/shared/AGENTS.md` (executor+critic
   default, triad, waiver, role+scope+evidence shape).
2. Extract the philosophy lens content from `agents/{ousterhout,carmack,grug,
   beck,cooper}.md` into `harnesses/shared/references/lenses.md`.
3. Rewire `skills/groom/SKILL.md` bench from `subagent_type:` to ad-hoc
   lens-critic dispatch reading the rubric.
4. Add the when-to-dispatch-which-role/lens guidance to code-review, refactor,
   diagnose.
5. Retire the global install in `bootstrap.sh` with the a11y allowlist; delete/
   convert the retired `agents/*.md`.
6. Add the milestone critic gate to `implement` then `deliver` (depends on the
   critic role being defined in step 1).
7. Run `bootstrap.sh`, `generate-index.sh`, `dagger call check`.

## Risk + Rollout
- **Primary under-dispatches** if the doctrine is soft prose. Mitigation: make
  it a routing-table row in AGENTS.md Layer 3 (`051`), not a paragraph; eval it.
- **Milestone gate over-engineers** trivial work (the explicit user warning).
  Mitigation: the <20 LOC / single-file skip threshold + waiver-in-receipt.
- **Cross-harness regression**: a harness without `subagent_type` already
  ignored the files; removing them only helps. Rollback = restore `agents/*.md`
  + the bootstrap loop (one revert).

## Delegation Evidence
- Roster providers used: codex (GPT-5.5), agy (Gemini), grok-build (Grok-4.3) —
  all independently corroborated retiring static agents / the static-vs-skill
  duplication; codex: "do not add more static agents unless `/critique` or
  bench-map consumes them."
- Web research: Claude Code sub-agents docs, Antigravity async subagents,
  Codex AGENTS.md+skills direction — all point to lead-agent-managed dynamic
  delegation over static catalogs.
- Accepted: the roles-not-files framing + executor/critic default (operator
  directive, 2026-05-29).
- Rejected: grow-the-catalog (077/078/079 original direction) — folded into
  this line instead.

## Related
- Supersedes the original 061 (bare retirement) with the operator's
  roles-not-files doctrine (session 2026-05-29).
- Folds in: original `078` (security/perf/api/threat → lens rubrics, not agent
  files) and the gfodor milestone-review gate.
- Composes with `051` (AGENTS.md three-layer + routing — placement & DRY) and
  `077` (`/critique` consumes the lens rubric).
- a11y trio kept per `skills/a11y/SKILL.md` three-agent protocol.
