# /harness create (in spellbook)

Create a new skill at `skills/<name>/` or a new agent at
`agents/<name>.md`. You are editing the canonical source.

## Before you create

- Justify the addition against description tax. Current catalog:
  26 skills × ~100 tokens description = ~2.6k tokens always loaded.
  Each new skill pays that tax on every conversation in every repo.
- Check `index.yaml` first. Search existing descriptions for the
  domain. If another skill could reasonably own this workflow with a
  1-3 line addition, extend it instead of adding a sibling.
- Read `harnesses/shared/AGENTS.md` — "When to act directly" — before
  assuming a new skill is needed. Mechanical transformations and
  single-concern fixes don't need a skill.

## Skill layout

```
skills/<name>/
├── SKILL.md          # ≤500 lines (enforced by check-frontmatter.py)
├── references/       # Progressive disclosure — loaded on demand
│   └── mode-*.md     # Router pattern when >3 modes
└── scripts/          # Optional. Must be self-contained.
```

Agent layout: one file at `agents/<name>.md` with frontmatter and
markdown body. No directory, no references.

## Frontmatter (skills)

Required (enforced by `scripts/check-frontmatter.py`):

```yaml
---
name: my-skill               # must match dir basename
description: |
  What it does. When to use it. Concrete trigger phrases.
  Use when: "debug this", "why broken", "production down".
argument-hint: "[mode] [target]"        # optional — autocomplete hint
disable-model-invocation: true          # optional — user-only
model: sonnet | opus                    # optional — per-skill model pin
---
```

Description is the trigger. Write it assertively. If the skill doesn't
fire on real utterances, the description is wrong — not the model.

**Good:** `"Use when: 'debug this', 'investigate', 'production down'"`
**Bad:** `"A debugging utility for code analysis"`

## Frontmatter (agents)

Same required fields (`name`, `description`). Common additions:

```yaml
---
name: my-agent
description: "Persona + invocation trigger in one line."
tools: Read, Grep, Glob, Bash          # allowlist
disallowedTools: Edit, Write, Agent    # denylist (evaluated after tools)
model: opus | sonnet
color: blue                             # UI hint (Claude-specific)
---
```

Agent body is a system prompt for the persona. See `agents/critic.md`
and `agents/ousterhout.md` as reference forms.

## What to encode

Encode the judgment the model lacks — not the procedure it already
knows. Gotcha lists outperform happy-path instructions. Enumerate:

- Failure modes and how to recognize them.
- Invariants that cannot be derived from context.
- Boundary decisions (which phase for which symptom).
- Cross-harness differences the author must account for.

**Avoid:** Step-by-step recipes. Phase-0/Phase-N state machines.
Exit-code routing tables inside SKILL.md. Regex recovery of agent
output. These are smells — you are writing for an intelligent agent.

## In-repo exemplars (read before drafting)

- `skills/flywheel/SKILL.md` — ~43 lines. Outer-loop composer. States
  invariants, delegates phase logic. Reference thin-harness form.
- `skills/shape/SKILL.md` — ~117 lines. Invariant-first at larger
  scale. Double diamond: diverge-before-converge twice.
- `skills/diagnose/SKILL.md` — routing table IS the skill.
- `skills/settle/SKILL.md` — mode detection IS the skill.
- `skills/tailor/SKILL.md` — the hot recent work; per-repo tailoring
  doctrine (repo brief → iterative rewriters → reconciliation →
  cross-harness install).

## Post-create checklist

1. Run `python3 scripts/check-frontmatter.py` — required fields, line
   cap. Fix any error before continuing.
2. Run `./scripts/generate-index.sh` (or let the pre-commit hook) —
   verify `index.yaml` contains the new entry.
3. If the skill interacts with install mechanics, also run
   `scripts/check-harness-agnostic-installs.sh`.
4. Run `dagger call check --source=.` for the full 14-gate preflight
   before opening a PR.
5. If it's a workflow skill with a clear delta, add an `evals/`
   directory with a representative prompt and run `/harness eval`.

## Cross-harness check

Can Claude load this? (SKILL.md under a skills dir, yes.) Can Codex
load this? (Same.) Can Pi load this? (Pi uses glob allow/deny in
`harnesses/pi/settings.json:skills[]` — does the skill name belong in
the allowlist?) If the answer to any is "only with runtime feature X,"
redesign at the filesystem layer first.
