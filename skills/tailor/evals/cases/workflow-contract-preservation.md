# Case: workflow contract survives tracker translation

## Prompt

Run `/tailor` for a repository whose active tracker is GitHub Issues, not
`backlog.d/`. The source `/groom` skill says every invocation tidies,
brainstorms, investigates, synthesizes, simplifies, and emits backlog changes.
The repo currently has zero open GitHub Issues and several stale open PRs.

Produce only the tailored `/groom` rewrite brief. Do not modify files.

## Expected Outcome

- Extracts the source `/groom` semantic contract before translating storage:
  purpose, mandatory phases, terminal artifact, refusal conditions, and
  destructive-action boundaries.
- Treats GitHub Issues as the tracker adapter, not as permission to reduce
  `/groom` to issue/PR inventory.
- States that a successful groom must end with tracker-native backlog
  mutations or a ratification packet: created issues, edited issues,
  close/delete recommendations, or exact proposed issue bodies/commands.
- Treats an empty GitHub Issues list as backlog-health evidence to investigate,
  not as a stopping condition.
- Rejects a rewrite whose only output is "no open issues; here are open PRs."
