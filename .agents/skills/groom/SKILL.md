---
name: groom
description: |
  Backlog management, brainstorming, architectural exploration, project bootstrapping.
  File-driven backlog via backlog.d/. Parallel investigation bench, synthesis protocol,
  themed recommendations. Product vision + technical excellence.
  Use when: backlog session, "groom", "what should we build", "rethink this",
  "biggest opportunity", "backlog", "prioritize", "tidy", "scaffold".
  Trigger: /groom, /backlog, /rethink, /moonshot, /scaffold, /tidy.
argument-hint: "[explore|rethink|moonshot|scaffold|tidy] [context]"
---

# /groom

Strategic backlog management. Parallel investigation, synthesis, themed recommendations.

## Execution Stance

You are the executive orchestrator.
- Keep synthesis, prioritization, and recommendation decisions on the lead model.
- Delegate investigation and evidence gathering to focused subagents.
- Run independent investigators in parallel by default.
- When `.spellbook/agents.yaml` exists, consult the provider roster before
  strategic fanout. Prefer at least one external provider lane for unfamiliar
  domains, major prioritization calls, or backlog items that will drive
  implementation. Record meaningful provider lanes with the repo receipt tool.

## Modes

| Mode | Intent |
|------|--------|
| **explore** (default) | Parallel investigation → synthesized themes → prioritized backlog items |
| **rethink** | Deep architectural exploration of a target system → one clear recommendation |
| **moonshot** | Explore variant — Strategist thinks from first principles, ignoring current backlog |
| **scaffold** | Project bootstrapping — quality gates, test infrastructure, CI, linting |
| **tidy** | Prune, reorder, archive completed items |

## Backlog Format: backlog.d/

```
backlog.d/
├── 001-fix-auth-rotation.md
├── 002-add-webhook-retry.md
└── _done/
    └── 000-initial-scaffold.md
```

Each file:
```markdown
# Fix auth token rotation

Priority: P0 | P1 | P2 | P3 | high | medium | low
Status: pending | ready | blocked | in-progress | done | shipped | abandoned
Estimate: S | M | L | XL

## Goal
<1 sentence — what outcome, not what mechanism>

## Non-Goals
- <what NOT to do>

## Oracle
- [ ] <mechanically verifiable criterion — prefer executable commands over prose>
- [ ] <"How will we know this is done?" — even rough oracles make items buildable>

## Notes
<context, constraints, prior art>
```

Closure markers for manual landings:
- `Closes backlog:<item-id>`
- `Ships backlog:<item-id>`

When work lands outside `/flywheel`, the landing commit should carry one of
those markers so `tidy` and `/flywheel` can detect stale active items.

When grooming Spellbook itself, shape items for downstream-repo usefulness first.
Spellbook-local work should survive only if it creates reusable primitives,
scaffolds, references, or policies; validates a proving-ground pattern meant to
transfer outward; or removes debt that materially blocks downstream adoption or
trust. See `references/backlog-doctrine.md` under "Spellbook Product Lens."

## Context Loading (all modes except tidy)

Before investigation, the orchestrator gathers baseline context:

1. Read `project.md` if it exists — store as project context for investigator prompts
2. Read `backlog.d/` — note existing items, their status, and any gaps
3. If `git-bug` is installed, read `git-bug bug status:open --format json` — note open issues
   alongside backlog.d items.
4. Read `.groom/retro/` if it exists — extract effort calibration and blocker patterns
5. Read `.groom/review-scores.ndjson` if it exists — pass summary statistics (average scores, verdict distribution, trend direction) to investigator prompts as baseline context
6. Read `exemplars.md` if it exists — note existing exemplars and pass to investigators as baseline context
7. **Cap check:** if >30 backlog items open, declare a reduction session (no new items until under cap)
8. Ask the user: "Anything on your mind? Bugs, friction, missing features?"

This takes <2 minutes. Do not block on missing artifacts — note their absence and proceed.

## Investigation Bench

Three named investigators per mode, all launched **in parallel** via the Agent tool
in a single message. See `references/investigation-bench.md` for full prompt templates.

**MANDATORY PARALLEL FANOUT.** All three investigators must launch simultaneously.
A grooming session that only runs one investigator has failed the investigation goal.

### Explore Investigators

| Investigator | Lens | Mandate | Agent Type |
|---|---|---|---|
| **Archaeologist** | Codebase health | Complexity hotspots, test gaps, coupling smells, dead code. "What's fragile?" | Explore |
| **Strategist** | Product opportunity | User journey friction, missing capabilities, competitive gaps. "What would users pay more for?" | Explore |
| **Velocity** | Effort patterns | Fix-to-feature ratio, churn hotspots, stalled work. "Where is effort not producing value?" | Explore |

For **moonshot** mode: the Strategist gets a modified prompt — "Forget the current backlog.
What's the single highest-leverage thing we're not building?"

### Rethink Investigators

| Investigator | Lens | Mandate | Agent Type |
|---|---|---|---|
| **Mapper** | System topology | Deep dependency trace, data flows, coupling points. "What breaks if you pull any thread?" | Explore |
| **Simplifier** | Radical simplicity | From-scratch rebuild perspective. "What layers can be deleted?" | Plan |
| **Scout** | External perspectives | Invokes `/research thinktank`. "What has the industry learned that this codebase hasn't?" | general-purpose |

### Investigator Output Format (shared)

Every investigator returns this exact shape:

```markdown
## [Name] Report
### Top 3 Findings
1. [finding] — Evidence: [file:line / commit / metric]. Impact: high/med/low.
2. ...
3. ...
### Strategic Theme
[One sentence: the overarching theme these findings point to]
### Single Recommendation
[One concrete action. Not a list. Not "consider." A specific thing to do.]
```

## Synthesis Protocol

After all investigators return, the **orchestrator** (you) synthesizes. Do NOT present
raw findings. Do NOT delegate synthesis to a sub-agent — this requires product judgment.

0. **Premise challenge** — Audit the request's framing before theming. Is this
   the root problem or a downstream symptom? Five-whys the stated goal. If it's
   a symptom ticket, name the underlying need and re-anchor the synthesis there.
   Converging on solutions to the wrong problem is worse than silence.
1. **Cross-reference** — Which findings appear across 2+ investigators? (highest signal)
2. **Theme extraction** — Group findings into 2-4 strategic themes. A theme is a cluster
   of findings that share a root cause or a shared solution. Not individual items — themes.
3. **Dependency map** — Do any themes depend on others? (e.g., "test infrastructure" enables
   "safe refactoring")
4. **Rank** — Order by: (impact on product vision) × (feasibility) / (effort)
5. **Exemplar check** — For the highest-impact theme, check whether reference
   implementations would inform the recommendation. If so, invoke
   `/research exemplars` for that domain. If `exemplars.md` doesn't exist yet
   and worthy exemplars are found, offer to create it. Include discovered
   exemplars in theme presentation.
6. **Present** — One theme at a time. For each: evidence from investigators, recommended
   action, rough effort (S/M/L). Ask the user: explore deeper, write backlog item, or skip?

Output format:

```markdown
## Grooming Synthesis

### Investigator Convergence
[Findings that appeared from 2+ investigators — these are highest signal]

### Theme 1: [Name]
**Evidence:** Archaeologist found X, Strategist found Y, Velocity confirms Z.
**Recommendation:** [one concrete action]
**Effort:** S/M/L
**Impact:** [why this matters for the product vision]

### Theme 2: ...

### Dependency Order
Theme A enables Theme B. Recommend executing A first.
```

## Workflow: Explore

Phase-gated. Each phase must complete before the next begins.

### 1. CONTEXT — Load baseline (see Context Loading above)
### 2. INVESTIGATE — Launch all three explore investigators in parallel
Gate: all three returned structured reports.
### 3. SYNTHESIZE — Cross-reference, theme, rank (see Synthesis Protocol)
Gate: themes extracted with evidence and recommendations. Not raw findings.
### 4. DISCUSS — Present one theme at a time. Recommend, don't list.
Gate: user decides per theme (explore deeper / write item / skip).
### 5. WRITE — Create backlog.d/ files or git-bug issues for approved themes
Shaped work (goal + oracle + sequence) → `backlog.d/` files.
Raw bugs/findings (need investigation) → `git-bug bug new -t "..." -m "..." --non-interactive`,
then label with `git-bug bug label new <id> "priority/pN" "domain/X"`.
After creating git-bug issues, sync: `git-bug push origin`
(best-effort — if push fails, issues are safe locally; log the failure but don't block).
Gate: every item has Goal + Non-Goals + Oracle.
For Spellbook backlog items, make the downstream leverage or proving-ground rationale explicit.
Use `references/agent-issue-writing.md` for issue quality standards.
### 6. PRIORITIZE — Reorder backlog.d/ by value/effort ratio

## Workflow: Rethink

### 1. CONTEXT — Load baseline + user specifies the target system
### 2. INVESTIGATE — Launch all three rethink investigators in parallel
Gate: all three returned structured reports.
### 3. SYNTHESIZE — Distill into 2-3 architectural options with honest tradeoffs
Always include "do nothing" as a viable option.
### 4. RECOMMEND — Pick one option. Argue for it. Be opinionated.
Gate: one clear recommendation with reasoning.
### 5. DISCUSS — User approves, modifies, or rejects
### 6. WRITE — One backlog.d/ item for the recommended change

## Workflow: Scaffold

Bootstrap a new project with quality gates:
1. Test infrastructure (framework, coverage gates)
2. Linting (ESLint/Biome/clippy with strict rules)
3. Type checking
4. Pre-commit hooks
5. Local CI via Dagger (`dagger init --sdk=python`, then define quality gates as Dagger Functions).
   See spellbook's own `ci/` for reference. Prefer Dagger over GitHub Actions for the inner loop.
6. CLAUDE.md with project-specific instructions
7. `backlog.d/` directory for file-driven backlog
8. `exemplars.md` — invoke `/research exemplars` for the project's domain to seed initial reference implementations

## Workflow: Tidy

1. **Backlog audit** — count open items, check against 30-item cap
2. Archive completed items (`Status: done` or `Status: shipped`) to `backlog.d/_done/`
3. Archive active items that already carry `## What Was Built` or are closed
   by current-branch commit markers (`Closes backlog:<item-id>`,
   `Ships backlog:<item-id>`)
4. Delete stale items (>30 days untouched, no longer relevant)
5. Flag items stuck in `in-progress` with no recent commits — these are abandoned, not active
6. Verify each remaining item has Goal + Oracle
7. Verify completed items have a "What Was Built" section — if not, add one from git log
8. **git-bug audit** (if installed):
   - Close stale bugs (>30 days untouched, no activity)
9. Reorder remaining by priority
10. If BACKLOG.md / icebox exists, review it once, migrate any still-relevant items, then delete the legacy file so `backlog.d/` remains the only backlog source of truth

## Gotchas

- **Accepting the ticket's framing as given** — A `/groom X` request is the user's first-draft articulation, not a locked problem statement. Before the investigation bench, five-whys the request: is this the root problem or a downstream symptom? If the user asked to groom "feature X," investigate whether X is the right lens or a proxy. Symptom-level grooming produces symptom-level backlogs.
- **Investigators returning "everything is fine"** — Red flag. Push harder. Every codebase has findings; an investigator that found none didn't look.
- **Synthesis that lists findings without theming** — That's a report, not synthesis. Group into themes before presenting.
- **Themes without recommendations** — That's a menu, not grooming. Pick one action per theme and argue for it.
- **Running one investigator and calling it done** — Mandatory parallel fanout. All three, every time.
- **Items without oracles** — If you can't write a "definition of done" with checkable criteria, the item isn't scoped. Go back and scope it.
- **Over-decomposing** — An agent-hour of work is one item, not three. Compression ratios make most splits unnecessary.
- **Backlog as graveyard** — Items >30 days old with no progress are dead. Archive or delete during tidy.
- **Backward-compatible backlog artifacts** — Keeping `BACKLOG.md` around "for history" creates split-brain planning. Migrate what matters, then delete the legacy file.

## Principles

- **Investigate before opining** — parallel investigation first, opinions after evidence
- **Theme, don't itemize** — strategic themes, not feature laundry lists
- **Recommend, don't list** — always have an opinion, argue for it
- **One theme at a time** — don't overwhelm during discussion
- **Product vision is the ranking function** — rank by impact on the user, not technical elegance
- **Every item needs an oracle** — if you can't verify done, the item isn't ready
- **File-driven** — backlog.d/ is the source of truth
