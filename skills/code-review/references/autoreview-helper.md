# Autoreview Helper

`harness-kit-checks autoreview` is the structured single-bundle review helper vendored
from OpenClaw `agent-skills` and adapted for Harness Kit base-branch discovery.
It is a review engine path inside `/code-review`, not a second review owner.

Use it when a branch, commit, PR, or dirty local patch needs a compact
structured review artifact before final synthesis, especially when:

- the human asks for `autoreview`, Codex review, Claude review, or a
  second-model review;
- a non-trivial code edit needs one frozen diff bundle before closeout;
- `/code-review` wants a target-normalized local, branch, or commit review
  result to compare against roster lanes.

Do not use it to bypass the delegation floor, verdict refs, executable-path
verification, `/deliver` clean loop, or Dagger gate. A clean helper result means
only: the selected helper engine reported no accepted/actionable findings for
the reviewed bundle.

## Commands

Dirty local patch:

```bash
harness-kit-checks autoreview --mode local
```

Branch or PR work:

```bash
harness-kit-checks autoreview --mode branch --base origin/master
```

Committed change:

```bash
harness-kit-checks autoreview --mode commit --commit HEAD
```

Structured result:

```bash
harness-kit-checks autoreview --mode branch --base origin/master --json-output .evidence/review/autoreview.json
```

Optional focused tests may run in parallel with the review:

```bash
harness-kit-checks autoreview --mode local --parallel-tests "cargo test"
```

If review-triggered fixes change code, rerun the focused proof and rerun the
helper. Stop when the final helper run exits 0 with no accepted/actionable
findings, or when the lead consciously rejects remaining findings with a
reason grounded in the live code.

## Fit Rules

- Treat output as advisory. Verify every accepted finding against live files.
- Reject speculative findings, broad rewrites, and fixes that over-complicate
  the owner boundary.
- If one accepted finding reveals a bug class, inspect sibling instances in
  the current diff scope before fixing.
- Do not invoke nested review commands inside the helper review.
- Do not push just to review.
- Do not call the helper's optional panel a substitute for Harness Kit roster
  evidence; panels are extra signal only.
