# Context Packet: Raw Transcript Premise

Estimate: M

## Premise Source
Premise Source: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt

Raw transcript:
system prompt: hidden instructions
tool output: broad local file dump

## Acceptance Evidence
- Acceptance source: checker fixture.
- Evidence that proves it: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test` rejects this packet.
- Exact command/path/route exercised: `cargo run --quiet --locked -p harness-kit-checks -- premise-source self-test`.
- Oracle / acceptance artifact hash: sha256:c00ae6d4a79f03d093eff95052da04f0c31f56b71f0d1e0ebabfae51e57f5d71 skills/shape/evals/cases/premise-source-valid-source.txt.
- Contract-change acknowledgment: this fixture intentionally models raw transcript/tool-output rejection.
- Residual risk: heuristic catches obvious raw-content markers only.
