---
name: refactor
description: |
  Branch-aware simplification and refactoring workflow. On feature branches,
  compare against base and simplify the diff before merge. On primary branch,
  scan the full codebase, research prior art, and identify the highest-impact
  simplification opportunity.
  Use when: "refactor this", "simplify this diff", "clean this up",
  "reduce complexity", "pay down tech debt", "make this easier to maintain",
  "make this more elegant", "reduce the number of states", "clarify naming".
  Trigger: /refactor.
argument-hint: "[--base <branch>] [--scope <path>] [--report-only] [--apply]"
---

# /refactor

Reduce complexity without reducing correctness. Favor fewer states, clearer
names, stronger invariants, better tests, and current docs. Deletion first,
then consolidation, then abstraction, then mechanical cleanup.

## Delegation Floor

When a provider roster is available (repo `.spellbook/agents.yaml` or system `~/.spellbook/agents.yaml`), `/refactor` starts by probing the
roster and dispatching two or more available roster members before choosing
or applying a simplification. Use one lane to map complexity and another to
challenge behavior-preservation risk; add a builder lane for the selected
change. The lead agent owns the final cut, proof, and receipts. Direct
lead-only refactoring is limited to mechanical command execution, emergency
unblocks, explicit user-forbidden delegation, or fewer than two available
roster members.

## Deslop Pass

When invoked after implementation or review, run a behavior-preserving deslop
pass before broader refactoring. Remove:

- unnecessary AI-style comments and restatements of obvious code
- defensive checks or try/catch blocks that are abnormal for trusted internal
  paths
- casts, optionality, and fallback branches that paper over unclear invariants
- deeply nested code that can become early returns or a smaller model
- bespoke helpers or wrappers that duplicate canonical local utilities

Keep behavior unchanged unless fixing a clear bug. If cleanup would alter
behavior, stop and route the finding back through `/shape` or `/implement`.

## Branch-Aware Routing

Detect the current branch and primary branch first:
1. Current: `git rev-parse --abbrev-ref HEAD`
2. Primary: `git symbolic-ref --short refs/remotes/origin/HEAD | sed 's#^origin/##'`
   (fallback `main`, then `master`)

If current branch != primary branch: run **Feature Branch Mode**.
If current branch == primary branch: run **Primary Branch Mode**.

If current branch resolves to `HEAD`, the primary branch cannot be discovered,
or the detected base is ambiguous, stop and require `--base <branch>`. Fail
closed rather than computing the wrong diff.

`--base <branch>` overrides detected base branch for feature-branch comparisons.
`--scope <path>` limits analysis and edits to one subtree.
`--report-only` disables file edits.
`--apply` allows edits in primary-branch mode (otherwise report + backlog shaping only).

Detailed simplification methodology lives in `references/simplify.md`.

## Feature Branch Mode (default on PR branches)

Goal: simplify what changed between `base...HEAD` before merge.

### 1. Map the delta

- Compute full diff stats and touched files: `git diff --stat <base>...HEAD`
- Identify high-leverage simplification targets in the diff:
  - pass-through layers
  - duplicate helpers
  - unclear naming
  - unnecessary state branches
  - over-modeled state or mode flags
  - tests that assert implementation instead of behavior
  - stale docs/comments in changed areas

### 2. Parallel exploration bench

Launch at least three subagents in parallel:
- **Diff Cartographer (Explore):** map responsibilities and complexity smells in changed modules
- **Simplification Planner (Plan):** propose deletion/consolidation-first refactor options
- **Quality Auditor (Explore):** spot test/documentation gaps introduced by the diff

Each subagent returns: top findings, one recommended change, confidence, risk.

### 3. Synthesize and choose

Rank opportunities by:
`(complexity removed * confidence) / implementation risk`

Prefer:
1. deletion
2. consolidation
3. state-space reduction and invariant tightening
4. naming clarification
5. abstraction
6. mechanical refactor

### 4. Execute (unless `--report-only`)

Dispatch a builder subagent for exactly one bounded refactor.
Each refactor must include:
- behavior-preserving tests (new or updated)
- obvious naming improvements where needed
- doc updates for changed contracts
- state reduction when the existing design encodes more modes than the behavior needs

### 5. Verify

Minimum:
- relevant tests pass
- no new lint/type failures
- complexity is reduced, not moved

If available, run `assess-simplify` and require `complexity_moved_not_removed=false`.

## Primary Branch Mode (default on `main` / `master`)

Goal: find the single highest-impact simplification for the codebase itself.
This mode is designed to be safe for scheduled runs.

### 1. Build a hotspot map

Use evidence from:
- churn (`git log --name-only`)
- module size and fan-out
- flaky or slow tests
- recurring failure domains from backlog/issues

### 2. Parallel strategy bench

Launch at least three subagents in parallel:
- **Topology Mapper (Explore):** locate architectural complexity hotspots
- **Deletion Hunter (Explore):** identify dead code and compatibility shims with no active contract
- **Rebuild Strategist (Plan):** propose the cleanest from-scratch shape for one hotspot

### 3. External calibration

Invoke `/research` for the target domain before final recommendation. Do not
assert architecture choices from memory.

### 4. Produce outcome

Default (safe): no code edits. Instead:
- choose one winning candidate
- optionally list up to two runners-up as appendix only
- shape the winning opportunity into a concrete backlog item (`backlog.d/`) with oracle

If `--apply` is explicitly set:
- implement exactly one low-risk, bounded simplification for the winning candidate
- verify tests/lint
- record residual risk and follow-up items

## Required Output

```markdown
## Refactor Report
Mode: feature-branch | primary-branch
Target: <branch or scope>

### Candidate Opportunities
1. [winning candidate] — complexity removed, risk, confidence

### Optional Runners-Up
1. [runner-up]
2. [runner-up]

### Selected Action
[what was applied, or backlog item created]

### Verification
[tests/lint/assessment results]

### Completion Gate
- Exact developer/operator behavior changed:
- Live evidence that proves it:
- Exact command/path exercised:
- Repo-fit check:
- Residual unverified paths:

### Residual Risks
[what remains and why]
```

## Gotchas

- **Complexity moved, not removed:** splitting one complex module into two equally complex modules is not simplification.
- **"Refactor everything":** broad edits destroy reviewability. Keep each pass bounded.
- **Skipping branch mode detection:** primary and feature branches have different risk envelopes.
- **Applying risky changes on primary by default:** primary mode defaults to report + shaping.
- **No oracle for a proposed refactor:** if you cannot state how success is measured, the proposal is not ready.
- **Chasing aesthetic churn:** clearer names and fewer states matter; style-only motion does not.
- **Parallelizing dependent edits:** only parallelize disjoint slices.
