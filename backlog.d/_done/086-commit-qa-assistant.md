# Commit-triggered QA assistant evidence lane

Priority: P1
Status: ready
Estimate: M

Inspired by Peter Steinberger's Codex QA-assistant workflow: every commit
generates a user-test scenario, drives the app through webVNC / computer /
browser-use tooling like a QA person, and opens follow-up PRs with fixes.

The screenshot referenced in the shaping request was not available at
`/Users/phaedrus/Desktop/HJrhp9AXoAQLA50.jpg`; the quoted post is the source
evidence for this ticket.

## Goal

Make Harness Kit able to attach user-like QA evidence to a commit by generating
and running a repo-fit scenario against the changed surface, then reporting
findings in the existing evidence and delivery flow.

## Non-Goals

- Do not build hosted VNC, browser-farm, or always-on SaaS infrastructure.
- Do not let an agent push, merge, or auto-open fix PRs in the first slice.
- Do not replace `/qa`, `/deliver`, `/demo`, `/code-review`, or the Dagger gate.
- Do not treat screenshots alone as proof that behavior worked.
- Do not make this browser-only; browser automation is one route after Step 0
  resolves the repo surface to a browser app.
- Do not store raw screen recordings, credentials, or private browser state
  outside the repo's evidence contract.

## Constraints / Invariants

- Cross-harness first: the contract must be filesystem + skill prose; Codex,
  Claude, Pi, and other provider CLIs are execution lanes, not the source of
  truth.
- Scenario generation must be grounded in live repo evidence: changed files,
  repo-local QA skill or `/qa` Step 0, existing routes/commands, and acceptance
  sources.
- Canonical artifacts live under `.evidence/<branch>/<date>/`; PR comments or
  draft releases are mirrors only.
- Findings must classify pass, fail, or inconclusive. A blank page, failed
  launch, missing route, or missing expected element is inconclusive/fail, never
  pass.
- Fix generation is a later slice and must route through `/deliver
  --polish-only <branch|PR>` plus `/code-review` and `dagger call check
  --source=.`.
- The lane must preserve clean-tree closeout and never discard user work.

## Authority Order

tests > type system > code > evidence artifacts > docs > lore

## Repo Anchors

- `skills/qa/SKILL.md` - Step 0 app-shape routing, running-surface QA, and
  completion evidence fields.
- `skills/qa/references/browser-tools.md` - browser/computer automation choices
  when the changed surface is browser-shaped.
- `skills/qa/references/evidence-capture.md` - screenshot, GIF, trace, and
  transcript capture conventions.
- `skills/deliver/SKILL.md` - clean-loop phase routing and boundary: delivery
  composes phases, never creates a side-channel merge path.
- `skills/deliver/references/evidence.md` - canonical `.evidence/` storage and
  per-phase emission rules.
- `skills/demo/references/pr-evidence-upload.md` - optional PR visual mirror
  pattern after canonical evidence is committed.
- `backlog.d/058-work-ledger-mission-control.md` - future mission-control
  surface for background QA attempts, findings, and follow-up state.

## Prior Art

- Steinberger / OpenClaw workflow - commit-triggered, user-like QA using
  webVNC and computer/browser-use tools, with background fix PRs.
- `/qa` - already defines "tests pass is not QA" and routes verification by
  app shape rather than by one favored tool.
- `/demo` PR evidence mirror - demonstrates how visual evidence can be
  packaged for reviewers without making GitHub the canonical store.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Literal hosted VNC copy | Every commit runs in hosted VNC and a background agent opens fix PRs | Closest to the post; strong user-like coverage for GUI apps | Cost, flake, credentials, and write-token risk; browser-only bias | Reject for first slice |
| Always-on PR watcher | Agent watches branches/PRs, runs QA, and proposes fixes | Convenient reviewer experience | Permission creep and noisy PR churn; duplicates `/deliver` | Reject |
| Local-first per-commit evidence lane | Developer or outer loop runs a generated scenario locally and stores structured evidence | Fits Harness Kit's filesystem-first model; low infra burden | May be skipped unless wired into delivery/ledger later | Choose |
| Project-local QA skill scaffold only | Teach `/create-repo-skill qa` to add scenario templates per repo | Durable repo fit; no new runtime | Does not answer commit-triggered workflow by itself | Defer as companion |
| Deterministic Playwright regression tests | Generate or update Playwright tests from scenario plans | Re-runnable CI oracle when browser app has stable routes | Brittle selectors; misses non-browser repos | Defer after evidence lane |
| Visual regression snapshots | Compare screenshots before/after each commit | Cheap signal for visual breakage | Functional failures can pass if pixels look plausible | Reject as primary |
| Manual QA checklist in PR template | Agent writes a human checklist for each commit/PR | Boring, low risk, works everywhere | Pushes execution back to humans; no artifact proof | Reject as fallback only |
| Post-merge monitor | Run user-like checks after merge/deploy and open follow-up work | Catches production-only failures | Too late for regressions; different risk class | Defer to `/monitor` |

## Tradeoff Matrix

| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| Literal hosted VNC copy | 2 | 1 | 1 | 2 | 2 | 3 | 1 |
| Always-on PR watcher | 3 | 2 | 2 | 2 | 2 | 3 | 2 |
| Local-first per-commit evidence lane | 5 | 4 | 4 | 5 | 5 | 4 | 4 |
| Project-local QA skill scaffold only | 4 | 4 | 5 | 4 | 5 | 4 | 4 |
| Deterministic Playwright regression tests | 3 | 3 | 4 | 4 | 4 | 5 | 3 |
| Visual regression snapshots | 2 | 3 | 4 | 4 | 4 | 3 | 3 |
| Manual QA checklist | 2 | 5 | 5 | 2 | 5 | 1 | 3 |
| Post-merge monitor | 3 | 3 | 3 | 4 | 4 | 3 | 3 |

The chosen first slice scores best because it adds the missing commit-scoped
QA contract without adding hosted infrastructure, write-token automation, or a
second workflow engine. Playwright tests and auto-fix PRs become more useful
after the evidence shape proves which scenarios are stable and valuable.

## Delegation Evidence

- Roster providers used:
  - `claude` as repo investigator, receipt
    `2ed3f132-bdb9-48b8-b341-7f3636162f2c`, transcript
    `.harness-kit/traces/provider-lanes/20260603T151930.563642Z-claude-993f2d86.txt`.
  - `pi` as product/premise critic, receipt
    `68474d16-6fcd-4253-a413-80f3d8910d1b`, transcript
    `.harness-kit/traces/provider-lanes/20260603T151932.248673Z-pi-4e74cf83.txt`.
  - `codex` as fresh oracle critic, receipt
    `ef2e38dd-feae-4756-80a3-1573745401eb`, transcript
    `.harness-kit/traces/provider-lanes/20260603T152313.303081Z-codex-4b3090fa.txt`.
- Accepted evidence: keep the first slice local-first and evidence-only; plug
  into `/qa`, `.evidence/`, `/deliver`, and backlog 058 instead of creating a
  hosted VNC or autonomous PR-writing system. Tighten the oracle so result
  records include the scenario/route-selection transcript and a committed text
  summary, not only binary artifacts.
- Rejected evidence: the literal "background opens PRs with fixes" part is
  deferred because Harness Kit already reserves push/merge/fix-loop authority
  for existing delivery and review flows. The `agy` draft-critic lane is
  rejected because its transcript timed out at OAuth despite the wrapper
  recording an exited provider attempt.
- Waivers: no external web lookup was used; the user-provided quote is the
  source, the screenshot path was unavailable, and the native Beck critic role
  was unavailable because its pinned model is not supported on this account.

## Oracle (Definition of Done)

- [ ] A reference such as `skills/qa/references/per-commit-lane.md` defines the
      commit QA lane: inputs, app-shape routing, scenario plan schema, evidence
      schema, status values, and safety boundaries.
- [ ] `/qa` links the reference and states when to use it: commit-triggered,
      PR-triggered, or outer-loop-triggered user-like scenario evidence.
- [ ] The scenario plan schema requires: commit SHA, changed files summary,
      app shape, launch command or target URL/command, persona/user goal,
      steps, expected observable outcomes, and expected evidence artifacts.
- [ ] The result schema requires: `pass` / `fail` / `inconclusive`, severity,
      exact command/path/route/tool call exercised, evidence refs, expected
      element or output assertions, scenario/route-selection transcript ref,
      committed `qa-report.md` or equivalent text summary, and residual risk.
- [ ] A synthetic fixture under `skills/qa/evals/cases/` proves that a blank
      page or missing expected element cannot be reported as `pass`.
- [ ] A second fixture under `skills/qa/evals/cases/` proves the lane can choose
      a non-browser QA path for a CLI/library repo instead of forcing browser
      tooling.
- [ ] A formatter/schema test feeds a browser-shaped changed-files summary plus
      a missing expected selector into the lane and asserts the emitted result is
      `fail` or `inconclusive`, includes the exact route/tool/evidence ref, and
      cannot serialize as `pass`.
- [ ] Any mention of fix PRs points to a later slice that routes through
      `/deliver --polish-only` and never bypasses `/code-review`, `/ci`, or
      clean-tree closeout.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Implementation Sequence

1. Add `skills/qa/references/per-commit-lane.md` with the commit QA lane
   contract, schema examples, and safety boundaries.
2. Update `skills/qa/SKILL.md` to route commit/PR-triggered scenario QA to that
   reference after Step 0 resolves the app shape.
3. Add focused eval fixtures for the two core failure modes: blank-page false
   pass and browser-only overreach.
4. Update any evidence or demo references only if the new schema needs a
   pointer to existing `.evidence/` or PR mirror conventions.
5. Run `python3 scripts/check-agent-roster.py`, then `dagger call check
   --source=.`.

## Risk + Rollout

- False confidence from shallow visual checks: require expected interactive
  element/output assertions and classify missing proof as inconclusive.
- Flaky local environments: record launch command, target, tool, timeout, and
  failure mode so infra failures are visible rather than converted to product
  failures.
- Privacy leakage from browser state or recordings: keep canonical evidence
  repo-local, redact sensitive outputs, and make PR mirrors opt-in.
- Scope creep into autonomous PR factories: keep fix generation out of this
  ticket and route later work through `/deliver --polish-only`.
- Rollback: remove the reference, `/qa` link, and eval fixtures; no persistent
  runtime service or schema migration should be required.
