# Public static docs companion

Priority: P1
Status: pending
Estimate: L

## Goal

Create a static HTML documentation companion for Spellbook that makes the
catalog usable by people who do not already speak AI-ops or harness
engineering. The unit is a docs site plus build-time catalog checks: it
indexes every local skill and agent, explains what Spellbook is from zero
to one, and walks through concrete workflow examples that clients can read
without opening the repo.

This is public-facing packaging for Spellbook as implementation
infrastructure, not a replacement for the repo or a Brandt-specific handoff
package.

## Problem Challenge

The first-order request sounds like "make docs," but the underlying
failure mode is buyer and operator translation. Spellbook already has a
rich catalog, gates, doctrine, and workflow loops; the missing layer is a
clean explanation surface that turns those internals into:

1. a plain-language model of AI-agent harness engineering,
2. a navigable index of primitives,
3. a few opinionated example workflows, and
4. credible client-facing proof that this is how production AI operations
   are made inspectable, governable, and repeatable.

Alphabetical reference pages alone do not solve this. The site must teach
the system before it catalogs the parts.

## Research Notes

External research converged on four constraints:

1. Agent systems should start simple and composable before growing into
   autonomous orchestration. Anthropic's effective-agents guidance favors
   workflow clarity and incremental complexity.
2. Evals and traces are release controls, not appendix material. OpenAI's
   eval and trace-grading docs, plus OpenTelemetry GenAI guidance, support
   making observability and evaluation visible in the main user journey.
3. Documentation should separate tutorials, how-to guides, reference, and
   explanation. Diataxis is the right information-architecture baseline.
4. Governance has to be first-class for client trust. NIST AI RMF, OWASP
   GenAI, and MCP security guidance all point toward clear trust
   boundaries, least privilege, approvals, traceability, and operational
   controls.

Daybook positioning adds the buyer-facing frame: the strongest wedge is
"AI operations implementation for teams stuck between demo and deployment."
The public language should emphasize mapping the workflow, building the
agent, adding evals, permissions, monitoring, and human approval points,
then handing over something a team can actually operate.

Sources:
- https://www.anthropic.com/research/building-effective-agents/
- https://platform.openai.com/docs/guides/evaluation-best-practices
- https://platform.openai.com/docs/guides/trace-grading
- https://www.nist.gov/itl/ai-risk-management-framework
- https://genai.owasp.org/
- https://modelcontextprotocol.io/docs/tutorials/security/security_best_practices
- https://opentelemetry.io/blog/2026/genai-observability/
- https://diataxis.fr/
- Daybook: `misty-step/growth/ai-ops-consulting-experiment-playbook-2026-05-18.md`

## Solution Divergence

### Option A: Static catalog from `index.yaml`

Generate a searchable static catalog directly from `index.yaml`, with
per-skill and per-agent pages linking back to source files.

Failure mode: accurate reference that still feels opaque to newcomers.

### Option B: Narrative docs site with generated catalog

Build a docs-first static site with a zero-to-one learning path, workflow
walkthroughs, governance pages, and a generated catalog section backed by
repo sources.

Failure mode: requires more content discipline, but solves the actual
audience problem.

### Option C: Interactive product microsite

Create a polished marketing-style microsite with animations, demos, and
manual case-study pages.

Failure mode: drifts from source of truth, becomes a sales site instead of
the durable documentation companion.

Chosen: Option B. It teaches first, indexes second, and keeps the catalog
source-derived so Spellbook does not inherit a second manual content system.

## Design

1. Add a static docs app under `docs/` or `site/` using a minimal generator
   that can be built locally and hosted as static HTML. Prefer the smallest
   stack that supports generated pages, routing, search, and clean visual
   polish without a server.
2. Add a catalog extraction step that reads canonical repo sources:
   `skills/*/SKILL.md`, `agents/*.md`, `index.yaml`,
   `harnesses/shared/AGENTS.md`, `ci/src/spellbook_ci/main.py`,
   `.githooks/*`, `bootstrap.sh`, and open `backlog.d` items.
3. Render generated reference pages for every skill and agent:
   purpose, trigger, when to use, inputs/outputs where present, related
   workflows, source path, and expected evidence.
4. Hand-author the teaching spine:
   - Spellbook in 60 seconds
   - What is a harness?
   - Skills, agents, gates, traces, oracles, and workflows
   - From messy workflow to governed AI operation
   - Inner loop: `/shape` -> `/deliver` -> `/ci` -> `/qa`
   - Outer loop: `/flywheel` -> `/monitor` -> `/reflect`
   - Governance: evals, approvals, observability, safety boundaries
   - Client handoff: what a team gets and how they operate it
5. Include example workflows as first-class pages, not buried snippets:
   - "Choose the first workflow worth automating"
   - "Turn a raw idea into a shaped ticket"
   - "Ship a repo change with review and gates"
   - "Install the system-wide harness with `bootstrap.sh`"
   - "Audit an AI workflow for evals, monitoring, and approval points"
6. Add visual polish appropriate for a public technical docs site:
   restrained layout, excellent typography, high-contrast diagrams,
   simple interactive filters, and workflow cards. Avoid marketing-page
   hero bloat; the first screen should immediately explain Spellbook.
7. Include an AI/agent-readable surface such as `llms.txt` or a compact
   generated manifest so coding agents can consume the docs without
   scraping the whole site.

## Cross-Harness

The docs site is not a harness-native runtime feature. It documents the
filesystem-level Spellbook catalog and must explain Claude, Codex, and Pi
behavior from one source. Generated pages should explicitly show when a
primitive is universal versus when a harness-specific setting affects how
it is installed or invoked.

No site feature may rely on Claude-only, Codex-only, or Pi-only metadata
as the primary source of truth. Harness-specific files are explanatory
inputs; `skills/`, `agents/`, shared doctrine, and generated catalog data
remain canonical.

## Oracle

- [ ] A local command builds the static site from a clean checkout and
      writes only deterministic output.
- [ ] Every `skills/*/SKILL.md` and `agents/*.md` has exactly one generated
      reference page with a source link.
- [ ] The docs build fails when a skill or agent exists without a generated
      page, or when a generated page points at a missing source file.
- [ ] The site includes at least five workflow walkthroughs listed in the
      design section.
- [ ] A non-specialist first-run path explains Spellbook, harnesses,
      skills, agents, and gates without requiring prior AI-ops vocabulary.
- [ ] The site includes a governance section covering evals, traces,
      approvals, least-privilege tool boundaries, and rollback/escalation.
- [ ] The catalog exposes client-safe positioning without copying private
      Daybook prose verbatim.
- [ ] `dagger call check --source=.` passes after adding the site and any
      new docs checks.

## Non-Goals

- No hosted SaaS, dashboard, database, or authenticated docs product.
- No manual re-entry of the full skill catalog.
- No public exposure of private client notes, Daybook paths, raw research
  logs, or provider transcripts.
- No Brandt-specific package in this ticket.
- No new harness runtime mechanism; this is documentation plus build-time
  verification.

## Related

- Supports: `backlog.d/051-agents-md-three-layer-restructure.md`
- Related: `backlog.d/056-agent-session-trace-lifecycle.md`
- Related: `backlog.d/058-work-ledger-mission-control.md`
