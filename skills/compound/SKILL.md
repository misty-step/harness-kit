---
name: compound
description: |
  Capture one compounding repo-technical learning while a solved problem is
  still fresh. Use when: after a bug fix, diagnosis, delivery, review, or
  incident reveals a reusable pattern worth adding to `docs/solutions/`.
  Trigger: /compound, /capture-learning, /learning.
argument-hint: "[solved-problem summary]"
---

# /compound

Capture the learning, not the whole story.

## Contract

- One solved problem -> one learning.
- Corpus: `docs/solutions/<category>/<slug>.md`; schema:
  `harnesses/shared/references/learnings.md`.
- Overlap first:
  `rg -n --glob '*.md' '^(title|tags|applies_when):|<module>|<failure-mode>' docs/solutions`.
- If an existing learning covers it, update that file only when the fresh
  evidence contradicts or sharpens it. Otherwise do not add a near-duplicate.
- If new, use grep-ready frontmatter: `title`, `tags`, `module`,
  `problem_type`, `applies_when`, `severity`, `date`.
- Cite checkable evidence: `repo@SHA`, PR, command, route, and `file:line`
  where possible. Session notes may provide context; they are not the anchor.
- Pattern docs are promoted from multiple learnings. Never author a broad
  doctrine page cold from one bug.

## Write Discipline

Present evidence wins at read. Coherence-neighborhood at write: refresh only
nearby learnings likely to conflict. On contradiction, scope the refresh to
the smallest corpus slice that can be wrong.

## Output

Report the path written or updated, the overlap query used, the evidence
anchor, and 3-6 retrieval terms future agents should grep.
