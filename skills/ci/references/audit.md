# CI Audit Rubric

What makes CI strong vs weak. Use this to score a repo's pipeline before
trusting a green run. Every gap is an audit finding; severity drives
whether to block on remediation or file as follow-up.

## The Five Concerns

A strong pipeline covers five concerns. Each missing concern is a gap.

| Concern       | Strong                                 | Weak                                | Gap (fail)               |
|---------------|----------------------------------------|-------------------------------------|--------------------------|
| Lint + format | Configured, strict, run in CI, no opt-outs | Configured locally only, or rules silently disabled | Not configured           |
| Type check    | Strict mode, covers all prod source    | Non-strict, or partial coverage     | Not configured           |
| Tests         | Hermetic, deterministic, behavior-focused | Network/DB-dependent, flaky, implementation-coupled | No test runner in CI     |
| Coverage      | Floor gated (≥70% typical)             | Reported but not gated              | Not measured             |
| Secrets scan  | gitleaks / trufflehog gate             | Pre-commit only                     | Not scanned              |

Secondary concerns: dependency audit, SBOM, licence check, container
scan. Flag if absent but don't block on them for typical repos.

## Concern-by-Concern Checks

### Lint + format

- Config file present (`.eslintrc`, `ruff.toml`, `.golangci.yml`, etc.).
- Rules not silently disabled (scan for `# noqa` / `eslint-disable`
  density; spot-check suspicious suppressions).
- Gate runs in Dagger pipeline, not just pre-commit.
- Auto-format pass available (for self-heal).

**Strengthening:** add a `lint` Dagger function, wire into `check`.

### Type check

- Strict mode on (mypy `strict=true`, tsc `strict: true`, pyright strict).
- Covers all prod source directories, not just a subset.
- `# type: ignore` / `any` density isn't pathological.

**Strengthening:** add a `typecheck` Dagger function. If the repo has
no types today, this is a bigger project — file a separate backlog item,
don't try to fix it inline.

### Tests

- Runner configured (pytest, go test, vitest, etc.).
- Hermetic: no live network, no external DB, no shared state between
  runs. Use fixtures, testcontainers, or in-memory stubs.
- Deterministic: seeded randomness, fake clocks.
- Behavior-focused: test assertions describe behavior, not internal
  structure. (This is a `/code-review` concern — flag but don't block.)

**Strengthening:** if tests aren't hermetic, that's a re-architecture,
not a quick fix. File backlog and defer.

### Coverage

- Measured (coverage.py, c8, go cover).
- Gated at a floor. Typical: 70% for mature repos, 50% for young ones.
- Floor is explicit in pipeline config, not implicit.

**Strengthening:** raise the floor gradually. Never lower to pass.

### Secrets

- Gate runs gitleaks or trufflehog.
- Scans full history on first run, diff-only afterwards.
- Pre-commit catches most, but CI catches what pre-commit missed.

**Strengthening:** add a `secrets` Dagger function.

## Structural Checks

### Dagger-first

- `dagger.json` present and `dagger functions` enumerates a `check`
  entrypoint that composes all gates.
- Pipeline code is tested (yes, the CI itself has tests). See the
  Harness Kit repo's `ci/tests/` for the pattern.
- Each gate is a named function so it can be invoked in isolation
  for debugging: `dagger call lint-python`, `dagger call test-bun`, etc.

### Pre-push hook

- `.githooks/pre-push` (or equivalent) invokes `dagger call check`.
- Hook skips gracefully when Dagger/Docker absent (don't block commits
  in environments that can't run containers).

### Speed

- Full pipeline under ~10 minutes on a cold run, under ~3 on warm.
- If slower: parallelize independent gates, cache layers aggressively,
  split slow-integration from fast-unit.
- Not a blocker for correctness, but a blocker for ergonomics — flag
  as med-severity when over 10 minutes.

## Severity Matrix

| Severity | Meaning                                          | Action              |
|----------|--------------------------------------------------|---------------------|
| high     | Missing concern (types, tests, secrets)          | Offer to fix inline |
| med      | Weak concern (coverage not gated, slow pipeline) | Offer or defer      |
| low      | Polish (better names, faster cache)              | Defer to backlog    |

User approves per-finding. High-severity gaps that the user defers
should be recorded as a backlog item, not silently dropped.

## Anti-Findings

Things that sound like gaps but aren't:

- **"No integration tests"** — if the repo's architecture is such that
  unit tests cover behavior and the integration surface is thin, this
  is a judgment call, not a universal gap.
- **"Coverage below 100%"** — 70–85% is healthy; 100% usually means
  testing implementation.
- **"Pipeline doesn't run on Windows"** — only a gap if the project
  targets Windows.

Judgment call: is this a load-bearing concern for *this* repo, or a
checklist item from someone else's context?
