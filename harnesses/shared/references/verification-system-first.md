# Verification System First

A verification system is the repeatable loop that can prove the work wrong.
It is not a confidence phrase, a checklist, or a green command by itself.

Use this reference when shaping, delivering, refactoring, QAing, designing
evals, writing benchmarks, or changing harness primitives.

## Contract

Before implementation, name the smallest credible system that will decide
whether the work actually works:

1. **Claim:** the behavior, quality, or operator outcome that must be true.
2. **Falsifier:** the concrete failure the system would catch.
3. **Driver:** command, route, browser walk, request replay, fixture runner,
   benchmark, eval, migration dry run, consumer build, or production probe.
4. **Grader:** exact assertion, rubric, golden, threshold, human calibration
   note, or observed artifact that turns the driver into pass/fail evidence.
5. **Evidence packet:** screenshots, transcripts, logs, request/response pairs,
   benchmark output, eval report, verdict, or receipt path another agent can
   inspect later.
6. **Cadence:** when it runs: before edits, after each milestone, pre-merge,
   post-ship, or on a recurring Mode B loop.

If the repo has no system for the changed surface, building or naming that
system is the first milestone. A feature shipped before its proof loop is a
guess with a diff.

## What Counts

| surface | verification system |
|---|---|
| Web UI | dev/preview URL, scripted or manual browser path, console/network check, screenshots or video |
| API/service | representative request replay, contract assertions, error-path cases, logs |
| CLI | documented happy path, malformed-input path, exit codes, stderr/stdout checks |
| Library/SDK | consumer build or throwaway install that exercises the public API |
| MCP/agent tool | harness registration plus replayed tool calls and structured-error checks |
| Model/agent behavior | held-out task, transcript, grader, rubric calibration, and outcome artifact |
| Performance | benchmark with workload, baseline, threshold, variance note, and raw output |
| Migration/data | dry run, fixture snapshot, rollback path, and invariant checks |
| Ops/monitoring | health/readiness/log/metric/alert probe tied to the changed behavior |

Use multiple systems when one boundary cannot see the failure. Unit tests,
typechecks, and lint catch structural regressions; QA, evals, benchmarks, and
probes catch failures at runtime, judgment, scale, or integration boundaries.

## Design Rules

- **Falsifiability first.** A system that passes when the values are wrong is
  theater. Mutate a fixture, route, expected value, or threshold when cheap to
  prove the check can fail.
- **Live before decorative.** A beautiful report is worthless if no driver
  exercised the changed surface.
- **Repo-shaped, not tool-shaped.** Start from the app shape and operator
  workflow, then choose browser, shell, HTTP, eval, benchmark, or monitor
  tools.
- **Leave receipts.** The evidence packet is part of the deliverable. Future
  agents should not need chat context to judge the claim.
- **Escalate recurring checks.** A repeated manual QA path becomes a repo-local
  verification skill, script, gate, benchmark, or Mode B loop.
- **Do not weaken gates.** If the current system is too slow, split fast and
  heavyweight lanes; do not delete the only proof that catches the failure.

## Minimum Artifact

Every substantial plan or closeout should include:

```markdown
Verification system:
- Claim:
- Falsifier:
- Driver:
- Grader:
- Evidence packet:
- Cadence:
- Gaps / waiver:
```

For tiny mechanical changes, a focused structural gate or exact inspection can
be enough, but the closeout still names why no live loop was needed.

## Failure Modes

- **Green aggregate:** "tests passed" with no route, command, artifact, or
  changed surface named.
- **Eval-shaped directory:** folders and prompts with no grader or held-out
  task.
- **Benchmark theater:** one run, no baseline, no variance note, no threshold.
- **QA anecdote:** "looked good" with no screenshot, transcript, or path.
- **Instrumentation debt:** no post-ship signal would reveal the behavior
  breaking.
- **Author-only judgment:** the same context that built the work also grades
  the open-ended outcome without held-out artifacts or fresh critique.
