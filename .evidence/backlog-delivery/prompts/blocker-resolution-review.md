Role: fresh-context pre-ship verifier.

Objective: verify that the three blocking gaps from the prior pre-ship review
are closed, and do not re-open unrelated broad critique unless the fix creates
a new blocker.

Inputs:

- The candidate repo is `/Users/phaedrus/Development/harness-kit`.
- The diff input contains the follow-up patch for:
  - `skills/research/references/default-fanout.md`
  - `skills/research/references/exa-tools.md`
  - `meta/CONTRACTS.md`

Check:

1. `default-fanout.md` has an Exa Agent / agentic acquisition lane that can be
   labeled complete/partial/failed/skipped in the source matrix.
2. `exa-tools.md` documents that Exa `/research/v1` is not for new work and
   names the deep/deep-reasoning search fallback.
3. `meta/CONTRACTS.md` links Mode B loop guardrails to
   `harnesses/shared/references/loop-readiness.md`.

Output:

- `BLOCKING: yes|no`
- If yes, list exact file/path and missed condition.
- If no, one paragraph confirming closure and any residual non-blocking risk.
