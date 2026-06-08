# Context Packet: Focused Lane Harness Projection

Priority: P1
Status: shaped
Estimate: L

## PRD Summary
- User: lead agents composing delegated provider lanes in Harness Kit.
- Problem: dispatched lanes inherit the globally installed skill/primitives
  catalog. Prompt instructions can discourage irrelevant skills, but cannot
  remove them from model-visible discovery surfaces. This pollutes lane context
  and causes confused subagents.
- Goal: create a thin, testable way for the primary agent to dispatch a lane
  with a bespoke visible harness: selected provider/model, selected local
  skills, selected external aliases, selected tools, one oracle, and explicit
  failure evidence.
- Why now: roster delegation is becoming stronger doctrine, but the provider
  lane should not always carry shape/groom/ship/refactor/etc. once the lane is
  scoped to one job.
- Success signal: a fake provider proves only the requested skill set is
  visible inside a projected lane root; real provider failures such as exhausted
  Claude credits become typed receipts and do not count as successful evidence.

## Product Requirements
- P0: define a `lane_harness` manifest schema for role, provider target, model
  override, allowed local skills, allowed external skill aliases, allowed tool
  labels, oracle, evidence expectations, and fallback policy.
- P0: implement a Rust materializer that creates an ephemeral projected harness
  root containing only the selected visible skills/config needed for one lane.
- P0: support a fake provider smoke path that lists the visible projected skills
  and proves excluded skills are absent without contacting an external model.
- P0: extend delegation receipts with `lane_harness_ref`,
  `lane_harness_sha256`, `projection_status`, and `failure_kind` or equivalent
  fixture-compatible fields.
- P0: classify provider failures explicitly: `missing_binary`, `probe_failed`,
  `probe_timeout`, `spawn_failed`, `auth_required`, `credits_exhausted`,
  `entitlement_missing`, `dispatch_timeout`, `nonzero_exit`,
  `sentinel_mismatch`, and `projection_failed`.
- P0: keep provider CLIs thin. The helper may materialize a requested
  environment and launch one requested lane; it must not select providers,
  score outputs, retry semantically, generate skills, or mutate global harness
  state.
- P1: add conditional smoke probes for Codex, Claude, Pi, and Antigravity that
  report adapter support or typed failure without modifying the user's real
  `$HOME`.
- P1: include lane harness references, projection status, and failure kinds in
  `summarize-delegations`.

## Non-Goals
- No semantic workflow engine.
- No automatic provider ranking, retry tree, or budget optimizer.
- No generated source-repo `.codex/skills`, `.claude/skills`, `.pi/skills`, or
  `.antigravitycli/skills` bridges.
- No global skill installation changes.
- No provider credential repair.
- No worktree provisioning in the first implementation.
- No claims that context bloat is solved before fake-provider and at least one
  real-provider smoke demonstrate the visible-skill boundary.

## Core Design
The primary agent owns composition. Harness Kit provides a narrow primitive:

1. Validate a lane manifest.
2. Materialize an ephemeral projected harness root.
3. Optionally launch exactly one provider command with an environment overlay.
4. Record a receipt.

The primitive is not allowed to infer the team, choose fallback providers, or
read model output as policy. It is closer to `cargo test --manifest-path ...`
than to an orchestrator.

## Manifest Sketch
```yaml
schema: lane_harness.v1
role: "critic"
provider_target: "codex"
model_override: null
allowed_local_skills: ["code-review", "ci"]
allowed_external_aliases: []
allowed_tools: ["shell.readonly", "git.diff"]
oracle:
  kind: "path"
  value: "backlog.d/101-focused-lane-harness-projection.md"
evidence_expectations:
  - "blocking_findings_or_none"
  - "commands_read"
fallback:
  on_provider_failure: "record_and_return"
  replacement_policy: "lead_explicit"
```

## Projection Shape
- Create ignored runtime roots under `.harness-kit/tmp/lane-harness/<id>/`.
- Symlink selected first-party skill folders from `skills/<name>/` into the
  projected provider skill root. Reject path escapes and duplicate aliases.
- Copy only minimal provider config needed for discovery. Prefer generated test
  config over copying user settings when possible.
- Pass env overlay per dispatch, for example `HOME=<projected-root>` or
  provider-specific config dir variables, only for the child process.
- Never rename, hide, or rewrite directories in the user's real home.
- Delete projected roots after dispatch unless `--keep-lane-root` is explicitly
  set for debugging; all runtime roots must be gitignored.

## Provider Adapter Hypotheses
| Provider | First smoke target | Risk | Initial verdict |
|---|---|---|---|
| Fake provider | list projected skill dirs | none | build first |
| Pi | explicit model/tool surface plus env-key auth | tool allowlist is promising but skill discovery must be verified | likely easiest real lane |
| Codex | projected `HOME` / config dir | credential/config behavior may vary | smoke after fake provider |
| Claude | projected `HOME` | spend limits and session-file auth risk; plugin collisions visible today | smoke but never rely on it alone |
| Antigravity | projected `HOME` / Gemini config root | zero exit is insufficient; needs sentinel output | smoke with strict sentinel |
| Cursor | workspace/keychain coupling | poor fit for first slice | defer |
| Grok | env-key auth but unclear skill discovery | good critic, weaker isolation target | defer |

## Failure And Fallback Semantics
- Roster probe means "binary/config appears callable", not "lane can run now".
- A provider failure writes a receipt and does not satisfy the evidence floor
  unless the failure itself was the intended evidence.
- `credits_exhausted` and `auth_required` are normal lane outcomes. The lead
  dispatches a replacement provider explicitly and reports the failed receipt.
- Automatic fallback is limited to syntactic behavior: record, return nonzero,
  and let the lead decide. No semantic replacement inside the helper.
- A projection failure is not a provider failure. Record `projection_failed`,
  preserve the normal provider state, and fall back only by lead decision.

## Implementation Slices

### Slice A: Evidence Contract
- Add a `lane_harness` manifest validator in
  `crates/harness-kit-checks/src/`.
- Validate provider ids against roster, local skill names against `skills/`,
  external aliases against `registry.yaml`, non-empty oracle/evidence fields,
  closed schema fields, and secret-like text rejection.
- Add fixture validation to `check-agent-roster --repo .`.
- Add receipt fields and `summarize-delegations` output.

### Slice B: Projection Materializer
- Add a Rust `materialize-lane-harness` command that takes a manifest and emits
  a projected root path plus sha256 digest.
- Implement fake provider visibility tests: included skills appear, excluded
  skills do not, nested files remain under each skill, path escapes fail.
- Keep materialization independent from real provider dispatch.

### Slice C: Single-Lane Dispatch Integration
- Add optional dispatch args such as `--lane-harness <path>` and
  `--keep-lane-root`.
- Dispatch still runs one requested provider command and appends one receipt.
- Add typed failure classification and process cleanup evidence.
- Add conditional real-provider smokes guarded by local availability and env.

## Tests / Acceptance Oracle
- [ ] Invalid lane manifests fail for unknown provider, unknown skill, unpinned
      external alias, empty oracle/evidence, unknown field, duplicate skill,
      path escape, and secret-like text.
- [ ] Fake provider with manifest `["ci"]` sees `ci` and cannot see `shape`,
      `groom`, or any other global skill.
- [ ] Materializer never writes to real `$HOME`, generated `index.yaml`, or
      source-repo harness bridge dirs.
- [ ] Provider failure fixtures classify `credits_exhausted`, `auth_required`,
      `dispatch_timeout`, `sentinel_mismatch`, and `projection_failed`.
- [ ] Failed Claude spend-limit lane is recorded and excluded from successful
      evidence counts.
- [ ] `summarize-delegations --backlog-ref 101 --format text` surfaces provider,
      model, lane harness ref, projection status, failure kind, and lead verdict.
- [ ] `cargo test --workspace --locked lane_harness agent_roster summarize_delegations`
      passes.
- [ ] `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`
      passes.
- [ ] `cargo run --locked -p harness-kit-checks -- check-runtime-primitives --repo .`
      passes if runtime projection paths are introduced.
- [ ] `dagger call check --source=.` passes before shipping.

## Delegation Evidence
- Roster probe: `cargo run --locked -p harness-kit-checks -- probe-agent-roster
  --validate-only` reported the repo roster valid.
- Claude adapter lane failed with monthly spend limit; receipt
  `2f17cadc-aa7c-42da-adec-78d30e17fa1f`. Accepted as failure evidence only.
- Codex failure/fallback lane succeeded; receipt
  `db512cc0-9ba8-4158-9cf9-5892c18920ad`. Accepted for typed failure taxonomy
  and "record, return, lead decides" fallback semantics.
- Grok thinness critic succeeded; receipt
  `6be7989a-f286-4617-991a-48671dd4a36c`. Accepted as overbuild guard;
  rejected where it says projection should be killed entirely, because prompt
  scoping cannot remove globally loaded skills.
- Antigravity adapter lane succeeded; receipt
  `0a63b823-05ff-4661-8336-22c6c932ef42`. Accepted for projected-root adapter
  facts and auth-risk warnings; defer provider-specific claims until smoke.

## Open Research Questions
- Which providers honor `HOME` or config-dir projection without breaking auth?
- Can Pi's explicit tool allowlist give us true tool isolation while filesystem
  projection handles skill isolation?
- Should retained projected roots redact config/auth paths before evidence is
  shared?
- What is the minimal context-bloat metric: visible skill count, prompt token
  delta, wrong-skill invocations, or task success variance?

## Verdict
Build this in stages. The receipt-only design is too weak because it measures
intent while leaving global skill inheritance untouched. A full capsule
launcher is too large and violates the "no semantic workflow engine" line. The
right middle path is a typed lane manifest plus an ephemeral projected harness
root, proven first by fake-provider visibility tests, then wired into one-lane
dispatch with typed failure receipts.
