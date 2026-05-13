# Demo Scaffold Template

Template for `/demo scaffold`. Generates a project-local demo skill
by investigating the codebase and designing capture strategy with the user.

## Investigation Prompts

Launch these three investigators in parallel as Explore sub-agents.

### App Mapper

> Map this project's application type and demo-relevant tooling.
>
> Find and report:
> - **App type:** web app (SPA, SSR, static), CLI, API, library, monorepo?
> - **Dev command:** how to start (read package.json scripts, Makefile,
>   Cargo.toml, go.mod, pyproject.toml, README, CLAUDE.md)
> - **Port:** what port does the dev server listen on?
> - **Key features:** what are the 3-5 most important user-visible features?
> - **Framework:** Next.js, Express, FastAPI, Gin, Axum, SvelteKit, etc.?
> - **Has UI?** Does this project render anything visible to users?
>
> Return structured findings. No prose — facts only.

### Feature Scout

> Identify demo-worthy features and visual deltas in this project.
>
> For web apps: read routes, components, and pages. Which have visual state
> changes worth demonstrating? For CLIs: which commands produce interesting
> output? For APIs: which endpoints show the core value proposition?
>
> Return a table:
>
> | Feature | Location | Visual Delta | Complexity |
> |---------|----------|-------------|------------|
>
> Classify complexity as: static (screenshot), interactive (GIF),
> multi-step (video walkthrough).

### Tooling Scout

> Map demo capture tooling available in this project.
>
> Find and report:
> - **Browser tools configured:** check `.mcp.json`, `mcp.json`,
>   `.claude/settings.json` for Chrome MCP, Playwright, Stagehand
> - **Recording tools available:** check for ffmpeg, asciinema, VHS (charm),
>   Playwright video support, Remotion
> - **Existing demo artifacts:** any GIFs, screenshots, demo dirs, videos?
> - **CI artifacts:** does CI produce screenshots or visual diffs?
> - **Upload targets:** GitHub Releases configured? S3? Other hosting?
>
> Return structured findings. No prose — facts only.

## Design Conversation Guide

After investigation, present findings to the user and iterate.

### 1. Confirm Findings

Present the merged investigation results:

> Here's what I found about your project:
> - **Type:** [app type] built with [framework]
> - **Key features:** [list]
> - **Demo-worthy deltas:** [count] features ([count] static, [count] interactive)
> - **Capture tools available:** [list or "none configured"]
> - **Existing demo artifacts:** [any found or "none"]
>
> Anything to correct or add?

### 2. Select Artifact Strategy

Recommend based on app type:

| App Type | Primary Artifact | Secondary | Tool |
|----------|-----------------|-----------|------|
| Web app (UI-heavy) | GIF walkthrough | Route screenshots | Chrome MCP gif_creator |
| Web app (data-heavy) | Screenshots + annotations | Console captures | Chrome MCP |
| CLI | Terminal session GIF | Command output captures | VHS or asciinema |
| API | cURL examples + response captures | Sequence diagrams | Script |
| Library | Before/after test output | Benchmark charts | Test runner |
| Monorepo | Per-app strategy | Combined walkthrough | Mixed |

Ask: "Which format best shows your project's value?"

### 3. Scope Demo Features

Present the feature table and ask:

> Which features are must-demo (shown every time)?
> Which are situational (shown for specific PRs only)?

### 4. Upload Strategy

Recommend based on project context:

- **PR evidence:** Draft GitHub release + PR comment (default)
- **README/docs:** Committed GIF in `docs/` or repo root
- **External:** S3, Cloudflare R2, or hosted service
- **None:** `/tmp/demo-evidence/` only

## Generated Skill Template

The Deliver phase writes these files to `.claude/skills/demo/` in the target project.

### SKILL.md Structure

```markdown
---
name: demo
description: |
  Generate demo artifacts for [project]. Captures [artifact types]
  for [key features].
  Use when: "make a demo", "demo this", "PR evidence", "record walkthrough".
  Trigger: /demo.
disable-model-invocation: true
argument-hint: "[feature|PR-number] [--format gif|screenshot|video] [upload]"
---

# /demo

[One-line description of what demo means for this project.]

## Capture Methods

| Feature | Method | Tool | Output |
|---------|--------|------|--------|
| [feature 1] | [GIF/screenshot/terminal] | [tool] | [file type] |
| [feature 2] | [method] | [tool] | [file type] |

## Workflow: Planner → Implementer → Critic

### 1. Plan

For each feature to demo:
- What state change to show (before → after)?
- What route/command to capture?
- What text/element proves it worked?

### 2. Capture

[App-type-specific capture instructions]

Rules:
- Every "after" has a paired "before" at the same location
- Every screenshot has a programmatic text assertion
- GIFs need real browser recording (not slideshow)
- Target: GIFs < 5MB, PNGs < 500KB

### Upload

[Upload strategy from design phase]

## FFmpeg Quick Reference

\`\`\`bash
# WebM → GIF (800px, 8fps, 128 colors)
ffmpeg -y -i input.webm \
  -vf "fps=8,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
  -loop 0 output.gif
\`\`\`

## Gotchas

- [App-specific gotchas from investigation]
- Default-state evidence proves nothing — show the delta
- GIFs must have >10 frames (not a slideshow) and be < 5MB
- Self-grading is worthless — critic must inspect artifacts cold
- WebM doesn't render in PR comments — convert to GIF
- Never commit binary artifacts to the repo
```

### references/capture-plan.md Structure

```markdown
# Demo Capture Plan — [project]

## Must-Demo Features

| # | Feature | Route/Command | Before State | After State | Artifact |
|----|---------|--------------|-------------|------------|----------|
| 1 | [feature] | [location] | [state] | [state] | [GIF/screenshot] |

## Situational Features

| # | Feature | When to Demo | Artifact |
|----|---------|-------------|----------|
| 1 | [feature] | [trigger condition] | [type] |

## Environment Setup

- Dev server: `[command]`
- Port: [port]
- Auth: [how to authenticate for captures]
- Prerequisites: [any setup needed]
```

## Quality Gates (self-check before finishing)

Before declaring the scaffold complete, verify:

- [ ] SKILL.md frontmatter has trigger phrases (not just the name)
- [ ] Feature table has real features from investigation (not placeholders)
- [ ] Capture methods match the app type (GIF for web, terminal for CLI)
- [ ] Dev command is the actual command (not guessed)
- [ ] Upload strategy matches the project's infrastructure
- [ ] Gotchas section has project-specific failure modes
- [ ] Total SKILL.md is under 500 lines
- [ ] No generic placeholders remain ("TODO", "[fill in]", "your-app")
- [ ] Files are written to `.claude/skills/demo/` (not global skills/)
