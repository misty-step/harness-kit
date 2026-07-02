# Retire or fold stale project.md into VISION.md/README

Priority: P3 (shipped) · Status: done · Estimate: S · Shipped: 2026-07-02

## Goal

Resolve the duplicate, stale north-star document at repo-root `project.md` —
it predates the current `VISION.md` and still carries pre-June themes
(rebrand, "remove global static agents," Antigravity framing) that no longer
match the shipped repo.

## Oracle

- [x] `project.md` is either deleted (its live content is fully superseded by
      `VISION.md` + `README.md`) or reduced to a stub pointer at `VISION.md`
      — pick one and state which in the PR, do not leave both as competing
      sources of truth. **Deleted** — not stubbed. A stub would have added a
      permanent file that exists only to say "look elsewhere," which the
      repo's own "delete as progress" doctrine (`VISION.md`) argues against
      when nothing depends on the file existing (see below).
- [x] Anything in `project.md`'s "Domain Glossary" section still accurate and
      not already covered by `CODEBASE.md`/`README.md` is preserved
      (migrated, not silently dropped). **Nothing to migrate** — checked each
      term live: `collections.yaml`, `.harness-kit.yaml`, and `skills/seed`
      (the "Collection", "Manifest", and "Seed" glossary entries) are all
      confirmed gone from the tree; "Skill" and "Harness" are already defined
      in `CODEBASE.md`'s Source Of Truth table; "DMI" was deliberately
      removed per `CHANGELOG.md` and its underlying mechanism
      (`disable-model-invocation` frontmatter) is documented properly in
      `skills/harness-engineering/references/mode-create.md` and
      `skills/design/references/scaffold.md`; "Delivery pipeline" is
      superseded by `CODEBASE.md`'s own (more current) Workflow Lifecycle
      section.
- [x] No remaining file references `project.md` as the current vision/status
      doc (`grep -rn "project.md" --include='*.md' .` reviewed by hand).
      Three live mentions found and reviewed individually — all are generic
      cross-repo pattern examples (`orient`'s reading-order guidance,
      `groom/references/investigation-bench.md`'s investigator prompt
      template, `harnesses/antigravity-cli/README.md`'s `--add-dir` usage
      example), none asserting *this* repo's own root `project.md` must
      exist — orient/groom run across many fleet repos and already handle a
      missing `project.md` gracefully. None needed editing.
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Notes

Confirmed live: `project.md`'s "Current Focus" line reads "Rebrand, remove
global static agents, and make skills + AGENTS.md the durable layer" — themes
`git log` shows resolved months ago (harness-kit v2 Mode A/B consolidation,
917b152a). `VISION.md` (root) is the actively maintained, richly cross-linked
north star (`Where The Depth Lives` section, 35 doctrine commits/month per
tonight's earlier groom teardown) and has fully superseded `project.md`'s
role.

**Why:** the teardown that seeded tonight's decisions flagged this exact file
("`project.md` claims 'last updated 2026-03-16,' content is pre-June themes
… delete it or fold into VISION.md/README") as a docs-staleness finding; it
was not in the decisions-overlay action list for tonight and remains
unresolved on the live tree as of this grooming pass.
