# Clean Loop

The clean loop runs `/code-review`, `/ci`, `/refactor`, conditional
`/design` + `/a11y`, `/qa`, and `/demo` iteratively until all green, capped at
**3 iterations**, then a final **hindsight sanity pass**, verdict freshness
check, and bounded `/reflect` pass before declaring merge-ready.

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
| `/demo` | Missing `.evidence/<branch>/<date>/evidence-index.md`, missing demo/text artifact, or artifact does not prove the changed behavior. |
| `/reflect` | Missing learning packet or packet lacks codification/backlog/skillify/memory/non-action disposition. |

## Iteration Logic

1. Run `/code-review` → capture verdict. If dirty: dispatch a builder (or
   re-run `/implement` with the findings) to fix, then loop.
2. Run `/ci` → capture receipt. If dirty: fix (a phase that hard-fails
   structurally — e.g. missing tool — is exit 10, not dirty).
3. Run `/refactor` — skip for trivial diffs (<20 LOC, single file).
4. Check whether the diff touches UI surfaces. Prefer
   `cargo run --locked -p harness-kit-checks -- detect-ui-surfaces --base <repo-default-base>` when available
   (Harness Kit's base is `master`); otherwise use the same path patterns
   manually (`*.tsx`, `*.jsx`, `*.vue`, `*.svelte`, stylesheets, `app/**`,
   `pages/**`, `components/**`, stories, tokens, and theme config). If UI
   surfaces are present, run `/design` and `/a11y` before deciding the branch is
   clean. Record rendered evidence or an explicit repo-fit waiver. If the
   detector cannot resolve the base ref, inspect `git diff --name-only`
   manually instead of treating the detector failure as "no UI".
5. Run `/qa` — choose the repo-fit running surface. Browser is only one
   possible driver; CLI, API, library, MCP, docs, and infra changes still need
   an exercised surface or an inconclusive/fail result.
6. Run `/demo` — create the audience-shaped artifact. For non-visual work, use
   the text-artifact path; do not skip proof.
7. **Hindsight sanity pass.** Once the phases are green, read the full branch
   diff one last time with fresh eyes — `git diff $(git merge-base HEAD
   <base>)...HEAD` — and ask **"what production embarrassment would justify
   rejection here?"** Look for shallow modules, pass-through layers, hidden
   coupling, tests asserting implementation instead of behavior, stale
   comments/docs in changed areas, and debug artifacts the phases missed. If
   anything non-trivial surfaces, fix it and return to step 1 (it counts
   against the cap). This is the named successor to `/settle`'s adversarial
   self-review; it is distinct from a `/critique` lens dispatch. Skip only for
   trivial diffs (<20 LOC, single file).
8. **Verdict-ref freshness** (only if `harness-kit-checks verdict` is available):
   confirm `refs/verdicts/<branch>` reads `ship`/`conditional` and its SHA
   matches HEAD. A stale SHA means changes landed after review → return to
   step 1.
9. Run bounded `/reflect` distill — emit one learning packet with
   codification, backlog, skillify, memory, or explicit non-action
   dispositions. If branch evidence changes after this point, return to step 1
   and replace the stale packet instead of accumulating duplicate learning
   packets.
10. If all gates are green → exit 0, `merge_ready`. Else increment iteration
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
- Skip proof on library-only diffs. Pick a text, command, import, generated
  docs, or consumer-build artifact instead.
- Treat `ui_surface:true` as a routing signal, not a verdict. `/design` and
  `/a11y` still own findings and evidence.
- Treat a detector error as unknown, not clean. Fall back to path inspection.
