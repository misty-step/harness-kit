# Clean Loop

The clean loop runs `/code-review`, `/ci`, `/refactor`, conditional
`/design` + `/a11y`, and `/qa` iteratively until all green, capped at
**3 iterations**.

## Iteration Cap

Maximum 3 iterations. No 4th. Loops without caps produce slop.

On cap-hit:
- Exit code **20** (`clean_loop_exhausted`)
- Receipt `phases[*]` records last verdict / CI tail / QA findings,
  iteration count
- Diff stays on the feature branch, unpushed, untouched — human inspects
- `state.json` records `phase.failed` on the last dirty phase; re-invoke
  without `--resume` refuses to clobber (exit 41 on merge-ready,
  explicit --resume or --abandon otherwise)

## Dirty-Detection (per phase)

A phase is **dirty** when:

| Phase | Dirty signal |
|---|---|
| `/code-review` | Receipt verdict contains `blocking` findings (severity ≥ blocking). "nit" / "consider" / "suggestion" is NOT dirty. |
| `/ci` | Non-zero exit from `/ci`. Any dagger check red. |
| `/refactor` | Non-zero exit. Clean refactor → green even if no-op. |
| `/design` | UI surface is present and design findings are unresolved, or no rendered-artifact evidence / waiver exists for a UI diff. |
| `/a11y` | UI surface is present and critical/serious accessibility findings remain unresolved. |
| `/qa` | P0 or P1 findings in its receipt. P2 does NOT block; gets recorded in receipt `remaining_work` for human attention. |

## Iteration Logic

1. Run `/code-review` → capture verdict. If dirty: dispatch a builder (or
   re-run `/implement` with the findings) to fix, then loop.
2. Run `/ci` → capture receipt. If dirty: fix (a phase that hard-fails
   structurally — e.g. missing tool — is exit 10, not dirty).
3. Run `/refactor` — skip for trivial diffs (<20 LOC, single file).
4. Check whether the diff touches UI surfaces. Prefer
   `scripts/detect-ui-surfaces.sh --base <repo-default-base>` when available
   (Harness Kit's base is `master`); otherwise use the same path patterns
   manually (`*.tsx`, `*.jsx`, `*.vue`, `*.svelte`, stylesheets, `app/**`,
   `pages/**`, `components/**`, stories, tokens, and theme config). If UI
   surfaces are present, run `/design` and `/a11y` before deciding the branch is
   clean. Record rendered evidence or an explicit repo-fit waiver. If the
   detector cannot resolve the base ref, inspect `git diff --name-only`
   manually instead of treating the detector failure as "no UI".
5. Run `/qa` — skip when the diff has no user-facing surface (pure
   library / infra / refactor).
6. If all phases are green → exit 0, `merge_ready`. Else increment iteration
   counter and repeat from step 1.

## Escalation Protocol

- **Iteration 1 dirty:** normal. Fix, loop.
- **Iteration 2 dirty:** note in receipt; fix, loop.
- **Iteration 3 dirty:** exit 20. Receipt explains what remains. Human
  handoff.
- **Fundamental re-shape needed** (detected at any iteration): stop the
  loop, exit 20 with `recommended_next: human-review` and
  `remaining_work` describing the re-shape. Do not spin the loop trying
  to fix a wrong-shaped design.
- **Hard phase failure** (tool missing, infra broken, crash): exit 10
  immediately, do not count against iteration cap. These are
  infrastructural, not "dirty output".

## What the Composer Does Not Do

- Invent a 4th iteration
- Mask a dirty phase as green
- Push on cap-hit "so the human can see it"
- Run `/qa` unconditionally on library-only diffs (judgment: if no
  runtime surface, skip)
- Treat `ui_surface:true` as a routing signal, not a verdict. `/design` and
  `/a11y` still own findings and evidence.
- Treat a detector error as unknown, not clean. Fall back to path inspection.
