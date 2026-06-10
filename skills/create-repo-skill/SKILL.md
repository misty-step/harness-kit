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

Agentic compiler: repo evidence + user intent -> one repo-local skill. Not
global. Not a runtime. Rust tools scaffold and typecheck artifacts; the lead
agent still owns discovery, product judgment, oracle design, and final report.

## Route

| Need | Load |
|---|---|
| repo-specific QA skill | `references/qa.md` |
| persona/value-proposition acceptance skill | `references/persona-acceptance.md` |
| skill-writing rules | `references/authoring.md` |
| readiness profile present | Load `.harness-kit/agent-readiness.yaml` for gate commands, mock policy, stack feedback strength, and waivers |

## Contract

- Discover the live repo before drafting.
- Ask the user for missing product truth: value proposition, target users,
  critical workflows, production/local target, and allowed side effects.
- Dispatch independent lanes: repo mapper, product/persona mapper, critic,
  and oracle designer when the workflow is not obvious.
- Decide whether a local skill is warranted. If global `/qa`, `/demo`,
  `/design`, or `/agent-readiness` already owns the job with minor parameters,
  do not generate a repo-local shadow.
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

## Build Tools

Use the tools as rails, not the generator:

```sh
cargo run --locked -p harness-kit-checks -- repo-skill scaffold <name> --kind qa|persona-acceptance|generic --repo <target-repo>
cargo run --locked -p harness-kit-checks -- repo-skill validate <target-repo>/.agents/skills/<name>
cargo run --locked -p harness-kit-checks -- eval-grader create-repo-skill <target-repo>/.agents/skills/<name>
```

`scaffold` creates the manifest for the agent to fill. `validate` rejects
placeholders, missing gates, missing eval seeds, missing concrete repo anchors,
and copied harness bridges. It cannot prove product fit; the agentic critic
must still attack the generated skill for generic wording, guessed commands,
invented routes, and weak oracles.

## Delegation Judgment

delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use split lanes: one maps repo truth, one attacks the generated skill for generic wording, missing oracles, and workflow mismatch.

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

The final report includes:

- generated paths and bridge paths;
- repo facts accepted and guesses rejected;
- validation command output;
- critic blockers resolved or waived;
- residual product truth still requiring the user.

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
- Do not confuse scaffold success with generated skill acceptance.
