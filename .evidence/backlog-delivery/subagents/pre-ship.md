# Pre-Ship Critic Chain

## Full Diff Review

- Provider target: `goose`
- Delegation id: `0a5ba811-e94b-4371-9ba3-1d0849390bf7`
- Receipt: `.harness-kit/traces/provider-lanes/20260616T214039.232669Z-goose-5c7fc948.txt`
- Input: `/tmp/harness-kit-backlog-final.diff`
- Verdict: `BLOCKING: yes`

Blocking findings accepted:

- `skills/research/references/default-fanout.md` needed an explicit Agentic
  acquisition lane with `complete/partial/failed/skipped` status.
- `skills/research/references/exa-tools.md` needed the Exa `/research/v1`
  deprecation warning and deep/deep-reasoning fallback note.
- `meta/CONTRACTS.md` needed to link Mode B loop guardrails to
  `harnesses/shared/references/loop-readiness.md`.

## Blocker Resolution Review

- Provider target: `opencode`
- Delegation id: `4cbbd80b-3a0b-4657-8752-9993c56bc555`
- Receipt: `.harness-kit/traces/provider-lanes/20260616T214924.213209Z-opencode-903f5ebc.txt`
- Input: `/tmp/harness-kit-blocker-fixes.diff`
- Verdict: `BLOCKING: no`

Verified closures:

- `default-fanout.md` now includes `Agentic acquisition` in the source invariant
  and report-shape source matrix.
- `exa-tools.md` now bars new work on `/research/v1` and names `deep` /
  `deep-reasoning` search fallback.
- `meta/CONTRACTS.md` now points loop guardrails to
  `harnesses/shared/references/loop-readiness.md`.
