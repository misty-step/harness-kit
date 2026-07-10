# 098 - Fold branch six-pack delivery into /deliver

## Priority

High.

## Problem

Operators want a high-rigor "branch six-pack" delivery loop: specifier, coder,
cleaner, architect, hardener, and QA lanes, with explicit evidence packet
construction and optional PR/final-mile handoff.

Harness Kit already has the right primitives:

- `/shape` for the specifier/oracle lane.
- `/implement` for the coder lane.
- `/refactor` for the cleaner lane.
- `/code-review` and `/critique` for the architect lane.
- `/hardening` for mutation/property/acceptance hardening.
- `/qa` for running-surface verification.
- `/ci`, `/agent-readiness`, `/demo`, `/trace`, `/yeet`, and `/ship` for gates,
  evidence, remote packaging, and final-mile landing.

A new top-level `/swarm-forge` skill would not be MECE. It would be an
orchestration style over `/deliver`, not a distinct artifact, side-effect
boundary, or lifecycle phase.

## Desired Outcome

Make `/deliver` grok "swarm-forge", "branch six-pack", "six agents",
"specifier coder cleaner architect hardener QA", and "Uncle Bob loop" as a
high-rigor delivery mode without adding a new top-level skill.

## Acceptance

- `/deliver` frontmatter triggers include the six-lane phrases above.
- `/deliver` gains a `--six-lane` or `--swarm` mode that remains a thin
  composer over existing leaf skills.
- Mode guidance lives in `skills/deliver/references/six-lane.md`; the main
  `SKILL.md` stays terse.
- The mode requires:
  - an executable oracle or shaped context packet;
  - Gherkin or equivalent user-oriented QA procedure when product behavior is
    affected;
  - lane verdicts for specifier, coder, cleaner, architect, hardener, and QA;
  - same-HEAD evidence before PR or merge claims;
  - `agent-readiness` delta when a readiness profile exists;
  - an evidence packet path or explicit repo-standard equivalent.
- PR and merge remain outside `/deliver` authority:
  - `/yeet` packages, commits, pushes, and prepares PR evidence.
  - `/ship` owns squash merge, backlog archive, trace handoff, and reflect.
- Regression coverage proves the generated index routes six-lane trigger
  phrases to `/deliver`, not a new top-level skill.

## Non-goals

- No new `/swarm-forge` source skill.
- No semantic workflow engine around provider CLIs.
- No duplicate implementations of review, CI, hardening, QA, PR, or merge
  logic.

## Validation

- `python3 scripts/check-frontmatter.py`
- `bash scripts/generate-index.sh && git diff --exit-code index.yaml`
- `bash scripts/check-docs-site.sh --self-test`
- `dagger call check --source=.`

