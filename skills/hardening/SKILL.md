---
name: hardening
description: |
  Agent-driven test hardening for functions, specs, and acceptance surfaces:
  property testing, mutation testing, acceptance mutation, CRAP/SCRAP risk,
  and structural DRY analysis. Use when: "property test this", "mutation
  test", "CRAP analysis", "DRY analysis", "harden tests", "find weak tests",
  "kill surviving mutants", "acceptance mutation", "uncle bob hardening".
  Trigger: /hardening.
argument-hint: "[property|mutation|acceptance|risk|dry|full] [target]"
---

# /hardening

Find places where the code or tests can lie, then add the smallest hardening
loop that makes the lie executable.

This skill absorbs the Uncle Bob pattern: agents are good at selecting a
bounded target, deriving domains and invariants, generating focused hardening
checks, running them, and repairing the bugs or weak tests they expose. The
output is not a generic "more tests" report. It is a short list of hardened
surfaces with commands, survivors, fixes, and residual risk.

## Execution Stance

You are the lead hardening engineer.
- Keep target selection, oracle quality, and final acceptance on the lead model.
- Dispatch roster lanes for independent target discovery and critic review.
- Prefer one high-risk target fully hardened over broad shallow coverage.
- Use repo-native tools first; add new hardening tools only when the repo has
  no reasonable equivalent and the target earns the dependency.

## Delegation Floor

When a provider roster is available (repo `.spellbook/agents.yaml` or system
`~/.spellbook/agents.yaml`), `/hardening` starts by probing the roster and
dispatching two or more available roster members. Use one lane to identify
hardening candidates and one lane to challenge the chosen oracle, domains,
mutants, and residual risk. The lead owns synthesis, edits, verification, and
receipts. Direct lead-only work is limited to mechanical command execution,
emergency unblocks, explicit user-forbidden delegation, or fewer than two
available roster members.

## Routing

| Mode | Use When | Primary Output |
|---|---|---|
| `property` | Pure or mostly pure behavior has a describable domain, range, invariant, round trip, monotonicity, idempotence, parser law, or conservation rule. | Property tests plus counterexample fixes. |
| `mutation` | Existing tests may be shallow, branch logic changed, or a file has enough coverage to justify mutant execution. | Mutation run, survivor fixes, and killed-mutant evidence. |
| `acceptance` | Gherkin, examples, fixtures, API contracts, CLI transcripts, or golden paths claim user behavior. | Mutated examples/contracts prove acceptance checks are connected. |
| `risk` | You need to find risky code before choosing targets. | CRAP/SCRAP/complexity/coverage-ranked target list. |
| `dry` | Duplication may hide copy-paste bugs or force scattered fixes. | Structural duplicate candidates and refactor/no-refactor calls. |
| `full` | User asks for a hardening pass without naming a mode. | Risk-ranked sequence: risk -> property/mutation/acceptance -> dry if relevant. |

## Upstream Signals

Other workflow skills may route here without making hardening part of the
default fast path:

| Signal | Route |
|---|---|
| `/implement` names broad-domain or invariant-heavy behavior | `/hardening property` |
| `/code-review` finds branch-heavy changed code with shallow tests | `/hardening mutation` or `/hardening risk` |
| `/qa` relies on examples, fixtures, Gherkin, contracts, or golden files whose values should matter | `/hardening acceptance` |
| `/ci` reports missing hardening visibility for a repo that needs it | `/hardening risk` |
| `/deliver` carries a blocking phase verdict naming a test-strength gap | the mode named by that phase |

## Target Selection

Start narrow. Pick a function, module, spec file, feature, route, or CLI
surface that satisfies at least two of:

- recently changed or likely to change;
- high complexity, low coverage, high CRAP/SCRAP, or dense branching;
- accepts broad input domains, parses/serializes/transforms data, computes
  totals, validates boundaries, schedules, allocates, or normalizes names;
- has examples or acceptance fixtures whose values should matter;
- has had production bugs, flaky tests, or hand-written test smells.

Reject targets that are mostly I/O glue unless you can isolate a deterministic
core or test the boundary with recorded fixtures. Do not property-test
framework wiring just because the mode was requested.

## Property Testing Protocol

For each candidate, write the oracle before code changes:

```markdown
## Property Oracle
- Target:
- Domain:
- Exclusions / invalid inputs:
- Generator strategy:
- Invariant or relation:
- Shrinking / counterexample capture:
- Exact command:
- Why examples alone are insufficient:
```

Good property targets usually have one of these shapes:

- parser/printer round trip;
- encode/decode or serialize/deserialize equivalence;
- normalization idempotence;
- sorted/filter/map laws;
- monotonicity or conservation of totals;
- commutativity or associativity where the domain actually supports it;
- equivalent implementations, slow reference model, or differential oracle;
- boundary rejection: invalid values fail with stable diagnostics.

If you cannot name the property in one sentence, do not invent a vague random
test. Fall back to example tests or mutation testing.

## Mutation Protocol

Mutation is a deliberate hardening workflow, not the default fast gate.

1. Baseline tests must pass before mutation starts.
2. Prefer changed files or high-risk functions over whole-repo mutation.
3. Use scan/dry-run modes first when the tool supports them.
4. Run bounded mutants with timeouts and, where supported, differential
   manifests or line selection.
5. Treat survivors as evidence. Fix the production bug, strengthen the test, or
   document an equivalent-mutant filter.
6. Do not mark done until the rerun kills the targeted survivors or the residual
   equivalent mutants are explicitly named.

## Acceptance Mutation Protocol

Acceptance mutation checks whether user-facing examples are connected to real
assertions.

- Mutate fixture/example/contract values, not application source.
- Run the same generated or scripted acceptance path against mutated data.
- A passing mutated example is a survivor: the acceptance path failed to notice
  a meaningful behavior change.
- Keep project-specific step handlers thin and exact. Unsupported steps,
  missing values, malformed values, and failed assertions must fail loudly.
- Use machine-readable reports when available; otherwise capture the exact
  command transcript as evidence.

## Risk And DRY Protocol

Risk analysis chooses where to spend hardening effort.

- CRAP-style metrics combine complexity and coverage to find risky production
  functions.
- SCRAP-style metrics apply similar pressure to test/spec code: long examples,
  no assertions, multiple phases, heavy stubbing, and hidden helper indirection.
- DRY analysis is candidate discovery, not an automatic refactor command.
  Confirm duplicated structure has duplicated intent before changing code.

## Completion Gate

Every hardening report includes:

```markdown
## Completion Gate
- Target hardened:
- Hardening mode:
- Oracle / property / mutant / metric used:
- Exact command exercised:
- Bug or weak-test evidence found:
- Fixes made:
- Rerun evidence:
- Survivors / residual risk:
- Roster receipt ids:
```

No "hardened" claim without the exact command and rerun evidence.

## Gotchas

- Random input without an invariant is noise.
- A property that restates the implementation is not an oracle.
- Mutation survivors are not automatically production bugs; equivalent mutants
  exist, but they must be explained rather than ignored.
- High CRAP points at risk. It does not prove bad design by itself.
- DRY tools report structural similarity. Shared shape can still represent
  different concepts.
- Acceptance mutation should mutate business examples, not generated test
  mechanics.
- Do not add slow mutation gates to the normal fast path unless a repo-local
  ticket explicitly chooses that tradeoff.

## Source Inspiration

- `unclebob/Acceptance-Pipeline-Specification`: portable Gherkin-to-IR
  acceptance mutation with persistent runner workers.
- `unclebob/mutate4go`, `mutate4java`, and `clj-mutate`: bounded file-level
  mutation with coverage, scans, differential manifests, and survivor loops.
- `unclebob/crap4go`, `crap4java`, and `crap4clj`: complexity plus coverage as
  a target-selection metric.
- `unclebob/scrap`: structural test/spec smell scoring for refactor decisions.
- `unclebob/dry4go` and `dry4java`: structural duplicate discovery as evidence,
  not an automatic refactor.
