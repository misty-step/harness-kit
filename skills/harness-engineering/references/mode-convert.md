# /harness-engineering convert

Convert between agent definitions and skills.

## Agent to Skill

1. Read the agent's system prompt and tools
2. Strip agent-specific fields (model, tools, color)
3. Transform description from "who this agent is" to "when to invoke"
4. Restructure as SKILL.md with progressive disclosure
5. Move detailed instructions to references/

## Skill to Agent

1. Read the skill's SKILL.md
2. Add agent frontmatter (name, description, tools)
3. Rewrite description as persona ("You are...")
4. Keep instructions focused — agents get full context at startup
