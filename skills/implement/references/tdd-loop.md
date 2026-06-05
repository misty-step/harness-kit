# TDD Loop

Red → Green → Refactor → Commit. The only loop.

## The cycle

1. **Red.** Write one failing test for one behavior. Run it. Confirm it
   fails for the expected reason (not a typo, not an import error).
   A test that fails for the wrong reason is a false red.

2. **Green.** Write the minimum production code that makes the test pass.
   Not the "right" code, not the elegant code — the minimum. Elegance
   comes in refactor.

3. **Refactor.** With the test green, improve the code you just wrote.
   Kill duplication, clarify names, tighten invariants. Tests stay green
   through this step — if a test goes red in refactor, revert.

4. **Commit.** One logical unit. Message names the behavior, not the
   mechanic. `feat: reject empty usernames` not `add validation to user.js`.

Then: next behavior, next red. Never two reds in a row.

## Vertical, Not Horizontal

Do not treat RED as "write every test" and GREEN as "write all code." That
horizontal slice outruns the feedback loop: tests start describing imagined
structure, failures blur together, and green can mask behavior that was never
exercised.

Correct TDD is vertical:

```text
RED -> GREEN -> REFACTOR: behavior 1
RED -> GREEN -> REFACTOR: behavior 2
RED -> GREEN -> REFACTOR: behavior 3
```

Each test responds to what the last cycle taught you. If several tests are red
at once, stop and shrink the cycle until one behavior has one clear failure.

## One behavior per test

A test asserts one observable behavior. If you find yourself writing
"and" in the test name, split it. Examples:

- Good: `rejects empty usernames`
- Good: `trims trailing whitespace from usernames`
- Bad: `validates and normalizes usernames`

One behavior = one clear failure message = one clear fix.

## Test behavior, not implementation

Tests that assert the shape of the code break on every refactor and
teach you nothing. Test the outside of the module:

- Good: `returns 404 when the user does not exist`
- Bad: `calls userRepo.findById once`
- Good: `the exported function returns a sorted array`
- Bad: `uses Array.prototype.sort internally`

Mocks and spies are tools of last resort, not the default. If you need
to mock everything, the module under test has too many collaborators —
that's a design smell, not a testing problem.

## When to skip TDD

TDD is the default. Skip only when the feedback loop TDD provides adds
no value. Document every skip inline in the commit message.

**Skip OK:**
- **Config files** — JSON, YAML, .env schemas. The test is "does the app
  start." That's covered by integration smoke.
- **Generated code** — protobuf output, OpenAPI clients, migrations from
  a schema diff. Test the generator, not the generated.
- **UI layout** — visual placement, CSS, component trees. Test behavior
  (clicks produce events, data renders) but not pixel positions.
- **Pure exploration** — spike branches that will be thrown away.
  Commit the learning, delete the spike, then TDD the real version.
- **Trivial plumbing** — one-line re-exports, type-only changes. If the
  type system catches it, the test is redundant.

**Do not skip for:**
- "It's simple" — simple code is where silent bugs hide
- "I'll add tests later" — you won't
- "The existing tests cover it" — if you can't point at the specific
  assertion that would fail, they don't

## Failure-mode tests

Happy-path tests prove the code works when inputs are good. That's
half the job. Every behavior that can fail loudly needs a test:

- Invalid input → specific error, not generic crash
- External dependency down → loud failure, not silent fallback
- Invariant violation → assertion fires, not data corruption

Silent catch-and-return is a red flag. If new code handles an exception,
there's a test that asserts the handling is correct.

## Hardening triggers

Example tests are enough for many behaviors. Route to `/hardening property`
before calling implementation done when the target has broad input domains or
law-shaped behavior: parsers, encoders, serializers, normalization, allocation,
scheduling, state transitions, arithmetic totals, sorting, filtering, or
round trips. The property oracle must name the domain, exclusions, generator,
and invariant. If no invariant can be stated in one sentence, stay with
example tests and let review decide whether mutation or risk hardening is
worth a separate pass.

## The refactor step's scope

Local. Improve the code you just wrote in this cycle:

- Rename a variable that reads awkwardly now that the behavior is clear
- Extract a helper when the function grew past one idea
- Collapse duplicated branches

Not in scope:
- Renaming modules, restructuring directories
- Changing public APIs that other code depends on
- Cross-cutting simplification

Those are `/refactor`'s job. Stay bounded.

## Multi-behavior features

When a feature has several behaviors, sequence them:

1. List the behaviors (the oracle usually gives you this list)
2. Pick the smallest that can fail independently
3. Red-Green-Refactor-Commit that one
4. Next behavior

Don't write five tests red at once. You lose the signal on which change
fixed which test, and "green-ish" becomes acceptable.

## Exit

A TDD cycle ends when every oracle criterion maps to at least one
passing test, the tree is clean, and commits are atomic. "It works on
my machine without the tests running" is not done.
