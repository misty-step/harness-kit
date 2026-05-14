# Case: ignored `.spellbook/` repo brief and unmarked scripts

## Prompt

Run `/tailor` for a repo whose `.gitignore` contains `.spellbook/`, whose
legacy `.claude/.tailor/repo-brief.md` points at an old worktree, and whose
`scripts/lib/backlog.sh` exists, is unmarked, and differs from spellbook.

Produce only the install/repair decision. Do not modify files.

## Expected Outcome

- Writes `.spellbook/repo-brief.md` for local runtime use.
- Also writes a tracked compatibility copy at `.claude/.tailor/repo-brief.md`
  with the same brief content, not a breadcrumb to ignored state.
- Does not silently overwrite the unmarked `scripts/lib/backlog.sh`.
- Surfaces the script conflict as `preserve / replace / diff`.
- Does not declare self-audit green while an unclassified ownership conflict
  remains.
