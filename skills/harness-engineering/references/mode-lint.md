# /harness-engineering lint

Validate a skill against quality gates.

## Quality gates

| Gate | Check | Fix |
|------|-------|-----|
| **Description triggers** | Does description include trigger phrases? | Add "Use when:" with concrete phrases |
| **Size** | SKILL.md < 500 lines? | Extract to references/ |
| **Gotchas** | Does it enumerate failure modes? | Add a gotchas section |
| **Judgment test** | Does it encode judgment the model lacks? | If not, delete the skill |
| **Oracle** | Can you verify the skill worked? | Add success criteria |
| **Freshness** | Do instructions match current model capabilities? | Strip non-load-bearing scaffold |
| **Mode bloat** | >4 modes with inline content, or any single mode >60 lines inline? | Extract mode content to references/mode-*.md; use router pattern (see /diagnose, /settle) |
| **Reference integrity** | Do all referenced local files in routing tables, gotchas, and examples exist? | Create the missing file, fix the path, or delete the stale reference |
| **Self-containment** | Do scripts source only paths under `skills/<name>/`? Do they resolve `SCRIPT_DIR` via `readlink -f` and `STATE_ROOT` from the invoking project? | Move shared libs into the skill tree; rewrite source paths to use `$SCRIPT_DIR/lib/…`; decouple state root from script dir. |
| **Delegation floor** | For substantial workflow skills, does `## Delegation Floor` include roster floor, direct-work exceptions, lane responsibilities, context boundary, output/evidence contract, and lead verification? | Add the missing contract terms, or state why the skill is not a substantive workflow skill. Validate with `python3 scripts/check-agent-roster.py`. |

## Delegation floor lint

Run the committed roster audit before approving harness or workflow-skill
changes:

```bash
python3 scripts/check-agent-roster.py
```

The check flags substantial workflow skills that lack a delegation floor, have
a weak floor, omit explicit exception rationale, blur in-thread subagents with
roster providers, or lack runtime delegation references for Claude Code, Codex,
Antigravity CLI, and Pi. A skill that truly does not need roster lanes should
say why instead of inheriting the default silently.

## Self-containment check

The skill must survive being symlinked into a foreign project. Two greps
catch most violations:

```bash
# Scripts that source files outside their own skill tree
rg -n 'source.*\$REPO_ROOT|source.*/scripts/lib/' skills/*/scripts/

# Scripts that walk up past skills/<name>/ via $SCRIPT_DIR/../..
rg -n 'SCRIPT_DIR/\.\./\.\.' skills/*/scripts/
```

Either match is a lint failure. The fix is structural, not a suppression.

Every scripted skill should also ship a distribution smoke test at
`skills/<name>/scripts/distribution_test.sh` that symlinks the skill into
a throwaway project and verifies `--help` works from there.

## Batch lint

Run on all skills: `for s in skills/*/SKILL.md; do /harness-engineering lint "$s"; done`

For duplicate skills, long descriptions, unused candidates, and Codex
prompt-budget pressure, sync and run the external `steipete-skill-cleaner`;
lint checks one skill's shape, not catalog tax.
