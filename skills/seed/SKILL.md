---
name: seed
description: |
  Vendor the system-wide Spellbook harness into the current repo when a
  project needs checked-in local copies instead of relying on global
  bootstrap symlinks. Copies first-party skills and agents into a repo-local
  shared skill layer, then bridges harness-specific entrypoints back to that
  shared copy. Use when: "seed this repo", "vendor spellbook here",
  "initialize the agent here", "set me up offline".
  Trigger: /seed.
---

# /seed

Vendor Spellbook into this repo. Most repos should rely on system-wide
bootstrap instead; use `/seed` only when the repo needs checked-in local
copies for offline work, review, or repository-specific persistence.

## What to do

1. Find `$SPELLBOOK`: `readlink -f` this SKILL.md, walk up until you
   see `skills/` + `agents/` + `harnesses/`.

2. Resolve the repo-local **shared skill root**. Use `.agents/skills/`.
   If a repo already has legacy `.agent/skills/`, stop and ask whether to
   migrate it. The shared root is the canonical storage for vendored
   Spellbook skills; `.claude/skills/`, `.codex/skills/`, and `.pi/skills/`
   are bridges.

3. Copy every skill in `$SPELLBOOK/skills/` into the shared skill root,
   preserving each skill's full directory (`references/`, `scripts/`,
   `evals/`, everything). Skip `seed` itself; it lives globally.

4. Ensure `.claude/skills/<name>`, `.codex/skills/<name>`, and
   `.pi/skills/<name>` point at each shared skill when those harness dirs
   exist. Do not duplicate skill contents into harness-specific dirs.

5. Copy every agent in `$SPELLBOOK/agents/` into the repo's existing
   agent directory. In most repos today that is `.claude/agents/`.
   Do not invent a second copy of agents unless the repo already has
   a documented shared-agent convention.

6. Copy `$SPELLBOOK/harnesses/shared/AGENTS.md` to `./AGENTS.md`
   only if one doesn't already exist.

7. Print what you installed.

## Invariants

- Never modify `$SPELLBOOK` or `~/.claude` / `~/.codex` / `~/.pi`.
  Writes only to the current repo.
- Shared skill root first. Spellbook-distributed skills live in the
  repo-local shared skill layer; `.claude/skills/` is only a bridge.
- Don't clobber existing shared skill roots, `.claude/`, or
  `AGENTS.md` — ask first.
- Don't filter, don't judge, don't specialize. Global bootstrap already
  installs all first-party skills; `/seed` exists only for repo-local
  vendoring.

Typical time: seconds. Typical cost: zero LLM tokens beyond the
skill body — just file copies.
