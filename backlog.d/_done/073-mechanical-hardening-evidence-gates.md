# Mechanical hardening evidence gates

Priority: P1
Status: ready
Estimate: M

## Goal

Make Harness Kit's hardening and acceptance-evidence claims mechanically harder
to fake. A workflow that says a surface is accepted, hardened, or merge-ready
should carry non-empty evidence fields, and later phases should detect when the
oracle or acceptance artifact changed without an explicit contract-change note.

## Non-Goals

- Do not add a semantic workflow engine.
- Do not make mutation testing part of the default fast CI path.
- Do not mandate one property-testing, mutation-testing, CRAP, or DRY tool
  across all languages.
- Do not store raw provider transcripts, secrets, or private runtime artifacts
  in committed files.

## Oracle

- [x] Add a Dagger-backed check that scans workflow `SKILL.md` files and
      references for required `Completion Gate` / `Acceptance Evidence` block
      fields when those blocks are present.
- [x] The check fails on blank, `TBD`, `unknown`, or placeholder-only evidence
      fields in committed skill templates.
- [x] `/hardening` carries the same field names the checker recognizes.
- [x] Shape/deliver/ship guidance defines how to carry an oracle or acceptance
      artifact hash when the workflow has one.
- [x] `/ship` guidance refuses to land changed acceptance criteria or weakened
      assertion surfaces without an explicit contract-change acknowledgment.
- [x] `dagger call check --source=.` passes.

## Notes

This follows the Uncle Bob hardening pattern without putting slow or
language-specific tools in the global gate. The fast Harness Kit gate should
enforce that evidence claims have a real shape. Repo-local hardening workflows
remain responsible for their own property-test, mutation, CRAP/SCRAP, DRY, and
acceptance-mutation commands.

Provider research also argued for an outer acceptance loop above unit-level
TDD: fail a feature or acceptance scenario first when the work changes user
behavior, then drill down into red/green/refactor. This ticket is the mechanical
gate half of that idea; the workflow-prose half can land with the same change
or be split if it grows.

## Progress

- Added `scripts/check-evidence-blocks.py` to scan committed skill markdown
  for `Completion Gate` and `Acceptance Evidence` blocks, require canonical
  evidence fields, and reject blank/TBD/unknown/placeholder-only field values.
- Added `check-evidence-blocks` to the Dagger pipeline and covered parser,
  missing-field, placeholder, `Acceptance Evidence`, fenced-template, and h3
  report-template behavior in Rust `evidence_blocks` tests.
- Updated committed Completion Gate templates across workflow skills to carry
  non-empty evidence descriptors using the shared field names recognized by
  the checker.
- Added shape/deliver/ship guidance for oracle or acceptance artifact hashes,
  plus `/ship` refusal language for changed acceptance criteria or weakened
  assertion surfaces without `Contract-change acknowledgment:`.

## Delegation Evidence

- Initial design critics: `claude` receipt
  `1f173967-bd87-4d30-917e-f666012aefc2` and `grok-build` receipt
  `6123d24a-04a3-420a-aef5-3433f2888a1a`. Accepted the shared-field and
  placeholder-set risks; rejected renaming `/hardening`'s Completion Gate to
  Acceptance Evidence because the common report boundary remains useful.
- Final diff critics: `grok-build` receipt
  `8c73b89c-a6ad-43ed-9726-9634d7fe297d` and `claude` receipt
  `efdb598c-c6c7-42f8-a10c-bc35b11555b4`. Both returned `BLOCKING: no`.
  Accepted the h3 parser gap and committed a follow-up parser/test fix before
  rerunning verification.

## Verification

- `cargo test --workspace --locked evidence_blocks`
- `cargo run --locked -p harness-kit-checks -- check-evidence-blocks skills`
- `python3 -m py_compile ci/src/harness_kit_ci/main.py`
- `cargo test --workspace --locked`
- `cargo run --locked -p harness-kit-checks -- build-docs-site`
- `cargo run --locked -p harness-kit-checks -- check-docs-site --repo .`
- `dagger call check --source=.` -> `16 passed, 0 failed`
