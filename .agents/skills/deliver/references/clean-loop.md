# Clean Loop (spellbook)

The clean loop runs `/code-review`, `/ci`, `/refactor`, and `/qa`
iteratively until all green, capped at **3 iterations**. Spellbook's
`/qa` path is non-browser library/harness verification: gate receipts,
skill eval suites, generated artifact drift, symlink bridge topology, and
command-level smoke evidence.

## Iteration Cap

Maximum 3 iterations. No 4th. Loops without caps produce slop.

On cap-hit:

- Exit code **20** (`clean_loop_exhausted`)
- Receipt `phases[*]` records last verdict, last `dagger call check`
  per-gate tail, iteration count
- Diff stays on the `<type>/<slug>` branch, unpushed, untouched —
  human inspects
- `state.json` records `phase.failed` on the last dirty phase;
  re-invoke without `--resume` refuses to clobber (exit 41 on
  merge-ready, explicit `--resume` or `--abandon` otherwise)

## Dirty-Detection (per phase)

A phase is **dirty** when:

| Phase | Dirty signal |
|---|---|
| `/code-review` | Verdict ref `refs/verdicts/<branch>` missing, pointing at non-HEAD SHA (stale), or carrying `verdict: dont-ship`. Blocking findings in bench synthesis. "nit" / "consider" / "suggestion" severity is NOT dirty. |
| `/ci` | Non-zero exit from `/ci`. Any of the 13 sub-gates red (`lint-yaml`, `lint-shell`, `lint-python`, `check-frontmatter`, `check-index-drift`, `check-vendored-copies`, `test-bun`, `check-exclusions`, `check-portable-paths`, `check-harness-install-paths`, `check-deliver-composition`, `check-no-claims`, `check-skill-evals`). `/ci` may self-heal lint-style gates via `dagger call heal` before reporting; only the post-heal result counts. |
| `/refactor` | Non-zero exit. Clean refactor → green even if no-op. |
| `/qa` | Non-zero exit. Missing evidence for the changed harness surface is dirty even when `/ci` is green. |

## Iteration Logic

1. Run `/code-review` → capture verdict ref + synthesis. If dirty:
   dispatch a builder (or re-run `/implement` with the findings) to
   fix, then loop. After a fix, the verdict ref is stale by
   construction; re-run review, not just CI.
2. Run `/ci` → capture receipt. If dirty: fix (a phase that
   hard-fails structurally — Dagger engine unavailable, missing
   `dagger` binary — is exit 10, not dirty).
3. Run `/refactor` — skip for trivial diffs (<20 LOC, single file).
   On a branch, `/refactor` runs in feature-branch mode against
   `master...HEAD`.
4. Run `/qa` → capture evidence. If dirty: fix or document the
   unverified path, then loop.
5. If all four green → exit 0, `merge_ready`. Else increment
   iteration counter and repeat from step 1.

## Escalation Protocol

- **Iteration 1 dirty:** normal. Fix, loop.
- **Iteration 2 dirty:** note in receipt; fix, loop.
- **Iteration 3 dirty:** exit 20. Receipt explains what remains.
  Human handoff.
- **Fundamental re-shape needed** (detected at any iteration): stop
  the loop, exit 20 with `recommended_next: human-review` and
  `remaining_work` describing the re-shape (e.g. "shape assumes
  Claude-only plugin API; violates cross-harness-first invariant").
  Do not spin.
- **Hard phase failure** (Dagger engine crashed, missing tool,
  process killed): exit 10 immediately, do not count against the
  iteration cap. Infrastructural, not "dirty output".
- **`check-deliver-composition` red:** this is special — the failing
  file is `skills/deliver/SKILL.md` itself. Fix requires editing the
  canonical source (not this `.agents/skills/deliver/SKILL.md`
  variant). Treat as dirty; dispatch builder with a pointer to the
  violating regex from `ci/src/spellbook_ci/main.py` (`DENYLIST`).

## What the Composer Does Not Do

- Invent a 4th iteration.
- Mask a dirty phase as green.
- Push on cap-hit "so the human can see it".
- Call `dagger call check` or `dagger call heal` directly — route
  through `/ci`. Inlining trips `check-deliver-composition` on the
  source SKILL.md and is a composition violation in spirit here too.
