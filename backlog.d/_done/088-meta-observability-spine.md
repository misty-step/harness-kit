# Meta-observability spine for skill analytics

Priority: P1
Status: ready
Estimate: M

## Goal

Make Harness Kit able to answer, from local evidence, which skills are used
across repos, how they chain together, and which work items they touched.

## Why Now

Harness Kit already has useful but separate observability primitives:

- Claude skill invocation rows in `~/.claude/skill-invocations.jsonl` via
  `harness-kit-checks claude-hook skill-invocation-tracker`.
- Delegation receipts in `.harness-kit/traces/delegations.jsonl` via
  `scripts/record-delegation.py` and `scripts/summarize-delegations.py`.
- Work lifecycle events in `.harness-kit/work/ledger.jsonl` via
  `scripts/work-ledger.py`.
- Trace/evidence references via `skills/trace/scripts/trace_record.py`.
- Observability coverage report shape in
  `skills/monitor/references/observability-coverage.md`.

Those streams are locally useful but not joined. The operator cannot ask:
"show skill frequency by repo", "what usually runs before `/ci`?", or "what
skill sequence did backlog 087 actually use?" without hand-grepping logs.

Backlog `031` is parked until real cycle data exists. This ticket creates the
queryable signal layer that later auto-tuning would consume.

## Non-Goals

- Do not build a dashboard, hosted service, daemon, cron watcher, or semantic
  workflow engine.
- Do not ingest raw prompts, raw tool output, raw transcripts, screenshots,
  browser state, secrets, credentials, or customer data.
- Do not require Langfuse, Phoenix, OpenTelemetry, Tessl, ClickHouse, or any
  other backend to use the local report.
- Do not make missing token/cost fields count as zero. Unknown is `unknown`.
- Do not unpark `031` or generate harness mutations from analytics yet.

## Constraints / Invariants

- Filesystem-first: source records stay JSONL and repo/user-local.
- Append-only raw evidence; reports are derived artifacts.
- Cross-repo aggregation stores metadata only: repo id, project, skill,
  session id, backlog/work refs, timestamps, status, evidence refs.
- The analyzer must tolerate missing stores and older schema rows.
- Raw source paths in reports are repo-relative where possible.
- All schema examples must pass a deterministic self-test.

## Authority Order

tests > schemas/fixtures > code > reports > docs > external prior art > lore

## Repo Anchors

- `harness-kit-checks claude-hook skill-invocation-tracker` - current passive skill
  invocation event source.
- `skills/harness-engineering/references/mode-audit.md` - currently describes
  Hot/Warm/Cold/Dead usage reports but has no script behind it.
- `scripts/work-ledger.py` - work lifecycle event store and `work_id` shape.
- `scripts/summarize-delegations.py` - existing narrow receipt report pattern.
- `skills/monitor/references/observability-coverage.md` - says Harness Kit
  observability is a report/config contract, not a dashboard.
- `skills/reflect/references/distill.md` - says retros gather skill invocation
  logs when available.
- `.harness-kit/examples/` - fixture home for schema and report self-tests.

## Prior Art

- Langfuse traces capture LLM calls, retrieval, tool executions, custom logic,
  timing, inputs, outputs, and metadata; its metrics layer slices quality,
  cost, latency, and volume by trace name, user, tags, release, model, prompt,
  and other dimensions:
  https://langfuse.com/docs/observability/overview and
  https://langfuse.com/docs/metrics/overview.
- Langfuse's agent-facing API/CLI/MCP posture is useful prior art for making
  analytics queryable by agents, especially full-text observation search and
  API-covered metrics:
  https://langfuse.com/agents.
- Arize Phoenix accepts OpenTelemetry traces and treats a trace as model calls,
  retrieval, tool use, and custom logic, plus evals over traces/spans:
  https://arize.com/docs/phoenix.
- OpenTelemetry GenAI semantic conventions define events, exceptions, metrics,
  model spans, and agent spans, but the current status is development and the
  spec warns about stability opt-in:
  https://opentelemetry.io/docs/specs/semconv/gen-ai/.
- Tessl's useful lesson is to treat skills as versioned, evaluated, distributed
  artifacts and to track adoption from registry to activation:
  https://tessl.io/ and
  https://docs.tessl.io/introduction-to-tessl/quickstart-skills-docs-rules.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Local JSONL analyzer | Read existing stores and emit markdown/json reports | Fits repo, private, easy to test | Limited ad hoc querying | Choose first |
| Local SQLite cache | Ingest JSONL into `~/.harness-kit/analytics/*.sqlite` | Stronger queries, still local | More schema/migration surface | Defer until JSONL report hurts |
| Langfuse-first export | Send every event to Langfuse | Good dashboards and cost/trace APIs | Requires hosted/self-hosted backend and privacy review | Defer adapter |
| Phoenix/OpenTelemetry-first | Emit OTLP spans from every harness | Vendor-neutral and interoperable | Premature collector/runtime burden | Defer adapter |
| Tessl-style registry analytics | Measure skill install/adoption/eval lift | Strong skill-product lens | Does not answer local invocation joins by itself | Borrow concepts |
| Transcript-mining first | Mine chat logs for patterns directly | Rich qualitative signal | Privacy and content-retention risk | Split to later ticket |
| Dashboard app | Build UI over all runs | Attractive at-a-glance surface | Becomes product/control plane, high maintenance | Reject now |

## Tradeoff Matrix

| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| Local JSONL analyzer | 5 | 4 | 5 | 5 | 5 | 5 | 5 |
| Local SQLite cache | 4 | 3 | 5 | 4 | 4 | 4 | 4 |
| Langfuse-first export | 3 | 2 | 3 | 4 | 3 | 3 | 3 |
| Phoenix/OpenTelemetry-first | 4 | 2 | 4 | 3 | 3 | 3 | 3 |
| Tessl-style registry analytics | 3 | 3 | 4 | 4 | 4 | 3 | 3 |
| Transcript-mining first | 3 | 2 | 2 | 3 | 3 | 3 | 2 |
| Dashboard app | 2 | 1 | 3 | 2 | 2 | 3 | 1 |

The first slice should be the local JSONL analyzer because it lights up dark
data immediately, keeps raw evidence local, and creates a stable schema for
later export adapters.

## Proposed Shape

Add `scripts/analyze-skill-invocations.py` with:

- Inputs:
  - `--skill-log`, defaulting to `~/.claude/skill-invocations.jsonl`.
  - `--work-ledger`, defaulting to `.harness-kit/work/ledger.jsonl`.
  - `--delegations`, defaulting to `.harness-kit/traces/delegations.jsonl`.
  - `--since`, `--repo`, `--project`, `--skill`, `--format text|json|markdown`.
- Output sections:
  - skill frequency by repo/project;
  - hot/warm/cold/dead classification using the mode-audit thresholds;
  - top skill transitions grouped by session;
  - per-backlog/work skill sequence when `backlog_ref` or `work_id` exists;
  - unmatched rows and missing-join warnings;
  - source coverage summary naming which stores were present or absent.
- Fixture:
  - `.harness-kit/examples/skill-invocations.jsonl` with two sessions and at
    least one repeated transition.
- Self-test:
  - `python3 scripts/analyze-skill-invocations.py --self-test`.

## Agent Readiness

- Profile source: no `.harness-kit/agent-readiness.yaml` inspected; use existing
  Harness Kit Python script/test style.
- Stack feedback strength: Python stdlib scripts plus shell self-tests.
- ADR decision: not required; backlog ticket plus tests are enough.
- Infrastructure path: CLI script over local files; no service setup.
- Gate: `python3 scripts/analyze-skill-invocations.py --self-test`,
  `python3 scripts/check-agent-roster.py`, then `dagger call check --source=.`
- Evidence storage: `.harness-kit/examples/skill-invocations.jsonl` plus
  generated report fixture output in the script self-test.
- Mock policy impact: preserved; fixture files are external evidence, not
  mocked internal collaborators.

## Delegation Evidence

- Roster providers used:
  - `claude`, receipt `c1bc871f-4122-4786-a2df-e04e62a03c91`, repo
    investigator.
  - `codex`, receipt `004cb27a-ff40-4918-9ed7-40478b196a7f`, product/premise
    critic.
- Native subagents used:
  - `explorer` repo mapper found the existing receipt, trace, ledger, monitor,
    and reflect surfaces plus missing cross-repo analytics.
  - default architecture critic recommended a normalized append-only local
    event/report layer and warned against dashboards/daemons.
- retired bench:
  - `research/quick`, systems and verification agents complete; accepted
    finding that existing streams lack a join key and that
    review-score/skill-usage loops are starved.
- External research:
  - Langfuse, Phoenix, OpenTelemetry, and Tessl docs were used for prior art.
  - Exa surfaced adjacent open-source skill/agent observability repos but did
    not change the first-slice recommendation.
  - xAI returned a broad stack recommendation; accepted only where it aligned
    with official docs and repo constraints.
- Rejected evidence:
  - Hosted dashboard/control-plane first.
  - Raw transcript mining as the first slice.
  - Treating token/cost absence as zero.
- Waivers:
  - One native Ousterhout role failed because its pinned model is unavailable
    on this account; a default native critic replaced it.

## Oracle

- [ ] `scripts/analyze-skill-invocations.py` exists and reads the skill log,
      work ledger, and delegation receipts without requiring any one store to
      exist.
- [ ] `.harness-kit/examples/skill-invocations.jsonl` contains deterministic
      fixture rows covering at least two sessions, repeated skill transitions,
      and one row with `backlog_ref`/`work_id`.
- [ ] `--format markdown` emits a frequency table, transition table, source
      coverage section, and missing-join warnings.
- [ ] `--format json` emits stable keys for `skills`, `transitions`,
      `coverage`, and `warnings`.
- [ ] Missing token/cost data is rendered as `unknown`, never `0`.
- [ ] `skills/harness-engineering/references/mode-audit.md` points to the
      script rather than asking the model to hand-roll the report.
- [ ] `python3 scripts/analyze-skill-invocations.py --self-test` passes.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Observability Plan

- Changed behavior to watch: whether skill analytics reports are generated
  from real rows instead of hand-authored summaries.
- Named signal or evidence surface:
  `.harness-kit/examples/skill-invocations.jsonl`, script self-test output,
  and future report artifacts.
- Instrumentation debt if no signal exists: Codex/Pi/Antigravity invocations
  remain invisible until the cross-harness adapter ticket ships.

## Risk + Rollout

- Privacy leak from raw args: report only counts and normalized refs by
  default; add `--include-args` only if a later ticket justifies it.
- Misleading cross-harness claims: label current source coverage explicitly.
- Schema drift: tolerate unknown fields and older rows.
- Overbuilding: stop at CLI report and fixtures; no dashboard or service.
- Rollback: remove script, fixture, and mode-audit pointer.
