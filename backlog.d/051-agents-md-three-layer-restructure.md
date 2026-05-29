# AGENTS.md three-layer restructure + routing tables

Priority: P1
Status: ready
Estimate: M

## Goal

Reshape `harnesses/shared/AGENTS.md` into three explicitly-signposted layers so it is addressable when extending and downstream repos clearly see what their layer-4 delta owns.

- **Layer 1 — Universal SWE principles** (always true regardless of AI): Ousterhout strategic design, TDD red-green-refactor, deletion > addition, test behavior not implementation, no mocking internal collaborators, state assumptions before acting, root-cause remediation.
- **Layer 2 — Agent-specific engineering gotchas** (things that bite AI agents specifically): delegation triggers + fresh-context rationale, parallelism default, 3-edit / 2-failure / solo-grind rules, after-compaction re-read, same-model self-critique is theater, plausible ≠ correct, continuous codification.
- **Layer 3 — Routing / decision tables**: concrete trigger → action lookups for common ambiguities (Plan vs Explore vs general-purpose subagent; named agent vs ad-hoc; when to invoke philosophy bench; when to /research vs grep locally). AuggieBench's load-bearing finding — decision tables with concrete triggers beat exhortation prose.

## Non-Goals

- NOT simple line-count compression. Length may be unchanged or slightly grow. Structure is the deliverable, not brevity.
- NOT changing doctrine content. Existing principles move to their layer; nothing is rewritten.
- NOT editing downstream repo AGENTS.md files. Those are layer 4 (repo-specific) and out of scope.
- NOT building a skill registry or catalog. Layer 3 is lightweight decision tables for common ambiguities, not a discovery system.
- NOT modifying `bootstrap.sh`, the Dagger gates, or the install paths. Same file, same symlink targets.

## Oracle

- [ ] `harnesses/shared/AGENTS.md` contains three top-level sections clearly marked Layer 1 / 2 / 3 (or named equivalently — e.g., "Universal Principles" / "Agent Gotchas" / "Routing"). A reader can tell in 10 seconds which layer any principle belongs to.
- [ ] "No mocking internal collaborators" is a first-class section in Layer 1 (not a bullet buried in Testing). Promoted from cooper's PR #120 addition.
- [ ] "State assumptions before acting" is a first-class section in Layer 1 (not a bullet under Doctrine).
- [ ] Layer 3 contains ≥3 concrete decision tables: each row is (trigger condition) → (specific action: tool, subagent type, skill name). Example row: "about to do >3 tool calls of exploration with unknown scope → `Agent(subagent_type=Explore, prompt=<question>)`".
- [ ] `bash scripts/generate-index.sh` still succeeds; no orphan references.
- [ ] `dagger call check --source=.` passes all 12 gates.
- [ ] Bootstrap re-run (`bash bootstrap.sh`) succeeds and the three harness targets (`~/.claude/CLAUDE.md`, `~/.codex/AGENTS.md`, `~/.pi/AGENTS.md`) resolve via their existing symlinks.
- [ ] A downstream repo (pick one — canary or misty-step) is read through after the change and the boundary between "what comes from global" vs "what's repo-specific" is visibly clearer; note the finding in the closing commit trailer.

## Notes

### Provenance

Derived from /groom session 2026-04-23 processing agent-guidance clippings. Must-reads: "A good AGENTS.md is a model upgrade" (AuggieBench study of 2500+ repos), "How to really stop your agents from making the same mistakes" (Skillify), "Coding Models Are Doing Too Much", "Multi-Agents What's Actually Working", Compound Engineering v3.

### Framing correction

Initial research recommendation was "compress 400 → 150 lines." That target was imported from AuggieBench's measurement of repo-specific AGENTS.md files guiding agents inside a single project. Harness Kit's `harnesses/shared/AGENTS.md` is a different animal — the global default brain symlinked into every harness, always loaded, everywhere. Length isn't the problem. Signposting and layer separation are. The load-bearing win isn't fewer tokens; it's that a model reading it knows *which kind of thing* it's reading and which layer a new principle belongs to.

### Axe-sharpening exemption

Infra:feature ratio as an axe-sharpening signal doesn't apply to harness-kit — harness-kit IS the axe, so harness work is feature work. The real validation metric is downstream: does the system-wide harness produce better agent behavior in real repos without bespoke generated scaffolding? This restructure is the proving ground for that hypothesis.

### What Layer 3 routing tables actually look like

Not a schema, not a skill registry. Example entries (sketch):

| Trigger | Action |
|---|---|
| About to open research rabbit hole (>3 tool calls, unknown scope) | `Agent(subagent_type=Explore, prompt=<explicit question>)` |
| About to review code you just wrote | Dispatch critic or philosophy bench persona — fresh context, diff + acceptance criteria only |
| Non-trivial architecture decision you haven't already made | `Agent(subagent_type=Plan, prompt=<design question>)` |
| Mechanical rename across N files | Direct `sed` / `git mv` — no subagent |
| Fuzzy problem, unknown root cause | `/diagnose` or dispatch Explore subagent with explicit hypothesis |

Exact set TBD during implementation; goal is 3–5 tables covering the most common ambiguity points.

### Downstream leverage

A signposted global AGENTS.md lets downstream repos (misty-step, cerberus, bitterblossom) write their own AGENTS.md as strictly "layer 4: what makes this repo different" — repo-specific build commands, framework conventions, voice/persona. Clears the current ambiguity about which universal principles to restate: none, they're in the global layer.

### Related

- **`061` (subagent roles, not files)** lands its dispatch doctrine here:
  the executor+critic default and planner/executor/critic triad belong in
  Layer 2 (agent gotchas) + a Layer 3 routing row. Coordinate placement so the
  two tickets don't fight over the same section.
- **`081` (delegation-floor DRY extraction)** depends on this: once the shared
  AGENTS.md is the signposted single source, `081` removes the 19 per-skill
  copies of the delegation-floor paragraph and replaces them with a one-line
  pointer. This ticket builds the home; `081` redirects the duplicates to it.
- Cooper agent (PR #120, `f904439`) added "no internal mocks" to the Testing subsection. This restructure promotes it to a first-class L1 section.
- Complements the global-harness bootstrap pivot by making the always-loaded
  source material tighter.
- Candidates from same /groom session explicitly dropped or deferred: skills/audit-resolver (wrong problem), skillify 10-step audit (no downstream demand yet), acceptance-example lineage (cosmetic), clean-context philosophy-bench review (local polish only, fold in if bench work surfaces later), MCP-first integration guidance (defer until first MCP integration lands or downstream pain voiced).
