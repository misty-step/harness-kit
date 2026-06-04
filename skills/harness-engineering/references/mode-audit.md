# /harness-engineering audit — Skill Health Assessment

Analyze skill invocation data to assess skill health, identify waste, and
recommend lifecycle actions.

For Codex skill prompt budget, duplicate copies, unused candidates, and loaded
roots, sync and run the external `steipete-skill-cleaner` first. Use this audit
when the question is lifecycle strategy rather than catalog mechanics.

## Data Source

Run `python3 scripts/analyze-skill-invocations.py --format markdown`. By
default it reads `~/.claude/skill-invocations.jsonl`,
`.harness-kit/work/ledger.jsonl`, and `.harness-kit/traces/delegations.jsonl`,
then reports coverage for Claude, Codex, Pi, and Antigravity. Only Claude has
a verified skill invocation hook today; unsupported harnesses must remain
explicit warnings until a real event surface and smoke path exists.
Each skill invocation line is a JSON object:
`{"schema_version": 2, "event_type": "skill_invocation", "ts": "ISO8601", "harness": "claude", "source_protocol": "post_tool_use", "skill": "name", "args": "...", "session_id": "...", "cwd": "...", "project": "..."}`.
The analyzer tolerates legacy rows without `schema_version` or
`source_protocol`, but fixture validation requires the v2 shape.

If the file doesn't exist or is empty, report: "No invocation data found.
Skill invocations are tracked automatically via PostToolUse hook. Once you
have data, re-run `/harness-engineering audit`."

## Flags

- `--since <duration>` — filter to recent data (e.g., `30d`, `7d`, `90d`).
  Default: all data.
- `--skill <name>` — deep-dive on a single skill instead of the full report.

## Full Report (default)

### 1. Frequency Table

| Skill | Invocations | Last Used | Projects |
|-------|-------------|-----------|----------|

Sort by invocation count descending.

### 2. Health Categories

Classify each installed skill (read `skills/*/SKILL.md` for the full list):

| Category | Criteria | Action |
|----------|----------|--------|
| **Hot** | >10 invocations in period | Keep, consider investing (deeper references, sub-modes) |
| **Warm** | 3-10 invocations | Keep, monitor |
| **Cold** | 1-2 invocations | Evaluate: niche or dead? |
| **Dead** | 0 invocations in period | Candidate for deprecation |

### 3. Consolidation Candidates

Flag skills that:
- Are always invoked in sequence (A then B in the same session → merge into A)
- Share >50% of trigger phrases with another skill (description overlap)
- Have complementary domains that could be one skill without exceeding 3 workflows

### 3b. Delegation Contract Coverage

For substantial workflow skills, run or mirror
`python3 scripts/check-agent-roster.py` and report any missing or weak
`## Delegation Floor` sections. Flag:

- no two-provider roster floor when a roster exists;
- missing exception rationale for direct lead-only work;
- vague lane responsibilities or context boundaries;
- absent output/evidence or receipt contract;
- no statement that the lead owns synthesis and verification;
- missing runtime delegation references for Claude Code, Codex, Antigravity
  CLI, or Pi.

Treat a missing floor as harness debt, not a stylistic note.

### 4. Recommendations

For each skill, emit one of:
- **keep** — healthy, earning its description tax
- **invest** — hot skill, would benefit from deeper references or sub-modes
- **deprecate** — dead or cold with no clear niche
- **merge [target]** — consolidation candidate, specify merge target
- **split** — exceeding mode-bloat gate
- **promote** — project-local skill used across >2 projects, promote to global

### 5. Description Tax Report

Count installed skills (from `skills/*/SKILL.md`). Estimate ~100 tokens per
skill description, always loaded. Report total. Flag if >2,000 tokens (20+ skills).

## Deep-Dive Report (--skill flag)

For a single skill:

- Invocation timeline (count by week or month)
- Project breakdown: which projects use it most
- Session co-occurrence: which other skills fire in the same sessions
- Recommendations: specific, actionable

## Output Format

Structured markdown. Tables and bullets only. No prose filler.
End with a **TLDR**: 3-5 bullet summary of the most actionable findings.
