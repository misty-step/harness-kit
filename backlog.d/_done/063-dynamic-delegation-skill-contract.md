# Add a dynamic delegation contract to workflow skills

Priority: P1
Status: merge-ready
Estimate: M

## Goal

Make every substantial workflow skill explicit that the lead agent is an
agent manager first: when a provider roster exists, the workflow dispatches
two or more roster members before substantive work is produced or validated.
The skill also states how to scope those lanes, how to record receipts, and
how the lead verifies the result.

## Non-Goals

- Do not hard-code Claude, Codex, Antigravity, or Pi-specific subagent syntax
  into generic skill bodies. Runtime-specific invocation details belong in
  harness projections or short references.
- Do not add a semantic workflow engine.
- Do not create one giant delegation reference that every skill blindly imports.
- Do not force roster lanes for pure mechanical command execution, emergency
  unblocks, explicit user-forbidden delegation, or fewer than two available
  roster members. Those are exceptions, not alternate defaults.

## Oracle

- [x] Each substantial workflow skill has a `## Delegation Floor` or equivalent
      section with:
      - two or more roster members as the default floor when a roster exists;
      - the narrow exception set for direct lead-agent work;
      - suggested roster-lane responsibilities;
      - context boundary;
      - output/evidence contract;
      - verification responsibility retained by the lead agent.
- [x] `/harness` lint or audit mode flags substantial workflow skills that
      lack a delegation floor or explicit exception rationale.
- [x] Runtime-specific references explain how to express dynamic delegation in
      Claude Code, Codex, Antigravity CLI, and Pi without changing the core
      skill semantics.
- [x] At least `/code-review`, `/research`, `/shape`, `/refactor`, `/diagnose`,
      `/qa`, and `/deliver` are updated as exemplars.
- [x] `dagger call check --source=.` passes.

## Notes

### Contract shape

Good dynamic delegation guidance is concrete without becoming a static persona
or workflow engine:

```text
For a security-sensitive diff, probe the roster and dispatch at least two
independent reviewers. One traces attacker-controlled input to sensitive sinks.
Another challenges the fix strategy and tests. Give both the diff, threat
model, and acceptance criteria. Ask for findings with file/line,
exploitability, and minimal fixes. Do not give them the author's reasoning.
```

That is better than globally installing `security-reviewer.md` forever. The
lead agent can tailor the role to the actual task, model, runtime, permission
mode, and context budget.

### Relationship to AGENTS.md

`harnesses/shared/AGENTS.md` should carry the general routing rule. Skills
should carry workflow-specific delegation recipes. Domain skills should carry
domain-specific reviewer lenses.

## What Was Built

- Strengthened `harness-kit-checks check-agent-roster` so the harness gate now validates
  delegation-floor contract fields across all 19 core workflow skills:
  provider floor, direct-work exceptions, lane responsibilities, context
  boundary, output/evidence contract, and lead verification.
- Added runtime-specific dynamic delegation references for Claude Code, Codex,
  Antigravity CLI, and Pi under `harnesses/*/README.md`.
- Updated `/harness lint` and `/harness audit` references to flag missing or
  weak delegation-floor coverage through `harness-kit-checks check-agent-roster`.
- Tightened workflow skill delegation floors so substantial workflows name
  lane context, evidence/receipt expectations, and lead-owned verification.
- Added regression coverage in Rust `agent_roster` tests for runtime
  delegation references.

## Verification

- `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`
- `cargo test --workspace --locked agent_roster`
- `cargo run --locked -p harness-kit-checks -- build-docs-site && cargo run --locked -p harness-kit-checks -- check-docs-site --repo .`
- `dagger call check --source=.`
