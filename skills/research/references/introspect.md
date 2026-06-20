# Introspect

Mine session transcripts for usage patterns. Output actionable improvement recommendations.

## Role

Staff engineer conducting a retrospective on how this machine is actually used,
not how it's supposed to be used. Evidence over intuition.

## Objective

Analyze all Claude Code session history and produce:
1. Frequency breakdown of actual usage patterns
2. Candidates for skill extraction (repeated multi-step workflows)
3. Candidates for autonomous agents (persistent loops, not one-shots)
4. CLAUDE.md/AGENTS.md updates (principles, not pragmatics)
5. Workflows to retire or consolidate

## Data Sources

Session transcripts live at `~/.claude/projects/`. Structure:
- Each project directory contains `.jsonl` files (one per session)
- `subagents/` subdirectories contain subagent transcripts (skip for top-level analysis)
- Record types: `user` (human messages), `assistant` (model responses), `progress` (hooks/system)

### JSONL Record Format

```
{
  "type": "user" | "assistant" | "progress",
  "message": {
    "role": "user" | "assistant",
    "content": "..." | [{ "type": "text", "text": "..." }, { "type": "tool_use", ... }]
  },
  "timestamp": "ISO-8601",
  "cwd": "/path/to/project",
  "sessionId": "uuid",
  "gitBranch": "branch-name"
}
```

Slash commands appear as: `<command-name>/skill-name</command-name>` in user message content.
Tool calls appear as `{ "type": "tool_use", "name": "ToolName", "input": {...} }` in assistant content.

## Durable Mining Command

For opt-in effectiveness mining, use the repo command instead of a one-off
`/tmp` script:

```bash
cargo run --quiet --locked -p harness-kit-checks -- mine-transcript-effectiveness \
  --transcript /path/to/session.jsonl \
  --skill-log crates/harness-kit-checks/tests/fixtures/skill-invocations.jsonl \
  --format markdown
```

The command requires explicit `--transcript` or `--source-root`, redacts before
reporting, fails closed on unresolved secret-like content, joins local evidence
stores when refs match, and reports missing source coverage instead of
inferring effectiveness from sparse data. Use `--allow-redacted-excerpts` only
for local operator review; default reports contain counts and refs, not raw
turn text.

## Durable Command Expansion Checklist

When the durable command needs broader coverage, extend
`harness-kit-checks mine-transcript-effectiveness` and its tests to extract:

### Quantitative
- **Sessions per project** (top 20)
- **Tool usage** by call count (top 25)
- **Slash commands** invoked by user (parse `<command-name>` tags)
- **Skill tool calls** invoked by Claude (from `Skill` tool_use blocks)
- **Subagent types** spawned (from `Agent` tool_use blocks)
- **Bash command frequency** (first word + git/gh subcommands)
- **File types edited** (by extension from Edit/Write tool calls)
- **Most edited files** (by filename)
- **User intent classification** (keyword-match into categories)
- **Repeated user messages** (normalized, count >= 3)

### Qualitative
- Sample 80 random user messages for manual pattern recognition
- Extract user corrections/frustrations (messages containing "wrong", "not what I", "still", "again")

### Filters
- Skip `subagents/` directories for top-level analysis
- Skip tool_result content (it's response data, not intent)
- Skip skill expansion text (messages containing "Base directory for this skill:")
- Cap sampled messages at 300 chars, skip messages < 3 chars

## Output Format

Present findings in this structure:

### 1. What You Do Most (ranked by frequency)
Table: Activity | % of usage | Evidence

### 2. Skill Candidates
Table: Proposed Skill | Repeated Pattern | Current State (ad-hoc / manual / partial)

### 3. Agent Candidates
Table: Proposed Agent | Why Autonomous | Evidence (not one-shot skills — persistent loops)

### 4. Instruction Updates
Table: Principle | Evidence (corrections, friction, repeated mistakes)
Keep to underlying principles, not specific pragmatic rules.

### 5. Retirements
Skills, workflows, or patterns that data shows are unused or superseded.

## Constraints

- Evidence-first. Every recommendation must cite session data.
- Agent-agnostic. Analysis applies to any coding agent, not just Claude Code.
- Principles over pragmatics. CLAUDE.md updates should be philosophical, not procedural.
- No fluff categories. If a recommendation doesn't have 3+ supporting data points, cut it.
- Respect the user's time. Findings should be scannable in under 2 minutes.
