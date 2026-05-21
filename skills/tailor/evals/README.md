# /tailor evals

Capability under test: `/tailor` produces a repo-specific harness plan with
deterministic install mechanics across workflow, universal, external, and
agent buckets.

Expected failure mode: a plan that omits external install mechanics
(`skills/.external/<alias>` absolute symlinks + sibling markers), picks
frontend externals for non-frontend repos, or drifts from cross-harness bridge
wording.

Post-install failure mode: a harness validates mechanically but is not bespoke.
The acceptance audit must use deterministic evidence as critic input, persist a
verdict, and reject generic-but-valid output without inventing numeric quality
scores.
