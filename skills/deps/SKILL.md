---
name: deps
description: |
  Analyze, test, and upgrade dependencies. One curated PR, not 47 version bumps.
  Reachability analysis, behavioral diffs, risk assessment. Package-manager agnostic.
  Use when: "upgrade deps", "dependency audit", "check for updates",
  "outdated packages", "security audit deps", "update dependencies",
  "vulnerable dependencies", "deps".
  Trigger: /deps.
argument-hint: "[audit|security|upgrade <pkg>|report] [--ecosystem npm|pip|cargo|go]"
---

# /deps

Analyze, test, and upgrade dependencies. One curated PR, not 47 version bumps.

**Target:** $ARGUMENTS

## Execution Stance

You are the executive orchestrator.
- Keep upgrade policy, risk acceptance, and final merge-readiness judgment on the lead model.
- Delegate package analysis and bounded upgrade work to focused subagents.
- Parallelize across disjoint packages/ecosystems where safe.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use specialized lanes for ecosystem discovery,
changelog/release-note analysis, CVE reachability, behavioral diffing,
upgrade-builder work, and regression-risk critique. Split by package or
ecosystem when independent; do not let any lane apply broad version churn
without the lead's curated upgrade policy.

## Routing

| Mode | Intent |
|------|--------|
| **audit** (default) | Full: discover outdated, analyze risk, upgrade, test, PR |
| **security** | Security-only: CVE/advisory-affected deps with reachability analysis |
| **upgrade** [pkg] | Targeted: upgrade a specific package with full analysis |
| **report** | Analysis only, no upgrades — produce the report |

If `--ecosystem` is specified, limit analysis to that ecosystem.
Otherwise, detect all ecosystems present.

### Mode → Phase Matrix

| Mode | Phase 0 | Phase 1 | Phase 2 | Phase 3 | Phase 4 | Phase 5 |
|------|---------|---------|---------|---------|---------|--------|
| audit | ✓ | ✓ | ✓ | ✓ | ✓ | PR |
| security | ✓ | ✓ (security only) | ✓ | ✓ | ✓ | PR |
| upgrade [pkg] | ✓ | skip | ✓ | ✓ | ✓ | PR |
| report | skip | ✓ | ✓ | skip | skip | Report only |

## Ecosystem Detection

Detect by lockfile/manifest presence. Multiple ecosystems in a monorepo:
analyze each independently, upgrade separately.

| Signal | Ecosystem |
|--------|-----------|
| `package-lock.json`, `yarn.lock`, `pnpm-lock.yaml`, `bun.lockb` | npm/Node |
| `requirements.txt`, `Pipfile.lock`, `poetry.lock`, `uv.lock` | Python |
| `Cargo.lock` | Rust |
| `go.sum` | Go |
| `Gemfile.lock` | Ruby |
| `composer.lock` | PHP |

No lockfile found → STOP. Tell the user to generate one first. Upgrading
without a lockfile is guessing — you can't diff what you can't pin.

## Workflow

Six phases, gated. Each phase must complete before the next begins.

### Phase 0: Baseline

Run the project's full test suite. If tests fail, **STOP** — report the
failures and exit. You cannot attribute regressions to upgrades if the
baseline is already red.

Gate: test suite passes.

### Phase 1: Discover

Run native audit/outdated commands for each detected ecosystem:

| Ecosystem | Outdated command | Audit command |
|-----------|-----------------|---------------|
| npm | `npm outdated --json` | `npm audit --json` |
| Python | `pip list --outdated --format=json` | `pip-audit --format=json` (or `safety check`) |
| Rust | `cargo outdated --root-deps-only` | `cargo audit --json` |
| Go | `go list -m -u all` | `govulncheck ./...` |

Categorize each outdated dependency:

- **Patch** (1.2.3 → 1.2.4): Safe. Apply without analysis.
- **Minor** (1.2.3 → 1.3.0): Usually safe. Quick changelog scan.
- **Major** (1.2.3 → 2.0.0): Needs full analysis. Breaking changes likely.

List all known CVEs with severity. Cross-reference with reachability
in Phase 2.

Gate: structured list of outdated deps with categorization + CVE list.

### Phase 2: Analyze

For each non-patch update AND all security-flagged dependencies, analyze
three concerns. Parallelize across packages, not within a single package.

**Changelog:** Read changelog/release notes. Summarize breaking changes,
deprecations. Verdict: `migration_required: yes | no | unknown`.

**Reachability:** Trace import chains to CVE-affected functions. See
`references/reachability-analysis.md`. Verdict: `reachable | not reachable | unknown`.

**Behavioral:** Compare API surface before/after. Check install scripts,
network calls, permission changes. See `references/behavioral-diff.md`.
Verdict: `risk: critical | high | medium | low`.

Gate: all packages have verdicts for all three concerns. Any `unknown`
reachability on critical/high CVEs → investigate deeper or escalate.

### Phase 3: Upgrade

Create branch `deps/upgrade-YYYY-MM-DD`. Apply upgrades in risk order:

1. **Patches** — all at once, single commit
2. **Security fixes** — one commit per fix (for clean revert if needed)
3. **Minors** — grouped by ecosystem, one commit per group
4. **Majors** — one commit per package (isolation for bisect)

Each commit message references the package, version range, and risk level.
If a major bump has no migration guide and significant API changes,
**escalate to human** — don't guess at migration.

Gate: all upgrades committed atomically per group.

### Phase 4: Test

After each upgrade group:

1. Run the project's test suite
2. If `dagger.json` exists, run `dagger call check`
3. If tests fail: bisect within the group, revert the failing package,
   note it in the report as "upgrade blocked — tests fail"

Do not proceed past a failing group. Fix or revert, then continue.

Gate: all upgrade groups pass tests (or are reverted with notes).

### Phase 5: Report

Produce a single PR with structured body:

```markdown
## Dependency Upgrades

### Summary
X packages upgraded, Y security fixes, Z blocked (with reasons).

### Security
| CVE | Package | Severity | Reachable? | Action |
|-----|---------|----------|------------|--------|
| CVE-2024-XXXXX | lodash | High | Yes — used in `src/utils.ts:42` | Upgraded 4.17.20 → 4.17.21 |
| CVE-2024-YYYYY | xmldom | Medium | No — only in devDependencies | No action (noted) |

### Upgrades
| Package | From | To | Type | Risk | Changelog |
|---------|------|----|------|------|-----------|
| react | 18.2.0 | 18.3.0 | Minor | Low | Perf improvements, no breaking changes |
| webpack | 5.x | 6.0.0 | Major | High | New config format — see migration guide |

### Reachability Report
[Which CVE-affected functions are actually called in this codebase]

### Behavioral Changes
[Install scripts added/removed, new network calls, permission changes]

### Test Results
[Pass/fail per upgrade group, any reverted packages]

### Risk Assessment
[Overall risk: low/medium/high. Rationale. Residual risks.]
```

For **report** mode: produce this output without creating a branch or PR.
For **security** mode: include only the Security and Reachability sections.

## Gotchas

- **Upgrading without running tests first.** Establish a green baseline
  before touching anything. If tests already fail, fix that first.
- **Treating all CVEs equally.** A critical CVE in an unreachable function
  is lower priority than a medium CVE in your hot path. Reachability
  analysis is not optional — 92-97% of CVEs are in functions never called.
- **Batch-upgrading everything at once.** You can't bisect a 40-package
  commit. Atomic groups by risk tier. Patches together, majors alone.
- **Trusting changelogs as complete.** Changelogs omit things. For major
  bumps, read the actual diff of breaking changes or migration guide.
  The behavioral analyst catches what changelogs miss.
- **Skipping transitive dependencies.** A direct dep upgrade can pull in
  a transitive major bump. Check `npm ls`, `pip show`, `cargo tree` after
  upgrading. The lockfile diff is the truth.
- **Major bumps without migration guides.** If a major version has
  significant API changes and no clear migration path, escalate to the
  human. Don't guess at migration — a wrong guess is worse than no upgrade.
- **Running without a lockfile.** No lockfile = no reproducible state =
  no meaningful diff. Generate one before starting.
- **Monorepos with shared dependencies.** Upgrading a shared dep in one
  workspace can break another. Analyze each workspace's usage independently,
  then upgrade at the root only when all workspaces are compatible.

## Verification

Semantic waiver: dependency safety is repo- and package-manager-specific.
Each run must prove the selected upgrade group with the target repo's test
suite and `dagger call check --source=.` when `dagger.json` exists.
