# 099 - Make the skill catalog grokkable, DRY, and MECE

## Priority

High.

## Problem

The Harness Kit skill catalog is mostly well carved, but it is not fully
grokkable or MECE at a glance.

Strong leaf names exist: `/shape`, `/implement`, `/code-review`, `/ci`,
`/refactor`, `/qa`, `/hardening`, `/monitor`, `/diagnose`, `/deploy`, `/trace`,
and `/debrief`.

The weak spots are where names are metaphorical, duplicated, or lifecycle
ambiguous:

- `/yeet` is memorable but unclear to new operators; it means classify, commit,
  push, and prepare remote review state.
- `/ship` can be confused with deploy; in Harness Kit it means land/archive/
  reflect after merge-readiness.
- `/deliver` and `/flywheel` are both composers and need sharper visible
  boundaries.
- `/settle` remains as a deprecated redirect and still costs catalog attention.
- `/karpathy-guidelines` is static doctrine, not a workflow.
- Roster/delegation boilerplate repeats across many skills.
- Long descriptions and long bodies increase always-loaded description tax and
  reduce trigger clarity.

## Desired Outcome

Make the catalog read like a small UNIX toolbelt: focused leaf skills plus a few
thin composers with obvious ownership boundaries.

## Acceptance

- Produce a lifecycle map with one owner per phase:
  `groom -> shape -> implement -> review/refactor/ci/qa/harden -> deliver ->
  yeet -> ship -> deploy -> monitor -> reflect`.
- For every first-party skill, classify it as:
  - leaf;
  - composer;
  - final-mile side effect;
  - meta/harness;
  - doctrine/reference;
  - deprecated redirect.
- Remove or schedule removal for `/settle` once compatibility criteria are met.
- Decide whether `/yeet` keeps its name, gains a clearer primary alias, or is
  renamed. The decision must preserve muscle memory while making the catalog
  understandable to a new operator.
- Tighten `/deliver`, `/flywheel`, `/yeet`, `/ship`, and `/deploy` descriptions
  so each advertises a distinct side-effect boundary.
- Extract repeated delegation-floor boilerplate into shared doctrine/reference
  language where feasible while preserving skill-specific lane guidance.
- Update `docs/skill-catalog-audit.md` with the current post-080 reality:
  `/settle` is deprecated, `/deliver --polish-only` is the merge-readiness
  owner, and six-lane delivery is a `/deliver` mode rather than a new skill.
- Add or extend a catalog lint/audit check that flags:
  - overlapping trigger phrases between composers;
  - long frontmatter descriptions;
  - deprecated skills without a removal target;
  - top-level skill proposals that lack a distinct artifact, side-effect
    boundary, or lifecycle phase.

## Non-goals

- No broad renaming without compatibility aliases and bootstrap/install impact
  review.
- No new top-level observability or swarm/composition skill.
- No changes to generated `index.yaml` by hand.

## Validation

- `python3 scripts/check-frontmatter.py`
- `python3 scripts/analyze-skill-invocations.py --format markdown`
- `bash scripts/generate-index.sh && git diff --exit-code index.yaml`
- `bash scripts/check-docs-site.sh --self-test`
- `dagger call check --source=.`
