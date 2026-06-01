# Skill Frontmatter Schema

Harness Kit skills are indexed from `SKILL.md` YAML frontmatter. Keep metadata
compact, literal, and routeable.

## Required Fields

### `name`

- Required string.
- Matches the skill directory name unless a migration note explains otherwise.
- Lowercase kebab-case for first-party skills.

### `description`

- Required string block.
- Starts with what the skill does, not "Use this skill to".
- Includes concrete `Use when:` or `Use for:` phrases in double quotes for
  natural-language routing.
- Includes a `Trigger:` line for slash-command routing.
- Does not claim phrases owned by another skill.

### `argument-hint`

- Recommended string for command-shaped skills.
- Uses bracketed optional arguments such as `[--fix] [target]`.
- Names modes only when the skill actually supports them.

## Routing Clauses

### `Use when:` / `Use for:`

- Lists quoted phrases an operator may type naturally.
- Phrases are part of the routing contract and must not duplicate another
  active first-party skill's exact phrase.
- Prefer domain-specific phrases over broad verbs.

### `Trigger:`

- Lists slash commands and aliases, comma-separated.
- The canonical trigger comes first.
- Aliases must include a reason in the skill body when they are not obvious.
- Deprecated redirect skills may keep legacy triggers only while they exist.
- Exact trigger collisions are invalid.

## Current Verb Families

- Package, commit, and push local work: `/yeet`.
- Land a merge-ready branch and close the learning loop: `/ship`.
- Release/deploy merged code to an environment: `/deploy`.
- Code review: `/code-review` and `/review`.
- Ad-hoc lens critique: reserved for `/critique`.
