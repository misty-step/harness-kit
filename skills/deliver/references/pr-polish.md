# pr-polish

Elevate a working PR from "CI green" to "exemplary." The deep reference for
`/deliver --polish-only`'s hindsight sanity pass (absorbed from the former
`/settle` Phase 2). Test-depth and coverage are owned by `/qa` and
`/hardening` — this doc is the architectural-hindsight + confidence lens.

## Hindsight Architecture Review

Read the full diff with one question: **"Would we build it the same way starting over?"**

Don't defend sunk cost. The code exists; evaluate it as if reviewing a proposal.

### Smell Catalog

**Shallow modules** — Interface is as complex as implementation. The module
doesn't hide anything. Fix: merge with caller or deepen by absorbing related logic.

**Pass-through layers** — Methods that just forward to another method. Each layer
adds cognitive cost without adding value. Fix: eliminate the middleman.

**Hidden coupling** — Two modules that must change together but don't declare
a dependency. Fix: make the coupling explicit (shared type, explicit interface)
or eliminate it.

**Temporal decomposition** — Code organized by *when* things happen rather than
*what information* they manage. Fix: reorganize around information hiding.

**Missing abstractions** — Same multi-step pattern repeated in 3+ places.
Fix: extract, but only after the pattern is stable (rule of three).

**Premature abstractions** — Generic framework for one use case. Fix: inline it.
Concreteness is a virtue until you have evidence for abstraction.

**Tests testing implementation** — Mocking internals, asserting on method call
counts, breaking when refactoring. Fix: test observable behavior through public
interfaces.

### Applying the Review

For each smell found:
1. Name the smell and the affected files
2. Assess severity: blocking (fix now) vs advisory (note for follow-up)
3. Blocking smells get fixed in this phase. Advisory smells get a `backlog.d/` item.

## Test Audit

### Coverage Gaps

Which code paths have no test? Prioritize:
- Error paths and failure modes
- Edge cases and boundary values
- Newly added branches

Happy-path-only coverage is a false signal. The bugs live in the paths nobody tested.

### Brittle Tests

Tests that break when implementation changes but behavior doesn't. Usual causes:
- Over-mocking (mocking internals instead of boundaries)
- Asserting on call counts or internal state
- Coupling to serialization format or log output

Fix: rewrite to assert on observable behavior through public interfaces.

### Edge Cases

Each represents a production failure mode:
- Boundary values (0, 1, max, off-by-one)
- Empty/null/missing inputs
- Concurrent access (if applicable)
- Error recovery and retry paths
- Resource exhaustion (full disk, OOM, timeout)

### Assertion Quality

Weak: `assert result is not None` — proves the code ran, not that it's correct.

Strong: `assert result.status == 401 and "expired" in result.body` — proves
specific behavior under specific conditions.

One vague assertion per test is a smell. Prefer specific outcome verification.

### Test Naming

Name describes the behavior being verified, not the method being called.

- Bad: `test_login`, `test_process`, `test_handle_request`
- Good: `test_login_with_expired_token_returns_401`
- Good: `test_empty_cart_checkout_raises_validation_error`

If you can't name the behavior, you don't understand what you're testing.

## Confidence Assessment

Confidence is an explicit deliverable. State it with evidence, not feelings.

### Levels

**High** — All behaviors tested, edge cases covered, matches existing patterns,
small blast radius. You'd merge without hesitation.

**Medium** — Core path tested but edge cases have gaps. Or: touches unfamiliar
code. Or: large blast radius with adequate rollback story.

**Low** — Untested paths, novel patterns, large blast radius, no rollback story.
Requires additional verification before merge.

### Evidence That Increases Confidence

- Passing tests (necessary but not sufficient)
- Live verification of affected features (manual or automated)
- Before/after comparison of observable behavior
- Explicit enumeration of what could go wrong and why it won't
- Reviewer approval from domain expert

### Evidence That Decreases Confidence

- "It compiles" as the only signal
- Tests that only cover happy path
- Large diff with no test changes
- Changes to shared utilities with no downstream verification
- "I think this is fine" without supporting evidence

### Reporting

State confidence per area when the PR spans multiple concerns:

```
Confidence:
- Auth changes: HIGH — full test coverage, matches existing pattern
- Cache invalidation: MEDIUM — core path tested, concurrent eviction untested
- Migration: LOW — no rollback tested, affects all users
```

## Agent-First Assessment

Run structured assessment tools when available. These produce scored,
machine-readable findings that complement human review.

| Tool | Purpose | When |
|------|---------|------|
| `assess-review` | Code quality scoring (triad, strong tier) | Always |
| `assess-tests` | Test quality scoring | Always |
| `assess-docs` | Documentation quality | When docs touched |

### Hard Gate

All `fail` findings from assess-* tools must be addressed before the polish pass exits.
`warn` findings are advisory — fix or note, but don't block on them.

## Feature Evidence

If the PR introduces new workflows, skills, or user-facing features:
produce visual evidence (screenshots, GIFs) demonstrating them in action.
Text logs are raw data, not reviewer-facing proof.

**GitHub mode:** Store in `.evidence/<branch>/<date>/`, commit to the branch,
then optionally mirror to a draft release for inline PR URLs. See
`skills/demo/references/pr-evidence-upload.md` for the recipe.

**Git-native mode:** Store in `.evidence/<branch>/<date>/`, commit to the branch.
Evidence becomes part of the auditable git history.

## Exit Criteria

The polish pass is complete when:
- [ ] Hindsight review performed, blocking smells fixed
- [ ] Test audit performed, gaps filled
- [ ] Confidence assessment stated with evidence
- [ ] All assess-* `fail` findings resolved
- [ ] Docs current with changes
- [ ] Feature evidence captured (screenshots/GIFs) if PR introduces new workflows

If this pass produced commits, loop back through CI — it must stay green.
