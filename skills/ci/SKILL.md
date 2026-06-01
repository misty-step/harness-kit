---
name: ci
description: |
  Audit CI gates, strengthen weak coverage, then drive green. Dagger owns
  the canonical pipeline; missing Dagger is auto-scaffolded. Acts directly on
  mechanical fixes and never returns red without structured diagnosis.
  Use when: "run ci", "check ci", "fix ci", "audit ci", "is ci passing",
  "run the gates", "dagger check", "why is ci failing", "strengthen ci",
  "tighten ci", "ci is red", "gates failing".
  Trigger: /ci, /gates.
argument-hint: "[--audit-only|--run-only]"
---

# /ci

Confidence in correctness. CI is load-bearing: a green CI is a claim about
code correctness. If CI is weak (no type check, shallow tests, no coverage
floor), green means nothing. So this skill **audits first, then runs**.

Stops at green CI. Does not review code semantics (→ `/code-review`), does
not address review comments (→ `/deliver --polish-only`), does not ship.

## Modes

- Default: audit → run. Full pass.
- `--audit-only`: produce audit report and gap proposals; do not run gates.
- `--run-only`: skip audit, just drive gates green.

## Stance

1. **Audit before run.** A weak pipeline passing is worse than a strong
   one failing. Inventory coverage before trusting green.
   For CI-audit, red-gate investigation, or gate-policy changes, use the
   Delegation Floor before changing policy or claiming green.
2. **Dagger-mandatory, auto-scaffolded.** Missing `dagger.json` is a gap
   the skill closes itself, not a blocker that halts work. Scaffold a
   TypeScript Dagger module, fold every existing gate into `check()`,
   thin the CI provider (GHA, CircleCI, etc.) to a single `dagger call check`
   step, and update pre-push hooks to match. Raw `npm run lint` / `pytest`
   / `go test` etc. are what GHA replaced, not what replaces GHA — they
   bypass the hermetic-container contract. CI providers must be thin
   wrappers that shell out to `dagger call check`, never the authoritative
   gate owner. Agent-first, local-first, provider-independent.
3. **Act, do not propose.** The skill has executive authority within its
   domain: mechanical strengthenings (adding missing gates, wiring
   uncovered scripts, consolidating duplicate workflows, hardening
   thresholds upward, scaffolding Dagger) are applied directly. Do NOT
   emit "proposals" or "diff previews" awaiting approval. Only escalate
   when the strengthening is a genuine product decision (e.g. disabling
   a currently-green test, materially changing scope, or a tradeoff
   the code alone cannot resolve).
4. **Fix-until-green on self-healable failures.** Don't report red and
   exit. Either fix or produce a precise diagnosis the human can act on.
5. **No quality lowering, ever.** Thresholds, lint rules, type strictness,
   coverage floors are load-bearing walls. Raising the floor is fine;
   lowering it to make CI pass is forbidden.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use lanes for audit, failure diagnosis, and gate-strengthening judgment; the lead may run exact validation commands and mechanical repairs after deciding from the evidence.

## Bounded Self-Heal

See `references/self-heal.md` for the fix-vs-escalate decision. Auto-fix
format/lint drift, regenerate lockfiles when deterministic, and retry flakes
within the bounded policy. Algorithm and logic failures escalate.

## Process

### Phase 1 — Audit (skip if `--run-only`)

Read `references/audit.md` for the full audit rubric. Inventory in parallel.
**Pipeline presence is the first gate.** If Dagger is absent, scaffold it
inline — do not stop, do not wait for approval. Scaffolding Dagger is
mechanical (`dagger init --sdk=typescript --source=.dagger`, fold existing
gates into `check()`, thin the GHA/CircleCI provider to a single
`dagger call check` step). The skill owns this.

- **Pipeline presence (blocking):** `dagger.json` / `.dagger/` exists?
  Entrypoint reachable? (`dagger functions` lists `check`?) Missing = HIGH,
  blocks green until scaffolded.
- **CI provider thinness:** GHA / CircleCI / etc. steps invoke
  `dagger call <func>` only. Inline `npm run X` / `pytest` / raw bash in
  workflow YAML = finding (pipeline lives in two places, drifts).
- **Gate coverage:** lint, format, type-check, tests, coverage floor,
  secrets scan — each one covered by a Dagger function, or a gap?
- **Thresholds:** coverage floor ≥70% (calibrate to repo maturity),
  type-check in strict mode, lint rules not silently disabled.
- **Hardening visibility:** property testing, mutation testing, CRAP/SCRAP,
  structural DRY, or acceptance-mutation commands are documented for repos
  whose changed surfaces need them. Missing tools are audit findings, not a
  reason to add slow hardening work to the default fast gate.
- **Hermeticity:** tests don't hit network/external DBs? Fixtures pinned?
- **Speed:** full pipeline under ~10 minutes? If not, flag (not blocking).
- **Pre-push hook:** invokes the Dagger pipeline? (see
  `.githooks/pre-push` pattern).

Emit a structured audit report:

```markdown
## CI Audit
| Concern        | Status | Severity | Proposed fix                    |
|----------------|--------|----------|---------------------------------|
| lint           | ok     | -        | -                               |
| type-check     | gap    | high     | Add mypy strict to dagger check |
| coverage floor | weak   | med      | Raise floor from 40% → 70%      |
| secrets scan   | gap    | high     | Add gitleaks gate               |
```

For each gap, apply the mechanical remediation directly. Do NOT emit
"proposals" awaiting approval. Mechanical strengthenings include:
adding a missing lint/typecheck/coverage/secrets gate, wiring new
scripts into the bash-syntax step, consolidating duplicate workflows,
raising thresholds upward when the current code already passes a higher
bar, and scaffolding Dagger. Escalate only when the strengthening would
disable a currently-green test, materially change scope, or encode a
product decision the code alone cannot resolve.

If audit finds no gaps worth fixing, say so and proceed.

### Phase 2 — Run (skip if `--audit-only`)

1. Run the pipeline end-to-end: `dagger call check`. No fallback. If
   Dagger is absent, Phase 1 failed to block — abort and re-audit.
2. Capture structured per-gate output (which gate, pass/fail, excerpt).
3. If green → emit report, exit 0.
4. If red → classify each failure per `references/self-heal.md`:
   - **Self-healable** (lint/format drift, stale lockfile, trivially
     fixable import/typo, flake retry): dispatch a focused builder
     subagent to fix, commit, re-run the failing gate.
   - **Escalatable** (logic/algorithm failure, failing test that
     encodes a behavior change, type error in hand-written code, test
     that is genuinely broken): stop. Emit structured diagnosis
     (file:line, gate name, excerpt, candidate cause). Exit non-zero.
5. Bounded retries: cap self-heal attempts at **3 per gate**. Further
   failures escalate even if classified as self-healable — if auto-fix
   isn't converging, the model needs a human look.

### Phase 3 — Verify

Final pass of `dagger call check` after any fixes. Green or bust. If any
gate was strengthened in Phase 1, the full pipeline must pass under the
new thresholds before the skill returns clean.

## What /ci Does NOT Do

- Review code semantics → `/code-review`
- Shape tickets or write specs → `/shape`
- Address review comments or coordinate merges → `/deliver --polish-only`
- Deploy or release → `/deploy`
- QA against a running app → `/qa`
- Write new test frameworks — uses what the repo has
- Lower any threshold to make a gate pass

## Verification Scope

**A green gate proves only the commands that actually ran.**

When a diff adds or materially changes an executable path — script
entrypoint, CLI, `package.json` / `make` target, Dagger function,
migration, runner, responder, or job entrypoint — the `/ci` report must:

- name the exact command that exercised the path
- or say explicitly that the runtime path is unverified

Helper fixtures, unit coverage, and adjacent lanes do **not** count as
runtime verification unless they invoke that exact path.

## Gotchas

- **Reporting red and exiting** when the failure was a trivially
  auto-fixable format drift.
- **Lowering a threshold** (coverage, lint severity, type strictness)
  to make the gate pass. Any suggestion to do this is a red flag.
- **Skipping audit on a repo you haven't run against before.** If the
  pipeline is weak, green is a lie.
- **Declaring green on a repo with no Dagger pipeline.** Missing Dagger
  = weak CI; "the GHA workflow passes" is not equivalent — it runs on
  someone else's infrastructure and can't be reproduced locally.
- **Running raw local shells** (`pytest`, `eslint`, `npm run X`) even
  when "faster" or "the pipeline isn't set up yet." Gates run via
  `dagger call` exclusively; raw shell bypasses the hermetic-container
  contract that makes green meaningful.
- **Treating GHA/CircleCI YAML as the source of truth.** If the workflow
  has inline bash beyond `dagger call ...`, Dagger is being undermined.
- **Unbounded self-heal loops.** 3 retries per gate. If it's not
  converging, stop and diagnose.
- **Auto-fixing a failing test by deleting it** or `@skip`-ing it. A
  failing test is either a bug in the code or a bug in the test —
  diagnose, don't suppress.
- **Auto-fixing a type error by casting to `Any` / `any`** or adding
  `# type: ignore`. Escalate — this is a logic/contract decision.
- **Claiming a new runner, CLI, or lane was tested when only helper
  fixtures ran.** Name the exact runtime command that executed or mark
  the path unverified.
- **Declaring "green"** while gates are still running. Wait for exit.

## Output

Report:

- **Audit:** gaps found, severity, what was strengthened, what was deferred.
- **Run:** gates attempted, per-gate status, self-heals applied (count +
  summary), escalations (file:line diagnosis).
- **Final:** green / red, exit code, residual risks, backlog items filed
  for deferred strengthenings.

```markdown
## /ci Report
Audit: 2 gaps found (type-check missing, coverage floor 40%).
  → Strengthened: type-check added.
  → Deferred: coverage floor raise (filed as backlog 0XX).
Run: 6 gates, 1 self-heal (ruff auto-fix), 0 escalations.
Final: green. dagger call check passes in 4m12s.
```

On failure:

```markdown
## /ci Report — RED
Gate: test-python
Failure: tests/widget/test_reducer.py::test_merge_conflict line 42
  AssertionError: expected {'a': 1}, got {'a': 1, 'b': 2}
Classification: logic failure (behavior change in reducer)
Action: escalated — human decision needed on reducer contract.
```
