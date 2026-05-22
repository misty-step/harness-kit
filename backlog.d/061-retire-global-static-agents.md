# Retire global static agents in favor of dynamic delegation guidance

Priority: P0
Status: ready
Estimate: M

## Goal

Stop installing global static agents from this repo. Current frontier harnesses
are increasingly good at creating and supervising task-specific subagents when
the lead agent has strong skills and clear AGENTS.md guidance. The durable
primitive is not "always install an Ousterhout agent"; it is "when this workflow
needs a design critic, spawn a fresh-context reviewer with this lens, this
scope, and this evidence contract."

Convert the useful content in `agents/*.md` into:

- workflow-specific delegation rubrics inside skills such as `/code-review`,
  `/refactor`, `/shape`, `/diagnose`, and `/research`;
- domain-specific lenses inside domain skills when a perspective only matters
  for that domain;
- general always-on principles inside `harnesses/shared/AGENTS.md` when they
  should guide every task.

## Non-Goals

- Do not ban subagents. The goal is the opposite: make subagents more
  task-specific, more dynamic, and better scoped.
- Do not remove project-local or runtime-native subagent support when a target
  repo genuinely needs a persistent named agent.
- Do not preserve global persona files as a compatibility crutch unless a
  runtime has no other practical way to express the guidance.
- Do not create a new orchestration DSL.

## Oracle

- [ ] `bootstrap.sh` no longer installs `agents/*.md` globally into
      `~/.claude/agents`, `~/.codex/agents`, `~/.pi/agents`, Antigravity, or
      any other runtime.
- [ ] `README.md`, `AGENTS.md`, and `project.md` describe agents as dynamic
      task-specific delegations by default, not as a global static catalog.
- [ ] `skills/code-review/SKILL.md` preserves the useful philosophy bench by
      instructing the lead agent to spawn fresh-context reviewers with specific
      lenses and responsibilities for the artifact under review.
- [ ] `skills/refactor/SKILL.md`, `skills/shape/SKILL.md`, and
      `skills/diagnose/SKILL.md` each name when to delegate, what the delegated
      agent owns, and what evidence it must return.
- [ ] `harnesses/shared/AGENTS.md` gets a compact dynamic-delegation routing
      section: when to spawn, when not to spawn, how to define scope, how to
      verify subagent output, and when to keep work local.
- [ ] Existing `agents/*.md` files are either deleted, moved to a
      `references/lenses/` style archive, or converted into skill references.
      The chosen disposition is documented.
- [ ] `dagger call check --source=.` passes.

## Notes

### Research signal

Current harnesses are moving toward lead-agent-managed delegation:

- Claude Code documents built-in subagents, CLI-defined subagents, independent
  context windows, tool/permission scoping, and the rule that a custom subagent
  is worth defining when the same worker keeps recurring:
  https://code.claude.com/docs/en/sub-agents
- Antigravity CLI documents asynchronous subagents where the main agent
  automatically spawns background agents for research, builds, and validation,
  and decides their tools and permissions:
  https://antigravity.google/docs/cli-features
- OpenAI Codex emphasizes durable `AGENTS.md` instructions and discoverable
  skills as the repeatable workflow layer:
  https://developers.openai.com/codex/guides/agents-md and
  https://github.com/openai/skills

The pattern is clear: keep stable workflow knowledge in skills and repo
instructions; let the lead agent instantiate the right subagents at runtime.

### What survives from the current agents

The philosophy is useful. The global installation mechanism is not.

- Ousterhout survives as a design lens: complexity, information hiding, shallow
  modules, change amplification.
- Carmack survives as a shipping lens: directness, what not to build, truth over
  sunk cost.
- Beck and Cooper survive as testing lenses: red-green-refactor, behavior tests,
  no internal mocks.
- Critic survives as a role contract: cold artifact review against acceptance
  criteria.

These belong close to the workflow that needs them, not in every user's global
agent list.

