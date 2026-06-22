---
name: refactor
description: |
  Architecture refactor mode: set a concrete improvement goal, refactor until
  the architecture is simpler and coherent, live-test after each significant
  step, autoreview, commit green milestones, and track progress in
  /tmp/refactor-{project}.md. Use when: "refactor this", "clean up the
  architecture", "make the design better", "refactor until you're happy",
  "pay down design debt", "simplify this subsystem". Trigger: /refactor.
argument-hint: "[scope or subsystem]"
---

# /refactor

Refactor architecture deliberately. Keep behavior stable unless the operator
explicitly asks for product change.

## Goal Articulation

Start by setting an explicit active goal when the harness supports goal
tracking. Do not write "refactor until happy." Write a goal with:

- **Outcome:** the architectural property that will be true.
- **Scope:** subsystem, files, routes, or public surface in bounds.
- **Fitness tests:** live command, route, consumer build, or browser path that
  must keep passing after each significant step.
- **Verification system:** when the fitness test is not already a credible
  proof loop, load `harnesses/shared/references/verification-system-first.md`
  and define claim, falsifier, driver, grader, evidence packet, and cadence.
- **Stop rule:** what evidence means the architecture is good enough.

Good shape:

```text
Refactor <subsystem> so <responsibility> has one owning module, public callers
use <named interface>, and <live tests/routes> still pass after each milestone.
Stop when the diff removes the duplicated seam, reviewers find no blocking
architecture concern, and no broader behavior change is needed.
```

## Progress File

Create `/tmp/refactor-{project}.md` immediately. Keep it current; it is the
handoff if the session dies.

Track:

- Goal, scope, fitness tests, and stop rule.
- Current architecture read: key modules, smells, constraints.
- Quality system: standards, proof methods, critic topology, and stop rules
  from `harnesses/shared/references/quality-system.md`.
- Milestones: planned, active, done.
- Live-test receipts after each significant step.
- Review findings, decisions, commits, and residual risk.

Do not put secrets, raw credentials, or private customer data in `/tmp`.

## Working Loop

1. **Read shape before edits.** Map module ownership, public interfaces,
   invariants, and the live verification path. If no live path exists, build
   or name the smallest credible one first; a refactor without a falsifiable
   behavior-preservation loop is a rewrite in disguise.
2. **Choose one architectural pressure.** Examples: split ownership, shallow
   wrapper, dependency direction, duplicated data shape, feature logic hiding
   in UI glue. Apply `harnesses/shared/references/delete-first.md` before
   optimizing or automating the pressure, and use the synced Ponytail skill
   (`skills/.external/dietrich-ponytail/SKILL.md`) as the default
   anti-overengineering lens before adding abstractions, files, dependencies,
   wrappers, or automation. Do not tidy everything.
3. **Make one significant step.** Significant means a moved boundary, deleted
   abstraction, renamed public concept, data-flow simplification, or large-file
   split. Mechanical formatting is not a milestone.
4. **Live-test immediately.** Use the repo's verification path, `/qa`, or the
   surface-specific route. Unit tests alone are insufficient for a milestone.
5. **Autoreview the milestone.** Use fresh-context critique when substantive:
   diff + architecture goal + fitness tests only, no author reasoning. Fix
   blockers before continuing. Scale the critic topology with
   `harnesses/shared/references/quality-system.md`; a risky boundary change
   earns more than one lens.
6. **Commit green milestones.** One concern per commit. The progress file stays
   in `/tmp`, not the repo, unless the operator asks for a durable plan.
7. **Reassess stop rule.** Continue only while another high-leverage
   architecture pressure remains inside scope and the live loop stays cheap.

## Delegation Judgment

Delegate on judgment per the shared Roster contract: native subagents by
default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Useful lanes:

- Explore lane: map ownership and coupling before edits.
- Critic lane: attack the architecture goal or a milestone diff.
- QA lane: exercise the live surface if the lead cannot drive it cheaply.

Default harsh critic: load the synced
`thermo-nuclear-code-quality-review` skill
(`skills/.external/cursor-thermo-nuclear-code-quality-review/SKILL.md`) for
milestone diffs that add abstractions, split modules, cross file-size
thresholds, or claim "cleaner architecture." Use `julius-caveman` to compress
progress-file summaries only when they get noisy.

## Stop Conditions

Stop and report instead of improvising when:

- The refactor requires product behavior changes.
- The live verification loop is absent and cannot be built cheaply.
- Three edits hit the same file without simplifying the architecture.
- A milestone breaks a public contract and no migration path is obvious.
- Review says the goal is vague, unmeasurable, or already satisfied.

## Gotchas

- **Vibes as oracle.** "Happy" is not evidence. The stop rule needs a diff,
  live proof, and review signal.
- **Architecture theater.** Renames, folders, and wrappers do not count unless
  they reduce coupling, clarify ownership, or delete a real failure mode.
- **Unfenced win.** A god-file split, a killed dependency, or a corrected
  dependency direction with no gate to stop it regrowing comes back. Ratchet the
  structural win into a standing gate — a fitness function, a god-file baseline —
  per `harnesses/shared/references/quality-gates.md`.
- **Big-bang rewrite.** If you cannot test and commit the step independently,
  the step is too large.
- **Test-only proof.** Refactors break integration seams; run the live surface
  after each meaningful boundary change.
- **Review after everything.** Late review finds foundational mistakes too
  late. Review milestone diffs while reversal is cheap.
- **Untracked progress.** If `/tmp/refactor-{project}.md` is stale, the
  refactor is no longer resumable.

## Completion

Done means:

- Goal stop rule satisfied or explicitly blocked.
- Every significant step has a live-test receipt.
- Blocking review findings fixed or rejected with a reason.
- Meaningful milestones committed.
- `/tmp/refactor-{project}.md` names final architecture, commits, verification,
  residual risk, and follow-up pressure outside scope.
- Final repo status is clean and branch/remote sync is reported when pushed.
