# /skill-eval run recipe

How to actually produce arm A, arm B, and the blind grader — cheap to serious.

## Cheap smoke (free, native subagents)

Proves the loop fires and can return A≠B. Single fixture. Family shared
(waiver). Use on every skill edit and to debug an eval before paying for the
decorrelated run.

1. **Arm A** — a fresh subagent given the fixture prompt + read-only repo, told
   to invoke the skill (`/<skill>`). Output the artifact only — no repo edits.
2. **Arm B** — a fresh subagent given the same fixture prompt + same repo and the
   bare instruction a sharp operator would type, no skill mention. Output the
   artifact only.
3. **Grader** — a fresh subagent given both artifacts (labeled X/Y, shuffled) +
   the fixture + the eval spec's checks and rubric. It is not told which arm is
   the skill. Returns objective-check results + rubric scores + a "which is more
   <claim>" verdict.

Run A and B in parallel; grader after. Save all three outputs to the evidence
packet. The smoke's honest limit: workers and grader share a family, so it
proves the mechanism and catches gross regressions — it does not certify the
margin. Margin needs the serious run.

## Serious run (paid, decorrelated)

For contract changes and model-upgrade re-audits. Uses `council.sh` so arms and
grader run on *different model families* than each other.

- Write each arm's full task to one file — lanes are cold; inline the fixture,
  repo context, forbidden edits, and what "done" means.
- `members.tsv` = one line per arm/grader: `label  cli  model  persona`.
- `skills/council/scripts/council.sh --task /tmp/<arm>.txt --members
  /tmp/members.tsv --outdir .evidence/harness-evals/<skill>/<date>/<arm>`
- Pull live slugs (roster index / OpenRouter MCP) — never hardcode; a lane
  failing instantly is usually a dead slug or auth lapse, not a verdict.
- Grader lane: a family distinct from *every* worker lane.

## The clean A/B knob (lane_harness)

When the harness should *enforce* skill visibility rather than trust the prompt,
use a `lane_harness.v1` manifest
(`crates/harness-kit-checks/src/lane_harness.rs`): arm A sets
`allowed_local_skills: ["<skill>"]`, arm B sets `allowed_local_skills: []`. Same
`provider_target`, same `oracle`, same `evidence_expectations`. "Skill on vs off"
becomes a manifest diff, not an honor-system instruction the worker can ignore.

## Evidence packet

```
.evidence/harness-evals/<skill>/<date>/
  fixtures/<id>/{prompt.md, repo-sha}
  arm-a/{artifact, transcript}
  arm-b/{artifact, transcript}
  arm-c/...                      # optional alternative primitive
  grader-<family>.md             # objective results + rubric + verdict
  report.md                      # score matrix, variance note, decision label
```

Sanitize: final artifacts + scored receipts only. No secrets, no raw provider
logs, no customer data (`backlog.d/112` non-goals). Decision label is one of
`keep` / `adapt` / `cut` / `needs-more-tasks` / `graduate-to-Daedalus`.
