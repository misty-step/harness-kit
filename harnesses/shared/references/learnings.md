# Learnings Corpus

Git-tracked repo-technical lessons live in `docs/solutions/`. They complement
session/operator memory: learnings are reusable engineering facts with
checkable anchors.

## Unit

One solved problem, one learning. Write the narrow reusable pattern that would
have changed the next agent's behavior before the bug, review miss, or design
mistake happened.

Pattern docs are promoted from multiple learnings; do not author broad doctrine
cold from one incident.

## Frontmatter

Every learning is Markdown with grep-ready YAML frontmatter.

```yaml
---
title: "Imperative reusable lesson"
tags: [domain, failure-mode, retrieval-term]
module: "repo/module-or-surface"
problem_type: bug-track # bug-track | knowledge-track
applies_when:
  - "2-4 concrete conditions that make this learning relevant"
  - "Name runtime shape, module shape, or review smell"
severity: medium # low | medium | high | critical
date: YYYY-MM-DD
repo_anchor: "owner/repo@sha"
pr: "owner/repo#123"
---
```

Required fields: `title`, `tags`, `module`, `problem_type`, `applies_when`,
`severity`, `date`. `repo_anchor`, `pr`, `commands`, or `evidence` are expected
when checkable. Session notes may provide context; they are not the anchor.

Use `problem_type: bug-track` for a lesson extracted from a concrete defect or
missed review. Use `problem_type: knowledge-track` for a stable fact or
integration constraint learned without a live bug.

## Retrieval

Grep is the floor. Semantic retrieval, QMD, or future indexers may run after it;
they do not replace it.

```sh
rg -n --glob '*.md' '^(title|tags|applies_when):|<module>|<failure-mode>' docs/solutions
```

Open likely matches and decide whether each applies to the current work. At
read time, present evidence wins over stale corpus prose.

## Write Discipline

Before adding a learning, check overlap:

```sh
rg -n --glob '*.md' '^(title|tags|applies_when):|<module>|<failure-mode>' docs/solutions
```

If a contradiction appears, refresh the smallest coherence neighborhood: the
candidate learning, directly overlapping learnings, and the checkable source
anchors. Avoid global rewrites unless multiple learnings prove the pattern has
changed.

## Body Template

```markdown
## Context

What failed, where, and why this lesson is reusable.

## Learning

The behavior future agents should apply.

## Evidence

- Anchor: owner/repo@sha
- Files: path:line
- Verification: command, route, artifact, or PR

## Retrieval Terms

Short list of terms future agents should grep.
```
