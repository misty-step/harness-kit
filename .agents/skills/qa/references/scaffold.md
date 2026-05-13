# QA Scaffold Template

Template for `/qa scaffold`. Generates a project-local QA skill
by investigating the codebase and designing with the user.

## Investigation Prompts

Launch these three investigators in parallel as Explore sub-agents.

### App Mapper

> Map this project's application type and dev tooling.
>
> Find and report:
> - **App type:** web app (SPA, SSR, static), CLI, API, library, monorepo?
> - **Dev command:** how to start the dev server (read package.json scripts,
>   Makefile, Cargo.toml, go.mod, pyproject.toml, README, CLAUDE.md)
> - **Port:** what port does the dev server listen on?
> - **Entry point:** main page, root route, or CLI entry
> - **Package manager:** npm, bun, pnpm, yarn, cargo, go, pip, uv?
> - **Framework:** Next.js, Express, FastAPI, Gin, Axum, SvelteKit, etc.?
>
> Return structured findings. No prose — facts only.

### Route Scout

> Map all user-facing routes, endpoints, or commands in this project.
>
> For web apps: read the router (Next.js `app/` or `pages/`, Express routes,
> SvelteKit routes, etc.). For CLIs: read command definitions. For APIs: read
> endpoint handlers.
>
> Return a table:
>
> | Route/Command | Description | Auth Required? | Complexity |
> |---------------|-------------|----------------|------------|
>
> Classify complexity as: trivial (static page), moderate (forms, state),
> complex (multi-step flows, real-time updates).

### Context Scout

> Map QA-relevant context for this project.
>
> Find and report:
> - **Auth mechanism:** session, JWT, OAuth, API key, none?
> - **Test credentials:** any test/dev users documented? (check .env.example,
>   README, seed files — never read actual .env)
> - **Existing test infrastructure:** unit tests, E2E, integration? Frameworks?
> - **Existing QA artifacts:** any QA skills, test plans, or checklists?
> - **Browser tools configured:** check `.mcp.json`, `mcp.json`,
>   `.claude/settings.json` for Playwright, Chrome MCP, agent-browser, Stagehand
> - **CI/CD:** what checks run on PR? (GitHub Actions, Dagger, etc.)
>
> Return structured findings. No prose — facts only.

## Design Conversation Guide

After investigation, present findings to the user and iterate.

### 1. Confirm Findings

Present the merged investigation results:

> Here's what I found about your project:
> - **Type:** [app type] built with [framework]
> - **Dev command:** `[command]` on port [port]
> - **Routes:** [count] user-facing routes ([count] critical, [count] moderate)
> - **Auth:** [mechanism]
> - **Browser tools available:** [list or "none configured"]
>
> Anything to correct or add?

### 2. Design Personas

Propose 2-3 personas based on the app's domain. These are NOT generic:

**Good personas** (domain-specific):
- "Restaurant owner managing their menu and checking tonight's reservations"
- "Student reviewing their progress and submitting an assignment"
- "DevOps engineer investigating a production alert"

**Bad personas** (generic):
- "New user exploring the app"
- "Admin user managing settings"
- "Power user using advanced features"

Ask: "Who uses this app? What are they trying to accomplish?"

### 3. Select Browser Tool

Browser tool decision tree (see `references/browser-tools.md` for details):

1. Need existing browser auth/cookies? -> Chrome MCP
2. Need hosted/stealth/anti-bot? -> Stagehand/Browserbase
3. Need deterministic test generation? -> Playwright MCP/CLI
4. Need annotated screenshots + lowest tokens? -> agent-browser
5. Not sure? -> Chrome MCP for exploration, Playwright for regression

Recommend one and explain why. Ask if the user has a preference.

### 4. Scope Critical Paths

Present the route table and ask:

> Which of these routes are critical path (must-test on every QA run)?
> Which are lower priority (test occasionally)?

### 5. Evidence Strategy

Recommend based on app type:

| App Type | Default Evidence |
|----------|-----------------|
| Web app (UI-heavy) | GIF walkthrough + route screenshots |
| Web app (data-heavy) | Screenshots + console/network checks |
| CLI | Terminal session capture |
| API | Curl output + response validation |
| Library | Test output diff |

## Generated Skill Template

The Deliver phase writes these files to `.claude/skills/qa/` in the target project.

### SKILL.md Structure

```markdown
---
name: qa
description: |
  QA for [project]. Navigate [app type], verify [key features].
  Use when: "run QA", "test this", "verify the feature", "QA this PR".
  Trigger: /qa.
disable-model-invocation: true
argument-hint: "[url|route|PR-number]"
---

# /qa

[One-line description of what QA means for this project.]

## Prerequisites

- [Dev server is running / preview deployment exists]
- [Browser tool] is available
- [Any auth/env setup needed]

## Dev Server

\`\`\`bash
[dev command]
\`\`\`

## Critical Paths

| # | Route | What to Check |
|---|-------|---------------|
| 1 | [route] | [specific checks from investigation] |
| 2 | [route] | [specific checks] |
| ... | ... | ... |

## Interactive Flows (if PR touches these)

| Flow | Steps |
|------|-------|
| [flow] | [specific steps for this app] |

## Evidence

[Evidence strategy from design phase]

Evidence goes to \`/tmp/qa-[project-slug]/\`.

## Output

- Status: PASS / FAIL
- Critical paths checklist
- Console errors found
- Evidence captured
- Recommendation: ready to merge / needs fix

## Gotchas

- [App-specific failure modes from investigation]
- Always check console for \`(Error|TypeError|ReferenceError|Failed|Unhandled)\` — runtime errors hide behind working UI
- If PR modifies a shared component, check ALL routes using it
- Test error states and loading states, not just happy paths
- [Project-specific anti-patterns]
```

### references/personas.md Structure

```markdown
# QA Personas — [project]

## [Persona 1 Name]

**Role:** [domain-specific role]
**Goal:** [what they're trying to accomplish]
**Common actions:**
- [action 1]
- [action 2]
**Edge cases:**
- [edge case 1]
- [edge case 2]

## [Persona 2 Name]

...
```

## Quality Gates (self-check before finishing)

Before declaring the scaffold complete, verify:

- [ ] SKILL.md frontmatter has trigger phrases (not just the name)
- [ ] Routes table has real routes from investigation (not placeholders)
- [ ] Dev command is the actual command (not "npm start" guessed)
- [ ] Personas reference real domain roles (not "new user")
- [ ] Evidence strategy matches the app type
- [ ] Gotchas section has project-specific failure modes
- [ ] Total SKILL.md is under 500 lines
- [ ] No generic placeholders remain ("TODO", "[fill in]", "your-app")
- [ ] Files are written to `.claude/skills/qa/` (not global skills/)
