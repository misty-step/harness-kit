# Mechanical hardening evidence gates

Priority: P1
Status: ready
Estimate: M

## Goal

Make Spellbook's hardening and acceptance-evidence claims mechanically harder
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

- [ ] Add a Dagger-backed check that scans workflow `SKILL.md` files and
      references for required `Completion Gate` / `Acceptance Evidence` block
      fields when those blocks are present.
- [ ] The check fails on blank, `TBD`, `unknown`, or placeholder-only evidence
      fields in committed skill templates.
- [ ] `/hardening` carries the same field names the checker recognizes.
- [ ] Shape/deliver/ship guidance defines how to carry an oracle or acceptance
      artifact hash when the workflow has one.
- [ ] `/ship` guidance refuses to land changed acceptance criteria or weakened
      assertion surfaces without an explicit contract-change acknowledgment.
- [ ] `dagger call check --source=.` passes.

## Notes

This follows the Uncle Bob hardening pattern without putting slow or
language-specific tools in the global gate. The fast Spellbook gate should
enforce that evidence claims have a real shape. Repo-local hardening workflows
remain responsible for their own property-test, mutation, CRAP/SCRAP, DRY, and
acceptance-mutation commands.

Provider research also argued for an outer acceptance loop above unit-level
TDD: fail a feature or acceptance scenario first when the work changes user
behavior, then drill down into red/green/refactor. This ticket is the mechanical
gate half of that idea; the workflow-prose half can land with the same change
or be split if it grows.
