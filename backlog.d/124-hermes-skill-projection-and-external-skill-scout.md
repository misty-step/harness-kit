# Hermes skill projection and external skill scout

Priority: P2
Status: pending
Estimate: M

## Goal

Make Harness Kit skills available to Hermes Agent through a bootstrap-owned,
collision-safe projection, and decide which popular external skill packs deserve
import after evidence rather than catalog popularity.

## Context

Hermes already has a native skill manager and local skill root. Live inspection
showed Hermes can list local skills, but several Harness Kit-ish skills are
currently Hermes-local copies rather than bootstrap-owned projections. A naive
`~/.hermes/skills/<skill>` symlink pass is unsafe because Hermes local skills can
shadow external dirs, and existing category directories such as
`~/.hermes/skills/research` must not be deleted or overwritten.

The first externally managed Matt Pocock import is intentionally small and
source-scoped. Superpowers and larger catalogs should be evaluated against
Harness Kit primitives instead of bulk-vendored by reputation.

## Oracle

- [ ] A Hermes projection design names the chosen mechanism:
      `skills.external_dirs`, profile-scoped symlink tree, or another Hermes
      native adapter.
- [ ] A fixture or dry run proves bootstrap does not delete, overwrite, or
      shadow an existing Hermes-local skill such as `research/research`.
- [ ] Collision behavior is explicit for first-party, externally managed, and
      Hermes-local skills.
- [ ] `hermes skills list` or the closest stable Hermes command proves projected
      Harness Kit skills are visible in a real Hermes profile.
- [ ] Superpowers, Addy Osmani agent-skills, and at least one catalog source are
      scored through the eval-bench backlog instead of bulk imported directly.
- [ ] The repo gate passes:
      `cargo run --locked -p harness-kit-checks -- check --repo .`.

## Candidate Sources

- `obra/superpowers`: cohesive methodology, but likely too universal for a bulk
  Harness Kit import without outcome evidence.
- `addyosmani/agent-skills`: high-quality lifecycle pack with heavy overlap
  against first-party delivery, QA, review, and design loops.
- `VoltAgent/awesome-agent-skills`: discovery index, not an import source.

## Links

- Related eval backlog: `backlog.d/112-harness-eval-bench.md`.
