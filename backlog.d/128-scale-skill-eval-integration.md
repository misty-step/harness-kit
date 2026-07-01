# Scale skill-eval integration across first-party skills

Priority: P0 · Status: pending · Estimate: L

## Goal

Make skill retention evidence-based by requiring every first-party skill to
carry an eval claim, runnable or waived proof surface, and telemetry-aware
disposition path.

## Oracle

- [ ] Every first-party `skills/*/SKILL.md` has either
      `skills/<skill>/evals/<skill>-eval.md` or an explicit waiver file that
      names why no runnable eval exists yet and when the waiver expires.
- [ ] The new-skill template or creation path includes the eval scaffold from
      `skills/skill-eval/templates/eval-spec.md`.
- [ ] `harness-kit-checks` exposes a check or report that lists first-party
      skills missing eval coverage or carrying expired waivers.
- [ ] Telemetry analysis can distinguish direct slash-command invocations from
      routed skill use well enough that routed helpers are not falsely marked
      unused.
- [ ] At least the hot first-party skills from the teardown (`deliver`,
      `code-review`, `research`, `shape`, `groom`, `design`,
      `harness-engineering`, `next`, `refactor`, `diagnose`, `ci`) have seeded
      eval specs or waivers.
- [ ] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Verification System

- Claim: first-party skills now have falsifiable retention evidence instead of
  surviving on plausible prose or historical taste.
- Falsifier: a first-party skill lacks an eval/waiver; a generated new skill can
  be created without an eval scaffold; routed usage remains invisible enough to
  make the cull ratchet unsafe.
- Driver: eval-coverage check/report, fixture skill creation from the template,
  telemetry fixture covering routed invocation records, and the aggregate repo
  gate.
- Grader: missing/expired eval coverage is reported deterministically; template
  output includes eval scaffolding; telemetry fixture classifies direct and
  routed records separately; gate output is green.
- Evidence packet: eval coverage report, fixture output, telemetry fixture
  transcript, and PR diff.
- Cadence: pre-merge for the integration, then on every skill edit and after
  major model releases.

## Children

1. Define the minimal eval coverage contract for first-party skills using
   `skills/skill-eval` and `skills/harness-engineering/references/mode-eval.md`.
2. Wire a coverage report or gate-adjacent warning in `harness-kit-checks`.
3. Extend the new-skill template/path so eval scaffolding is created with the
   skill, not as an afterthought.
4. Fix the routed-invocation telemetry blind spot so skills consumed by
   `/design`, `/groom`, `/shape`, or other routers are visible as routed use.
5. Seed eval specs or time-boxed waivers for the hot first-party skills.
6. Convert `backlog.d/112-harness-eval-bench.md` from a one-shot pilot into a
   tracked input or close it only after the coverage loop here owns the ongoing
   cadence.

## Notes

Operator decision, 2026-07-01: "Skill-eval integration: every first-party skill
has at least one eval; new-skill template includes it; wire the routed-invocation
telemetry blind spot fix."

PR #139 added the `skill-eval` skill and the first `/shape` eval evidence. This
epic scales that mechanism across the catalog and gives it teeth.
