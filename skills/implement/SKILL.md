---
name: implement
description: |
  Atomic TDD build skill. Takes a context packet (shaped ticket) and
  produces code + tests on a feature branch. Red → Green → Refactor.
  Does not shape, review, QA, or ship — single concern: spec to green tests.
  Use when: "implement this spec", "build this", "TDD this", "code this up",
  "write the code for this ticket", after /shape has produced a context packet.
  Trigger: /implement, /build (alias).
argument-hint: "[context-packet-path|ticket-id]"
---

# /implement

Spec in, green tests out. One packet, one feature branch, one concern.

## Invariants

- Trust the context packet. Do not reshape. Do not re-plan.
- If the packet is incomplete, **fail loudly** — do not invent the spec.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use one bounded builder lane and one adversarial validator/refactor lane; give them the context packet, oracle, boundaries, repo anchors, and what would embarrass us if green tests missed it.

## Completion Evidence

Completion evidence core applies: use `harnesses/shared/AGENTS.md`
(Completion Evidence) as the universal evidence shape, then fill the local
fields required by the implementation final message.

## Contract

**Input.** A context packet: goal, non-goals, constraints, repo anchors,
oracle (executable preferred), implementation sequence. Resolution order:

1. Explicit path argument (`/implement backlog.d/033-foo.md`)
2. Backlog item ID (`/implement 033`) → resolves via `backlog.d/<id>-*.md`
3. Last `/shape` output in the current session
4. **No packet found → stop.** Do not guess the spec from a title.

Required packet fields (hard gate — missing any = stop):
- `goal` (one sentence, testable)
- `oracle` (how we know it's done, ideally executable commands)
- `implementation sequence` (ordered steps, or explicit "single chunk")

If the packet contains `Formal Spec Required: yes`, also require a `Formal
Spec` block with: informal spec, formal examples, acceptance oracle, hardening
budget, and waiver path. Missing or vague formal-spec fields are packet
incompleteness, not builder judgment.

See `references/context-packet.md` for the full shape.

**Output.**
- Code + tests on a feature branch (`<type>/<slug>` from current branch)
- All tests green (new + existing)
- Working tree clean (no debug prints, no scratch files)
- Commits in repo convention — one logical unit per commit
- Final message: branch ref + oracle checklist status + Completion Gate

**Stops at:** green tests + clean tree. Does not run `/code-review`,
`/qa`, `/ci`, or open a PR.

## Workflow

### 1. Load and validate packet

Resolve the packet (order above). Parse required fields. If any are missing
or vague ("add feature X" with no oracle), stop with:

> Packet incomplete: missing <field>. Run /shape first.

Do not try to fill in the gaps. Shape is a different skill's judgment.

### 2. Create the feature branch

`git checkout -b <type>/<slug>` from the current branch. Builders never
commit to master/main. If you forget, create the branch after and
cherry-pick before handing off.

### 3. Dispatch the builder

Spawn roster-backed **builder** lanes with:
- The full context packet
- The executable oracle
- The TDD mandate (see below)
- File ownership (if the packet decomposes into disjoint chunks, spawn
  multiple builders in parallel — one per chunk, each with subset of oracle)

**Builder prompt must include:**
> You MUST write a failing test before production code. RED → GREEN →
> REFACTOR → COMMIT. Exceptions: config files, generated code, UI layout.
> Document any skipped-TDD step inline in the commit message.

For packets with `Formal Spec Required: yes`, the first red test must be an
acceptance test derived from the formal examples and acceptance oracle. Unit
tests come after the failing acceptance check exists; production code comes
after both acceptance and unit intent are executable. Waive this only through
the packet's named waiver path and record the waiver in the Completion Gate.

See `references/tdd-loop.md` for the full cycle and skip rules.

**Milestone critic gate.** After each chunk's builder returns, dispatch a fresh
read-only critic that sees ONLY the chunk diff + the packet oracle + the todo —
never the builder's reasoning. It returns blocking gaps (oracle clause unmet,
behavior lost, invariant violated). Do not advance to the next chunk until the
critic returns no blocking gap, or the gap is explicitly waived in the
Completion Gate. Skip for trivial diffs (<20 LOC, single file). This enforces
the milestone gate defined in `harnesses/shared/AGENTS.md` (Layer 2 — Dispatch).

### 4. Verify exit conditions

Before exiting, confirm:
- [ ] Every oracle command exits 0 (run them, don't trust the builder)
- [ ] `git status` clean (no untracked debug files)
- [ ] No `TODO`/`FIXME`/`console.log` added that isn't in the spec
- [ ] Commits are logically atomic (one concern per commit)
- [ ] Broad-domain or invariant-heavy behavior was routed through
      `/hardening property`, or the handoff names why example tests are enough

If any check fails, dispatch a builder sub-agent to fix. Max 2 fix loops,
then escalate.

### 5. Hand off

Output: feature branch name, commit list, oracle checklist (which commands
pass), Completion Gate, residual risks. Do not run review, do not merge, do
not push unless the packet explicitly says so.

```markdown
## Completion Gate
- Exact end-user behavior changed: behavior or internal operator behavior implemented.
- Evidence that proves it: failing-then-passing test, oracle command, or artifact proving the behavior.
- Exact command/path/route exercised: command, URL, route, file path, or tool call actually run.
- Repo-fit check: live repo pattern, contract, or boundary the implementation follows.
- Observability / instrumentation debt: named signal added, existing signal used, or debt recorded.
- Residual risk: unverified path, deferred edge case, or none with reason.
```

Completion evidence core applies: behavior changed or verified, live evidence,
exact command/path/route, repo-fit check, and residual risk. See
`harnesses/shared/AGENTS.md` (Completion Evidence).

Local fields include observability / instrumentation debt when the changed
behavior has no named signal.

For internal-only changes, replace "end-user" with the developer/operator
behavior the implementation changes.

## Scoping Judgment (what the model must decide)

- **Test granularity.** One behavior per test. If you can't name the
  behavior in one short sentence, the test is too big.
- **When to skip TDD.** Config, generated code, UI layout, pure
  exploration. Document the skip in the commit. Everything else: test first.
- **When to escalate.** Builder loops on the same test failure 3+ times,
  the oracle contradicts the constraints, or the spec requires behavior
  that violates an invariant. Stop and report, don't power through.
- **Parallelism.** Only parallelize when file ownership is disjoint and
  oracle criteria partition cleanly. Shared files → serial builders.
- **Refactor depth.** The refactor step in TDD is local — improve the
  code you just wrote. Broader refactors are `/refactor`'s job, not yours.

## What /implement does NOT do

- Pick tickets (caller's job, or `/deliver` / `/flywheel`)
- Shape or re-shape specs (→ `/shape`)
- Code review (→ `/code-review`)
- QA against the running app (→ `/qa`)
- CI gates / lint (→ `/ci`)
- Simplification passes beyond TDD refactor (→ `/refactor`)
- Ship, merge, deploy (→ human, or `/deliver --polish-only` → `/ship`)

## Stopping Conditions

Stop with a loud report if:
- Packet is incomplete or ambiguous
- Oracle is unverifiable (prose-only checkboxes with no executable form —
  write one, or stop)
- Builder fails the same test 3+ times after targeted fix attempts
- Spec contradicts itself or violates a stated invariant
- Tests hit an external dependency that isn't available

**Not** stopping conditions: spec is hard, unfamiliar codebase, initial
tests red. Those are the job.

## Gotchas

- **Reshaping inside /implement.** If the spec is wrong, stop. Don't
  silently rewrite the oracle to match what you built.
- **Declaring victory with partial oracle.** "Most tests pass" is not
  green. Every oracle command exits 0, or you're not done.
- **Silent catch-and-return.** New code that swallows exceptions and
  returns fallbacks is hiding bugs. Fail loud. Test the failure mode.
- **Testing implementation, not behavior.** Tests that assert the
  structure of the code break on every refactor. Test what the code
  does from the outside.
- **Committing debug noise.** `console.log`, `print("here")`, commented-out
  code. The tree must be clean before exit.
- **Skipping TDD without documenting.** Config and generated code are
  fine exceptions; silently skipping because "it was simpler" is not.
- **Parallelizing coupled builders.** Two builders editing files that
  import each other = merge pain and lost work. Partition by file
  ownership before parallel dispatch.
- **Branch drift.** Forgetting to create the feature branch and
  committing to the current branch. Always `git checkout -b` first.
- **Scope creep from builders.** Builder adds "while I'm here"
  improvements. The spec is the constraint — raise a blocker, don't
  silently expand the diff.
- **Trusting self-reported success.** Builders say "all tests pass."
  Verify by running the oracle yourself. Agents lie (accidentally).
