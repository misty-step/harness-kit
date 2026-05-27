---
name: harness
description: |
  Build and repair Spellbook primitives: skills, shared doctrine, provider
  roster, harness configs, gates, evals, bootstrap, and sync logic. Use for
  "improve the harness", "bootstrap is wrong", "AGENTS.md is stale", "skill
  health", "eval skill", "sync primitives", "roster defaults".
  Trigger: /harness, /skill, /primitive.
argument-hint: "[create|eval|lint|convert|sync|engineer|audit|models] [target]"
---

# /harness

Mutate the harness. Keep it thin.

## Route

| Need | Load |
|---|---|
| create skill/agent | `references/mode-create.md` |
| eval skill | `references/mode-eval.md` |
| lint skill | `references/mode-lint.md` |
| convert agent/skill | `references/mode-convert.md` |
| sync externals | `references/mode-sync.md` |
| engineer doctrine/gates/hooks | `references/mode-engineer.md` |
| audit catalog | `references/mode-audit.md` |
| open-model defaults | `references/open-model-roster.md` |

## Contract

- Fix root cause in the highest-leverage layer: type/test/hook/gate/skill/
  AGENTS, in that order.
- Prefer deletion. Harness prose is context tax.
- Cross-harness first: Claude, Codex, Pi. Filesystem + `SKILL.md` is primary.
- Skills stay self-contained: scripts/references under the skill; state roots
  from invoking repo.
- Spellbook source skills live in `skills/`; repo-local `.agents/skills/` and
  harness-specific skill bridge dirs are `/seed` output for consumer repos.
- Generated/root `AGENTS.md` is a router, not a manual. Keep non-obvious facts
  only.
- System bootstrap exposes the full first-party skill catalog; repo-local
  vendoring is exceptional and must earn its complexity.
- Provider CLIs are tools. Do not wrap them in semantic orchestration unless a
  shaped ticket explicitly asks.

## Delegation Floor

If a provider roster is available (repo `.spellbook/agents.yaml` or system `~/.spellbook/agents.yaml`), dispatch two or more providers for
substantive harness work and record receipts. Direct work is for mechanical
commands, emergency unblocks, explicit user waivers, or fewer than two
available providers.
Do not treat a missing repo-local roster as a waiver. Run the resolver-backed
probe, which falls back to the system Spellbook roster:
`python3 $SPELLBOOK_ROOT/scripts/probe-agent-roster.py`.
Native in-thread subagents may supplement the investigation, but they do not
satisfy the roster floor. Count only configured provider ids in receipts and
final roster reports.

## Quality Bar

- `SKILL.md` encodes judgment, not a procedure the model already knows.
- Put long mode detail in `references/`; keep the entry file short.
- Ad-hoc roster lanes beat static project subagents unless tool permissions
  must be isolated.
- New mechanisms include a gate, eval, or smoke path.
- Every run ends clean: no untracked or modified files.

## Gotchas

- Stale AGENTS prose is worse than missing prose.
- Duplicated repo-local skill copies are usually stale context unless a repo
  needs checked-in vendored harness state.
- Regexes over agent prose are usually the wrong boundary.
- If a rule matters, enforce it outside prose.
