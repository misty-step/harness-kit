> Template. Copy to
> `<target-repo>/.agents/skills/<repo>-<domain>/evals/<repo>-<domain>-eval.md`
> and fill every bracketed placeholder. Delete this line and every other
> `> ` guidance line before committing. This is a smaller instrument than
> Harness Kit's first-party `skills/skill-eval/templates/eval-spec.md` — one
> claim, cold-agent-run fixtures, no multi-arm A/B judge panel. See
> `../../../references/repo-local-skill-generation.md`.

# `<repo>-<domain>` eval

The one claim this generated skill must earn: **[one falsifiable sentence —
what a cold agent can do with this skill that it could not reliably do from
the bare repo alone, e.g. "correctly names and runs the real gate command on
the first try instead of guessing or inventing one"]**.

## Fixtures

> 1-2 fixtures. Each is a cold-agent run: a fresh-context agent gets only this
> skill file plus normal repo read access (no session memory of how the skill
> was generated) and a task that exercises it.

| # | Task given to the cold agent | Forbidden edits | What it stresses |
|---|---|---|---|
| 1 | [e.g. "use this skill to verify the repo's current state"] | [e.g. no writes outside a scratch dir] | [the flagship command path] |

## Objective checks (scriptable or human-observable, pass/fail)

- [ ] The agent names the exact command from the skill, not an invented or
      paraphrased one.
- [ ] The command actually runs (no 404/typo'd flag/missing env var the
      skill failed to mention).
- [ ] The agent reports a verdict in the skill's Report contract shape
      (verdict / command run / surface exercised / what was not covered).
- [ ] [repo-specific check, if any]

## Pass condition

The cold agent completes fixture 1 using only the skill + repo, with all
objective checks passing. A no-op "skill" (equivalent to the agent guessing
from the bare repo) fails because [name the concrete thing the bare repo
alone reliably gets wrong or omits — e.g. wrong default port, missing env
var, stale command from README vs. actual CI].

## Cadence

Re-smoke when the generating repo's real command/gate changes (drift check),
and whenever the provenance header's source SHA is more than a few months
stale.

## Run log

> Fill with the actual cold-agent smoke transcript/reference performed
> during generation — required before this skill is committed. Not
> "PENDING": generation without a completed fixture 1 run is not done.

[Date] — [fixture run], [pass/fail], [evidence: transcript path or command
output], [agent/model that ran it].
