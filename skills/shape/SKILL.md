---
name: shape
description: |
  Shape a raw idea into something buildable. Product + technical exploration.
  Spec, design, critique, plan. Output is a context packet.
  Use when: "shape this", "write a spec", "design this feature",
  "plan this", "spec out", "context packet", "technical design".
  Trigger: /shape, /spec, /plan, /cp.
argument-hint: "[idea|issue|backlog-item] [--spec-only] [--design-only]"
---

# /shape

Turn a raw idea into a **context packet** — the unit of specification that
`/deliver` consumes. Spec before code, always.

## Contract

A packet is ready when all of these are true. How you get there is your
judgment; size the effort to the stakes — a one-file fix needs a paragraph,
an architecture choice needs the full treatment.

- **Premise challenged.** The request is a first-draft framing, not a locked
  problem. Name the underlying user outcome before designing; the best path
  may not be the feature asked for. Shaping the wrong problem well is the
  failure this skill exists to prevent.
- **Grounded in live repo evidence.** You read the load-bearing source files,
  tests, and ADRs yourself. Subagent summaries add coverage; they do not
  replace direct reads of what the builder must understand.
- **Learnings checked before structuring.** Grep prior repo-technical lessons
  before locking the shape:
  `rg -n --glob '*.md' '^(title|tags|applies_when):|<product-term>|<failure-mode-term>' docs/solutions`.
  Open likely matches and state applies / does-not-apply in the packet.
- **Vision-aware when strategic.** For product direction, positioning,
  long-lived workflow, or project-identity choices, read root `VISION.md` when
  present. If the shape depends on a missing or stale north star, route to
  `/vision` or carry an explicit waiver.
- **Alternatives genuinely explored.** Real alternatives fail differently —
  include the boring/manual path and one that inverts a load-bearing
  assumption. Same idea in three outfits is one option. To break out of that,
  `nous-creative-ideation` is a routed library of named methods (TRIZ for
  parameter conflicts, premortem-and-inversion, first-principles) that
  generate options which actually fail differently. Kill the losers on
  the record and **recommend one**; a menu is not a shape. For automation or
  refactor-heavy ideas, apply `harnesses/shared/references/delete-first.md`
  and the synced Ponytail skill (`skills/.external/dietrich-ponytail/SKILL.md`)
  before choosing the build path; the lazy viable path must be a real
  alternative, not a throwaway paragraph.
- **Perceptual criteria prototyped.** When acceptance is taste — design,
  copy, feel, layout — prose alternatives are the wrong artifact; the
  operator can't evaluate what they haven't seen. Route through `/design`
  prototype variations, let the operator react, then lock what the reaction
  taught. When the operator can't articulate the want at all, hunt a
  reference artifact instead — source code beats docs beats screenshots —
  and anchor the packet to it (Repo Anchors carries it).
- **Scope is fenced.** Goal (outcome, not mechanism), explicit non-goals,
  and invariants that must survive the change.
- **Oracle is executable.** "It should work" is not an oracle; "these
  commands pass, this route returns X" is. If you can't write the oracle,
  the goal isn't clear yet — go back. See `references/executable-oracles.md`.
  When you cannot define the measuring stick, delegating its INVENTION is a
  legal oracle ("figure out how to measure X, then hit it") — and the
  builder never grades its own work: acceptance is a fresh context pointed
  at the real output, told to prove it fails
  (`harnesses/shared/references/prompting-frontier.md`).
- **Verification harness named.** The packet states which live-verification
  harness will prove the work (the repo's one-command evidence loop) — and
  when none exists, the packet's first milestone is building it
  (verification system first, shared AGENTS.md Layer 1), not the feature.
  Load `harnesses/shared/references/verification-system-first.md` for
  evals, benchmarks, QA paths, performance claims, agent-behavior claims, or
  any surface whose proof loop is not already obvious.
- **Deliverable visible up front.** Code, research, docs, or decision — a
  reader should not have to reach the implementation sequence to find out.
- **Executable by a stranger.** The packet is consumed without your
  context — by a remote lane, a different model, or you next month. Include
  current-state excerpts where the code would surprise, one exemplar file
  for conventions, commands you actually ran, and stop conditions: the
  surprises that should halt execution and come back rather than be
  improvised around.
- **"Works" has product dimensions.** For public API, CLI, UI, performance,
  compatibility, migration, or operator-workflow packets, load
  `harnesses/shared/references/works-critique.md` and include the likely
  review focus.
- **Loops cross the Mode A/Mode B boundary.** For recurring or unattended
  workflow ideas, load `harnesses/shared/references/loop-readiness.md` and
  shape a handoff instead of a Harness Kit scheduler.
- **Premise source named.** The packet cites the artifact that explains why
  this shape exists (`Premise Source: sha256:<digest> <path-or-url>`) or
  carries an explicit waiver with residual risk. Voice/raw-transcript
  premises take the metadata block from
  `references/voice-transcript-metadata.md`; never store raw audio paths.
  This is grader-enforced (see Verification).
- **HTML plan authored.** For non-trivial or contestable shapes, write the
  plan directly as a `.html` artifact from `templates/html-plan.html`,
  publish it to the Sanctum shelf (artifact skill; slug = the Powder card
  id), and attach the URL to the card (link or comment) so the plan is
  durably ticket-coupled. Never auto-open it in a browser (operator ruling
  2026-07-04: auto-opened planning artifacts are a distraction) — inspect
  the rendered hierarchy yourself via the published page; the operator
  opens it from Powder on his clock.
  The hero is the complete work contract: target outcome,
  chosen design, why it wins, proof surface, and stop conditions. Order the
  body by decision volatility: lead with what the operator is most likely to
  tweak — data model, public interfaces, UX flows — and bury the mechanical
  work at the bottom. The plan doubles as an unknown detector; put the
  contestable decisions where the reviewer's attention lands first. Below it,
  include the support needed for a stranger to execute without chat context:
  current state, change shape, repo anchors, alternatives and tradeoffs,
  acceptance, verification, communication cadence, risks, and adversarial
  review focus. Use layout as thinking: comparison tables, phase lanes, risk
  grids, diagrams, callouts, and links to repo anchors. Generate diagram images
  only for complex or contested plans where a labeled architecture / sequence /
  system map carries information HTML, mermaid, or ASCII cannot; keep mermaid
  for precise call-graphs (`harnesses/shared/references/image-generation.md`).
  Prefer the Misty Step aesthetic kit when the artifact is visual and local
  review can load it; keep any local CSS as thin plan-specific glue. This is not Markdown exported to
  HTML; the HTML is the planning medium. Skip only for trivial shapes,
  no-browser environments, or explicit operator waiver.

Interrogate before you design, and lock product direction before technical
design. For any substantial or contestable shape the default is a
`grill-me`-style interview: load
`harnesses/shared/references/interrogate-first.md` and walk the operator down
the decision tree one question at a time, each with your recommended answer and
what breaks if it's wrong, until the load-bearing product and architecture
choices are pinned. Explore the repo, vision, and commands to resolve what you
can; only the operator settles genuine product direction. The guard cuts both
ways: don't manufacture questions for a shape the evidence already locks — that
is railroading, not rigor.

## Packet Skeleton

Sections carry weight or they don't appear. For substantial work, follow the
PRD shape in `references/prd-ticket-quality.md`; for CLI surfaces, include
the block from `references/cli-design.md`.

```markdown
# Context Packet: <title>

## Goal            — one sentence, outcome not mechanism
## Non-Goals       — scope that stays out, even if tempting
## Constraints     — invariants that must remain true
## Repo Anchors    — the 3–10 files whose patterns must be followed
## Alternatives    — what was considered, how each fails, verdicts
## Design          — chosen shape, surfaces touched, data/control flow,
                     rejected alternatives and why, ADR decision if any
## Oracle          — executable definition of done
## Premise Source  — sha256 + artifact, or explicit waiver
## HTML Plan       — Sanctum shelf URL attached to the Powder card, or explicit waiver
## Risks + Rollout — how it fails, how to undo it
```

When the oracle depends on an acceptance artifact (fixture, golden file,
contract, screenshot), pin it: `sha256:<digest> <path>`. If implementation
intentionally changes that artifact, the handoff carries a contract-change
acknowledgment. High-risk work (money/auth/migrations, expensive-to-detect
regressions) earns formal examples and a test-strength budget — note it in
the packet for `/deliver` and `/qa` rather than inflating the packet itself.

## Delegation Judgment

Delegate on judgment per the shared Roster contract: native subagents by
default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: one lane to map repo constraints, one for prior art or
premise challenge; fresh-context critique of the draft packet when the
design is contestable.

## Critique

Your own design read is not a review. When the design is contestable, hand
the draft packet — artifact only, not your reasoning trail — to adversarial
fresh-context critique, preferably a different model family, and ask for the
production failure that would embarrass us. Lens prompts live in
`references/critique-personas.md`. Skip for trivial shapes.

## Gotchas

- **Over-speccing HOW.** Specify WHAT and WHY; let the builder own the how.
  Detailed pseudocode cascades its own bugs into implementation.
- **Speccing after building.** That's documentation, not specification.
- **Ready-but-vague.** A packet is not ready while a load-bearing choice
  still says "preferably" or "decide during implementation".
- **50 repo anchors.** If everything is an anchor, nothing is.
- **HTML as decoration.** A plan page that is just prose in a browser missed
  the point. Use spatial structure to show sequence, tradeoffs, risk, proof,
  communication, and critic focus at a glance. A plan page that needs the
  chat transcript to be understood is not ready for execution.
- **Editing live shape docs without ripple check.** Files marked
  `shaping: true` feed other streams; trace consequences after editing.

## Verification

For non-trivial packets, include the verification-system block from
`harnesses/shared/references/verification-system-first.md`: claim, falsifier,
driver, grader, evidence packet, cadence, and gaps/waiver.

Premise-source discipline is enforced by the Rust grader:

```sh
cargo run --locked -p harness-kit-checks -- premise-source validate <packet>
cargo run --locked -p harness-kit-checks -- premise-source self-test
```

HTML plan artifact:

```sh
cp skills/shape/templates/html-plan.html /tmp/<card-id>-plan.html
# fill it, then publish + attach (never `open` it):
python3 ~/Development/harness-kit/skills/artifact/scripts/artifact_create.py \
  --title "<title>" --slug <card-id> --tag "Plan" --html-file /tmp/<card-id>-plan.html
# attach the printed URL to the Powder card (add_link MCP tool or a comment)
curl -s -o /dev/null -w '%{http_code}' <printed-url>   # expect 200
```

Fill it as HTML, publish, and revise after reading the rendered page. Inspect
that the hero states the chosen design and proof path, the alternatives table
names tradeoffs, the verification section names exact commands/artifacts, and
the review section gives a useful artifact-only critic prompt. Do not auto-open
a browser; the shelf URL attached to the card is the deliverable. Keep source
links, commands, and oracles exact; use the rendered view to make the plan
clearer, not less precise.
