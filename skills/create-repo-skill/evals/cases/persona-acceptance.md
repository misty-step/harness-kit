# Case: Persona Acceptance Generator

Prompt:

> Create a local persona acceptance skill for a web app that helps clinic staff
> triage patient intake forms. The skill should test whether a front-desk
> coordinator can find a new intake, identify missing insurance details, and
> escalate it without touching production data.

Fixture/context:

- Target repo has a documented dev server, `/intake` route, seeded demo staff
  account, and Playwright available.
- Production writes are forbidden.

Expected artifact:

- `.agents/skills/persona-acceptance/SKILL.md`
- at least one `evals/cases/*.md`
- report-card output format
- safe-tenant or ask-user boundary
- completion gate with exact behavior, live evidence, route/command, repo-fit
  check, and residual risk
