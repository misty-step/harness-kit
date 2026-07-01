# Cull zero-use vendored skills from the default catalog

Priority: P0 · Status: ready · Estimate: M

## Goal

Remove the zero-use, zero-reference vendored skill imports from the default
Harness Kit catalog so every projected session pays less context tax without
losing a capability that live repo evidence shows is routed or used.

## Oracle

- [ ] `registry.yaml` no longer syncs the mattpocock suite, `steipete-skill-cleaner`,
      `openai-gh-address-comments`, `openai-gh-fix-ci`, `petekp-grill-me`, or
      `every-ce-dogfood-beta` as active default externals unless new live
      telemetry/reference evidence is added in the PR.
- [ ] `skills/.external/` no longer contains generated vendored copies for the
      culled aliases after a full `cargo run --locked -p harness-kit-checks --
      sync-external --repo .` run.
- [ ] `index.yaml` and `docs/site/` are regenerated from the post-cull source
      tree, and `cargo run --locked -p harness-kit-checks -- check --repo .`
      passes.
- [ ] The PR body or receipt reports before/after counts for installed skills,
      vendored externals, and generated catalog description bytes.
- [ ] Any referenced but culled external has a first-party replacement path or a
      visible follow-up note in this ticket.

## Verification System

- Claim: deleting the named default external imports reduces always-loaded skill
  catalog weight without removing a used or routed capability.
- Falsifier: a first-party skill, active reference, generated index entry, or
  telemetry row still depends on a removed alias; `sync-external --check` would
  re-create removed files; the repo gate fails.
- Driver: reference graph scan with `rg`, telemetry report, full
  `sync-external`, generated index/docs refresh, and the Harness Kit aggregate
  gate.
- Grader: removed aliases are absent from `registry.yaml`, `skills/.external/`,
  `index.yaml`, and docs output; retained routed externals still appear; gate
  output is green.
- Evidence packet: PR diff plus a short `.evidence/` receipt or PR section with
  before/after counts and the exact commands run.
- Cadence: pre-merge for this cull, then repeated whenever the skill half-life
  ratchet in `128` marks another import for quarantine.

## Children

1. Measure the current catalog: installed first-party count, vendored external
   count, generated description bytes, and reference graph for all kill-list
   aliases.
2. Remove or deactivate the kill-list sources in `registry.yaml`; prefer
   deletion for recoverable upstream imports unless a follow-up needs a dormant
   `active: false` record.
3. Run full external sync so orphaned generated skill directories are pruned
   by tooling instead of hand-edited.
4. Regenerate `index.yaml` and `docs/site/`; update any first-party references
   that still point at removed aliases.
5. Run the repo gate and report before/after catalog weight in the PR.

## Notes

Operator decision, 2026-07-01: "Cull the ~30 zero-use vendored skills
(mattpocock suite etc.) — delete now, recoverable from upstream."

The groom teardown identified the safe cull set as the mattpocock suite,
`steipete-skill-cleaner`, `openai-gh-*`, `petekp-grill-me`, and
`every-ce-dogfood-beta`. Keep routed design-bench externals, `julius-caveman`,
`dietrich-ponytail`, `cursor-thermo-nuclear-code-quality-review`, and
`nous-creative-ideation` unless fresh evidence says otherwise.

Do not hand-edit generated external skill directories as the primary mechanism.
`registry.yaml` is the source of truth and `sync-external` owns
`skills/.external/`.
