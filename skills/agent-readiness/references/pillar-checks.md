# Pillar Checks

Binary pass/fail checks for each pillar. Every check must cite evidence
(file path, config value, command output). Organized by maturity level —
L1 checks are foundational, L5 checks are aspirational.

Subagents: read YOUR pillar's section. Run every check. Report pass/fail
with evidence.

---

## 1. Style & Validation

### L1 — Foundational
- **Linter configured**: `.eslintrc*`, `biome.json`, `.rubocop.yml`, `pyproject.toml [tool.ruff]`, or equivalent exists
- **Formatter configured**: `prettier`, `black`, `gofmt`, `rustfmt`, or equivalent configured
- **Type checker enabled**: `tsconfig.json` with `strict: true`, `mypy.ini`, `pyright`, or language-native types

### L2 — Documented
- **Pre-commit hooks**: `.husky/`, `.pre-commit-config.yaml`, `lefthook.yml`, or lint-staged configured
- **Lint runs in CI**: CI config runs linter and fails on violations
- **Editor config**: `.editorconfig` exists for cross-editor consistency

### L3 — Standardized
- **Zero lint warnings policy**: CI fails on warnings, not just errors
- **Type coverage >80%**: For gradually-typed languages (TS, Python), most code is typed
- **Auto-fix on save/commit**: Formatter runs automatically, not manually

### L4 — Optimized
- **Custom lint rules**: Project-specific rules encoding architectural boundaries
- **Lint rule for agent patterns**: Named exports, explicit DTOs, predictable file naming

### L5 — Autonomous
- **Self-healing lint**: Agent can add/fix lint rules based on recurring violations

---

## 2. Build & CI

### L1 — Foundational
- **Build command exists**: `package.json` scripts, `Makefile`, `Cargo.toml`, or equivalent
- **CI config exists**: `.github/workflows/`, `.gitlab-ci.yml`, `Jenkinsfile`, etc.
- **Dependencies pinned**: Lockfile exists (`package-lock.json`, `yarn.lock`, `Cargo.lock`, `poetry.lock`)

### L2 — Documented
- **Build documented**: README or CLAUDE.md explains how to build
- **CI runs on PRs**: CI triggers on pull request events
- **Single build command**: One command builds the whole project (no multi-step dance)

### L3 — Standardized
- **CI runs lint + typecheck + test**: All three verification layers in CI
- **CI fails fast**: Lint/typecheck before tests (fail in seconds, not minutes)
- **Dependency vulnerability scanning**: `npm audit`, `pip-audit`, Dependabot, or equivalent

### L4 — Optimized
- **CI <5 minutes**: Total CI pipeline completes in under 5 minutes
- **Parallel CI jobs**: Lint, typecheck, test run in parallel
- **Build caching**: Turbo, nx, or CI-level caching for incremental builds

### L5 — Autonomous
- **CI auto-fixes**: Bot auto-fixes lint/format violations via PR
- **Flaky test detection**: CI tracks and quarantines flaky tests automatically

---

## 3. Testing

### L1 — Foundational
- **Test runner configured**: Jest, Vitest, pytest, go test, cargo test, or equivalent
- **At least one test exists**: Any test file that passes
- **Tests run locally**: `npm test` or equivalent works without external dependencies

### L2 — Documented
- **Test command documented**: README or CLAUDE.md explains how to run tests
- **Test directory structure**: Tests in `__tests__/`, `tests/`, `*_test.go`, or co-located
- **Coverage tool configured**: Coverage reporting set up (even if threshold is low)

### L3 — Standardized
- **Coverage >60%**: Line or branch coverage above 60%
- **Integration tests exist**: Tests that exercise multiple modules together
- **Tests run in CI**: CI runs the full test suite and fails on failure

### L4 — Optimized
- **Coverage >80%**: Line or branch coverage above 80%
- **Test suite <5 minutes**: Full suite completes locally in under 5 minutes
- **E2E tests exist**: End-to-end tests for critical user flows
- **Coverage enforced**: CI fails if coverage drops below threshold

### L5 — Autonomous
- **Mutation testing**: Tools like Stryker or mutmut to verify test quality
- **Test generation**: Agent can generate meaningful tests, not just coverage padding
- **Visual regression tests**: Screenshot comparison for UI components

---

## 4. Documentation

### L1 — Foundational
- **README exists**: `README.md` with project description
- **Setup instructions**: How to install dependencies and run the project
- **License file**: `LICENSE` or `LICENSE.md`

### L2 — Documented
- **CLAUDE.md or AGENTS.md exists**: Agent-specific instructions, conventions, gotchas
- **Architecture overview**: High-level description of system structure (even a paragraph)
- **API documented**: For libraries/services, public API is documented

### L3 — Standardized
- **Contributing guide**: `CONTRIBUTING.md` with PR process, style expectations
- **Environment variables documented**: All required env vars listed with descriptions
- **Commands cheat sheet**: Common dev commands in README or CLAUDE.md

### L4 — Optimized
- **Architecture Decision Records**: `docs/adr/` or equivalent with rationale for key decisions
- **Runbook for common tasks**: Deployment, debugging, data migration procedures
- **Docs freshness**: Documentation updated within the last 90 days

### L5 — Autonomous
- **Living specification**: Tests or types serve as executable documentation
- **Auto-generated API docs**: OpenAPI, TypeDoc, or equivalent generated from code

---

## 5. Dev Environment

### L1 — Foundational
- **Package manager configured**: `package.json`, `Cargo.toml`, `pyproject.toml`, `go.mod`
- **Gitignore exists**: `.gitignore` with appropriate entries
- **Works on clone**: `git clone && install && run` works without manual steps

### L2 — Documented
- **Env template**: `.env.example` or `.env.template` with all required variables
- **Node/Python/Go version specified**: `.node-version`, `.python-version`, `.tool-versions`, `engines` field
- **Setup script**: `scripts/setup.sh` or `make setup` for one-command environment setup

### L3 — Standardized
- **Devcontainer**: `.devcontainer/` config for reproducible environments
- **Docker compose**: `docker-compose.yml` for local services (DB, cache, etc.)
- **Seed data**: Database seeds or fixtures for local development

### L4 — Optimized
- **Isolated workspaces**: Support for git worktrees or parallel checkouts
- **Hot reload**: Dev server reloads on file changes
- **Local-first services**: All dependencies runnable locally (no mandatory cloud services)

### L5 — Autonomous
- **Ephemeral environments**: On-demand preview environments per PR
- **Self-provisioning**: Agent can set up the full environment from scratch

---

## 6. Code Quality & Architecture

### L1 — Foundational
- **No files >500 lines**: Or at most <5% of files exceed this
- **Entry points discoverable**: Main entry points are obvious (`src/index.*`, `main.*`, `app.*`)
- **No circular dependencies**: Or circular deps detected and documented

### L2 — Documented
- **Modular structure**: Code organized by feature/domain, not by type (no `controllers/`, `models/`, `services/` only)
- **Explicit exports**: Public API of modules is clear (index files, `__init__.py`, `mod.rs`)
- **Consistent naming**: Files and functions follow a consistent naming convention

### L3 — Standardized
- **Dependency direction enforced**: No upward imports (UI → domain ok, domain → UI not ok)
- **Max complexity threshold**: Cyclomatic complexity limits enforced via linter
- **Dead code detection**: Tools or CI checks for unused exports/code

### L4 — Optimized
- **Deep modules, simple interfaces**: Modules hide complexity behind small public APIs
- **<10 files changed per typical PR**: Architecture supports small, focused changes
- **Explicit error types**: Errors are typed/structured, not string messages

### L5 — Autonomous
- **Architectural fitness functions**: Automated tests that verify architectural constraints
- **Blast radius analysis**: Tooling that shows impact of changes before they're made

---

## 7. Observability

### L1 — Foundational
- **Error handling exists**: try/catch, error boundaries, or equivalent patterns
- **Errors propagate**: No silent swallowing of exceptions (no empty catch blocks)
- **Console/stdout logging**: Some logging exists for debugging

### L2 — Documented
- **Structured logging**: JSON logs or structured log library (winston, pino, structlog)
- **Error tracking**: Canary, Sentry, Bugsnag, or equivalent configured
- **Log levels used**: DEBUG/INFO/WARN/ERROR used appropriately

### L3 — Standardized
- **Health checks**: `/health` endpoint or equivalent for services
- **Request logging**: HTTP requests logged with method, path, status, duration
- **Error context**: Errors include enough context to diagnose (user, request, state)

### L4 — Optimized
- **Distributed tracing**: OpenTelemetry, Datadog, or equivalent for cross-service tracing
- **Metrics instrumented**: Key business and technical metrics tracked
- **Alerting configured**: Alerts fire on error spikes, latency, or health check failures

### L5 — Autonomous
- **Agent-accessible telemetry**: Agent can query logs/metrics to diagnose issues
- **Anomaly detection**: Automated detection of unusual patterns

---

## 8. Security & Governance

### L1 — Foundational
- **No secrets in code**: No hardcoded API keys, passwords, or tokens in source
- **Gitignore covers secrets**: `.env`, `*.pem`, `*.key` in `.gitignore`
- **Dependencies not wildly outdated**: No dependencies >2 major versions behind

### L2 — Documented
- **Branch protection**: Main branch requires PR (check via `gh api repos/{owner}/{repo}` or branch protection rules)
- **CODEOWNERS exists**: `CODEOWNERS` or equivalent for review routing
- **Security policy**: `SECURITY.md` with vulnerability reporting instructions

### L3 — Standardized
- **Secret scanning**: GitHub secret scanning, gitleaks, or equivalent enabled
- **Dependency auditing in CI**: `npm audit`, `pip-audit`, or Dependabot configured
- **Input validation**: User-facing inputs validated at system boundaries

### L4 — Optimized
- **SAST/DAST**: Static or dynamic security analysis in CI
- **Signed commits**: Commit signing encouraged or enforced
- **Least privilege**: Service accounts and API keys use minimal permissions

### L5 — Autonomous
- **Auto-patching**: Dependabot or Renovate auto-merges passing security patches
- **Runtime protection**: WAF, rate limiting, or equivalent for production services

---

## 9. Agent-Legible Source Of Truth

State surfaces are product, content, deployment, security, workflow, or
operational facts that determine what the repo does but may live outside the
repository. A repo is agent-ready only when agents can inspect and verify those
surfaces through code, local files, MCP, CLI, API, skill, or an explicit
waiver with an expiry.

When remediation crosses an external system boundary, route through
`meta/INTEGRATION_GUIDE.md` before choosing MCP, CLI, API, or skill.

### L1 — Foundational
- **State surfaces inventoried**: `.harness-kit/agent-readiness.yaml`
  contains `state_surfaces[]`, even if empty
- **System of record named**: Each state surface names its `system_of_record`
- **Agent access classified**: Each state surface labels `agent_access` as
  code, local-file, mcp, cli, api, skill, admin-ui-only, cms-only, or unknown

### L2 — Documented
- **Inspectable path provided**: Non-hidden state surfaces include
  `source_path`
- **Verification command provided**: Non-hidden state surfaces include a local
  command, MCP resource, API check, or skill path that proves current state
- **Hidden state labeled as debt**: `admin-ui-only`, `cms-only`, and `unknown`
  access are readiness debt unless they carry a non-expired waiver

### L3 — Standardized
- **Waivers expire**: Hidden-state waivers include `waiver` and
  `waiver_expires`
- **Integration remediation routed**: Hidden external systems point remediation
  to `meta/INTEGRATION_GUIDE.md`, not ad hoc UI instructions
- **Profile validation enforced**: `profile-crud.py validate` rejects stale or
  unwaived hidden state

### L4 — Optimized
- **Automated source exports**: External systems have repeatable export or
  query commands agents can run locally
- **Drift detection**: CI or scheduled checks detect when external truth
  diverges from repo-visible state

### L5 — Autonomous
- **Agent-accessible remediation**: Agents can propose or execute bounded
  fixes through the approved MCP/CLI/API/skill boundary
- **Fleet visibility**: Source-legibility debt is aggregated across repos
