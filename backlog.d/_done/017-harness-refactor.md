# Refactor /harness — extract scaffolds, router pattern, add audit mode

Priority: high
Status: done
Estimate: L

## Problem

/harness is over-scoped: 7 modes, 286 lines, 7 reference files spanning three
unrelated domains (skill lifecycle, project scaffolding, architectural design).
Research shows this violates every principle we want /harness itself to teach:

- One skill = one domain. /harness mixes skill lifecycle, scaffold generation,
  and harness engineering.
- Scaffold-qa and scaffold-demo are domain-specific workflows that belong with
  /qa and /demo respectively — not in infrastructure.
- 7 modes with inline content exceeds the mode-bloat threshold (>4 inline).
- The skill that teaches progressive disclosure doesn't practice it.

## Goal

1. Extract scaffold modes to their domain skills (/qa, /demo)
2. Refactor /harness to the internal router + references pattern (proven by
   /investigate and /settle)
3. Add an audit mode for skill health assessment
4. Codify research-backed skill design principles into the skill itself

## Non-Goals

- Rewriting /qa or /demo beyond adding scaffold capability
- Changing the scaffold workflow itself (investigate → design → deliver is solid)
- Modifying bootstrap.sh or harness installation

---

## Phase 1: Extract scaffold modes to domain skills

### What moves where

#### To skills/qa/

| Source | Destination | Action |
|--------|------------|--------|
| harness/references/scaffold-qa.md | qa/references/scaffold.md | Move + rename |
| harness/references/browser-tools.md | qa/references/browser-tools.md | Move (primary consumer is QA) |
| harness/references/evidence-capture.md | qa/references/evidence-capture.md | Move (primary consumer is QA) |

#### To skills/demo/

| Source | Destination | Action |
|--------|------------|--------|
| harness/references/scaffold-demo.md | demo/references/scaffold.md | Move + rename |
| harness/references/pr-evidence-upload.md | demo/references/pr-evidence-upload.md | Move (primary consumer is /demo) |
| harness/references/remotion.md | demo/references/remotion.md | Move (primary consumer is /demo) |
| harness/references/tts-narration.md | demo/references/tts-narration.md | Move (primary consumer is /demo) |

#### Stays in skills/harness/

Nothing. All 7 current reference files move out. New reference files are created
in Phase 2. The harness/references/ directory is rebuilt from scratch with
mode-specific content.

### Changes to skills/qa/SKILL.md

Replace the current thin redirect (45 lines) with a skill that owns its scaffold
workflow. The new SKILL.md has two modes: **run** (do QA) and **scaffold**
(generate project-local QA skill).

```markdown
---
name: qa
description: |
  Browser-based QA, exploratory testing, evidence capture, and bug reporting.
  Drive running applications and verify they work — not just that tests pass.
  Use when: "run QA", "test this", "verify the feature", "exploratory test",
  "check the app", "QA this PR", "capture evidence", "manual testing",
  "scaffold qa", "generate qa skill".
  Trigger: /qa.
argument-hint: "[url|route|feature|scaffold]"
---

# /qa

QA effectiveness depends on project-specific context. This skill either runs
QA (if a project-local skill exists) or scaffolds one.

## Routing

| Intent | Action |
|--------|--------|
| "scaffold qa", "generate qa skill" | Read `references/scaffold.md` and follow it |
| Run QA (project-local skill exists) | Defer to project-local `.claude/skills/qa/SKILL.md` |
| Quick one-off QA (no scaffold) | Use the quick protocol below |

If first argument is "scaffold" → read `references/scaffold.md`.

## Quick One-Off QA (no scaffold)

If you need to verify something right now without scaffolding:

1. Start the dev server
2. Navigate to affected routes
3. Verify: happy path, edge cases, console errors, network failures
4. Capture evidence to `/tmp/qa-{slug}/`
5. Classify findings: P0 (blocks ship), P1 (fix before merge), P2 (log)

For browser tool selection, read `references/browser-tools.md`.
For evidence capture patterns, read `references/evidence-capture.md`.

## Gotchas

- **"Tests pass" is not QA.** Tests verify code paths. QA verifies user experience.
- **This fallback is intentionally thin.** Generic QA instructions can't encode
  your app's routes, personas, or failure modes. Scaffold for real coverage.
- **Autopilot expects a scaffolded skill.** If `/autopilot` invokes `/qa` and
  hits this redirect, scaffold first: `/qa scaffold`.
```

### Changes to skills/demo/SKILL.md

Same pattern — add scaffold routing. The new SKILL.md has two modes: **run**
(generate demo artifacts) and **scaffold** (generate project-local demo skill).

```markdown
---
name: demo
description: |
  Generate demo artifacts: screenshots, GIF walkthroughs, video recordings,
  polished launch videos with narration and music. From raw evidence to
  shipped media. Also handles PR evidence upload via draft releases.
  Use when: "make a demo", "generate demo", "record walkthrough", "launch video",
  "PR evidence", "upload screenshots", "demo artifacts", "make a video",
  "demo this feature", "create a walkthrough", "scaffold demo",
  "generate demo skill".
  Trigger: /demo.
argument-hint: "[evidence-dir|feature|scaffold] [--format gif|video|launch] [upload]"
---

# /demo

Demo effectiveness depends on project-specific context. This skill either
generates demo artifacts or scaffolds a project-local demo skill.

## Routing

| Intent | Action |
|--------|--------|
| "scaffold demo", "generate demo skill" | Read `references/scaffold.md` and follow it |
| Run demo (project-local skill exists) | Defer to project-local `.claude/skills/demo/SKILL.md` |
| Quick one-off demo (no scaffold) | Use the quick protocol below |

If first argument is "scaffold" → read `references/scaffold.md`.

## Quick One-Off Demo (no scaffold)

If you need to capture evidence right now without scaffolding:

### Workflow: Planner -> Implementer -> Critic

Each phase is a **separate subagent**. The critic must inspect artifacts cold
(no context from the implementer) to prevent self-grading.

1. **Plan:** Identify the feature delta, build a shot list, choose capture method
2. **Capture:** Execute the plan — every "after" has a paired "before"
3. **Critique:** Fresh agent validates source, pairing, text delta, coverage, quality
4. **Upload:** `gh release create qa-evidence-pr-{N} --draft` + PR comment

### FFmpeg quick reference

```bash
# WebM -> GIF (800px, 8fps, 128 colors)
ffmpeg -y -i input.webm \
  -vf "fps=8,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
  -loop 0 output.gif
```

For detailed capture patterns, read `skills/qa/references/evidence-capture.md`.
For PR evidence upload, read `references/pr-evidence-upload.md`.
For Remotion video composition, read `references/remotion.md`.
For TTS narration, read `references/tts-narration.md`.

## Gotchas

- **Default-state evidence proves nothing.** Show the delta, not just defaults.
- **Self-grading is worthless.** The critic subagent inspects artifacts cold.
- **This fallback is intentionally thin.** Generic demo instructions can't encode
  your app's features, capture methods, or upload targets. Scaffold for quality.
- **Autopilot expects a scaffolded skill.** If `/autopilot` invokes `/demo` and
  hits this redirect, scaffold first: `/demo scaffold`.
```

### Cross-references to update

After moving reference files, update these references in other skills:

| File | Old reference | New reference |
|------|--------------|---------------|
| skills/settle/SKILL.md:149 | `skills/harness/references/pr-evidence-upload.md` | `skills/demo/references/pr-evidence-upload.md` |
| skills/autopilot/SKILL.md:95 | `Run /harness scaffold qa first` | `Run /qa scaffold first` |
| skills/autopilot/SKILL.md:109 | `Run /harness scaffold demo first` | `Run /demo scaffold first` |
| skills/autopilot/references/qa-and-demo.md:4-5 | `/harness scaffold qa` and `/harness scaffold demo` | `/qa scaffold` and `/demo scaffold` |
| skills/autopilot/references/qa-and-demo.md:15 | `redirect to /harness scaffold` | `redirect to /qa scaffold or /demo scaffold` |
| skills/autopilot/references/qa-and-demo.md:18-23 | `skills/harness/references/` path block | Update to `skills/qa/references/` and `skills/demo/references/` |

Total: 4 files with stale references (settle, autopilot SKILL.md, autopilot qa-and-demo.md, plus the qa/demo SKILL.md files being rewritten).

### Acceptance criteria

- [ ] `/qa scaffold` triggers scaffold workflow (reads references/scaffold.md)
- [ ] `/demo scaffold` triggers scaffold workflow (reads references/scaffold.md)
- [ ] `/harness scaffold qa` errors with: "Scaffold moved to /qa. Run `/qa scaffold`."
- [ ] All reference file reads in /qa and /demo resolve correctly
- [ ] No broken references in settle or other skills
- [ ] `git grep 'harness/references/scaffold-' skills/` returns zero results

---

## Phase 2: Router refactor of /harness

### New reference files to create

| File | Content source | Approximate lines |
|------|---------------|-------------------|
| `harness/references/mode-create.md` | Extract from SKILL.md lines 30-98 (Creating a Skill) | ~100 |
| `harness/references/mode-lint.md` | Extract from SKILL.md lines 114-127 (Linting) + new mode-bloat gate | ~60 |
| `harness/references/mode-eval.md` | Extract from SKILL.md lines 100-112 (Evaluating) | ~40 |
| `harness/references/mode-convert.md` | Extract from SKILL.md lines 129-144 (Converting) | ~40 |
| `harness/references/mode-sync.md` | Extract from SKILL.md lines 207-215 (Sync) | ~30 |
| `harness/references/mode-engineer.md` | Extract from SKILL.md lines 146-205 (Engineering) | ~80 |
| `harness/references/mode-audit.md` | New (Phase 3) | ~80 |

### The new /harness SKILL.md (complete, ready to write)

```markdown
---
name: harness
description: |
  Build, maintain, evaluate, and optimize the agent harness — skills, agents,
  hooks, CLAUDE.md, AGENTS.md, and enforcement infrastructure.
  Use when: "create a skill", "update skill", "improve the harness",
  "sync skills", "eval skill", "lint skill", "tune the harness",
  "add skill", "remove skill", "convert agent to skill",
  "audit skills", "skill health", "unused skills".
  Trigger: /harness, /focus, /skill, /primitive.
argument-hint: "[create|eval|lint|convert|sync|engineer|audit] [target]"
---

# /harness

Build and maintain the infrastructure that makes agents effective.

## Routing

| Intent | Reference |
|--------|-----------|
| Create a new skill or agent | `references/mode-create.md` |
| Evaluate a skill (baseline comparison) | `references/mode-eval.md` |
| Lint/validate a skill against quality gates | `references/mode-lint.md` |
| Convert agent ↔ skill | `references/mode-convert.md` |
| Sync primitives from harness-kit to project | `references/mode-sync.md` |
| Design harness improvements | `references/mode-engineer.md` |
| Audit skill health and usage | `references/mode-audit.md` |

If first argument matches a mode name, read the corresponding reference.
If no argument, ask: "What do you want to do? (create, eval, lint, convert, sync, engineer, audit)"

**Scaffold moved.** If user says "scaffold qa" or "scaffold demo", redirect:
"Scaffold is now owned by the domain skill. Run `/qa scaffold` or `/demo scaffold`."

## Skill Design Principles

These principles govern every mode. They are the quality standard for skills
this harness creates, evaluates, and lints.

### 1. One skill = one domain, 1-3 workflows

A skill that spans multiple domains is a skill that should be split. Three
workflows is healthy. Four is suspicious. Five is a refactor signal.

### 2. Token budget: 3,000 target, 5,000 ceiling

SKILL.md body should be ~3,000 tokens for optimal reasoning. 5,000 is the
hard ceiling, not the target. Every token competes for attention with the
user's actual problem.

### 3. Mode content in references, not inline

Mandatory for skills with >3 modes. The router pattern: thin SKILL.md with
a routing table, mode-specific content in `references/mode-*.md`. Proven by
/investigate (7 modes, thin router) and /settle (3 phases, reference-backed).

### 4. Every line justifies its token cost

Irrelevant-but-related content causes greater degradation than unrelated noise.
A skill about QA that also explains CI configuration is worse than a skill
about QA that says nothing about CI. Cut related-but-off-topic content first.

### 5. Description tax is always-on

The description field costs ~100 tokens per skill, loaded into every conversation
whether the skill fires or not. Don't split a skill unless domain coherence
demands it — each split permanently adds to the description tax.

### 6. Encode judgment, not procedures

If the model already knows how to do something, the skill is waste. Encode what
goes wrong, not what to do right. A gotcha list outperforms pages of happy-path
instructions.

### 7. Mode-bloat gate

>4 modes with inline content (not reference-backed) is a lint failure. Extract
to references/ or split the skill.

## Gotchas

- Skills that describe procedures the model already knows are waste
- Descriptions that don't include trigger phrases won't fire
- SKILL.md over 500 lines means you failed progressive disclosure
- Hooks that reference deleted skills will silently break
- Stale AGENTS.md instructions cause more harm than missing ones
- After any model upgrade, re-eval your skills — some become dead weight
- Regexes over agent prose are usually proof the boundary is wrong
```

### Acceptance criteria

- [ ] /harness SKILL.md is under 80 lines (routing table + principles + gotchas)
- [ ] Each mode reference file is self-contained and loadable independently
- [ ] `/harness create` reads references/mode-create.md and follows it
- [ ] `/harness lint` reads references/mode-lint.md and follows it
- [ ] `wc -l skills/harness/SKILL.md` < 80
- [ ] All 7 design principles appear in the SKILL.md body (always in context)
- [ ] No scaffold content remains in /harness

---

## Phase 3: Add audit mode

New mode: `/harness audit`

### references/mode-audit.md (complete)

```markdown
# /harness audit — Skill Health Assessment

Analyze skill invocation data to assess skill health, identify waste, and
recommend actions.

## Data Source

Reads `~/.claude/skill-invocations.jsonl`. Each line is a JSON object with
at minimum: `skill` (name), `timestamp`, `project` (optional), `duration_ms`
(optional), `outcome` (optional: success/failure/abandoned).

If the file doesn't exist or is empty, report: "No invocation data found.
Skill invocations are logged to ~/.claude/skill-invocations.jsonl. Once you
have data, re-run `/harness audit`."

## Flags

- `--since <duration>` — filter to recent data (e.g., `30d`, `7d`, `90d`).
  Default: all data.
- `--skill <name>` — deep-dive on a single skill instead of the full report.

## Full Report (default)

### 1. Frequency Table

| Skill | Invocations | Last Used | Avg Duration | Failure Rate |
|-------|-------------|-----------|-------------|-------------|

Sort by invocation count descending.

### 2. Health Categories

Classify each skill into one of:

| Category | Criteria | Action |
|----------|----------|--------|
| **Hot** | >10 invocations in period, low failure rate | Keep, invest |
| **Warm** | 3-10 invocations, acceptable failure rate | Keep, monitor |
| **Cold** | 1-2 invocations in period | Evaluate: niche or dead? |
| **Dead** | 0 invocations in period | Candidate for deprecation |
| **Failing** | >30% failure/abandon rate | Investigate, fix or kill |

### 3. Consolidation Candidates

Flag skills that:
- Share >50% of trigger phrases with another skill (description overlap)
- Are always invoked in sequence (A then B → merge into A)
- Have complementary domains that could be one skill without exceeding 3 workflows

### 4. Recommendations

For each skill, emit one of:
- **keep** — healthy, earning its description tax
- **invest** — hot skill, would benefit from deeper references or sub-modes
- **deprecate** — dead or cold with no clear niche
- **merge [target]** — consolidation candidate, specify merge target
- **split** — hot skill exceeding mode-bloat gate
- **fix** — high failure rate, needs investigation
- **promote** — project-local skill used across >2 projects, promote to global

### 5. Description Tax Report

Total description token cost across all installed skills (estimate ~100 tokens
per skill). Flag if total exceeds 2,000 tokens (20+ skills). This is the
permanent cost paid on every conversation.

## Deep-Dive Report (--skill flag)

For a single skill:

- Invocation timeline (histogram by week/month)
- Failure analysis: common error patterns, abandon points
- Duration distribution: p50, p90, p99
- Project breakdown: which projects use it most
- Trigger phrase analysis: which description phrases actually trigger it
  (requires matching invocation context to description)
- Recommendations: specific, actionable

## Output Format

Structured markdown report. No prose filler. Tables and bullets only.
End with a TLDR: 3-5 bullet summary of the most actionable findings.
```

### Acceptance criteria

- [ ] `/harness audit` produces a structured report from invocation data
- [ ] `--since 30d` filters correctly
- [ ] `--skill investigate` produces a deep-dive report
- [ ] Graceful handling when no invocation data exists
- [ ] Recommendations are one of the 7 defined actions (keep/invest/deprecate/merge/split/fix/promote)
- [ ] Description tax report calculates total token cost

---

## Phase 4: Add mode-bloat lint gate

### Changes to references/mode-lint.md

Add a new row to the quality gates table:

| Gate | Check | Fix |
|------|-------|-----|
| **Mode bloat** | >4 modes with inline content (not reference-backed)? | Extract mode content to `references/mode-*.md`, leave routing table in SKILL.md |

### Detection logic

Count sections in SKILL.md that match mode-like patterns (## headings that
correspond to routing table entries). If >4 such sections have >20 lines of
inline content each (not just "Read references/..."), flag as mode-bloat.

Reference-backed modes (heading + 1-2 line redirect to references/) don't count.

### Acceptance criteria

- [ ] `/harness lint` on the old (pre-refactor) harness SKILL.md flags mode-bloat
- [ ] `/harness lint` on the new (post-refactor) harness SKILL.md passes
- [ ] `/harness lint` on /investigate passes (router pattern, reference-backed)
- [ ] `/harness lint` on /settle passes (3 phases, reference-backed)
- [ ] Mode-bloat gate appears in the lint quality gates table

---

## File-by-File Change Summary

### Deleted files

| File | Reason |
|------|--------|
| skills/harness/references/scaffold-qa.md | Moved to skills/qa/references/scaffold.md |
| skills/harness/references/scaffold-demo.md | Moved to skills/demo/references/scaffold.md |
| skills/harness/references/browser-tools.md | Moved to skills/qa/references/browser-tools.md |
| skills/harness/references/evidence-capture.md | Moved to skills/qa/references/evidence-capture.md |
| skills/harness/references/pr-evidence-upload.md | Moved to skills/demo/references/pr-evidence-upload.md |
| skills/harness/references/remotion.md | Moved to skills/demo/references/remotion.md |
| skills/harness/references/tts-narration.md | Moved to skills/demo/references/tts-narration.md |

### New files

| File | Lines (est.) | Content |
|------|-------------|---------|
| skills/harness/references/mode-create.md | ~100 | Extracted from SKILL.md: description field, structure, encoding, progressive disclosure, frontmatter, dynamic context |
| skills/harness/references/mode-lint.md | ~60 | Extracted from SKILL.md: quality gates table + new mode-bloat gate |
| skills/harness/references/mode-eval.md | ~40 | Extracted from SKILL.md: baseline comparison protocol |
| skills/harness/references/mode-convert.md | ~40 | Extracted from SKILL.md: agent→skill and skill→agent conversion |
| skills/harness/references/mode-sync.md | ~30 | Extracted from SKILL.md: sync protocol, .harness-kit marker |
| skills/harness/references/mode-engineer.md | ~80 | Extracted from SKILL.md: codification hierarchy, Norman principle, Dagger, hooks, AGENTS.md, stress-test, thin harness |
| skills/harness/references/mode-audit.md | ~80 | New: skill health assessment from invocation data |
| skills/qa/references/scaffold.md | 225 | Moved from harness, renamed |
| skills/qa/references/browser-tools.md | 272 | Moved from harness |
| skills/qa/references/evidence-capture.md | 160 | Moved from harness |
| skills/demo/references/scaffold.md | 213 | Moved from harness, renamed |
| skills/demo/references/pr-evidence-upload.md | 89 | Moved from harness |
| skills/demo/references/remotion.md | 217 | Moved from harness |
| skills/demo/references/tts-narration.md | 177 | Moved from harness |

### Modified files

| File | Change |
|------|--------|
| skills/harness/SKILL.md | Complete rewrite: 286 → ~75 lines. Router + principles + gotchas. |
| skills/qa/SKILL.md | Rewrite: 45 → ~45 lines. Add scaffold routing, update reference paths. |
| skills/demo/SKILL.md | Rewrite: 64 → ~65 lines. Add scaffold routing, update reference paths. |
| skills/settle/SKILL.md | Update pr-evidence-upload reference path (line 149). |
| skills/autopilot/SKILL.md | Update scaffold references from `/harness scaffold` to `/qa scaffold` and `/demo scaffold` (lines 95, 109). |
| skills/autopilot/references/qa-and-demo.md | Update all `/harness scaffold` refs and reference material paths to new locations. |

---

## Risk Assessment

### What could break

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| `/harness scaffold qa` invocations in other skills/docs | Medium | Medium | grep for all "harness scaffold" references, update to `/qa scaffold` |
| `/autopilot` dispatching `/harness scaffold` | Certain | High | autopilot SKILL.md (lines 95, 109) and references/qa-and-demo.md both reference `/harness scaffold` — update all |
| Reference reads using old paths | Medium | High | `git grep 'harness/references/' skills/` — must be zero after refactor |
| Mode-create content loses context when extracted | Low | Medium | Keep the 7 design principles in SKILL.md body (always loaded), mode details in references |
| Description changes cause /harness not to trigger | Low | High | Keep existing trigger phrases, add new ones for audit |
| No invocation data exists for audit mode | Certain (initially) | Low | Graceful fallback message, audit mode is useful once data accumulates |

### Verification plan

1. `git grep 'harness/references/' skills/` — must return zero matches post-refactor
2. `git grep 'scaffold qa' skills/` — verify all point to `/qa scaffold`
3. `git grep 'scaffold demo' skills/` — verify all point to `/demo scaffold`
4. `wc -l skills/harness/SKILL.md` — must be < 80
5. `wc -l skills/qa/SKILL.md` — must be < 50
6. `wc -l skills/demo/SKILL.md` — must be < 70
7. For each mode: invoke `/harness <mode>` and verify the reference loads
8. `/harness lint skills/harness/SKILL.md` — must pass all gates including mode-bloat
9. `/harness lint skills/investigate/SKILL.md` — must pass (regression check)

### Build order

Phases must be executed in order. Phase 1 moves files that Phase 2 would
otherwise create broken references to. Phase 3 and 4 are independent of each
other but depend on Phase 2 (they add to the refactored skill).

```
Phase 1 (extract scaffolds) → Phase 2 (router refactor) → Phase 3 + 4 (parallel)
```

Estimated total: 7 new files, 7 moved files, 6 modified files, 7 deleted files.
Net change: ~0 lines (redistribution, not growth).
