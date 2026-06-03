# Close the review score → skill evolution feedback loop

Priority: medium
Status: pending
Estimate: M

## Goal

Turn `.groom/review-scores.ndjson` from passive data into an active feedback
loop that improves review quality over time. When review patterns emerge
(repeated low scores in a dimension, persistent false positives), surface
them and suggest skill instruction changes.

## Design

1. **Score enforcement**: `/code-review` MUST append to review-scores.ndjson
   after every review (currently "wired but operationally empty")
2. **Trend detection**: `/groom` Velocity investigator analyzes score trends
   and flags regressions (e.g., "correctness scores declining over last 5 reviews")
3. **Skill tuning suggestions**: When a pattern is detected, `/reflect` proposes
   concrete skill instruction changes (not just observations)
4. **Calibration**: Periodically compare agent review findings against actual
   bugs found post-merge to measure review effectiveness

## Oracle

- [x] Every `/code-review` invocation appends a score entry
- [x] `/groom` reports score trends when 5+ entries exist
- [x] `/reflect` proposes skill changes based on score patterns
- [x] False positive rate is tracked (reviews that flagged non-issues)

## Non-Goals

- Automatic skill modification (human approves changes)
- External dashboards

## What Was Built

- Added `scripts/review-score-trends.py`, a local NDJSON analyzer for
  `.groom/review-scores.ndjson` that reports schema coverage, 5-entry score
  trends, false-positive rate, and concrete skill-tuning targets.
- Added `check-review-score-trends` to the Dagger gate via the analyzer's
  `--self-test`.
- Tightened `/code-review` scoring so the row is mandatory and includes
  `branch`, `sha`, finding counts, false-positive counts, and post-merge bug
  calibration.
- Wired `/groom` and its Velocity investigator to run the analyzer and avoid
  trend claims before 5 entries exist.
- Wired `/reflect` and prompt-debt guidance to turn analyzer-reported score
  patterns into concrete skill/reference proposals.

## Delegation Evidence

- `claude` (`a1d875d1-e382-4357-9384-fc8e9b24c483`) identified enforcement,
  `/reflect` consumption, and 5-entry trend reporting as the missing work after
  `011`; accepted. Its separate findings-file recommendation was deferred as
  heavier than the current oracle.
- `grok-build` (`a3092900-0838-4ba6-bca8-6672b2b21097`) also recommended a
  small script/check, richer score schema, and `/groom`/`/reflect` consumption;
  accepted. Its pre-merge enforcement suggestion was deferred because this
  ticket only needs trend feedback, not a new merge blocker.

## Verification

- `python3 scripts/review-score-trends.py --self-test`
- `python3 scripts/review-score-trends.py .groom/review-scores.ndjson`
- `python3 -m py_compile scripts/review-score-trends.py ci/src/harness_kit_ci/main.py`
- `python3 scripts/check-frontmatter.py`
- `python3 scripts/check-agent-roster.py`
- `bash scripts/build-docs-site.sh`
- `bash scripts/generate-index.sh`
- `bash scripts/check-docs-site.sh`
- `git diff --check`
- `dagger call check --source=.` -> 17 passed, 0 failed

## Completion Gate

- Exact end-user behavior changed: `/code-review` now creates calibrated review
  score rows, `/groom` reports trends only when enough data exists, and
  `/reflect` proposes concrete skill changes from repeated score patterns.
- Evidence that proves it: analyzer self-test passes and Dagger includes
  `check-review-score-trends`.
- Exact command/path/route exercised: `python3 scripts/review-score-trends.py
  --self-test` and `.groom/review-scores.ndjson` legacy read.
- Oracle / acceptance artifact hash:
  `f70eff0dff81c97672357b5e2866ed2dc5fad02987bffd376bd6197f4cc91618`
  for this backlog item before closeout edits.
- Contract-change acknowledgment: changed the required `/code-review` score row
  schema by adding branch/sha and finding calibration fields.
- Repo-fit check: follows the existing `.groom/review-scores.ndjson` artifact,
  `/groom` Velocity lane, `/reflect` prompt-debt codification hierarchy, and
  generated docs/index flow.
- Hardening run / waiver: no `/hardening` run; the new analyzer self-test and
  Dagger lane cover the changed trend-detection invariant.
- Residual risk: false-positive counts are still human-calibrated; the script
  reports rates but does not prove whether a finding was semantically false.
