# Scale skill-eval integration across first-party skills

Priority: P0 · Status: in-progress · Estimate: L

## Goal

Make skill retention evidence-based by requiring every first-party skill to
carry an eval claim, runnable or waived proof surface, and telemetry-aware
disposition path.

## Oracle

- [x] Every first-party `skills/*/SKILL.md` has either
      `skills/<skill>/evals/<skill>-eval.md` or an explicit waiver file that
      names why no runnable eval exists yet and when the waiver expires.
      (24/24 first-party skills covered — PR #143.)
- [x] The new-skill template or creation path includes the eval scaffold from
      `skills/skill-eval/templates/eval-spec.md`. (Documented as a required
      step in `skill-design-principles.md`'s "New Skill: Eval Scaffold Is Not
      Optional" section and enforced by the gate below — PR #143.)
- [x] `harness-kit-checks` exposes a check or report that lists first-party
      skills missing eval coverage or carrying expired waivers.
      (`check-eval-coverage`, folded into `check` — PR #143.)
- [x] Telemetry analysis can distinguish direct slash-command invocations from
      routed skill use well enough that routed helpers are not falsely marked
      unused. (`invocation_kind` classification + per-skill
      direct/routed/unknown breakdown — PR #144.)
- [x] At least the hot first-party skills from the teardown (`deliver`,
      `code-review`, `research`, `shape`, `groom`, `design`,
      `harness-engineering`, `next`, `refactor`, `diagnose`, `ci`) have seeded
      eval specs or waivers. (`shape`/`design` already had specs;
      `deliver`/`code-review`/`research` got new seeded specs, plus `orient`
      as a bonus per the operator's hot-5 framing; the rest got time-boxed
      waivers — PR #143.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
      (Verified independently on each PR's branch, not just combined —
      PR #143, PR #144.)

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

1. [x] Define the minimal eval coverage contract for first-party skills using
   `skills/skill-eval` and `skills/harness-engineering/references/mode-eval.md`.
   (PR #143: eval spec or time-boxed `WAIVER.md`.)
2. [x] Wire a coverage report or gate-adjacent warning in `harness-kit-checks`.
   (PR #143: `eval_coverage::check_eval_coverage`, folded into `check`.)
3. [x] Extend the new-skill template/path so eval scaffolding is created with the
   skill, not as an afterthought. (PR #143: documented + gate-enforced; no
   literal `new-skill` scaffold generator exists in this repo yet, so
   enforcement is via the gate rejecting any skill with neither an eval nor a
   waiver, not a template stub.)
4. [x] Fix the routed-invocation telemetry blind spot so skills consumed by
   `/design`, `/groom`, `/shape`, or other routers are visible as routed use.
   (PR #144: `invocation_kind` classifier + per-skill breakdown.)
5. [x] Seed eval specs or time-boxed waivers for the hot first-party skills.
   (PR #143.)
6. [ ] Convert `backlog.d/112-harness-eval-bench.md` from a one-shot pilot into a
   tracked input or close it only after the coverage loop here owns the ongoing
   cadence. **Not yet done** — the coverage loop (children 1-5) now exists and
   `skill-eval/SKILL.md`'s Cadence section documents the ongoing re-audit
   triggers (edit-time smoke, contract-change full run, model-release
   re-audit), but no actual multi-condition bench run has happened at scale
   beyond the single `/shape` pilot from PR #139. 112 should stay open as the
   "run real evidence at scale" input; do not close it on this PR pair alone.

## Notes

Operator decision, 2026-07-01: "Skill-eval integration: every first-party skill
has at least one eval; new-skill template includes it; wire the routed-invocation
telemetry blind spot fix."

PR #139 added the `skill-eval` skill and the first `/shape` eval evidence. This
epic scales that mechanism across the catalog and gives it teeth.

**2026-07-01 — PR #143 + #144 merged.** Children 1, 2, 3, 4, 5 landed;
child 6 remains open (see above). Remaining follow-on work for a future pass:
run real A/B evidence (not just seeded specs) for `deliver`, `code-review`,
`research`, `orient`; renew/replace the 18 time-boxed waivers as they expire
(earliest: `roster`/`next` 2026-08-01); consider whether a literal
`new-skill` scaffold command is worth building versus the current
documented-process + gate-enforcement approach.
