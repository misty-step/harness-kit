# Fix dead petekp-grill-me reference in interrogate-first.md

Priority: P2 (shipped) · Status: done · Estimate: S · Shipped: 2026-07-02

## Goal

Remove or replace the dangling reference to the culled `petekp-grill-me`
vendored skill in `harnesses/shared/references/interrogate-first.md`, left
behind by tonight's zero-use vendored skill cull (backlog 127, `_done/`).

## Oracle

- [x] `harnesses/shared/references/interrogate-first.md:5-7` no longer cites
      `skills/.external/petekp-grill-me/SKILL.md` as a live exemplar (the
      directory is confirmed gone from `skills/.external/`).
- [x] The "interview, not questionnaire" posture keeps its exemplar sentence —
      either point it at a first-party skill that still embodies the stance
      (`/shape`'s grill step, per `skills/shape/SKILL.md:98`, was named as the
      redundant survivor in the cull's own evidence) or drop the file
      reference and keep the prose description. (Repointed to
      `skills/shape/SKILL.md` — `/shape` loads this reference and is the
      primary consumer of the stance, not a co-equal external exemplar.)
- [x] `grep -rn "petekp-grill-me" harnesses/ skills/ docs/` returns nothing
      outside `backlog.d/_done/` and `.evidence/`. (One hit remains,
      `docs/site/manifest.json` — the generated backlog index echoing this
      ticket's own title text, not a doctrine reference; the actual
      `interrogate-first.md` citation is gone, confirmed by a targeted grep
      against that file alone.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Notes

This file is loaded as shared, always-on reference material
(`harnesses/shared/references/`), not a one-off doc — a dangling path to a
deleted skill is exactly the kind of stale doctrine the repo's own gates
(`check-no-claims`, portable-paths family) are supposed to catch, but this
one is prose-only and slipped through since no gate checks reference-file
prose against `skills/.external/` contents.

**Why:** verified live — `test -d skills/.external/petekp-grill-me` returns
false (directory absent, confirming the cull landed), while `grep -n
"petekp-grill-me" harnesses/shared/references/interrogate-first.md` still
returns the citation at lines 5-6. This is the only surviving dead reference
found in a full repo grep for all six culled aliases (mattpocock suite,
steipete-skill-cleaner, openai-gh-address-comments, openai-gh-fix-ci,
petekp-grill-me, every-ce-dogfood-beta) across `*.md`/`*.yaml`/`*.rs`.
