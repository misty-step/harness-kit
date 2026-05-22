---
name: harness
description: |
  Build, maintain, evaluate, and optimize the agent harness — skills, agents,
  hooks, CLAUDE.md, AGENTS.md, and enforcement infrastructure.
  Use when: "create a skill", "update skill", "improve the harness",
  "sync skills", "eval skill", "lint skill", "tune the harness",
  "add skill", "remove skill", "convert agent to skill",
  "audit skills", "skill health", "unused skills",
  "lint this skill", "eval this skill", "evaluate this skill",
  "validate this skill", "check this skill".
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
| Sync primitives from spellbook to project | `references/mode-sync.md` |
| Design harness improvements | `references/mode-engineer.md` |
| Audit skill health and usage | `references/mode-audit.md` |

If first argument matches a mode name, read the corresponding reference.
If no argument, ask: "What do you want to do? (create, eval, lint, convert, sync, engineer, audit)"

**Scaffold moved.** If user says "scaffold qa" or "scaffold demo", redirect:
"Scaffold is now owned by the domain skill. Run `/qa scaffold` or `/demo scaffold`."

## Skill Design Principles

These principles govern every mode. They are the quality standard for skills
this harness creates, evaluates, and lints.

0. **Roster-aware workflows.** Core workflow skills should name where they
   consult `.spellbook/agents.yaml` for non-trivial provider lanes, while
   keeping schema details and receipt mechanics centralized in the roster
   scripts and shared AGENTS doctrine.
1. **One skill = one domain, 1-3 workflows.** A skill that spans multiple
   domains should be split. Three workflows is healthy. Five is a refactor signal.
2. **Token budget: 3,000 target, 5,000 ceiling.** Every token competes for
   attention with the user's actual problem. 5,000 is the hard ceiling, not target.
   *Exemplars in this repo:* `skills/flywheel/SKILL.md` (42 lines — states
   composition invariants, delegates phase logic elsewhere) and
   `skills/shape/SKILL.md` (117 lines — invariant-first even at larger scale).
3. **Mode content in references, not inline.** Mandatory for >3 modes. Thin
   SKILL.md with routing table, mode content in `references/mode-*.md`.
4. **Every line justifies its token cost.** Irrelevant-but-related content
   degrades more than unrelated noise. Cut related-but-off-topic content first.
5. **Description tax is always-on.** ~100 tokens per skill, loaded every
   conversation. Don't split unless domain coherence demands it.
6. **Encode judgment, not procedures.** If the model already knows how, the
   skill is waste. Gotcha lists outperform pages of happy-path instructions.
   *Exemplars:* `skills/diagnose/SKILL.md` (routing table is the core
   judgment — which phase for which symptom) and `skills/settle/SKILL.md`
   (mode detection — GitHub-PR vs git-native — is the core judgment).
7. **Mode-bloat gate.** >4 modes with inline content is a lint failure.
   Extract to references/ or split the skill.
8. **Self-contained.** Every file the skill needs lives under
   `skills/<name>/`. Scripts source libs from `$SCRIPT_DIR/lib/…`, not
   `$REPO_ROOT/…`. State roots (cycles, locks, backlog) resolve from the
   invoking project (`git rev-parse --show-toplevel`), not the skill's
   install location. Lint catches violations.
9. **Cross-harness first.** Skills and mechanisms target Claude Code,
   Codex, and Pi. If a design only works inside one harness's runtime
   (e.g. Claude `enabledPlugins`, Codex `/plugins`), find the
   filesystem-level equivalent that works across all three. Prior art:
   `harnesses/pi/settings.json:skills[]` glob. See
   `harnesses/shared/AGENTS.md` Cross-Harness First.
10. **Prose for an intelligent reader, not a program.** You're not
    writing code. You're writing instructions for an intelligent
    agent. The best skills read like a colleague explaining
    (`skills/.external/anthropic-skill-creator/SKILL.md` ends a
    section with "Cool? Cool."). Favor bullets, examples, and
    guardrails over flowcharts. If your skill has Phase 0 / Phase 1 /
    state-machine shape, you're describing a program. Strip to
    invariants + a "shape of the work" paragraph; the agent fills
    sequencing itself. *Exemplars:*
    `skills/.external/openai-playwright/SKILL.md` (147 lines, mostly
    copy-paste examples + guardrails) and
    `skills/.external/anthropic-skill-creator/SKILL.md`
    (colleague-voice throughout).

## Gotchas

- Skills that describe procedures the model already knows are waste
- Descriptions that don't include trigger phrases won't fire
- SKILL.md over 500 lines means you failed progressive disclosure
- Hooks that reference deleted skills will silently break
- Stale AGENTS.md instructions cause more harm than missing ones
- After any model upgrade, re-eval your skills — some become dead weight
- Regexes over agent prose are usually proof the boundary is wrong
- Scripts that source `$REPO_ROOT/…` or `../../…` outside the skill tree
  break the moment the skill is symlink-distributed. Test the distribution
  path, not just the in-repo path.
- Anchoring a new mechanism on one harness's runtime (Claude plugins,
  Codex plugins) is a design bug. All three harnesses scan SKILL.md
  from a skills directory — that's the common denominator. Runtime
  features are optimizations, not primary mechanisms.
- State-machine-shaped skills (Phase 0-N flowcharts, deterministic
  scoring scripts inside SKILL.md, exit-code routing tables) are a
  smell. You're writing for an intelligent agent, not Python.
  Rewrite as invariants + guardrails + a paragraph of "shape of
  the work." The agent fills sequencing itself.
