# Add a dynamic delegation contract to workflow skills

Priority: P1
Status: ready
Estimate: M

## Goal

Make every substantial workflow skill explicit about how the lead agent should
decide whether to spawn subagents, how to scope them, and how to verify their
work. This replaces static global personas with workflow-local delegation
contracts.

## Non-Goals

- Do not require subagents for every workflow. Many edits are faster and safer
  when handled locally.
- Do not hard-code Claude, Codex, Antigravity, or Pi-specific subagent syntax
  into generic skill bodies. Runtime-specific invocation details belong in
  harness projections or short references.
- Do not add a semantic workflow engine.
- Do not create one giant delegation reference that every skill blindly imports.

## Oracle

- [ ] Each substantial workflow skill has a `## Delegation` or equivalent
      section with:
      - when to delegate;
      - when to keep work local;
      - suggested subagent responsibilities;
      - context boundary;
      - output/evidence contract;
      - verification responsibility retained by the lead agent.
- [ ] `/harness` lint or audit mode flags substantial workflow skills that
      lack a delegation section or explicit skip rationale.
- [ ] Runtime-specific references explain how to express dynamic delegation in
      Claude Code, Codex, Antigravity CLI, and Pi without changing the core
      skill semantics.
- [ ] At least `/code-review`, `/research`, `/shape`, `/refactor`, `/diagnose`,
      `/qa`, and `/deliver` are updated as exemplars.
- [ ] `dagger call check --source=.` passes.

## Notes

### Contract shape

Good dynamic delegation guidance is concrete without becoming a static persona:

```text
For a security-sensitive diff, spawn a fresh-context reviewer whose only job is
to trace attacker-controlled input to sensitive sinks. Give it the diff, the
threat model, and the acceptance criteria. Ask for findings with file/line,
exploitability, and a minimal fix. Do not give it the author's reasoning.
```

That is better than globally installing `security-reviewer.md` forever. The
lead agent can tailor the role to the actual task, model, runtime, permission
mode, and context budget.

### Relationship to AGENTS.md

`harnesses/shared/AGENTS.md` should carry the general routing rule. Skills
should carry workflow-specific delegation recipes. Domain skills should carry
domain-specific reviewer lenses.

