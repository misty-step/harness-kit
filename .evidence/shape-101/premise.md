# Premise: Capability Capsules

Operator request: shape the prior ad hoc harness-generation research into a
buildable Harness Kit context packet.

The accepted research direction was a Rust-backed capability-capsule primitive
for roster dispatch. A capsule is a small launch contract for one delegated
lane: role, provider/model/harness, workspace isolation, allowed tools, allowed
skills, allowed external skills, acceptance oracle, and evidence/receipt
requirements. It is not runtime skill-prose generation and not a semantic
workflow engine.

Key premise constraints:

- Reduce context bloat by scoping capabilities per lane/session.
- Preserve Harness Kit doctrine: cross-harness first, filesystem plus SKILL.md,
  provider CLIs stay thin, receipts are evidence, and the lead owns synthesis.
- Avoid the prior per-project allowlist failure where global skill state was
  mutated and polluted other repos.
- Treat external skill libraries as discovery input unless pinned and allowed.
- Start with validation and receipt evidence before launcher behavior expands.
