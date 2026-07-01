<!--
AUTHORING NOTES — delete this whole block once the skill is filled in.

This is the template for a REPO-LOCAL QA skill: the project-specific knowledge the
generic /qa method structurally cannot hold — exact launch commands, ports, env
vars, seed/auth steps, per-surface golden paths, and known gotchas. It is the
highest-value local agent investment (see David Cramer, "project-local QA skills
are the most valuable thing I've done for agents locally").

Rules for a good one:
- CONCRETE over generic. `pnpm dev` → web on http://localhost:3200, api on :4300.
  Not "start the dev server." If a command isn't copy-pasteable, it isn't done.
- START THIN. Cramer: "describe a few basic things and refine as they go." A
  20-line skill that names the real launch command and one golden path beats a
  500-line speculative one. Grow it the next time QA is painful, not now.
- The operator interview IS the spec. The manual checks a human runs before
  merging — that sequence, verbatim — is what this skill encodes. Ask, don't guess.
- It's a SKILL, not AGENTS.md. QA guidance is on-demand (loaded only when QA'ing),
  so it must not tax always-on context. Keep it out of AGENTS.md.
- One skill per repo. Name it `<repo>-qa`. Delete sections that don't apply — an
  API-only repo has no UI section.
- The generic /qa skill defers to this one. This file wins; it is the source of
  truth for how to verify this repo.

Fill every <placeholder>, delete unused sections, then delete this block.
-->
---
name: <repo>-qa
description: |
  QA <repo> changes by exercising the real running surface, not just tests.
  <one line naming the repo's surfaces: web UI / API / CLI / worker / service>.
  "Tests pass" is not QA. Use when: "QA this", "verify the feature", "smoke
  test", "check the app", "test <repo>". Trigger: /<repo>-qa.
argument-hint: "[surface|route|command|feature]"
---

# <repo>-qa

QA in <repo> means verifying the surface that changed against reality.
<Name the deterministic gate that is necessary but not sufficient — e.g.
`pnpm test` / `make ci-smoke` / `./scripts/verify.sh` — and say why it isn't the
whole story here.>

## Surfaces

Map what changed to what you must exercise. One path per surface the diff touched.

| Changed area | Surface | QA path |
|---|---|---|
| <e.g. apps/api/**> | API | <replay representative requests locally; check status + contract + error paths> |
| <e.g. apps/web/**> | Web UI | <launch, log in, walk the golden path the change touched; watch console + network> |
| <e.g. packages/cli/**> | CLI | <run the documented invocations + a malformed one; audit exit codes> |
| <e.g. workers/**> | Worker/job | <trigger the job; confirm the side effect, not just that it ran> |

## Start local runtime

The exact commands — copy-pasteable, with ports, env, and any seed/auth step.

```sh
# full app
<pnpm dev>            # web: <http://localhost:PORT>, api: <http://localhost:PORT>
# isolated surfaces
<command>             # <what it starts, on what port>
```

- Env: <required env vars + where they come from; any mock flag, e.g.
  `<APP>_MOCK=true` to run without external creds>.
- Seed/auth: <how to get a usable logged-in state or seeded data>.
- Fallback: <what to do if the default port is taken / a service is down>.

## <Surface> QA

<Per-surface concrete steps. Repeat this section per surface that matters. Keep
each step a real command or a specific check, not a principle.>

1. <identify the exact route/command/screen the change claims to affect>
2. <run the smallest command that exercises it>
3. <inspect the artifact/DB side effect/response shape — not just the exit code>
4. <exercise one edge the change plausibly broke: bad auth, malformed input, empty state>

## Gotchas

- <repo-specific traps: a service that must be up first, a flaky port, a mock
  that silently passes, a surface that only fails in prod-like config>

## Report

Return: **verdict** (PASS / FAIL / UNVERIFIED) · exact commands run · surfaces
exercised · artifacts/evidence inspected · what was NOT covered and whether a
post-ship signal exists for it.
