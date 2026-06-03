# Agent-readiness SDLC contract and repo profile CRUD

Priority: P1
Status: done
Estimate: M

## Goal

Make agent readiness part of the core Harness Kit SDLC, not a standalone audit.
For each repository, Harness Kit should be able to create, read, update, and
delete a small repo-local readiness profile that captures what makes the codebase
easy for agents to understand and hard for agents to break. `/shape`,
`/deliver`, `/ship`, `/agent-readiness`, and repo-skill generation should use
that profile when choosing stacks, shaping acceptance gates, and reporting
readiness regressions.

## Non-Goals

- Do not build a new workflow engine or dashboard.
- Do not make Factory, Droid, GitHub, or any cloud service the source of truth.
- Do not require Rust for every project. Encode a strict-feedback bias and a
  decision gate, not dogma.
- Do not write ADRs for routine decisions. Require ADRs only when the decision
  is hard to reverse, surprising without context, and the result of a real
  tradeoff.
- Do not replace repo-local judgment with a universal score.

## Constraints / Invariants

- Git and filesystem state are the durable primitive. GitHub comments and hosted
  dashboards may be adapters, never the canonical evidence store.
- Agent-ready means two things at once: easy for agents to work on, hard for
  agents to break.
- Prefer CLI/API/SDK-managed infrastructure. Human-only dashboard setup,
  click-ops, or hidden vendor state is a readiness smell.
- Prefer stack choices with fast, strict feedback: compiler errors, strict type
  checking, format/lint gates, deterministic local tests, behavior coverage, and
  Dagger-backed CI.
- Keep modules deep and interfaces small. Smaller focused services beat broad
  context-heavy surfaces when the split has a real boundary.
- Mock only external boundaries. Internal mocks are a readiness regression
  because they hide integration failures from agents.

## Repo Anchors

- `skills/agent-readiness/SKILL.md` - current audit/report/fix skill.
- `skills/agent-readiness/references/agent-readiness-principles.md` - already
  captures the Factory-style readiness thesis.
- `skills/agent-readiness/references/pillar-checks.md` - binary readiness
  criteria by pillar.
- `skills/shape/SKILL.md` - context packet and stack/architecture shaping.
- `skills/deliver/SKILL.md` and `skills/ship/SKILL.md` - readiness regression
  and closeout reporting points.
- `backlog.d/052-harness-kit-config-contract.md` - broader `.harness-kit/*.yaml`
  config contract; this ticket should either land a narrow profile first or
  feed 052's schema.
- `backlog.d/065-repo-grounded-acceptance-contract.md` - live-repo evidence
  contract this should compose with.
- `backlog.d/075-skillify-mvp.md` - future skill CRUD primitive this should not
  block on.

## Readiness Profile CRUD

Implement a small, schema-checked repo-local profile such as
`.harness-kit/agent-readiness.yaml`:

- `create`: generate the profile from live repo evidence, detected stack,
  known gates, ADR locations, CI commands, coverage commands, and infrastructure
  management surfaces.
- `read`: show the current readiness contract in a terse, agent-readable form
  before `/shape`, `/deliver`, `/ship`, and `/agent-readiness` make claims.
- `update`: record deliberate improvements, accepted waivers, ADR links,
  coverage/gate threshold changes, and strict-feedback stack decisions.
- `delete`: remove stale waivers or obsolete readiness assumptions when the
  repo no longer matches them; do not silently preserve dead exemptions.

The first implementation can be a script under `skills/agent-readiness/scripts/`
or a shared config helper if 052 lands first. The important boundary is that the
profile is plain data, reviewable in Git, and usable by every harness.

## Oracle

- [ ] `skills/agent-readiness/SKILL.md` distinguishes assessment mode from SDLC
      contract mode, and routes profile CRUD through a deterministic script or
      schema-backed helper.
- [ ] A reference file defines the readiness profile schema, including stack
      feedback strength, gates, coverage/reporting commands, ADR policy,
      infrastructure manageability, module-boundary notes, mock policy,
      observability access, and known waivers with expiry.
- [ ] `/shape` context packets include an "Agent Readiness" section for new
      systems or substantial architecture changes: stack choice, ADR decision,
      CLI/API/SDK infrastructure path, Dagger/local gate, and evidence storage.
- [ ] `/deliver` and `/ship` report whether a change improved, preserved, or
      regressed the repo readiness profile; regressions require an explicit
      contract-change note.
- [ ] `/create-repo-skill` and future `/skillify` can read the profile to avoid
      generating repo-local instructions that contradict live readiness policy.
- [ ] The profile checker rejects blank, stale, expired, or placeholder-only
      waivers.
- [ ] The docs companion or generated index links the readiness profile without
      hand-editing `index.yaml`.
- [ ] `dagger call check --source=.` passes.

## Implementation Sequence

1. Add the readiness profile schema/reference and a minimal create/read/update/
   delete/validate script under `skills/agent-readiness/`.
2. Add focused tests for profile parsing, stale waiver rejection, and round-trip
   CRUD behavior.
3. Patch `/agent-readiness` to use the profile as the durable assessment record.
4. Patch `/shape`, `/deliver`, and `/ship` with one-line routing hooks and the
   required readiness evidence fields.
5. Add the check to Dagger only after the local script is stable.
6. Rebuild generated docs/indexes through the existing scripts and run the full
   gate.

## Alternatives Considered

- Standalone readiness report only: easy to add, but it does not affect SDLC
  behavior. Reject.
- GitHub comments/dashboard as source of truth: polished UX, but vendor-specific
  and hard to replay. Reject.
- Merge entirely into 052 config contract: avoids another file, but delays useful
  readiness semantics behind a broader config ticket. Partial.
- Build `/skillify` first and hide profile CRUD there: reuses a future CRUD
  primitive, but blocks the SDLC contract on a larger MVP. Reject for now.

## Research Notes

- Factory Agent Readiness measures repositories across eight pillars and five
  gated maturity levels, then prioritizes remediation. Level 3 is framed as the
  practical target for production-grade autonomous work.
- Droid `/readiness-report` persists historical reports and `/readiness-fix`
  remediates failing criteria. Harness Kit should copy the closed-loop shape,
  but keep the durable state in repo-local Git data.
- Factory's linter guidance treats lint rules as executable policy on the hot
  path: local dev, pre-commit, CI, PR automation, and agent toolchains.
- Factory Missions and OpenAI harness-engineering notes converge on milestone
  validation, fresh worker context, legible app/log/metric surfaces, and humans
  designing environments and feedback loops rather than hand-coding every line.
- OpenAI's Codex safety guidance reinforces bounded execution, managed config,
  network policy, and agent-native logs as readiness criteria for serious use.
- Kodus `agent-readiness` is useful prior art for local, configurable, CI-gated
  readiness checks.

## Delegation Evidence

- claude receipt `36409cc3-9916-4abb-be5a-a29a2c61b73a` ranked 081, 076, 068,
  072, and 065 as the compounding backlog path; accepted as sequencing pressure.
- pi receipt `1745ac96-93ea-46b7-ba4e-a8bea0ecd7d0` independently ranked 081,
  073, 061b, 072, and 069; accepted for the warning that mechanical evidence
  gates and bounded dispatch should precede broader runtime ambition.

## Related

- Depends on or should be sequenced after: `076`, `081`.
- Composes with: `052`, `065`, `073`, `075`.

## What Was Built

- Added `skills/agent-readiness/references/profile-schema.yaml` as the
  repo-local readiness profile contract for
  `.harness-kit/agent-readiness.yaml`.
- Added deterministic `skills/agent-readiness/scripts/profile-crud.py` with
  create/read/update/delete/validate support, live repo defaults, and strict
  waiver validation for missing, stale, expired, or placeholder waivers.
- Added `skills/agent-readiness/scripts/test-profile-crud.sh` and a Dagger
  lane that exercises profile creation, read/validate, waiver update/delete,
  and invalid waiver rejection.
- Patched `/agent-readiness`, `/shape`, `/deliver`, `/ship`, and
  `/create-repo-skill` to consume the profile as SDLC contract evidence while
  gracefully degrading when no profile exists.

Closes-backlog: 084
