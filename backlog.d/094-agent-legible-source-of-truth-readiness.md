# Agent-legible source-of-truth readiness pillar

Priority: P1
Status: shaped
Estimate: M

## Goal

Make `/agent-readiness` identify repo state that agents cannot inspect,
verify, or change through code, CLI, MCP, API, or another explicit source path.

## Source Evidence

- Lee Robinson's source-of-truth argument: hidden CMS/UI-only content forces
  agents over a network/UI boundary where grep, edits, diffs, and Git history
  stop working. Source: https://leerob.com/agents.
- User-provided Lee Robinson post summary: code should be source of truth, or
  external state must be made legible through MCP, CLI, or skill.
- Cursor security automation prior art: useful background agents need
  persistent data, dedupe, consistent output, and validated assumptions before
  blocking behavior. Source:
  https://cursor.com/blog/security-agents.

## Non-Goals

- Do not require every external system to be deleted or moved into code.
- Do not build MCP servers, CLIs, or CMS exporters in this ticket.
- Do not make `/agent-readiness` a semantic integration planner.
- Do not weaken existing readiness pillars or replace `meta/INTEGRATION_GUIDE.md`.
- Do not treat a human-only admin UI as acceptable without an explicit waiver
  and expiry.

## Constraints / Invariants

- Code, local files, tests, scripts, and Dagger gates remain the highest
  authority for local repo truth.
- External-system guidance must route through `meta/INTEGRATION_GUIDE.md`
  before choosing MCP, CLI, API, or skill.
- Profile validation must be deterministic and fixture-backed.
- Missing `.harness-kit/agent-readiness.yaml` continues to degrade to
  assessment-only behavior; once a profile exists, stale waivers are invalid.

## Authority Order

tests > schema/profile validation > code/scripts > integration guide > docs > lore

## Repo Anchors

- `skills/agent-readiness/SKILL.md` - readiness workflow and profile contract.
- `skills/agent-readiness/references/pillar-checks.md` - current readiness
  pillars.
- `skills/agent-readiness/references/profile-schema.yaml` - durable profile
  schema.
- `skills/agent-readiness/scripts/profile-crud.py` - deterministic profile CRUD
  and validation path.
- `meta/INTEGRATION_GUIDE.md` - MCP/skill/CLI decision boundary for external
  systems.
- `backlog.d/084-agent-readiness-sdlc-contract.md` - prior profile-contract
  work, now closed.

## Prior Art

- Lee Robinson / Cursor CMS migration: moving content back into code made
  content inspectable, editable, diffable, and revertible for agents.
- `meta/INTEGRATION_GUIDE.md`: external systems should expose a narrow,
  inspectable contract; MCP is for safe, portable access to external context
  and actions.
- `/agent-readiness`: existing pillars already score feedback loops,
  documentation, dev environment, observability, and security, but not source
  legibility.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Do nothing | Treat source legibility as already implied by doctrine | No new surface | Repos can score well while key state remains UI-only | Reject |
| AGENTS-only rule | Add prose saying external truth must be legible | Tiny change | Prose does not produce evidence or remediation | Reject |
| New readiness pillar | Add source-of-truth checks and profile fields | Fits existing assessment workflow | Adds schema surface that must stay small | Choose |
| Integration guide only | Expand `meta/INTEGRATION_GUIDE.md` | Good for design decisions | Does not make consumer repos auditable | Reject as insufficient |
| Repo scanner | Infer hidden state from code patterns | Automated discovery | High false positives and misses real SaaS/CMS state | Defer |
| Build MCP/CLI generators | Automatically scaffold integrations for hidden systems | High leverage after diagnosis | Premature mechanism before inventory | Reject |
| Background invariant monitor | Periodically check source-legibility drift | Good L5 behavior | Depends on profile fields existing first | Defer |

## Tradeoff Matrix

| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| Do nothing | 1 | 5 | 5 | 2 | 5 | 1 | 5 |
| AGENTS-only rule | 2 | 5 | 5 | 2 | 5 | 2 | 4 |
| New readiness pillar | 5 | 3 | 5 | 5 | 5 | 5 | 4 |
| Integration guide only | 3 | 4 | 5 | 3 | 5 | 3 | 4 |
| Repo scanner | 3 | 2 | 4 | 4 | 4 | 3 | 3 |
| MCP/CLI generators | 2 | 1 | 3 | 3 | 2 | 3 | 2 |
| Background monitor | 4 | 2 | 4 | 4 | 4 | 4 | 3 |

The readiness-pillar shape wins because it turns the Lee/Cursor insight into a
repeatable repo audit without choosing an integration mechanism too early.

## Agent Readiness

- Profile source: `.harness-kit/agent-readiness.yaml` if present, otherwise
  `missing`.
- Stack feedback strength: strong; existing Python profile CRUD and shell tests
  can carry fixture validation.
- ADR decision: not required; this is an extension of the existing readiness
  contract.
- Infrastructure path: local profile schema and CLI validation.
- Gate: `bash skills/agent-readiness/scripts/test-profile-crud.sh`,
  `python3 scripts/check-agent-roster.py`, then `dagger call check --source=.`
- Evidence storage: fixture profiles under `skills/agent-readiness/evals/` or
  the existing script test fixture path.
- Mock policy impact: preserved; tests use local profile fixtures only.

## Delegation Evidence

- Roster providers used:
  - `claude` repo investigator, receipt
    `c5a1708e-e046-4590-8141-1d08412317a5`, transcript
    `.harness-kit/traces/provider-lanes/20260603T195448.306288Z-claude-3adaf0ee.txt`.
  - `pi` premise critic, receipt `fe9a2a9a-c48e-41ce-a4a2-9feea8338884`,
    transcript
    `.harness-kit/traces/provider-lanes/20260603T195449.169195Z-pi-5de99953.txt`.
  - `codex` oracle critic, receipt `2920ae5b-d21c-46a6-9202-0861490134fa`,
    transcript
    `.harness-kit/traces/provider-lanes/20260603T195448.042662Z-codex-922a9e5e.txt`.
- Accepted evidence: Claude identified the missing source-legibility inventory
  in `/agent-readiness`; Pi warned that the rule is already doctrine and should
  not become a broad workflow engine; the selected slice keeps it as a profile
  audit with deterministic validation.
- Rejected evidence: automatic MCP/CLI generation and background monitoring are
  deferred until a profile can name the hidden surfaces.
- Waivers: external web research was lead-only; provider lanes used repo-local
  evidence and user-provided source themes.

## Oracle

- [ ] `skills/agent-readiness/references/pillar-checks.md` adds an
      agent-legible source-of-truth pillar or equivalent section with binary
      checks.
- [ ] `skills/agent-readiness/references/profile-schema.yaml` supports
      `state_surfaces[]` entries with at least: `name`, `system_of_record`,
      `agent_access`, `source_path`, `verification_command`, `waiver`,
      `waiver_expires`.
- [ ] `profile-crud.py validate` fails a fixture where `agent_access` is
      `admin-ui-only`, `cms-only`, or `unknown` without a non-expired waiver.
- [ ] `profile-crud.py validate` passes a fixture where external state has an
      MCP, CLI, API, or skill path plus an executable verification command.
- [ ] The readiness report labels hidden state as readiness debt and points to
      `meta/INTEGRATION_GUIDE.md` for remediation.
- [ ] `bash skills/agent-readiness/scripts/test-profile-crud.sh` passes.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Acceptance Evidence

- Acceptance source: profile schema fixture and profile validation command.
- Evidence that proves it: failing and passing profile CRUD fixture tests.
- Exact command/path/route exercised:
  `bash skills/agent-readiness/scripts/test-profile-crud.sh`.
- Contract-change acknowledgment: this intentionally expands the readiness
  profile contract to include source-legibility state.
- Residual risk: a repo can still omit hidden systems from the profile; later
  scanner or review prompts may reduce that but are not part of this slice.

## Observability Plan

- Changed behavior to watch: readiness reports start naming UI-only/CMS-only
  truth surfaces as debt instead of silently accepting them.
- Named signal or evidence surface: profile validation output and readiness
  report rows.
- Instrumentation debt: no fleet-wide aggregation until 088-091 land.

## Implementation Sequence

1. Add source-legibility fields to the profile schema with clear waiver expiry.
2. Add positive and negative profile fixtures.
3. Extend `profile-crud.py validate` and its shell self-test.
4. Add the readiness pillar text and remediation routing to the integration
   guide.
5. Run the profile test, roster check, and full Dagger gate.

## Risk + Rollout

- Risk: teams over-inventory trivial data sources. Mitigate by requiring only
  systems that hold product, content, deployment, security, or workflow truth.
- Risk: pressure to generate integrations immediately. Mitigate by making this
  ticket assessment-only; remediation is future backlog.
- Rollback: remove the schema fields and fixtures, then rerun profile tests and
  Dagger.
