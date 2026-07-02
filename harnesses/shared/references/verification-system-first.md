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
| API/service | representative request replay, contract assertions, local third-party API emulation when supported, error-path cases, logs |
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

## Live-Diff For Behavior-Preserving Refactors

When the oracle is "identical to before" and the target has no
characterization tests to pin current behavior (extract-to-module,
route-thinning, dedup, lifts), "unit tests pass" cannot catch the seam a lift
most often breaks — the integration between the refactored layer and its real
dependencies, which unit tests mock away.

**Technique:** exercise the same representative inputs against (a) the local
refactor branch and (b) the deployed or pre-refactor build, both pointed at
the same backing store. Diff responses byte-for-byte, including error and
not-found paths, not just the happy path. Identical responses across the set
= behavior preserved; any divergence is the bug, located precisely at the
diverging input. In verification-system terms: the deployed/pre-refactor
build is the grader's reference oracle; the diff itself is the falsifier.

**Precondition:** the pre-refactor behavior must still be runnable against the
same data — a deployed prod instance, a pinned build, or `git stash` + rerun
locally.

**Does not apply when:** the refactor is meant to change output (pin a golden
instead); or for write-path side effects, since a read-only diff says nothing
about them — diff those via post-state reads or a transaction-scoped probe.

**Pair, don't replace.** Live-diff is the integration net; keep
repository/unit tests as the structural net underneath it.

Proven live 2026-06-17/18 on a 6-route rewrite with zero route-level tests:
repository unit tests plus a before/after live diff (local branch vs deployed
prod, same backing store → byte-identical list/detail/404 responses across
representative reads) was the only thing that actually proved the rewrite
preserved behavior.

## Design Rules

- **Falsifiability first.** A system that passes when the values are wrong is
  theater. Mutate a fixture, route, expected value, or threshold when cheap to
  prove the check can fail.
- **Live before decorative.** A beautiful report is worthless if no driver
  exercised the changed surface.
- **Repo-shaped, not tool-shaped.** Start from the app shape and operator
  workflow, then choose browser, shell, HTTP, eval, benchmark, or monitor
  tools.
- **Emulate supported third-party APIs locally.** When `emulate.dev` covers the
  provider, the driver can be an emulator plus representative request replay;
  record services, ports, seed file, reset/teardown, and the docs used
  (https://emulate.dev/docs).
- **Leave receipts.** The evidence packet is part of the deliverable. Future
  agents should not need chat context to judge the claim.
- **Escalate recurring checks.** A repeated manual QA path becomes a repo-local
  verification skill, script, gate, benchmark, or Mode B loop.
- **Do not weaken gates.** If the current system is too slow, split fast and
  heavyweight lanes; do not delete the only proof that catches the failure.

## Eval & Benchmark Rigor

The grader is half the system; the *numbers* mislead unless you size and read
them honestly. A strong model will report "3.8 → 4.1, improved" off twelve
samples — don't.

**Statistical honesty.**
- No score without a confidence interval. For a rate, `SE = sqrt(p*(1-p)/n)`,
  95% CI `±1.96*SE`. A paired delta whose CI includes 0 is noise, not a result.
  (Miller, "Adding Error Bars to Evals", arXiv 2411.00640)
- Right-size n. Detecting a ~3% absolute change at 80% power needs ~1000 items;
  a ~12-item suite catches only large regressions — read small deltas as noise
  and say so. (Miller)
- Compare versions paired on identical items (McNemar / paired bootstrap), not
  as two independent rates; pairing is free variance reduction. (Miller)
- Cluster the SE by source when many graded items share one document — naive
  independence understated uncertainty >3x on real benchmarks. (Miller)
- Average K samples per item for nondeterministic graders (K=2 cuts variance
  ~1/3); never lower temperature to fake stability — that trades variance for
  bias. (Miller)

**Judge validity (model graders).**
- Validate the judge against human labels before trusting it: target Cohen's
  κ ≈ 0.80; report TPR/TNR (not raw % agreement) under class imbalance, then
  bias-correct the rate. (Hamel Husain, evals-faq; "Judge's Verdict", arXiv
  2510.09738)
- Binary pass/fail per atomic criterion, not a 1–5 Likert — Likert doesn't
  track expert judgment and isn't actionable. Have the judge write its rationale
  before the verdict. (Hamel Husain, llm-judge; G-Eval, arXiv 2303.16634)
- Judge model ≠ generator family (self-enhancement is +10–25% win rate);
  reference-guide objective items; one judge per dimension; give an
  "insufficient info" escape hatch. (Zheng et al., MT-Bench, arXiv 2306.05685;
  Anthropic, Demystifying Evals)
- A ~100% pass rate means the eval is too weak; aim where it bites (~70%).

**Anti-Goodhart.** A judge or threshold, once optimized against, stops
measuring. Keep a hidden held-out split and select by it (≈60/40); diversify and
rotate sources; n-gram-screen new fixtures for contamination; turn every shipped
defect into a permanent fixture-backed case so it cannot silently regress.

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
