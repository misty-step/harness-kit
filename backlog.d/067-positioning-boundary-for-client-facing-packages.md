# Positioning boundary for client-facing packages

Priority: P3
Status: ready
Estimate: S

## Goal

Document the boundary between Harness Kit as a shared harness primitive library
and any buyer-facing governed-AI workflow package that uses Harness Kit under the
hood.

Future agents should not pitch the raw repo as the complete enterprise
onboarding, usage-control, admin, or compliance deliverable. They should treat
Harness Kit as implementation infrastructure and shape a separate package when
the audience is non-operator, executive, admin, procurement, or client-facing.

## Non-Goals

- Do not build the Brandt-facing package in this repo.
- Do not add enterprise control-plane features, RBAC, spend limits, dashboards,
  or kill switches to Harness Kit.
- Do not dilute the repo with sales copy. This is a boundary note for agents
  and maintainers, not a landing page.
- Do not make the boundary Brandt-specific. Brandt is evidence; the rule is
  general.

## Oracle

- [ ] `README.md`, `project.md`, or a small `docs/positioning.md` states that
      Harness Kit is an operator-facing harness primitive library for senior
      engineers and platform teams.
- [ ] The same doc names what Harness Kit is not: an enterprise admin-control
      plane, spend-governance dashboard, procurement-ready onboarding package,
      or nontechnical training artifact.
- [ ] `AGENTS.md` points future agents to the positioning note before they
      answer "should we hand this repo to a client / enterprise / department?"
- [ ] The note gives the recommended split:
      client-facing governed workflow package outside this repo; Harness Kit
      underneath as implementation substrate; admin/control companion layer
      when usage governance is the real buyer need.
- [ ] The note lists concrete evidence a future agent must gather before
      deciding the boundary has changed: installed downstream usage, packaged
      onboarding docs, support/rollback path, security/trust story, and
      admin-control surfaces.
- [ ] `dagger call check --source=.` passes.

## Notes

### Why this belongs in Harness Kit

The prompt-debt reducer surfaced a repeated decision pattern: agents need to
distinguish tool substrate from buyer-facing package. That boundary should be
visible in Harness Kit because Harness Kit's repo identity is changing during the
rebrand and dynamic-delegation pivot.

The implementation of any governed AI workflow offer belongs elsewhere. The
boundary statement belongs here so future agents stop over-scoping this repo.

### Current positioning

Harness Kit is strongest for:

- senior AI platform and developer-enablement teams;
- pilot teams already comfortable with git, local gates, markdown specs, and
  agentic delivery loops;
- consulting delivery teams using a shared harness behind the scenes;
- repo maintainers who need cross-harness skills, system-wide harness setup, and review
  discipline.

Harness Kit is weak as a direct handoff for:

- executive or department-level onboarding;
- enterprise usage/spend/model-access control;
- incident reconstruction dashboards;
- nontechnical training;
- procurement or security-review-ready packaging.

### Relationship to other tickets

- Should land after or with `060-rebrand-harness-library.md`, because naming
  and positioning are tightly coupled.
- Should not block `061-retire-global-static-agents.md` or
  `063-dynamic-delegation-skill-contract.md`; those are architecture work.
- Provides durable guardrails for future Brandt-style packaging discussions
  without making this repo own the Brandt deliverable.
