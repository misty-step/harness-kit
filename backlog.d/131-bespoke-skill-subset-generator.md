# Generalize bespoke skill subset generation

Priority: P2 · Status: pending · Estimate: M

## Goal

Prove that Harness Kit can generate focused repo-local skill subsets for domain
agents, starting from the repo-local QA-skill pattern, without turning the
source repo into a semantic workflow engine.

## Oracle

- [ ] A prototype takes one real consumer repo and produces a repo-local
      `.agents/skills/` or equivalent subset with a focused QA or domain skill,
      using live repo commands and routes.
- [ ] The generated subset contains only portable skill folders and references;
      no source-repo `.codex/skills`, `.claude/skills`, `.pi/skills`, or other
      generated bridge directories are committed to Harness Kit.
- [ ] A cold-agent smoke verifies the generated subset by running the repo's
      actual QA/check path from the generated skill.
- [ ] The generation pattern is documented as a skill/template asset, not a
      general workflow engine around provider CLIs.
- [ ] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Verification System

- Claim: domain agents get better focused context from a generated skill subset
  than from the whole global library.
- Falsifier: the generated subset cannot run the target repo's real command; it
  copies stale generic prose; it creates committed harness bridge directories;
  or it requires provider-specific orchestration to work.
- Driver: one real consumer-repo generation run, cold-agent or scripted smoke
  through the generated skill, portable-path scan, and the Harness Kit gate.
- Grader: generated skill names exact repo commands/routes; smoke produces
  evidence in the consumer repo; Harness Kit remains free of generated bridge
  dirs; gate output is green.
- Evidence packet: generated subset diff in the consumer repo or scratch copy,
  smoke transcript, and Harness Kit template/reference diff.
- Cadence: prototype once, then promote only if telemetry shows repeated use.

## Children

1. Select the first consumer repo and domain where a focused generated skill
   beats the full library, preferably QA because the verification path is
   concrete.
2. Extract the repo-local QA-skill pattern into a reusable prompt/template under
   the owning Harness Kit skill.
3. Generate a subset for the consumer repo and run its live QA/check path.
4. Document boundaries: generated repo-local skill folders are consumer artifacts
   and should not be committed back as Harness Kit source bridges.
5. Decide whether to keep, adapt, or cut the generator based on the smoke.

## Notes

Operator decision, 2026-07-01: "Skills-that-generate-skills experiment:
repo-local QA-skill pattern generalized — domain agents get focused bespoke
skill subsets, not the whole library."

This is an experiment, not a new default install model. If one strong prompt
plus manual copy is enough, do not build a larger generator.
