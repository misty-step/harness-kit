# Add role-scoped bootstrap bundles

Priority: P1 · Status: pending · Estimate: L

## Goal

Let bootstrap project a small, role-appropriate subset of Harness Kit skills
instead of loading the entire catalog into every session by default.

## Oracle

- [ ] A bundle manifest or generated config defines at least five role-scoped
      bundles (`lead`, `implementer`, `critic`, `designer`, `ops` or equivalent)
      with 8-15 default skill descriptions each.
- [ ] `bootstrap` can install a selected bundle for a system or repo harness
      without losing the current full-catalog mode.
- [ ] A dry-run or fixture bootstrap reports before/after projected skill count
      and estimated description bytes.
- [ ] Existing detected harness installs keep working with the default behavior,
      or the migration is explicit and reversible.
- [ ] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Verification System

- Claim: role-scoped projection reduces session description tax while preserving
  a reversible full-catalog path for broad operator sessions.
- Falsifier: bootstrap cannot reproduce the selected bundle; a role bundle omits
  a required first-party dependency; default installs unexpectedly shrink; the
  generated index/docs disagree with projected skills.
- Driver: bootstrap fixture/dry run for each role, generated count/byte report,
  install-path checks, and the aggregate repo gate.
- Grader: bundle manifests resolve to existing skills; dry-run output shows the
  intended count reduction; default mode remains available; gate output is
  green.
- Evidence packet: bundle manifest diff, dry-run transcripts, and before/after
  count table.
- Cadence: pre-merge for each bundle change, then telemetry review after real
  sessions use bundles.

## Children

1. Decide the minimal role vocabulary from Weave/Factory usage rather than
   creating persona theater.
2. Add bundle declarations that are data, not duplicated skill prose.
3. Teach bootstrap to select a bundle for system and repo harness projections
   while preserving full mode.
4. Add a dry-run/count report so token savings are visible before installation.
5. Validate bundles against telemetry and update defaults only after evidence.

## Notes

Operator decision, 2026-07-01: "Role-scoped bootstrap bundles (~12k
tokens/session -> target <=5k)."

This extends, rather than reverts, the older skill-catalog tailoring work. Keep
the interface smaller than the implementation; avoid one-off hardcoded harness
branches when a manifest plus bootstrap selection is enough.
