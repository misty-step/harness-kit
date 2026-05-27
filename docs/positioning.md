# Harness Kit Positioning

Harness Kit is an operator-facing harness primitive library for senior engineers,
AI platform teams, developer-enablement teams, and consulting delivery teams
that need reusable agent workflows across multiple harnesses and repositories.

Harness Kit is implementation substrate. It is not an enterprise admin-control
plane, spend-governance dashboard, procurement-ready onboarding package,
security-review packet, or nontechnical training artifact.

## Boundary

Use Harness Kit when the audience is technical operators who can work with git,
local gates, markdown specs, skills, provider rosters, and agentic delivery
loops.

Shape a separate client-facing governed workflow package when the audience is
an executive, department lead, admin, procurement reviewer, security reviewer,
or nontechnical team. That package may use Harness Kit underneath, but it should
own the client-facing onboarding, workflow framing, support path, rollout story,
and usage-governance experience.

When usage governance is the buyer need, the right split is:

- client-facing governed workflow package outside this repo;
- Harness Kit underneath as the implementation substrate;
- admin/control companion layer outside this repo for usage, spend, model
  access, rollback, audit, and policy surfaces.

## Revisit Criteria

Do not change this boundary because a single client conversation sounds
enterprise-shaped. Revisit it only when there is concrete evidence that
Harness Kit itself has become a buyer-facing package:

- installed downstream usage by non-Harness Kit operators;
- packaged onboarding docs for nontechnical or procurement-facing audiences;
- explicit support, rollback, upgrade, and incident-response path;
- security and trust story suitable for external review;
- admin/control surfaces for usage governance, spend governance, model access,
  audit, and policy enforcement.

Until that evidence exists, keep Harness Kit small and operator-facing. Put
client packaging and enterprise control-plane work in a separate package.
