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

Shape a raw idea into something buildable. Output is a **context packet** —
the unit of specification that precedes implementation.

## Workflow

Size the shape before raising the floor. Trivial shapes are mechanical
one-file fixes, typo-level docs, or already-decided packet formatting.
Non-trivial/M+ shapes are everything with product judgment, architecture
choice, user workflow, data model, dependency, privacy, or rollout risk. The
diversity, roster, matrix, and review-bench gates apply to non-trivial/M+
shapes.

### Phase 1: Understand

Accept: raw idea, backlog.d/ item, issue ID, or observation.

## Delegation Judgment

delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use one lane to map repo constraints and another for prior-art or premise challenge.

Spawn parallel sub-agents to gather context fast: one to map the relevant
codebase area (files, patterns, constraints), another to search for prior art
(how do other projects solve this? check codebase first, then /research).
For non-trivial shapes, fill the bench instead of running a two-agent
formality: include at least one repo investigator, one product/premise critic,
one architecture critic, one implementation-risk reviewer, and one test/oracle
reviewer when the available harness can support it. Use the lead model for
synthesis; record meaningful external lanes as delegation receipts.
Synthesize their findings before proceeding.

If `exemplars.md` exists at project root, read it. Include relevant exemplar
techniques in the context packet with specific files to study during build.

### Phase 2: Product Exploration

**GATE: Do NOT write code until product direction is locked.**

1. **Investigate** — Problem space, user impact, prior art
2. **Brainstorm** — At least 6 materially different product forms,
   workflows, stacks, tools, or solution families for M+ shapes. Include the
   boring/manual path, the local-first path, the ideal/high-leverage path, and
   one path that inverts a load-bearing assumption. Describe how each fails.
   Candidates are structurally distinct only when they differ on at least one
   major axis: form factor, paradigm, placement, build/buy/rent/host,
   persistence model, dependency footprint, or trust boundary. Cull to 2-3
   finalists on the record; the kill list ships in the packet.
   **Recommend one.**
3. **Discuss** — One question at a time. Iterate until locked.
4. **Alignment pass** — For M+ shapes, ask at most five blocking
   architecture/product questions before drafting. Each question includes the
   recommended answer, the evidence behind it, and what breaks if the answer is
   wrong. If the recommendation follows an existing ADR or repo invariant,
   name the file/line; if it is an assumption, mark it as such.
5. **Draft spec** — Goal, non-goals, acceptance criteria

### Phase 3: Technical Exploration

1. **Explore** — At least 6 technical approaches or system forms for M+
   shapes. These must be structurally different, not cosmetic variants. For
   each: architecture sketch, files to modify, pattern alignment, effort,
   privacy/security posture, reversibility, operability, testability, and
   failure modes.
   **Recommend one.**

2. **Evaluate** — Build a tradeoff matrix before choosing. Score the viable
   options against outcome fit, implementation size, delete-ability, data
   ownership, privacy/security, agent-manageability, failure blast radius,
   migration path, verification cost, and long-term operating burden. Reject
   weak options explicitly; do not let them disappear. If the matrix shows
   fewer than 4 rows with at least 2 distinct values across the core axes, the
   brainstorm has not diverged enough.

3. **Validate** — For effort M or larger, spawn the adversarial design review
   bench in parallel:
   ousterhout reviews for module depth and information hiding, carmack for
   shippability and over-engineering, grug for complexity, beck for executable
   oracles, and at least one roster-backed external provider lane for an
   independent critique when available. Give each the design summary and ask
   for the production failure mode that would embarrass us, the evidence that
   would prove it, and a verdict + concerns. If any has blocking concerns,
   synthesize the evidence and revise the design before proceeding.

4. **Discuss** — No limit on rounds. Design isn't ready until user says so.

### Phase 4: Context Packet

The output of shape. This is what `/deliver` and `/implement` consume.
For M+ shaped tickets, read `references/prd-ticket-quality.md` and make the
packet a compact PRD plus technical design. The deliverable type, user, UX
enabled, technical architecture, ADR decision, alternatives, oracle, and
residual risk must be visible before implementation details.
The lead must read raw repo evidence directly. Subagent summaries can add
coverage, but they do not replace repo anchors, ADRs, tests, or source files
the builder must understand before implementation.

```markdown
# Context Packet: <title>

## PRD Summary
- User: <operator, maintainer, reviewer, customer, or agent workflow>
- Problem: <pain or opportunity>
- Why now: <priority reason>
- UX enabled: <what changes for the user>
- Deliverable type: <working code | research report | docs artifact | harness primitive | cleanup | migration | decision memo>
- Success signal: <first observable proof>

## Goal
<1 sentence — what outcome, not mechanism>

## Product Requirements
- P0: <non-negotiable user outcome or constraint>
- P1: <useful but bounded follow-on>
- Non-goals: <scope that stays out>

## Non-Goals
- <what NOT to do, even if it seems like a good idea>

## Constraints / Invariants
- <things that must remain true before, during, and after>

## Authority Order
tests > type system > code > docs > lore

## Repo Anchors
- `src/auth/middleware.ts` — current pattern to follow
- `tests/auth/` — existing coverage

## Lead Repo Read
- Source files read directly: <paths and why they matter>
- ADRs / invariants read directly: <paths or explicit none>
- Commands or rendered artifacts inspected: <exact command/path or none>
- Subagent summaries used only for: <critique, search, or none>

## Prior Art
- `src/payments/middleware.ts` — similar pattern

## Alternatives Considered
| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| <name> | <how it works> | <why it could win> | <how it fails> | <choose/reject/defer> |

Include at least 6 rows for M+ shapes.

## Tradeoff Matrix
| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| <name> | <1-5> | <1-5> | <1-5> | <1-5> | <1-5> | <1-5> | <1-5> |

Explain the scoring in prose; numbers alone are not evidence.

## Technical Design
- Chosen architecture: <selected system shape>
- Files/systems touched: <bounded surfaces>
- Data/control flow: <how behavior moves through the system>
- Build/check boundary: <what fails where>
- ADR decision: required / not required, with reason and escalation trigger.
- ADR-style invariants: <conditions, consequence if violated, file refs>
- Design X vs Y: <main rejected alternatives and why>
- Comprehension-required: <topic, only when the packet intentionally opts into `/reflect checkpoint`; omit otherwise>

## Alignment Questions
- Q1: <blocking question>
  Recommended answer: <answer>
  Evidence: <repo/source/user evidence>
  Risk if wrong: <failure mode>

Include no more than five. Use `none; assumptions accepted` only when the
shape is already locked by live repo evidence and the user request.

## Agent Readiness
- Profile source: `.harness-kit/agent-readiness.yaml` if present, otherwise `missing`.
- Stack feedback strength: compiler/type/lint/test strictness for the chosen stack.
- ADR decision: required / not required, with reason.
- Infrastructure path: CLI/API/SDK-managed setup, or explicit readiness smell.
- Gate: Dagger/local command the agent can run.
- Evidence storage: repo-local path for receipts, traces, screenshots, reports, or fixtures.
- Mock policy impact: preserved / improved / regressed.

## Delegation Evidence
- Roster providers used: <provider/model/role>
- Native subagents used: <role/purpose>
- Accepted evidence: <what changed the spec>
- Rejected evidence: <what was discarded and why>
- Waivers: <provider/model/role gaps>

No context packet ships without this synthesis artifact for non-trivial/M+
shapes. The recommendation must cite lane evidence, not only the lead's own
preference.

## Premise Source
Premise Source: sha256:<digest> <path-or-url>

The premise source is the raw or closest-available artifact that explains why
this shape exists: issue, PR, pasted text, screenshot, sanitized transcript
excerpt, or operator-supplied file. It is not acceptance evidence. Premise
source proves what problem was accepted, narrowed, or rejected; acceptance
evidence proves the implementation satisfies the oracle. For private
transcripts or chats, use `/agent-transcript` to render a scoped redacted
excerpt before referencing it.

For voice-derived or raw-transcript premise artifacts, add the metadata block
from `references/voice-transcript-metadata.md`. The block must name
`source_kind`, duplicate the `source_hash`, preserve explicit unknowns for
model/confidence/duration, name redaction status/tool, and record residual
risk. Do not retain raw audio paths in repo premise artifacts.

If no inspectable premise artifact can be safely named, use an explicit waiver
instead:

```markdown
Premise Source Waiver: <reason the raw/source artifact is unavailable or unsafe>
Residual risk: <what future implementers cannot verify because of the waiver>
```

## Exemplar Techniques
- <technique from exemplars.md> — <specific file to study during build>

## Oracle (Definition of Done)
- [ ] All existing auth tests pass
- [ ] New endpoint returns 200 with valid token
- [ ] Response time < 100ms p99
- [ ] Dogfood artifact captured for runtime-visible changes, or waiver explains why no running surface exists

## Deliverable
- Output: <exact artifact or behavior left behind>
- Acceptance oracle: <command, rendered artifact, report shape, or decision record>
- Evidence artifacts: <receipts, screenshots, fixtures, hashes, traces, or links>
- Residual risk: <what remains unproven>

## Observability Plan
- Changed behavior to watch:
- Named signal or evidence surface:
- Instrumentation debt if no signal exists:

When the oracle depends on a fixture, contract, golden file, screenshot,
Gherkin feature, transcript, or other acceptance artifact, include its
artifact hash in the packet. Prefer:

```sh
shasum -a 256 <artifact-path>
```

Name the source and hash near the oracle, for example:
`Oracle / acceptance artifact hash: sha256:<digest> <artifact-path>`.
If implementation intentionally changes that source, the handoff must carry
`Contract-change acknowledgment: <why the acceptance contract changed>`.

When an acceptance artifact exists, include this block:

```markdown
## Acceptance Evidence
- Acceptance source: fixture, contract, golden file, transcript, screenshot, Gherkin feature, or executable oracle path.
- Evidence that proves it: command output, mutation result, QA artifact, or trace proving the acceptance path is connected.
- Exact command/path/route exercised: command, URL, route, file path, or tool call that exercised the acceptance source.
- Oracle / acceptance artifact hash: sha256 digest plus artifact path for each source the oracle depends on.
- Contract-change acknowledgment: reason for an intentional acceptance-contract change, or statement that no contract changed.
- Residual risk: unverified path, accepted survivor, or none with reason.
```

For high-risk or ambiguity-heavy changes, require the formal-spec ladder when
two or more are true:

- core business rules, money/security/auth behavior, data migrations,
  permissions, or cross-service contracts change;
- user-facing behavior is best expressed as examples, scenarios, CLI
  transcripts, API fixtures, or golden files;
- a regression would be expensive to detect manually after merge;
- changed code has high complexity, low coverage, or a known weak oracle;
- implementation needs multiple agents or long-running milestones where
  context drift is likely.

When the ladder triggers, the packet must include this block before
implementation starts:

```markdown
## Formal Spec
- Formal Spec Required: yes (cite the trigger criteria)
- Informal spec: plain-language behavior and business rule.
- Formal examples: Gherkin scenarios, fixture paths, CLI transcripts, API examples, or golden files.
- Acceptance oracle: executable command or route that must fail before implementation and pass after.
- Hardening budget: named hardening modes and bounded time/scope cap.
- Waiver path: who/what may waive acceptance-first, property, mutation, or acceptance-mutation evidence and how residual risk is recorded.
```

If the ladder does not trigger, omit the block or state `Formal Spec Required:
no` in the risk notes with the reason. Do not make Gherkin, property testing,
or mutation testing the default path for low-risk work.

## Implementation Sequence
1. <first chunk>
2. <second chunk>

## Risk + Rollout
- <how it could fail, how to undo it>
```

### Phase 5: Visual HTML Handoff

For non-trivial shaped work that explicitly includes visual documentation,
generated-image documentation, public docs, workflow-diagram, or reader-facing
handoff requirements, render a static HTML handoff after the context packet:

```sh
cargo run --locked -p harness-kit-checks -- shape-render <packet-or-backlog.md> \
  --output .evidence/shape-<id>/context-packet.html
```

Use `/design` on the rendered artifact before closeout. Give `/design` the file
path, audience, intent, source packet, and the PRD hierarchy to protect. The
critique must cover hierarchy, typography, density, table/code fit, mobile
layout, content fidelity, and residual visual risk. For other M+ shapes, this
handoff is optional; do not rationalize every packet into visual docs.

Open the generated HTML in a browser before closeout. Inspect desktop and
mobile-width layout; verify long tables, code/path text, implementation steps,
and the review gate are visible and non-overlapping. Record the exact
file URL/path inspected and the evidence in the context packet's Acceptance
Evidence or closeout. The HTML handoff is reviewer-facing documentation, not
the source of truth; the Markdown packet/backlog item remains authoritative.
If no browser is available, record an explicit waiver plus the strongest
available static/render check; do not claim design verification.

For CLI work, load `references/cli-design.md` and include its `## CLI Surface`
block in the context packet.

If you can't write an oracle, the goal isn't clear enough. Go back to Phase 2.

## Gotchas

- **Premise unchallenged:** A shape request accepts the stated framing by default. Before Phase 2, five-whys the goal. If the request says "feature X," name the underlying user outcome — the best path to it may not be X. A solid shape of the wrong problem is the failure mode this skill exists to prevent, not cause.
- **Alternatives-in-name-only:** Three "options" that are the same idea in three outfits is one option. Real divergence means structurally distinct approaches — typically one minimal-viable, one ideal, and one that inverts a load-bearing assumption. If you can't articulate how each would fail differently, go back. For M+ effort, the philosophy bench (ousterhout/carmack/grug) is persona diversity, not foundation diversity — also use `/research` and the shared Roster contract for genuinely heterogeneous signal.
- **Vague oracles:** "It should work" is not an oracle. "These 3 tests pass and this endpoint returns 200" is. See `references/executable-oracles.md`.
- **Checkbox oracles:** Prose checklists drift. Write oracles as commands that return pass/fail, not prose that requires interpretation.
- **Buried deliverable:** If a reader must reach the implementation sequence to
  know whether the output is code, research, docs, or a decision, the packet is
  not ready.
- **Ready-but-vague PRD:** `Status: ready` cannot contain unresolved target
  language like "preferably", "confirm later", or "pick during implementation"
  for a load-bearing scope choice.
- **Speccing after building:** A context packet written after implementation is documentation, not specification. Spec first.
- **50 repo anchors:** If everything is an anchor, nothing is. Pick 3-10 files whose patterns MUST be followed.
- **Skipping non-goals:** Agents drift toward scope expansion. Non-goals are load-bearing constraints. Write them.
- **Over-speccing implementation details:** Specify WHAT and WHY. Let the builder figure out HOW. Detailed pseudocode cascades errors.
- **Editing shape docs without ripple check:** Files with `shaping: true` frontmatter are live specs. Before editing, check: do affordance tables need updating? Does this change ripple to other work streams or context packets? Edit the doc, then trace the consequences.

## Principles

- Minimize touch points (fewer files = less risk)
- Design for deletion (easy to remove later)
- Favor existing patterns over novel ones
- YAGNI ruthlessly
- Recommend, don't just list options
- One question at a time
