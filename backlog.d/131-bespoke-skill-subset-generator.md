# Generalize bespoke skill subset generation

Priority: P2 · Status: done · Estimate: M

## Goal

Prove that Harness Kit can generate focused repo-local skill subsets for domain
agents, starting from the repo-local QA-skill pattern, without turning the
source repo into a semantic workflow engine.

## Oracle

- [x] A prototype takes one real consumer repo and produces a repo-local
      `.agents/skills/` or equivalent subset with a focused QA or domain skill,
      using live repo commands and routes. (Two: `canary-deploy` —
      misty-step/canary#182 — and `powder-qa` — misty-step/powder#12.)
- [x] The generated subset contains only portable skill folders and references;
      no source-repo `.codex/skills`, `.claude/skills`, `.pi/skills`, or other
      generated bridge directories are committed to Harness Kit. (Both PRs add
      only `.agents/skills/<repo>-<domain>/**`; Harness Kit itself carries only
      the reference + template under `skills/harness-engineering/`.)
- [x] A cold-agent smoke verifies the generated subset by running the repo's
      actual QA/check path from the generated skill. (Both: fresh-context
      subagents, zero session memory of the generation work, ran the skills'
      exact commands verbatim and passed — `canary-deploy` against the real
      production `canary-obs` Fly app, `powder-qa` through the full 12-step CLI
      lifecycle. See both PRs' eval-stub run logs for full transcripts.)
- [x] The generation pattern is documented as a skill/template asset, not a
      general workflow engine around provider CLIs.
      (`skills/harness-engineering/references/repo-local-skill-generation.md`.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

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

1. [x] Select the first consumer repo and domain where a focused generated skill
   beats the full library, preferably QA because the verification path is
   concrete. (Canary's hand-authored `.agents/skills/canary-qa/SKILL.md` is
   the exemplar this generalizes from — see child 2's reference doc's
   "Exemplar" section. Two live proof repos chosen: canary — a second domain,
   `canary-deploy`, since QA is already proven there — and powder — first
   domain, `powder-qa`, since it has none yet.)
2. [x] Extract the repo-local QA-skill pattern into a reusable prompt/template under
   the owning Harness Kit skill.
   (`skills/harness-engineering/references/repo-local-skill-generation.md` +
   `skills/harness-engineering/templates/repo-local-skill/` — a documented
   judgment process + copy-and-fill templates, explicitly NOT a Rust
   verb/workflow engine; the reference names why, citing the two prior
   `/tailor`/`/focus` retirements for over-engineering the same territory.)
3. [x] Generate a subset for the consumer repo and run its live QA/check path.
   (`canary-deploy` — misty-step/canary#182 — a second domain, additive
   alongside the existing `canary-qa`; `powder-qa` — misty-step/powder#12 —
   powder's first repo-local skill. Both cold-agent smokes PASS; both agents
   self-reported zero guessing/invention was needed.)
4. [x] Document boundaries: generated repo-local skill folders are consumer artifacts
   and should not be committed back as Harness Kit source bridges.
   (Reference doc's "Name and place the file" + "Anti-goals" sections.)
5. [x] Decide whether to keep, adapt, or cut the generator based on the smoke.
   **Verdict: keep, as-is — no scope growth.** Both cold-agent smokes passed on
   the first try with the skill as the *only* input (no guessing, no digging
   outside the file, per both agents' explicit self-reports); the one real gap
   either smoke surfaced (`powder-qa`: `init-db --show-secret` prints the
   bootstrap key) was cheap enough to fold straight back into the generated
   skill's own Gotchas rather than into the generator process. Nothing in
   either proof motivated more machinery (no manifest, no A/B eval, no lint
   hook) — the boundary section's bet holds. Re-open this ticket only if a
   future repo needs a domain with no drivable oracle, or if more than ~3
   generated skills start accumulating in one repo (the signal to reach for a
   role-scoped bundle instead, per the reference doc).

## Notes

Operator decision, 2026-07-01: "Skills-that-generate-skills experiment:
repo-local QA-skill pattern generalized — domain agents get focused bespoke
skill subsets, not the whole library."

This is an experiment, not a new default install model. If one strong prompt
plus manual copy is enough, do not build a larger generator.

**2026-07-02 — generator asset landed** (harness-kit#151). Also introduces a
provenance-header + eval-stub convention for generated repo-local skills (the
evals-per-skill floor, backlog 128, extended past Harness Kit's own catalog)
that `canary-qa` predates and is not retrofitted with — it stays as its
authoring lane committed it.

**2026-07-02 — proved on canary + powder, closing.** `canary-deploy`
(misty-step/canary#182) and `powder-qa` (misty-step/powder#12) both generated,
both cold-agent-smoked PASS, both PRs open with the target repo's own gate
green (canary: `./bin/validate --strict`; powder: local `cargo fmt`/CLI
lifecycle, matching its CI). Keep verdict recorded in child 5. This experiment
ticket is done; the pattern lives on as documented process
(`skills/harness-engineering/references/repo-local-skill-generation.md`), not
as a new standing default — future repos generate on demand, not via any
new install-time step.
