# Document bootstrap --bundle/--dry-run in the README Quick Start

Priority: P2 (shipped) · Status: done · Estimate: S · Shipped: 2026-07-02

## Goal

Role-scoped bootstrap bundles (backlog 130, landed tonight — `.harness-kit/bundles.yaml`,
`bootstrap --bundle NAME [--dry-run]`) shipped with zero mention in
`README.md`. A cold operator or agent following the README's Quick Start has
no way to discover the feature exists.

## Oracle

- [x] `README.md`'s Quick Start section documents `--bundle NAME` and
      `--dry-run` alongside the existing bootstrap one-liner, naming the
      available bundles (`lead`, `implementer`, `critic`, `designer`,
      `vault`) and the default (full catalog, unchanged) behavior when the
      flag is omitted.
- [x] The doc states the measured effect from 130's own oracle (full catalog
      ~21.4k description bytes vs. ~4-8k per bundle) so the value proposition
      is legible without reading `bundles.yaml`. (Used fresh live numbers
      from `bootstrap --bundle <name> --dry-run` for all 5 bundles rather
      than trusting 130's possibly-stale estimate: full catalog is 51
      skills/~21471 bytes; bundles range 8–21 skills/~4.0–7.8k bytes today.)
- [x] `cargo run --locked -p harness-kit-checks -- bootstrap --help` output
      and the README stay consistent (cross-check by hand, not just gate).
      (`--help`'s usage line reads `bootstrap [--repo PATH] [--bundle NAME]
      [--dry-run]` — README's flag names/casing match exactly.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Notes

Confirmed live: `harness-kit-checks --help` lists
`bootstrap [--repo PATH] [--bundle NAME] [--dry-run]` as a real, working
flag; `grep -rn "bundle" README.md CODEBASE.md bootstrap.sh` returns nothing.
`backlog.d/130-role-scoped-bootstrap-bundles.md` itself is still
`Status: in-progress` (child 5 — telemetry-driven default correction — is the
only remaining gap) and its own notes call this "opt-in only until usage
evidence exists," which makes discoverability the actual blocker to that
usage evidence ever accumulating.

**Why:** a feature that reduces standing session tax by 60-81% (130's own
measured numbers) is unreachable by any reader who only follows the README —
the single most-read entry point for a cold clone.
