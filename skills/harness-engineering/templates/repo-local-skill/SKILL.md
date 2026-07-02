> Template. Copy to `<target-repo>/.agents/skills/<repo>-<domain>/SKILL.md`
> and fill every bracketed placeholder from the live target repo. Delete this
> line and every other `> ` guidance line before committing. See
> `../../references/repo-local-skill-generation.md` for the full process.

---
name: <repo>-<domain>
description: |
  [One paragraph: what this skill verifies/runs/operates for <repo>, stated
  in terms of the repo's real shape (service/CLI/library/etc.), not generic
  process. End with explicit trigger phrasing: "Use when: '<phrase>',
  '<phrase>'. Trigger: /<repo>-<domain>."]
argument-hint: "[<surface1>|<surface2>|<surface3>]"
---

<!--
Generated via harness-kit's repo-local skill generation pattern
(skills/harness-engineering/references/repo-local-skill-generation.md).
Source repo: <owner/repo> @ <sha>. Generated: <YYYY-MM-DD>.
Generator ref: harness-kit@<sha used to generate this>.
Facts below are repo-derived at generation time, not invented. Re-verify
commands against the live repo before trusting this if it has aged — a
generated skill is a snapshot, not a live view.
-->

# <repo>-<domain>

[One or two sentences: what "<domain>" means in this repo specifically, and
the one deterministic gate/command that is necessary-but-not-sufficient
(name it, name what it can't prove).]

## Surfaces

> Only include rows for surfaces this repo actually has. Delete the rest.
> Every cell is a real path/command from the live repo, not a placeholder
> genre.

| Changed area | Surface | Verification path |
|---|---|---|
| `<path-glob>` | [HTTP service / CLI / library / worker / SDK] | [exact command(s)] |

## Commands

> Exact, copy-pasted invocations. Include the one-time setup a cold agent
> needs (env vars, local DB path, ports) and the fallback if the default
> port/resource is taken. If the repo's CI workflow differs from what
> AGENTS.md/README claims, name the mismatch here instead of picking one.

```sh
[real command 1]
[real command 2]
```

## Gotchas

> Repo-specific footguns a cold agent would otherwise rediscover the hard
> way — pulled from AGENTS.md/CLAUDE.md's own footgun list, docs/, or the
> generation author's own dry run. Not generic advice ("write tests").

- [gotcha 1]
- [gotcha 2]

## Report

Return: **verdict** (PASS / FAIL / UNVERIFIED) · exact command(s) run ·
surface(s) exercised · artifact/output inspected · what was NOT covered.
