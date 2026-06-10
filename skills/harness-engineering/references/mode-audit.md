# /harness-engineering audit — Harness Health

Measure what the catalog actually earns, then recommend lifecycle actions.

## Data

```sh
cargo run --locked -p harness-kit-checks -- telemetry --format markdown
```

Reads `~/.claude/skill-invocations.jsonl` (written by the Claude
`skill-invocation-tracker` PostToolUse hook) plus delegation receipts in
`.harness-kit/traces/delegations.jsonl`. Filter with `--since 30d`,
`--skill NAME`, `--project NAME`.

**Codex has no invocation hook.** Its usage signal comes from session-log
mining — count distinct sessions that actually read a skill file:

```sh
cd ~/.codex/sessions && find . -name '*.jsonl' -mtime -60 -print0 \
  | xargs -0 grep -EHo '(cat|view|sed -n) [^"]*skills/[a-z-]+/SKILL.md' \
  | awk -F: '{match($0,/skills\/[a-z-]+\/SKILL/); print $1" "substr($0,RSTART+7,RLENGTH-13)}' \
  | sort -u | awk '{print $2}' | sort | uniq -c | sort -rn
```

Raw occurrence counts without the per-session `sort -u` are catalog noise,
not usage — the skill list rides in every prompt.

## Judgment

- Usage is a power law; that's normal. The per-skill question is "low usage
  with a value-when-used story, or low usage with no story?" Only the
  second is a deletion candidate.
- Recency matters: a skill created last week with zero usage is unproven,
  not dead.
- Check staleness the other way too: heavy usage of a skill whose prose has
  rotted is a rewrite signal, not a health signal.
- Cross-check the primitive test (SKILL.md): a "skill" only ever invoked
  explicitly by the operator, never auto-triggered, may really be a prompt.

Output: per-skill verdict (keep / rewrite / demote to prompt / delete) with
the usage number, recency, and story behind each. Findings feed `/groom`
tickets; do not auto-fix.
