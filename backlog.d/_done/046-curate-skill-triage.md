# Triage legacy `curate` skill (pre-`.harness-kit`-marker era)

Priority: low
Status: retired
Estimate: S

## Goal

Decide fate of `.agents/skills/curate/SKILL.md` + `.claude/skills/curate/SKILL.md`.
Both predate the `.harness-kit` marker convention (`installed-by: tailor`) and
are not in the canonical `skills/` catalog. They reference `scripts/
`harness-kit-checks generate-embeddings`, which replaced the old Python generator.

## Outcome

Retired with the global-skill bootstrap pivot. The legacy repo-local `curate`
copy was deleted when `.agents/skills/` became source-backed links to the
canonical `skills/` catalog. No canonical `skills/curate/` exists.

## Why this isn't closed by `/tailor`

`/tailor` preserves unmarked content by default — it cannot delete content
it didn't install. The triage is a human decision:

1. **Promote to canonical** — move content into `skills/curate/` under the
   repo's skills catalog, add frontmatter, register via `scripts/generate-
   index.sh`. Appropriate if the skill encodes judgment still useful for
   evolving the library.
2. **Archive** — move under `skills/.external/` or delete; content is
   stale and `/harness audit` can be invoked directly instead.
3. **Preserve with marker** — leave content in place but stamp a
   `.harness-kit` marker with `installed-by: human` so future `/tailor`
   runs stop surfacing it as unknown.

## Oracle

- [ ] `.agents/skills/curate/` + `.claude/skills/curate/` reach a marked
      state (marker present) or are removed.
- [ ] If promoted to canonical: `skills/curate/SKILL.md` passes
      `dagger call check-frontmatter` and appears in `index.yaml`.
- [ ] Subsequent `/tailor` runs do not flag `curate` as unmarked legacy.

## Non-Goals

- Re-implementing curation logic (scope: status-quo decision only).
- Extending `/curate` to new sources.
