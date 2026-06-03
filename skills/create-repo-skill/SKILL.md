---
name: create-repo-skill
description: |
  Generate a repository-local skill from live repo discovery and user intent.
  Use when: "create QA skill", "generate repo skill", "make a local skill",
  "persona acceptance skill", "value proposition QA", "bespoke repo QA",
  "scaffold local skill". Creates concrete `.agents/skills/<name>/` guidance
  with harness-specific bridges when useful. Trigger: /create-repo-skill,
  /repo-skill, /create-qa-skill.
argument-hint: "[qa|persona-acceptance|<skill-name>] [target]"
---

# /create-repo-skill

Generate a local skill for this repo, not Harness Kit globally. This is a thin
skill authoring lane, not a persona runtime or scheduling engine.

## Route

| Need | Load |
|---|---|
| repo-specific QA skill | `references/qa.md` |
| persona/value-proposition acceptance skill | `references/persona-acceptance.md` |
| skill-writing rules | `references/authoring.md` |

## Contract

- Discover the live repo before drafting.
- Ask the user for missing product truth: value proposition, target users,
  critical workflows, production/local target, and allowed side effects.
- Dispatch independent lanes: repo mapper, product/persona mapper, critic.
- Write only repo-local skill artifacts in the target repo.
- Prefer `.agents/skills/<name>/` as shared root; bridge `.claude/skills/`,
  `.codex/skills/`, and `.pi/skills/` when those dirs exist.
- Do not shadow a first-party global skill unless the local repo explicitly
  needs the same command name, such as `.agents/skills/qa/`.
- Include at least one eval seed or smoke oracle in the generated skill.
- Include an acceptance block with live repo evidence, exact command/path,
  repo-fit check, and residual risk.
- Name the repo's observable surfaces when the generated skill verifies
  behavior that should be watched after ship: healthchecks, logs, analytics
  coverage, receipts, evidence directories, benchmarks, or release smoke.
- Keep generated `SKILL.md` under 300 lines unless the repo proves otherwise.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use split lanes: one maps repo truth, one attacks the generated skill for generic wording, missing oracles, and workflow mismatch; native in-thread subagents may supplement but do not satisfy the roster floor.

## Output

Generated local skill includes:

- `SKILL.md` with trigger phrases, repo commands, live surfaces, output format.
- `references/` only for material too large for the entry file.
- `evals/README.md`, `evals/cases/<case>.md`, and a small grader or rubric.
- A closeout note naming what the lead accepted, rejected, and left risky.
- A generated-skill quality gate: fail on placeholders, guessed commands,
  missing live surfaces, or no eval seed.
- A post-generate acceptance block comparing the generated skill to live repo
  language, commands, docs, shared root, bridge topology, observable surfaces,
  and known user corrections.

## Post-Generate Acceptance

Before claiming the generated repo skill is usable, compare it against live repo
evidence. Passing frontmatter or scaffold validation is structural proof only.

```markdown
## Acceptance Evidence
- Live repo evidence read: repo files, docs, commands, routes, configs, harness roots, and user corrections inspected.
- Acceptance source: user request, repo-local workflow, value proposition, ticket, or explicit absence.
- Evidence that proves it: diff, smoke output, eval result, generated artifact path, or transcript proving the repo-local workflow is connected.
- Exact command/path/route exercised: command, URL, route, tool call, generated path, or smoke oracle run.
- Oracle / acceptance artifact hash: sha256 digest for any fixture, transcript, screenshot, or contract used by the generated skill, or state that no artifact-backed oracle exists.
- Contract-change acknowledgment: reason when generating the skill changes an existing repo-local acceptance contract, or state that no contract changed.
- Repo-fit check: language, commands, docs, shared root, bridge topology, and observable surfaces match this repo.
- Structural gate: frontmatter, eval seed, scaffold validation, or smoke check result.
- Residual risk: missing product truth, untested command, skipped bridge, or none with reason.
```

## Gotchas

- Generic local skills are worse than no local skill.
- Personas are not job titles. They need a goal, context, constraint, and task.
- Value propositions are claims to test, not marketing copy to repeat.
- Browser automation is a tool choice after workflow mapping, not the default.
- Do not generate a semantic workflow engine. Generate instructions and oracles.
