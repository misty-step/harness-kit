# Crystallize frontmatter schema and trigger contracts

Priority: P1
Status: merge-ready
Estimate: S

## Goal

Formalize the metadata contract for Harness Kit skills by documenting and validating trigger phrases, use-cases, and naming specifications. This ensures that the global catalog can be indexed structures and matches John Ousterhout's *deep module* doctrine.

## Oracle

- [x] Create `skills/skillify/references/frontmatter-schema.md` formalizing the exact schema requirements for `name`, `description`, `argument-hint`, `Use when:`, and `Trigger:` structures.
- [x] Extend `scripts/check-frontmatter.py` to check for trigger definitions and emit a warning (does not fail CI in this phase) on omissions.
- [x] Ensure that no trigger collisions exist across active first-party skills.
      The known collisions this gate must catch and force-resolve:
      - `/critique` — claimed by `code-review` AND the new `/critique` skill
        (`077`). Resolution: `code-review` drops `/critique`, keeps `/review`.
      - "ship it" / `/ship-it` — claimed by `ship`, `yeet` (`/ship-local`), and
        `deploy` (`/ship-it`, `/release`). Resolution: one clear owner per verb
        (package/push = yeet, land/learn = ship, release = deploy).
- [x] `dagger call check --source=.` runs green.

## Notes

A trigger collision occurs when two skills claim identical trigger phrases (e.g. `ship` vs `deploy` using overlapping words without clear routing). This ticket ensures we warn on drift before Phase 3 strictly enforces collision limits.

## Progress

- Added a frontmatter schema reference covering `name`, `description`,
  `argument-hint`, `Use when:` / `Use for:`, and `Trigger:` routing clauses.
- Extended `check-frontmatter.py` to warn on missing trigger definitions while
  failing exact trigger/use-when claim collisions.
- Normalized slash, hyphen, and phrase forms so `/ship-it` and "ship it"
  collide intentionally.
- Resolved current metadata collisions by removing `/critique` from
  `/code-review`, `/ship-it` from `/deploy`, stale `"fix CI"` from `/settle`,
  and plain `"ship it"` from `/yeet`.

## Verification

- `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`
- `cargo test --workspace --locked frontmatter`
- `cargo test --workspace --locked`
- `dagger call check --source=.`
