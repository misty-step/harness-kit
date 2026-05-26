---
name: shape
description: |
  Shape a raw spellbook idea into a buildable backlog.d/ file — new skill,
  new Dagger gate, new hook, doctrine extension, or cross-harness mechanism.
  Solution-side double diamond: diverge on approach, converge on a shape
  with an executable oracle before /implement touches it.
  Use when: "shape this", "write a shape", "design this skill",
  "spec out a gate", "plan this doctrine change", "shape a harness mechanism".
  Trigger: /shape, /spec, /plan.
argument-hint: "[idea|issue|backlog-item] [--spec-only] [--design-only]"
---

# /shape

Take an idea and produce `backlog.d/NNN-<kebab-title>.md` — a shaped ticket
that `/deliver` (inner loop) or `/flywheel` (outer loop) can consume. You
are the solution-side double diamond. `/groom` is the problem diamond;
`/shape` is what happens after the problem framing is locked.

Spellbook ships harness primitives — skills, agents, Dagger gates, hooks,
settings, doctrine. A shape here is not "a product feature." It is
typically one of: **a new skill, a new atomic sub-gate under
`dagger call check`, a pre-commit hook rule, a doctrine paragraph in
`harnesses/shared/AGENTS.md`, or a cross-harness mechanism that emits
per-harness artifacts from one source.** Price your shape to that unit.

## Workflow

### Phase 1 — Orient

Accept a raw idea, a `backlog.d/NNN-*.md` to refine, or an observation
("the gate let X through"). Before exploring solutions:

- Read `/Users/phaedrus/Development/spellbook/.spellbook/repo-brief.md`
  (or the equivalent under the invoking repo) for the current gate set,
  invariants, known debts, and session-signal corrections.
- Read 2-3 recent `backlog.d/_done/*.md` as format exemplars. Good
  references: `_done/017-harness-refactor.md` (extract-and-restructure),
  `_done/032-deliver-inner-composer.md` (skill rename + recompose with
  Contract/Composition/Receipt sections), `_done/034-ci-skill-redesign.md`
  (new skill with Stance + Contract + Oracle), `_done/045-harness-pivot-minimal-globals.md`
  (doctrine pivot with Before/After).
- If the idea touches an existing skill, read its `SKILL.md` plus any
  `references/` that apply. If it touches a Dagger gate, read
  `ci/src/spellbook_ci/main.py`. If it touches bootstrap or install flow,
  read `bootstrap.sh`.
- Dispatch parallel Explore subagents only when the context is genuinely
  uncharted. For a scoped skill tweak, skip — the three-file read above
  already beats a subagent round-trip.
## Delegation Floor

When `.spellbook/agents.yaml` exists, `/shape` starts by probing the roster
and dispatching two or more available roster members before converging on a
problem frame or solution. Use one lane to map repo constraints and another
for prior-art or premise challenge; add more lanes when product, technical,
or customer-risk perspectives diverge. The lead agent owns synthesis,
questions to the user, final context-packet wording, and receipts. Direct
lead-only shaping is limited to mechanical packet formatting, emergency
unblocks, explicit user-forbidden delegation, or an explicit waiver when
fewer than two roster members are available.

### Phase 2 — Problem Challenge

**Five-whys the ticket before accepting its framing.** Tickets here
routinely encode symptoms:

- "Add a lint rule for X" — what *user-visible* failure is X a proxy for?
- "Refactor /settle" — is the real outcome "polish for merge" or is it
  "delete /settle because `/ci` + `/code-review` cover it"?
- "New skill /deploy" — does the invoking repo actually have a deploy
  target? Cross-harness parity included?

If the stated solution survives the five-whys, proceed. If not, the shape
is now of a different thing — write it down and continue with the real
target.

**Recognize the Doctrine Shortcut.** Many "new mechanism" requests should
be a single paragraph added to `harnesses/shared/AGENTS.md` plus a lint
rule, not a new skill. If the fix is "the agent should know X," the shape
is an AGENTS.md patch + a gate that proves it. `_done/045` is the canonical
example — pivot doctrine, delete the over-built thing it replaced.

**GATE:** Do not proceed to Phase 3 until you can state the shape's unit
in one sentence: *"a new `/foo` skill"*, *"a new sub-gate `check-bar`
under `dagger call check`"*, *"a pre-commit hook rule that rejects Z"*,
*"doctrine: paragraph in harnesses/shared/AGENTS.md plus enforcement lint"*.

### Phase 3 — Solution Divergence

Produce **≥2 structurally distinct approaches**, with one chosen. Real
divergence means the shapes fail differently:

- **Minimal-viable** — smallest change that delivers the outcome. Often
  doctrine + lint, or extend an existing skill.
- **Ideal** — what you'd build if the rest of the system were already
  there. Often reveals missing prerequisites.
- **Inverted** — flip a load-bearing assumption. "What if the gate is
  agent-judgment instead of deterministic lint?" "What if we delete the
  skill instead of growing it?" "What if this is per-harness config
  instead of shared code?"

Examples of real divergence from repo history:
- 032 (/deliver rename) chose **compose-atomic-phase-skills** over
  **grow /autopilot** — structural guarantee via `check-deliver-composition`
  gate enforces the choice.
- 045 (harness pivot) chose **minimal globals + per-repo population**
  over **global catalog + per-project allowlist**. Two architectures;
  the inverted one won after six weeks of the first one misbehaving.
- 034 (/ci) chose **new skill** over **grow /settle** on domain-
  coherence grounds; `/settle` was then slated for deprecation.

Alternatives-in-name-only is the failure mode: three approaches wearing
the same idea in three outfits. If you cannot articulate a different
failure mode per option, you have one option.

**For M+ effort, invoke the solution-divergence bench in parallel:**
- `ousterhout` — is the skill a deep module with a simple interface, or
  a shallow pass-through?
- `carmack` — is this shippable in the claimed estimate, or design-
  astronaut territory?
- `grug` — is the complexity load-bearing, or is there a smaller thing
  that does the same job?
- `beck` — can this be TDD'd? Where is the first red test?

Give each the shape summary + repo-brief. Proceed only after blocking
concerns are addressed. **Philosophy bench is persona diversity, not
foundation diversity.** For genuinely novel designs, also consult a
cross-model second voice (provider roster / Thinktank / fresh-context
subagent) — same-model self-debate collapses to consensus.

### Phase 4 — Cross-Harness Check (Red Line)

Before writing the shape, answer: **"What does this do on Codex and
Pi?"** If you cannot answer, the shape is incomplete. Cross-harness
parity is a Red Line.

- Skill content is cross-harness by construction (filesystem + SKILL.md).
  Works on Claude, Codex, Pi.
- Runtime features (Claude's `enabledPlugins`, Codex's `/plugins`, Pi's
  `settings.json:skills[]`) are optimizations *on top of* filesystem
  selectivity, not the primary layer. Reference pattern in repo:
  `harnesses/pi/settings.json:skills[]` globs.
- If the mechanism needs runtime toggling, the shape must describe a
  single source emitting per-harness artifacts. "Only works on Claude"
  fails the check.
- `bootstrap.sh` symlinks by default. Anything that relies on copy-mode
  (`harnesses/claude/settings.json` is COPIED not symlinked) must say so
  and name the re-bootstrap consequence.

Fold the answer into the shape body (typically under `## Design` or its
own `## Cross-Harness` subsection).

### Phase 5 — Write the Shape

Output file: `backlog.d/NNN-<kebab-title>.md` where `NNN` is the next
monotonic integer (check `backlog.d/` + `backlog.d/_done/` for highest).
Use the template below. A shape of a small scoped change (new lint rule,
doctrine paragraph) is 40–80 lines. A shape of a new skill with a
durability contract (see 028, 032) runs 300–500 lines. Match scope.

```markdown
# <concise-title>

Priority: <P0|P1|P2|P3|high|medium|low>
Status: <pending|in-progress|shipped|done|abandoned>
Estimate: <S|M|L|XL>
<optional: Aliases: /foo, /bar>
<optional: Supersedes: NNN-other-shape.md>
<optional: Shipped: YYYY-MM-DD>

## Goal

<1-3 sentences. What outcome, not mechanism. Name the unit: "a new
`/foo` skill", "a new sub-gate", "doctrine + lint rule".>

## Why This Isn't X
<optional; REQUIRED when superseding or adjacent to a prior/killed
artifact. Reference the prior shape ID or killed skill (e.g. "/focus
killed March 2026"). Name the failure mode being avoided.>

## Design

<For small shapes (like 023, 025): a single numbered list of
mechanisms is enough.>

<For larger shapes: use sections like Composition, State Model,
Contract, Receipt, Components, Durability Guarantees, Worktree
Behavior. Match the concerns of 028-flywheel / 032-deliver.>

## Cross-Harness

<REQUIRED unless the shape is Claude-specific by construction (e.g.
`harnesses/claude/settings.json`). Answer: what does this do on Codex
and Pi? What ships to `.agents/skills/`? What per-harness artifacts,
if any, are emitted?>

## Oracle

<Executable checkbox list. Commands that exit 0, or dagger call
check sub-gates that turn green. Not prose ("it should work").>

- [ ] `dagger call check --source=.` passes with the new
      `check-<name>` sub-gate included
- [ ] <skill> symlink-installs into a foreign project and runs
      without `../..` or `$REPO_ROOT` errors
- [ ] pre-commit hook rejects a test commit that <violates rule X>

## Non-Goals

- <Scope boundaries that agents will otherwise drift into>
- <Load-bearing. Agents expand scope by default; write the non-goals.>

## Related
<optional>
- Blocks: NNN-other.md
- Depends on: NNN-other.md
- Supersedes: NNN-dead.md
```

If you cannot write an executable oracle, the goal isn't clear enough.
Go back to Phase 2. See `references/executable-oracles.md`.

## Gotchas

- **Premise unchallenged.** Shape requests arrive as solutions wearing
  problem masks. "Shape `/deploy`" from a repo with no deploy target is
  the wrong ticket. Five-whys first.
- **Missing Cross-Harness section.** Red Line. If the shape doesn't name
  Codex + Pi behavior, reviewers will bounce it. Bake this into the body
  before submitting.
- **Alternatives-in-name-only.** Three "options" that share the same
  failure mode are one option. Force an inverted-assumption alternative;
  if you can't, call the single option what it is and justify.
- **Shape of a product feature.** This repo ships harness primitives.
  If you find yourself writing a feature spec with users, routes, and
  a product surface, you are in the wrong repo — the shape is probably
  for the consuming project, not spellbook itself.
- **New skill when doctrine + lint would do.** Default to the cheaper
  shape. A paragraph in `harnesses/shared/AGENTS.md` plus a `check-<name>`
  sub-gate is often the whole fix. `_done/045` pivoted from a skill
  architecture to a doctrine pivot; the replaced machinery was deleted.
- **Shape of a skill-that-procedures.** Skills encode judgment. If the
  shape reads like a runbook ("step 1: run X, step 2: parse Y"), the
  skill is waste — the model already knows how. Shape the gotchas and
  invariants instead. Reference form: `skills/flywheel/SKILL.md` (43
  lines).
- **No oracle or vague oracle.** "Runs on spellbook" is not an oracle.
  "`dagger call check --source=.` green with the new sub-gate; foreign-
  project symlink install succeeds" is. The oracle is what `/deliver`'s
  clean loop terminates against.
- **50 anchors in the design.** If every file matters, none do. Name the
  3-10 load-bearing files whose patterns the builder must match.
- **Skipping non-goals.** Agents drift toward scope expansion — a shape
  without `## Non-Goals` is a shape that grows silently. Bound it.
- **Over-speccing implementation.** The shape specifies WHAT and WHY.
  `/implement` + `/deliver` figure out HOW. Pseudocode in a shape
  cascades errors.
- **Shaping after building.** A shape written after the diff exists is
  documentation, not specification. `/deliver` reads the shape first;
  `/implement` cannot run without it. Shape first.
- **Marking it `done` prematurely.** `Status: done` belongs to
  `backlog.d/_done/` only. `/flywheel`'s `update-bucket` phase moves the
  file when a cycle ships (see `_done/028`). Don't stamp `done` in-place.
- **Forgetting Red Lines.** "Cross-harness first", "no `../..` in skill
  libs", "no claim-coordination primitives", "`.spellbook/deliver/<ulid>`
  is agent-written and gitignored" — these are structural. The shape
  that violates one is dead on arrival.

## Principles

- Unit-price the shape: skill | gate | hook | doctrine | cross-harness
  mechanism. Mixing units is a refactor signal.
- Minimize touch points — fewer files = less blast radius.
- Design for deletion. Skills get killed (see `/focus`, `/autopilot`-old,
  `/settle`-slated). A shape that cannot be cleanly reversed is a
  liability.
- Favor existing patterns. The repo already has gate-lint, pre-commit-
  hook, doctrine, and skill-composition patterns — use them before
  proposing new infrastructure.
- YAGNI ruthlessly. Phase 2+ features in the shape are fine; Phase 2+
  features in the *MVP section* are scope creep.
- Recommend, don't just list. The shape document picks an option, names
  the alternatives, says why.
- One question at a time during convergence. Silent absorption of user
  feedback is not ratification.

## References

- `references/executable-oracles.md` — oracle-as-command discipline
- `references/critique-personas.md` — divergence bench personas (the
  full set; Phase 3 above names the default four)
- `references/shaping-methodology.md` — the R/S/fit-check methodology
  when shaping something product-shaped rather than harness-shaped
  (rare in this repo; applies when shaping downstream-consumer artifacts)
- `references/breadboarding.md` — affordance-mapping for UI/system work
  in consuming repos (not typically used for spellbook's own shapes)
- `references/writing-plans.md` — bite-sized TDD task decomposition for
  the rare case where a shape needs a full implementation plan attached
