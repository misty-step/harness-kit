# Host-Agnostic CI Design

Use this when designing, auditing, or repairing CI across local machines,
GitHub Actions, Azure Pipelines, self-hosted runners, or future hosts.

Harness Kit remains a focused agent primitive catalog. Do not build a CI
framework here. The `/ci` skill teaches agents how to shape repo-local CI; the
implementation belongs in the consumer repo.

## North Star

One repo-owned contract runs everywhere:

```text
local shell / hook
GitHub Actions
Azure Pipelines
self-hosted runner
future runner
        |
        v
repo-owned gate command / Dagger function / build-system target
```

Provider YAML is a thin caller. If changing CI behavior requires editing GitHub
or Azure YAML instead of the repo-owned gate, the contract is in the wrong
place.

## Substrate Choice

Choose the smallest substrate that preserves the invariant.

| Substrate | Use When | Avoid When |
|---|---|---|
| Direct host command (`scripts/check.sh`, `bin/gate`, package script, Make/Just/Task) | Tools are already native, fast, deterministic, and easy to install locally and in hosted CI. | Environment drift is the real bug, or service orchestration dominates. |
| Dagger | The repo needs portable execution across local and hosted runners, pinned container/service dependencies, graph caching, service orchestration, or traceable pipeline functions. | It only wraps ordinary lint/typecheck/unit/build commands and makes the inner loop slower. |
| Build-system target (Bazel/Pants/Nix-backed) | The repo already earns that build graph through scale, language mix, remote cache/execution, or reproducible environments. | The build system would be introduced only to satisfy CI taste. |
| Provider reusable workflow/template | Repeating provider boilerplate across repos, while the real gate stays repo-owned. | It becomes the only source of truth or hides logic from local agents. |

Dagger is neither mandatory nor legacy by default. It is a strong candidate for
host-agnostic CI when its benefits are load-bearing: typed functions,
containerized dependencies, services, cache graph, and OpenTelemetry-backed
observability. It is a bad default when it turns a 20-second local gate into a
multi-minute Docker bootstrap for ordinary checks.

## Two-Tier Gate

Every non-trivial repo should expose:

- **Fast gate:** seconds to low minutes; deterministic; no network unless local
  emulation is the point; runs from hook and agent inner loop. Typical checks:
  format, lint, typecheck, changed/focused tests, shell syntax, cheap secret
  scan, generated drift for touched surfaces.
- **Full gate:** required before merge, deploy, or release; can use Dagger,
  browser, network, provider sandboxes, mutation, full coverage, performance,
  packaging, and live-readiness checks.

Moving a check out of the fast gate is valid only when the same invariant stays
required in the full gate. If the full gate is too slow for every push, split
it; do not delete it.

## Comprehensive Coverage

Pick checks for failures this repo can actually suffer:

- Correctness: unit, integration, contract, e2e, replay, golden fixtures.
- Type and static checks: typecheck, lint, shell syntax, schema validation.
- Coverage quality: diff coverage first; project coverage as trend; mutation
  or fuzzing on core logic when feasible.
- Generated drift: docs, indexes, clients, schemas, fixtures, lockfiles.
- Security: secret scan over source, logs, generated artifacts, commit
  messages, PR/release metadata; dependency and license scan; IaC policy where
  relevant.
- Supply chain: pinned tool versions, lockfiles, provenance, artifact hashes,
  release packaging parity.
- Performance: benchmark deltas with confidence or explicit noise floor;
  bundle size/perf budgets where user-facing.
- Reliability: flaky test detection, retries reported not hidden, cancellation
  for stale PR runs, no cancellation for deploy/main release unless safe.

Avoid global vanity gates. Global coverage, complexity, or maintainability
numbers are reports unless ratcheted or diff-scoped.

## Observability And Reports

A strong gate leaves context for future agents:

- Run digest: command, host, commit/range, duration, critical path, queue time
  if available, cache hit/miss summary if available.
- Test report: JUnit/TAP/native JSON plus failing test logs.
- Coverage: project trend plus patch/diff coverage; raw LCOV/Cobertura when
  possible.
- Mutation/fuzz/perf: survivors, seeds, input corpus, confidence/noise note.
- Security: redacted findings with rule, field, path/metadata source, and
  severity.
- Artifact evidence: build outputs, checksums, SBOM/provenance if relevant,
  deploy manifest, rollback/readiness proof.
- Trace/log link: Dagger trace, hosted workflow run, OTel trace, or structured
  logs with correlation id when the substrate supports it.

Do not require every report in every repo. Require enough evidence that a
fresh agent can answer: what ran, why it failed, what changed, and what remains
unverified.

## Host Portability Checks

When auditing or designing a gate, ask:

- Can I run the same repo-owned contract locally and on GitHub/Azure?
- Does CI install the same tool versions as local, or pin them explicitly?
- Are secrets required only for lanes that truly need them?
- Are network/provider checks isolated from offline deterministic gates?
- Can path filters skip a required workflow into a pending or false-green state?
- Are advisory workflows visibly advisory?
- Are duplicate job names avoided so branch protection is unambiguous?
- Are stale PR runs cancelled while main/deploy runs are protected?
- Is cache configuration keyed by real inputs, not mutable outputs?
- Is the report artifact durable and linked from the completion summary?

## Agent-Specific Requirements

Agents need CI to be legible:

- Name the one command the agent should run before claiming done.
- Name the fast command and the full command separately.
- Keep sidecar workflows classified: required, advisory, scheduled, deploy,
  release, bot/review.
- Fail with actionable, redacted diagnostics; do not make agents inspect a
  provider UI for basic errors.
- Keep generated artifacts and evidence packets in predictable paths.
- Record skipped heavyweight checks as residual risk, not as green.
- Treat hooks as convenience, not trust boundaries; CI or branch protection
  must enforce the merge/deploy floor.

## Source Anchors

- Dagger docs: https://docs.dagger.io/
- Dagger CI quickstart: https://docs.dagger.io/0.16.3/ci/quickstart/
- Dagger observability: https://docs.dagger.io/features/observability/
- DORA metrics: https://dora.dev/guides/dora-metrics/
- GitHub reusable workflows: https://docs.github.com/en/actions/concepts/workflows-and-actions/reusing-workflow-configurations
- GitHub status checks: https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/collaborating-on-repositories-with-code-quality-features/about-status-checks
- Azure templates: https://learn.microsoft.com/en-us/azure/devops/pipelines/process/templates?view=azure-devops
- Azure branch policies: https://learn.microsoft.com/en-us/azure/devops/repos/git/branch-policies-overview?view=azure-devops
- OpenTelemetry Collector: https://opentelemetry.io/docs/collector/
