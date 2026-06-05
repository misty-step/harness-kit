---
name: harness-engineering
description: |
  Harness engineering for Harness Kit primitives: skills, shared doctrine,
  provider roster, harness configs, gates, evals, bootstrap, and sync logic.
  Use for "improve the harness", "harness engineering", "bootstrap is wrong",
  "AGENTS.md is stale", "skill health", "skill usage", "undertriggering skill",
  "description tax", "eval skill", "sync primitives", "roster defaults".
  Trigger: /harness-engineering, /harness, /skill.
argument-hint: "[create|eval|lint|convert|sync|engineer|audit|models] [target]"
---

# /harness-engineering

Engineer the harness. Keep it thin.

## Route

| Need | Load |
|---|---|
| create global skill/agent | `references/mode-create.md` |
| eval skill | `references/mode-eval.md` |
| lint skill | `references/mode-lint.md` |
| apply skill-design lessons | `references/skill-design-principles.md` |
| clean Codex skill catalog | external `steipete-skill-cleaner` |
| convert agent/skill | `references/mode-convert.md` |
| sync externals | `references/mode-sync.md` |
| engineer doctrine/gates/hooks | `references/mode-engineer.md` |
| measure skill usage/health/staleness | `references/mode-audit.md` |
| open-model defaults | `references/open-model-roster.md` |

Use `/create-repo-skill` for generated repository-local skills such as bespoke
QA and persona acceptance probes. Use this skill when changing Harness Kit
itself. Use `/skill-cleaner` before catalog deletions, duplicate cleanup, or
description-budget work when the external `steipete-skill-cleaner` skill is
synced locally.

## Contract

- Fix root cause in the highest-leverage layer: type/test/hook/gate/skill/
  AGENTS, in that order.
- Prefer deletion. Harness prose is context tax.
- Cross-harness first: Claude, Codex, Pi. Filesystem + `SKILL.md` is primary.
- Skills stay self-contained: scripts/references under the skill; state roots
  from invoking repo.
- Treat a skill as a folder, not a markdown file. Use scripts, references,
  examples, templates, assets, evals, or append-only data when prose would
  make the agent reconstruct repeatable work.
- Harness Kit source skills live in `skills/`; repo-local `.agents/skills/` and
  harness-specific skill bridge dirs are `/seed` output for consumer repos.
- Generated/root `AGENTS.md` is a router, not a manual. Keep non-obvious facts
  only.
- System bootstrap exposes the full first-party skill catalog; repo-local
  vendoring is exceptional and must earn its complexity.
- Provider CLIs are tools. Do not wrap them in semantic orchestration unless a
  shaped ticket explicitly asks.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use lanes for doctrine critique, runtime compatibility, gate design, and regression risk; native in-thread subagents may supplement but do not satisfy the roster floor. Do not treat a missing repo-local roster as a waiver; use the resolver-backed probe.

## Quality Bar

- `SKILL.md` encodes judgment, not a procedure the model already knows.
- Frontmatter descriptions are model trigger classifiers, not human summaries:
  include explicit `Use when:` phrases and `Trigger:` aliases.
- Instruction prose is compression, not literature. Sacrifice grammar before
  clarity; keep terse imperatives, named failure modes, and concrete oracles.
- Put long mode detail in `references/`; keep the entry file short.
- Build gotchas from repeated agent failures. If a gotcha can be asserted by a
  script, hook, or eval, codify it there and point the skill at the artifact.
- Ad-hoc roster lanes beat static project subagents unless tool permissions
  must be isolated.
- New mechanisms include a gate, eval, or smoke path.
- Every run ends clean: no untracked or modified files.

## Post-Sync Acceptance

After changing skills, shared doctrine, generated docs, bootstrap, roster, or
harness projections, prove the output is repo-fit, not merely structurally
valid.

```markdown
## Acceptance Evidence
- Live repo evidence read: source skill, shared doctrine, generated docs, bootstrap output, roster, or harness projection inspected.
- Acceptance source: backlog oracle, skill contract, generated index/docs contract, bootstrap contract, or explicit absence.
- Evidence that proves it: command output, diff, generated artifact, bootstrap transcript, eval result, or Dagger output.
- Exact command/path/route exercised: check, generator, bootstrap, smoke path, projection path, or route run.
- Oracle / acceptance artifact hash: sha256 digest for any fixture, generated artifact, transcript, or contract used as the oracle, or state that no artifact-backed oracle exists.
- Contract-change acknowledgment: reason when the change alters an acceptance contract, generated source, or assertion surface, or state that no contract changed.
- Repo-fit check: source/generator/projection agree; no stale generated docs, wrong skill root, stale command, or copied bridge remains.
- Structural gate: frontmatter, roster, evidence-block, docs, index, eval, or Dagger gate result.
- Residual risk: skipped harness, external dependency, or none with reason.
```

## Gotchas

- Stale AGENTS prose is worse than missing prose.
- Duplicated repo-local skill copies are usually stale context unless a repo
  needs checked-in vendored harness state.
- Generated catalog/docs drift means the source skill changed but the harness
  projection did not.
- Unsupported invocation hooks mean usage telemetry is structurally shaped but
  not empirically proven for that harness.
- Structural eval trees are not semantic proof; objective graders must assert
  behavior or carry an explicit waiver.
- Helper scripts that are not wired into a gate become optional folklore.
- Regexes over agent prose are usually the wrong boundary.
- If a rule matters, enforce it outside prose.
