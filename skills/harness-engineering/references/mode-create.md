# /harness-engineering create

Create a new Harness Kit source skill or agent from scratch.

For a project-local skill in a consumer repo (bespoke QA drivers, persona
probes), write it directly into the repo's `.agents/skills/<name>/` with the
repo's real routes and commands — same craft, local facts. This mode is for
first-party catalog primitives under `skills/`.

## The description field is everything

The description determines when the model loads the skill. Write it assertively.
Include trigger phrases users actually say. If the skill doesn't fire, the
description is wrong — not the model.

**Good:** `"Use when: 'debug this', 'why is this broken', 'investigate', 'production down'"`
**Bad:** `"A debugging utility for code analysis"`

## Structure

```
skill-name/
├── SKILL.md          # < 500 lines. Core routing + judgment.
├── references/       # Deep context loaded on demand.
├── scripts/          # Executable code for deterministic tasks.
├── examples/         # Representative inputs/outputs when useful.
├── templates/        # Copyable artifacts the model should not recreate.
├── assets/           # Images, fixtures, schemas, or static resources.
└── evals/            # Optional eval cases/graders for load-bearing skills.
```

Do not create empty folders. Add the folder when the skill has real reusable
material for that layer. If the workflow benefits from memory, define an
append-only JSONL/schema path and privacy rule; otherwise leave it stateless.

## What to encode

Encode judgment the model lacks. Not procedures it already knows.

**Highest signal:** Gotchas — what goes wrong, not just what to do right.
A gotcha list is more valuable than pages of happy-path instructions.
Enumerate failure modes, common mistakes, things the model consistently
gets wrong without the skill.

**Avoid:** Step-by-step procedures the model can derive from context.
If you're writing "1. Read the file 2. Find the function 3. Edit it" —
that's not a skill, that's a task description.

**In-repo exemplars worth reading before drafting:**
- `skills/sprites/SKILL.md` — one primitive, a routing table, gotchas.
  No daemon, no ceremony.
- `skills/diagnose/SKILL.md` — judgment encoded as a routing table plus
  phase protocol with concrete stop conditions.
- `skills/next/SKILL.md` — tiny reusable trigger where app-visible discovery
  earns the primitive.

**External exemplars (installed under `skills/.external/`):**
- `anthropic-skill-creator` — the "theory of mind" framing:
  explain the *why* before the *how* so the model can handle
  edge cases the rules don't enumerate.
- `anthropic-claude-api` — stratified progressive disclosure across
  SKILL.md body → language-specific reference folders → code examples.
- `vercel-dogfood` — repro-first discipline: document immediately
  before moving on, so findings survive session handoff.

## Progressive disclosure

Three layers. Each loads only when needed:

1. **Description** (~100 tokens) — always in context. Decides triggering.
2. **SKILL.md body** (< 500 lines) — loads when skill fires.
3. **References/scripts/assets/templates/examples** — loaded or run on demand
   when the specific situation requires them.

Keep SKILL.md focused on what to do and what goes wrong. Move deep reference
material, examples, boilerplate, schemas, and repeatable mechanics out of the
entry file and into the skill folder.

## Brevity doctrine

Model instructions are not essays.

- Prefer fragments over paragraphs when the meaning survives.
- Use imperative verbs: "Probe roster", "Write receipt", "Run gate".
- Name the failure mode directly.
- Delete throat-clearing: "it is important to", "you should consider".
- Keep examples shorter than the rule they explain.
- Put citations, long rationale, and variant-specific detail in references.

Useful source patterns:

- JuliusBrussee/caveman: token compression and terse agent dialect.
  https://github.com/JuliusBrussee/caveman
- petekp/claude-code-setup `grill-me`: a tiny, standalone interrogation skill
  that forces one question at a time with a recommended answer.
- cursor/plugins `thermo-nuclear-code-quality-review`: harsh maintainability
  review focused on structural simplification, file-size pressure, and
  spaghetti-growth blockers.
- Anthropic skill authoring: description selection and progressive disclosure.
  https://anthropic.mintlify.app/en/docs/agents-and-tools/agent-skills/best-practices
- Vercel Agent Skills: concise `SKILL.md`, reusable versioned context, and
  avoiding duplicated reference content.
  https://vercel.com/kb/guide/agent-skills-creating-installing-and-sharing-reusable-agent-context

## Frontmatter fields that matter

```yaml
---
name: my-skill
description: |
  What it does. When to use it. Trigger phrases.
argument-hint: "[arg1] [arg2]"      # shown in autocomplete
context: fork                        # run in isolated subagent (optional)
agent: Explore                       # which subagent type (optional)
disable-model-invocation: true       # user-only invocation (optional)
allowed-tools: Read, Grep, Glob     # restrict tool access (optional)
hooks:                               # skill-scoped lifecycle hooks (optional)
  PostToolUse:
    - matcher: "Edit|Write"
      hooks: [{type: command, command: "bash scripts/validate.sh"}]
---
```

## Dynamic context injection

Skills support shell injection: wrap a command in backticks prefixed with `!`
and the output replaces the placeholder at skill load time. For example, a
skill can inject the current git branch or recent commits so the model sees
live data, not the command. See the Claude Code skills docs for syntax.
