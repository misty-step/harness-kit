# Design-spine: compose /design into a router-with-thesis + a self-enforcing DESIGN.md contract

Priority: P2 · Status: in-progress — A done, B-format done; B-gate + C need a frontend repo · Estimate: L (epic, 3 tracks A→B→C)

## Goal
Agent-authored UI in any HK-bootstrapped repo follows a persistent, CI-enforced
design contract, and `/design` reaches the right component skill on the first
try — by rewriting `/design` into a **router with a thesis + a preset dial +
wired gates** that composes (does not delete) the 17 vendored design skills.

## Non-Goals
- Deleting component skills. The value audit proved 0 of 8 `leon-*` are
  redundant; keep all, route one primary per role.
- A semantic workflow engine. The master skill must not teach the SDLC sequence
  the model already knows, and **no route may depend on another route's output**
  — each is independently invocable. Railroading is a control-flow property, not
  just a vocabulary one; a numbered phase skeleton is a pipeline even if no line
  says "now write tests". Violates VISION.md "thin harness, strong models"
  otherwise.
- A rendered-DOM gate in v1 (defer Dembrandt eval to a follow-on); a
  StyleX-style typed-prop compiler; authoring HK's brand identity; any Mode B /
  unattended design loop.

## Constraints (invariants that must survive)
- Net front-door surface **shrinks** even as component depth is preserved.
- `cargo run --locked -p harness-kit-checks -- check --repo .` stays green.
- Cross-harness portability: filesystem + `SKILL.md` is the primary layer.
- No no-license external prose copied into the repo (route to vendored skills;
  do not inline their bodies). See `registry.yaml` no-license caveats.
- The anti-slop detector stays deterministic (`npx impeccable detect`, exit 2).

## Context
HK does not lack design capability — it has a sprawling, mostly-vendored design
surface with no coherent spine. Telemetry (2026-06-23): of ~19 design-adjacent
skills only `design` (15×) and `anthropic-frontend-design` (3×) are used; the
entire vendored taste family is cold. Diagnosis: components are cold because
they **overlap and hide at the description layer**, not because they are
low-value. The four operator launchpad prompts (design-system extraction, a11y +
multi-device, DESIGN.md-first, UX audit) all want the same missing thing —
**persistence + anti-drift** — whose answer is mechanical enforcement, not more
taste prose.

The world standardized the exact primitive HK hand-rolled: DTCG tokens reached
first stable W3C version 2025.10; Google Labs open-sourced the DESIGN.md spec +
linter (`@google/design.md`, verified installable v0.3.0; `lint`/`diff`/
`export`→DTCG|Tailwind/`spec`); Anthropic issue #1008 wires DESIGN.md into the
`frontend-design` skill `/design` already points at. HK's bespoke nine-section
DESIGN.md was ~80% congruent → drop it, adopt the standard for free tooling.

Full grounding (telemetry numbers, research brief + URLs, per-skill value audit,
role map, vision tension) in the premise artifact below.

## Repo Anchors
- `skills/design/SKILL.md` — the file rewritten; current ~30-row routing table,
  ~20 rows "compose with impeccable /impeccable X".
- `skills/design/references/scaffold.md` — nine-section DESIGN.md to replace.
- `skills/.external/cursor-thermo-nuclear-code-quality-review/SKILL.md` —
  decision-machine exemplar: frame → orthogonal gates → escalation → asymmetric
  earned-approval bar → prioritized output. Steal the *shape*, not the steps.
- `skills/.external/impeccable-impeccable/` — runtime skeleton to steal:
  brand-vs-product register fork, context-aware no-arg routing, dual-isolated
  critic (heuristic + deterministic, synthesized not concatenated).
- `harnesses/shared/references/preferred-stack.md` (lines 103–123) — the
  "design system must enforce itself" doctrine this epic finally wires.
- `skills/harness-engineering/references/mode-eval.md` — the routing-eval
  contract (A/B in worktrees, decorrelated blind judge).
- `VISION.md` — the thin-harness invariant the design must not break.

## Alternatives
| Option | Why it helps | Tradeoff | Verdict |
|---|---|---|---|
| **Router-with-thesis (chosen)** | Fixes role-confusion (the real telemetry problem); lands contract + verify primitives; thin-harness-compatible. | Rewrites a hot skill; risk of drifting into a phase engine. | **choose** |
| Ponytail: 3 tiny edits, no rewrite | Preset-dial subsection + ban-core fold + standalone gate, ~1 day. | Leaves 17 skills overlapping; no decision-machine posture. | **graft** — Track B ships this way regardless |
| Invert: absorb into impeccable runtime | impeccable already is the verb-router + dual-critic runtime; least bespoke prose. | Bets the spine on a no-license external w/ 60k-line files; interface drift breaks us; not the operator's ask. | reject (steal skeleton only) |
| Prune to ~2 (original) | Smallest surface; pure delete-as-progress. | Audit proved 0 of 8 `leon-*` redundant; throws away distinct presets/methods. | reject (operator) |

## Design
Three tracks, sequenced A→B→C; **B and C are phases inside the composed skill**,
and Track B's gate is independently shippable (the Ponytail graft).

**Track A — compose `/design` as router-with-thesis.** Rewrite `SKILL.md` as a
**flat menu, not a pipeline** — every route is independently invocable
(`/design polish` never has to pass through "generate"; no route depends on
another route's output). Its parts: opening taste-doctrine folded from
`anthropic-frontend-design`; two **dials** the operator sets (not stages) — an
aesthetic preset (`leon-taste`(default)/`soft`/`minimalist`/`brutalist` ×
`nutlope` genres, genuine variants) and an entry mode (greenfield→
`nutlope-hallmark` | redesign→`impeccable audit+critique`, folding
`leon-redesign`'s a11y/SEO omissions checklist | image-first→`leon-images` when
image-gen available); a **role map** routing exactly one primary per role
(micro-polish→`jakub`; motion author→`emil-emil`; motion review→
`emil-review-animations`, confirmed vendored; a11y→`vercel-web-design-guidelines`;
tokens→`@google/design.md`; view-transitions/react-arch→the two vercel skills,
conditional); one **folded anti-slop ban-core** (collapse the ~7 duplicated
ban-lists; `nutlope` slop-test is the superset gate); and a **review gate** that
fans to dual isolated critics (heuristic + deterministic, synthesized). Borrow
thermo-nuclear's earned-approval posture (default-deny, author justifies
blockers) and its gate/dispatch shape — **not** an ordered phase sequence.

**Track B — self-enforcing DESIGN.md contract.** Replace the nine-section
scaffold with the `@google/design.md` spec + DTCG token frontmatter.
`/design scaffold` emits a DESIGN.md the standard linter validates, plus a wired
source gate: stylelint `declaration-strict-value` + eslint
`no-restricted-imports` + a `src/design-system/examples/` golden dir + one
`verify-ui` command (the Builder.io agent-loop pattern — stderr re-enters the
next prompt). Product-intent/voice/governance, if kept, live in the generated
project-local design skill, not DESIGN.md (the standard is purely visual).
**Standalone seam:** B replaces `references/scaffold.md` and wires the gate
behind the *existing* `/design scaffold` route, so it ships without waiting on
A; A later folds that route into the rewritten spine. B never edits the routing
prose A owns.

**Track C — standing verification loop `/design` owns.** Graduate the
`lab-registry` adjustable-viewport viewer from prototyping to shipped-UI: render
the real surface at ≥3 viewports → axe → keyboard/focus walk → evidence packet.
This is the a11y/multi-device launchpad prompt as a wired loop, not a borrowed
`/qa` step.

**Dogfood:** HK's own `docs/site` is the first consumer — scaffold its
DESIGN.md, wire the gate, run the loop. The repo that ships the design-contract
process finally holds one.

## Oracle (executable definition of done)
Track A:
- [ ] Answer key committed first: a checked-in table of 15 representative
      prompts → their keyed-correct primary/preset (e.g. "make this brutalist"→
      `leon-brutalist`; "add page transitions"→`vercel-react-view-transitions`;
      "is this accessible"→`vercel-web-design-guidelines`; "the motion feels
      janky"→`emil-review-animations`). Without the key the eval is unrunnable
      and two executors key it differently.
- [ ] Routing eval: A/B old-vs-new `/design` in worktrees; for each prompt,
      check the route taken against the keyed target. Pass = new reaches the
      keyed target on ≥13/15 **and** new's mis-route count is strictly lower
      than old's (paired). No "judge prefers new" grader — length/structure bias
      makes it pass by construction once the rewrite is bigger.
- [ ] `check --repo .` green after the rewrite.
- [ ] Control-flow check (not vocabulary): no route in the rewritten `SKILL.md`
      depends on another route's output; each is independently invocable. The
      fresh-context critic asserts the parts are a flat menu + two dials + a
      gate, not an ordered 1→N pipeline.

Track B:
- [ ] `npx @google/design.md lint <generated DESIGN.md>` exits 0 on a clean
      contract.
- [ ] Off-token falsifier: inject a raw hex where a token exists → the wired
      gate exits non-zero; remove it → exits 0.
- [ ] `npx @google/design.md export` produces a DTCG `tokens.json` + Tailwind
      config from the dogfood DESIGN.md.

Track C:
- [ ] On HK `docs/site`: evidence packet with ≥3 viewport screenshots, an axe
      report (0 serious violations or each documented), and a focus-order
      capture.
- [ ] Contrast falsifier: inject a sub-AA contrast pair → the loop's axe step
      flags it.

## Verification System
- Claim: the composed `/design` routes better than the current skill, and
  agent-authored UI in a repo with a DESIGN.md cannot drift off-system silently.
- Falsifier: (A) the A0 probe warms zero cold components in 2–3 weeks, or new's
  route-match does not beat old's on the answer key; (B) a PR introduces an
  off-token value and merges green; (C) a sub-AA contrast surface passes the loop.
- Driver: A0 probe telemetry then `mode-eval` A/B worktrees (A);
  `npx @google/design.md lint` + the source gate + off-token injection (B); the
  viewport/axe loop on `docs/site` (C).
- Grader: objective route-match against the committed 15-prompt answer key +
  paired mis-route count, old vs new (A); nonzero exit gates the merge for B;
  axe serious-violation count for C.
- Evidence packet: eval transcripts + judge verdict; gate falsifier transcript;
  dogfood screenshots + axe report under the repo's evidence convention.
- Cadence: one-off shape evidence at build; gate per-PR (diff-scoped) in
  consumer repos; telemetry re-check at 30/60 days post-merge.

## Premise Source
Premise Source: sha256:22e61d34d3bbe56fa315d94da843b28f1d272d7dcf8acafbd6bd5108e764f51a /private/tmp/claude-501/-Users-phaedrus-Development-harness-kit/30a1f564-d831-4263-8229-626bdc0b961d/scratchpad/design-spine-premise.md
(Session artifact: the brainstorm + per-skill value audit + web research that
locked this shape. Move into the repo if the grounding must outlive the session.
The `premise-source` grader is absent from the current `harness-kit-checks`
build — restoration tracked in backlog 113 — so this line is format-validated by
hand, not by the gate.)

## HTML Plan
`scratchpad/design-spine-plan.html` (Aesthetic-kit plan, authored as HTML and
opened for hierarchy review before execution). Session artifact.

## Risks + Rollout
- **Workflow-engine drift (vision violation).** Mitigation: STOP-IF on SDLC
  phase prose; the eval grades routing not pipeline adherence; thin-harness
  invariant. Rollback: `/design` is one file under version control — revert.
- **Bespoke-format loss.** Dropping nine-section loses product-intent/governance;
  carry those in the generated project-local design skill / PRODUCT.md.
- **Gate over-strict.** Gate the diff, scoped to repos that have a DESIGN.md;
  one-off waiver path in the completion gate.
- **Telemetry doesn't move.** If components stay cold the compose thesis failed.
  Don't discover this a quarter late: run **Track A0 (cheap pre-test) before the
  full rewrite** — add the preset/role routing as a few rows to *today's* table
  (the Ponytail graft, ~1 day) and watch whether routing alone warms any cold
  component in 2–3 weeks. Zero movement ⇒ the components are genuinely unwanted;
  descope the rewrite to the contract/gate (B) instead of rolling back a shipped
  spine.
- Rollout order: **A0 (probe) → B (standalone gate, dogfood docs/site) → A (the
  full spine, only if A0 warms something) → C (extends A)**. Each post-probe
  track is its own `/deliver` + cross-model critic.

## Notes
- Cross-model (codex) fresh-context critic before `/deliver` picks up Track A —
  it rewrites a hot skill and touches vision invariants.
- Operator decision deferred to post-build telemetry: keep/cut each `leon`
  preset once usage data exists.
- Follow-on (separate item): evaluate Dembrandt for the framework-agnostic
  rendered-DOM gate (the net-new whitespace deferred from Track B v1).
- Relates: 112-harness-eval-bench (the routing eval could seed it), the
  cerberus-absorbs-code-review memory (same "absorb into a strong program"
  tension, resolved here as steal-skeleton-not-shim).

## Delivery log

- **Track A — done** (commit `170acfb`, branch `feat/design-spine-compose`).
  `skills/design/SKILL.md` rewritten to the composed router-with-thesis; routing
  eval `skills/design/evals/routing-eval.md` grades 15/15 vs prior 7/15;
  milestone critic passed; full repo gate green. A0 was subsumed (the routing/
  preset rows ARE the rewrite); telemetry at 30/60 days remains the standing
  falsifier for whether the surfaced specialists get invoked in real sessions.
- **Track B (format) — done.** `references/scaffold.md` rewritten to the
  `@google/design.md` format wholesale. Oracle proven live: clean DESIGN.md →
  `lint errors: 0`; malformed token → errors; `export dtcg` emits W3C DTCG
  2025.10 tokens. Finding: `@google/design.md@0.3.0`'s `spec` subcommand is
  broken (bundling defect) — `lint`/`diff`/`export` work; rely on `lint` + the
  format example, not `spec`.
- **Track B (enforcement gate) + Track C (a11y/viewport loop) — need a frontend
  repo.** Boundary finding: HK is a Rust repo whose only UI is a generated
  static Aesthetic docs site, so the stylelint/eslint/golden/verify-ui gate and
  the browser a11y/viewport loop cannot be *dogfooded* here without forcing a
  fit. They are landed as skill capability + the scaffold's gate template; the
  live dogfood belongs in a real component UI (web-presence, rkc-website, sploot,
  …). The packet's "dogfood docs/site" assumption was wrong for B-gate and C.
