# Project: Spellbook

## Vision
Portable harness primitive library that makes AI coding agents reliably
excellent across any runtime without locking the operator into static
personas or a bespoke orchestration layer.

**North Star:** Every recurring workflow pattern a senior engineer uses is captured as a tested,
composable primitive — so any strong agent can adapt it to Claude Code, Codex,
Antigravity, Pi, or the next harness first-try.
**Target User:** Senior+ engineers running multi-agent workflows across multiple repos.
**Current Focus:** Rebrand, remove global static agents, and make skills +
AGENTS.md the durable layer for dynamic task-specific delegation.
**Key Differentiators:** Agent-agnostic (works across harnesses), manifest-driven (`.spellbook.yaml`),
research-backed (web + multi-model validation before codifying).

## Domain Glossary

| Term | Definition |
|------|-----------|
| Skill | A markdown-first module (SKILL.md + optional references/) that gives agents domain expertise |
| Delegation guidance | Natural-language guidance that tells the lead agent when to spawn task-specific subagents, what responsibilities to assign, what evidence to require, and what philosophical lenses to apply |
| Collection | Named group of skills in collections.yaml (payments, web, agent, infra, etc.) |
| Harness | An AI agent runtime (Claude Code, Codex, Antigravity CLI/IDE, Pi) |
| Manifest | `.spellbook.yaml` — declares which primitives a project needs |
| Tailor | Meta-skill that reads a target repo, picks primitives, rewrites workflow skills, and projects harness-specific entrypoints |
| DMI | Disable-model-invocation — user-only skills that cost zero budget |
| Delivery pipeline | groom → deliver (shape → implement → review + ci + refactor + qa) → merge |

## Active Focus

- **Milestone:** Harness library reset — rebrand, dynamic delegation, Antigravity migration
- **Key work:** Pick the new name, stop installing global agents, convert static persona files into reusable lenses inside skills and AGENTS.md, add Antigravity CLI/IDE projections
- **Theme:** Skills and repo guidance are durable; static global subagent catalogs are not
- **Recent:** Restored the repo under `~/Development/spellbook`; backlog pivoted toward Antigravity and dynamic subagent management

## Quality Bar

- [ ] Every skill has clear trigger conditions (when to invoke, when NOT to)
- [ ] All descriptions ≤1024 chars with trigger phrases
- [ ] Skills compose — orchestrators call primitives, never reimplement
- [ ] Agent-agnostic — no Claude-specific assumptions leak into skill bodies
- [ ] Static personas become lenses and delegation rubrics unless a runtime requires a projection
- [ ] Retro patterns flow back into skill definitions

## Patterns to Follow

### Progressive Disclosure
```
description (budget cost) → SKILL.md body → references/ (on-demand)
```

### Skill Structure
```
skills/{name}/
├── SKILL.md          # Required. Frontmatter + instructions.
└── references/
    ├── sub-cap-1.md  # Loaded on demand
    └── sub-cap-2.md  # Zero additional budget cost
```

### AC Tags for Machine Verification
```markdown
- [ ] [test] Given X, when Y, then Z
- [ ] [command] Given X, when `cmd`, then output matches
- [ ] [behavioral] Given X, when user does Y, then Z
```

## Lessons Learned

| Decision | Outcome | Lesson |
|----------|---------|--------|
| Umbrella pattern (research/) | 4→1 budget savings | Progressive disclosure works; add sub-caps at zero cost |
| Core pruning 50+ → 11 | Budget 10.2K → 1.4K | Aggressive pruning + pack architecture works |
| Standalone pipeline skills | Fragmented, hard to compose | Absorb related skills into umbrellas |
| Symlink distribution | Fragile, machine-specific | Manifest-driven pull from GitHub (focus) is more portable |
| Flat skills/ directory | Simpler than core/packs | One level of indirection is enough; collections handle grouping |

---
*Last updated: 2026-03-16*
*Updated during: Spellbook architecture refactor*
