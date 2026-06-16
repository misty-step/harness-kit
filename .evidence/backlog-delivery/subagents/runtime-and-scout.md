# Runtime + Scout Critic Lane

- Provider target: `opencode`
- Delegation id: `65c417ef-ad13-4bb7-933e-239bcea3cfd5`
- Receipt: `.harness-kit/traces/provider-lanes/20260616T212710.342343Z-opencode-b8719c0e.txt`
- Input: `.evidence/backlog-delivery/delivery-plan.html`
- Verdict: `BLOCKING: no`

Accepted findings:

- Exa Agent private-context consent must be visible in the returned artifact,
  not only in environment variables or prompts.
- The research Bun suite should be part of the canonical Harness Kit gate,
  because backlog 105's oracle lives there.
- `scout-skills` should not make live network calls by default; live metadata is
  an evidence mode.
- The Exa Agent timeout/cost exposure after run creation should be explicit
  residual risk unless a documented cancel endpoint is verified.

Disposition:

- `AgenticResearchBlock` now includes `private_context_allowed`; Bun tests cover
  both default false and explicit `EXA_AGENT_PRIVATE_CONTEXT_OK=1`.
- `cargo run --locked -p harness-kit-checks -- check --repo .` now includes
  `bun-test-research`.
- `scout-skills` defaults offline and requires `--live` for GitHub metadata
  evidence.
- Exa references document the verified Agent endpoint family and the remaining
  timeout cost risk.

Non-evidence:

- Provider target `pi`, delegation id `12eeb696-4139-4744-bb24-4ce0be294cc3`,
  timed out with `failure_kind=dispatch_timeout`; it is not counted as review
  evidence.
