# Repo Skill Authoring

## Write Compressed Instructions

- Cut essays.
- Use commands, paths, routes, endpoints, personas, and oracles.
- Prefer fragments when grammar adds no precision.
- Put rationale in references, not the hot path.
- Each section earns context by preventing a real error.

## Skill Shape

```text
.agents/skills/<name>/
  SKILL.md
  references/
  evals/
    README.md
    cases/<case>.md
    graders/<check>
```

Bridge harness-specific dirs only when present:

```text
.claude/skills/<name> -> ../../.agents/skills/<name>
.codex/skills/<name>  -> ../../.agents/skills/<name>
.pi/skills/<name>     -> ../../.agents/skills/<name>
```

## Quality Checks

- Frontmatter description has real trigger phrases.
- The body names repo commands and surfaces.
- The skill says what not to touch.
- Output format is checkable.
- Eval seed names the expected artifact and grader.
- No placeholders remain.

## Sources

- JuliusBrussee/caveman: terse token dialect. https://github.com/JuliusBrussee/caveman
- Anthropic skill authoring: description selection and progressive disclosure. https://anthropic.mintlify.app/en/docs/agents-and-tools/agent-skills/best-practices
- Vercel Agent Skills: reusable, versioned skills and concise references. https://vercel.com/kb/guide/agent-skills-creating-installing-and-sharing-reusable-agent-context
