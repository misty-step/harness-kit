# HarnessX Evaluation Design

Backlog: `110`
Status: review-only eval artifact; no code added, so the focused Rust
`harnessx` parser/checker test is explicitly waived for this slice.

## Candidate Primitive Family

Skill frontmatter descriptions and reference routing lines. They are typed
enough to diff and test, but small enough to review manually.

## Trace Schema

Use sanitized existing artifacts only:

- skill invocation analytics summary;
- delegation receipt summaries;
- failed review receipts;
- generated docs/index drift results.

Do not include raw private transcripts, secrets, prompts containing customer
data, or unredacted provider logs.

## Held-Out Task Set

1. A skill-trigger collision fixture.
2. A missing reference-routing fixture.
3. An over-broad external skill import fixture.
4. A stale generated-docs fixture.
5. A no-op edit negative control.

## Baseline and Candidate

- Baseline model: current open-model roster lane used for cheap critique.
- Candidate model: the same lane plus a trace-proposed patch.
- Strong-model guard: current lead/critic flow must not regress on the same
  held-out tasks.

## Scorer

A candidate only passes if it:

- improves at least one held-out weak-lane outcome;
- does not regress any negative control;
- passes `cargo run --locked -p harness-kit-checks -- check --repo .`;
- receives `BLOCKING: no` from a fresh critic that sees only patch plus oracle.

## Safety Gate

- No self-merging edits.
- Human review before any source diff is accepted.
- Candidate patches are produced as review artifacts, not applied by the eval
  worker.
- Rollback is deleting the candidate patch/artifact; no runtime state changes.

## Dry-Run Candidate Patch

Not applied. Predicted first candidate: tighten one skill frontmatter trigger
description where analytics show under-triggering or collision, then compare
held-out trigger fixtures before and after.
