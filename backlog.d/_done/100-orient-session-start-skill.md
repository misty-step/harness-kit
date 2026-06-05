# Add a session-start /orient skill

Priority: P1
Status: done
Estimate: S

## User

Senior+ engineer or lead agent opening a fresh Harness Kit session after time,
compaction, worktree switching, or unclear prior state.

## Problem / Why Now

Harness Kit has strong delivery, grooming, debrief, readiness, transcript, and
reflection primitives, but no deliberately small session-start reconnaissance
skill. Agents either over-read broad context or under-read live state, which
risks starting from stale memory instead of the current repo.

## UX Enabled

The operator can ask `/orient` or "orient yourself" and get a terse report of
the live repo state, current focus, open/closed backlog signal, roster state,
blocking gaps, and the most likely next skill to run.

## Deliverable Type

First-party Harness Kit skill with a semantic eval grader.

## Chosen Technical Design

Add `skills/orient/SKILL.md` plus `skills/orient/evals/`. Add an
`orient-session-start` grader to `harness-kit-checks eval-grader` so candidate
reports must be read-only, live-source grounded, and distinct from `/groom`,
`/debrief`, `/agent-readiness`, `/reflect`, and transcript mining.

## ADR Decision

No ADR. This is a small first-party skill and eval surface following existing
skill/eval patterns.

## Alternatives

- Add a heavy `/session-brief`: rejected for first slice because it duplicates
  `/debrief` and transcript mining.
- Add an automatic session hook: rejected for first slice because auto-loaded
  context can become noise and context tax.
- Fold into `/groom`: rejected because `/groom` explicitly says it is not an
  orientation report.

## Oracle

- [ ] `skills/orient/SKILL.md` exists with concrete `Use when:` and `Trigger:`
      phrases.
- [ ] The skill is read-only by contract and outputs a short orientation report.
- [ ] The skill names live sources: scoped `AGENTS.md`, `project.md`, git
      status/branch, recent commits, active backlog, recent done backlog, and
      roster.
- [ ] The skill explicitly avoids readiness scoring, full debriefs, transcript
      mining, state storage, provider-wrapper behavior, and workflow DSLs.
- [ ] `skills/orient/evals/` includes README, case, and grader docs.
- [ ] `cargo run --locked -p harness-kit-checks -- eval-grader orient-session-start <candidate>` accepts a good candidate and rejects ceremony/generic output.
- [ ] `cargo run --locked -p harness-kit-checks -- check-skill-evals --repo .`
      passes.
- [ ] `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`
      passes.
- [ ] `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`
      passes.
- [ ] Generated `index.yaml` and docs site are in sync.

## Evidence Artifact Plan

- Eval candidate fixtures under `/tmp` for command proof.
- Roster delegation receipts under `/tmp` for read-only critic lanes.
- Final generated index/docs diff plus command output.

## Risks

- Ceremony theater: the skill becomes mandatory throat-clearing instead of a
  useful fast read.
- Duplicate behavior: the skill expands into `/groom`, `/debrief`,
  `/agent-readiness`, or `/reflect`.
- Hidden state: the skill starts storing session memory instead of reporting
  live repo evidence.

## What Was Built

- Added first-party `skills/orient/SKILL.md` as a read-only, fast
  session-start orientation skill.
- Added `skills/orient/evals/` with README, session-start case, and grader
  docs.
- Added `orient-session-start` to `harness-kit-checks eval-grader`, including
  positive and negative Rust coverage for concrete evidence, forbidden
  readiness/transcript expansion, unknown placeholders, and shallow source-label
  reports.
- Regenerated `index.yaml` and `docs/site`.

## Verification

- `cargo test --workspace --locked eval_graders` - passed.
- `cargo run --locked -p harness-kit-checks -- eval-grader --self-test` -
  passed.
- `cargo run --locked -p harness-kit-checks -- eval-grader orient-session-start <good-candidate>` -
  passed.
- `cargo run --locked -p harness-kit-checks -- eval-grader orient-session-start <generic-candidate>` -
  rejected with `orientation output lacks concrete command/path evidence`.
- `cargo run --locked -p harness-kit-checks -- eval-grader orient-session-start <shallow-backticked-candidate>` -
  rejected with `missing required pattern: git status --short --branch`.
- `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .` -
  passed.
- `cargo run --locked -p harness-kit-checks -- check-skill-evals --repo .` -
  passed.
- `cargo run --locked -p harness-kit-checks -- check-index-drift --repo .` -
  passed.
- `cargo run --locked -p harness-kit-checks -- check-docs-site --repo .` -
  passed.
- `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .` -
  passed.
- `cargo run --locked -p harness-kit-checks -- check-runtime-primitives --repo .` -
  passed.
- `cargo run --locked -p harness-kit-checks -- check-evidence-blocks skills` -
  passed.

## Delegation Evidence

- Pi risk critic receipt `8889254a-dc83-472c-a73b-660d13bec305` found no
  blockers after the first tightening pass.
- Codex repo-fit critic receipt `5537eb5c-f0ec-4a14-ba23-6662d53c9015` found
  the grader accepted generic reports and missed two required report fields;
  both were fixed.
- Codex re-review receipt `45b0c556-a35b-4fe8-95c4-fdc0373fcc42` found the
  missing-field issue fixed but identified that shallow source-label reports
  could still pass.
- Codex final re-review receipt `b7bdaa55-d2c6-4031-a1bf-014b551aef78` found no
  blockers after the grader required concrete status/log/backlog evidence.

Closes-backlog: 100
