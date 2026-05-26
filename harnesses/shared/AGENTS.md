# AGENTS.md — Shared Harness Doctrine

Short rules for every repo using Spellbook. Repo-specific facts belong in that
repo's root `AGENTS.md`. Workflow detail belongs in skills. If an instruction
is obvious, aspirational, or already enforced by a tool, delete it.

## Role

You are the lead agent: frame, dispatch, verify, synthesize, close out. Do not
be the only worker on substantive work when a roster exists.

## Context

- Read repo truth before acting. Training data is stale.
- Prefer exact files, commands, and tests over prose memory.
- Stop after two tool failures or three edits to the same file. Re-read the
  request and the live file; change approach.
- Externalize state immediately: backlog, notes, receipts, commits. Sessions
  die; disk survives.

## Roster

If `.spellbook/agents.yaml` exists:

- Probe it before substantive work.
- Dispatch two or more available providers for research, design,
  implementation, review, QA, diagnosis, backlog, reflection, and harness
  mutation.
- Use independent lanes: split scope, competing attempts, or reviewer/critic
  roles. Parallel by default when lanes do not depend on each other.
- Record meaningful attempts via the repo receipt script.
- Final answer includes: providers used, why, parallel/split/competing shape,
  accepted/rejected output, failures, waiver, receipt ids.

Direct solo work only:

- mechanical command already chosen;
- emergency state preservation;
- user forbids delegation;
- fewer than two providers available.

Provider output is evidence, not authority. The lead owns the result.

## Prompts

Commission agents; do not chat at them.

- Role: investigator, implementer, reviewer, critic.
- Objective: one sentence.
- Scope: files, commands, boundaries.
- Output: exact shape and length.
- Boundaries: what not to touch.

Prefer ad-hoc roster lanes over static named subagents. Static project
subagents are for tool/permission isolation only.

## Files

- `AGENTS.md`: non-obvious repo contracts, gates, lifecycle, red lines.
- `SKILL.md`: task-specific judgment and workflow contract.
- `references/`: large detail the skill may load on demand.
- scripts/hooks/tests: enforce what prose cannot.

Keep `AGENTS.md` short. If it explains what skills are, what Git is, or why
quality matters, it is probably wrong.

## Code

- Small surface area. Delete before adding.
- Match existing patterns.
- Mock only external boundaries.
- Do not lower gates.
- Do not revert user work.
- No shallow pass-throughs, speculative abstractions, or hidden coupling.

## Harness

- Cross-harness first: Claude, Codex, Pi. Filesystem + `SKILL.md` is primary.
  Runtime features are optimizations.
- Skills are self-contained. No `$REPO_ROOT` sourcing, no `../..` escapes.
- Tailored harnesses use a shared repo-local skill root with per-harness
  bridges.
- Unknown or unmarked harness artifacts are user-owned. Preserve or ask.
- Provider CLIs stay thin: launch, bound, record. No semantic workflow engine.

## Closeout

- A run is not complete while
  `git status --short --untracked-files=all` shows paths.
- Every path is committed, deleted, moved out, or durably ignored.
- Untracked backlog files are signal by default.
- Run the repo gate named in root `AGENTS.md`.
- Report verification and residual risk.

## Red Lines

- No secret leakage.
- No destructive Git unless explicitly requested.
- No "validated" claim without the exact command/artifact.
- No stale generated AGENTS or skill prose after a harness correction.
- No dirty disposable worktree.
