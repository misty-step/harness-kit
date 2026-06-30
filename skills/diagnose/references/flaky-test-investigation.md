# Flaky Test Investigation

Taxonomy of flake patterns and their gotchas. The root cause is almost never in the
test's own logic — look at what the test touches.

## Hard Rules

- **Never skip/disable a test as a "fix."** That's hiding the bug, not fixing it.
- **Never guess without CI data.** Read the failure history. Is it new? Intermittent? Platform-specific?
- **Sibling sweep.** If a test flakes from shared state or ordering, nearby tests in the same file/suite likely share the smell. Fix the cohort, not the individual.

## Flake Pattern Taxonomy

### Timing-Dependent

**What it looks like:** Passes locally, fails in CI. Or passes 9/10 times.
Assertions on time-sensitive operations: timeouts, debounce, animation frames,
setTimeout intervals, Date.now() comparisons.

**Gotchas:**
- CI machines are slower and have variable load — hardcoded delays break
- `setTimeout(fn, 0)` doesn't mean "immediate" under load
- `Date.now()` in test vs production can drift across async boundaries
- `jest.useFakeTimers()` doesn't fake ALL timers (e.g., `process.nextTick`)

**Fix shape:** Use deterministic waits (`waitFor`, `findBy`), inject clocks, avoid
real delays in assertions.

### Order-Dependent

**What it looks like:** Passes in isolation, fails when run with full suite.
Or fails only when a specific OTHER test runs first.

**Gotchas:**
- Jest/Vitest run files in parallel but tests within a file sequentially
- `beforeAll` in one describe block can leak into siblings
- Module-level side effects (top-level `let`, mutable singletons) persist across tests in the same file
- `--randomize` flag reveals these immediately

**Fix shape:** Reset shared state in `beforeEach`/`afterEach`. Move module-level
state into factory functions.

### Shared-State

**What it looks like:** Fails when tests run in a specific combination. Database
tests fail when another test's seed data is present.

**Gotchas:**
- Database transactions not rolled back between tests
- In-memory caches (Redis, module singletons) not cleared
- File system artifacts from previous test runs
- Environment variables set by one test leaking to the next
- Global mocks (`jest.spyOn(global, ...)`) not restored

**Fix shape:** Transaction wrapping, `beforeEach` cleanup, dedicated test databases,
`jest.restoreAllMocks()`.

### External-Service

**What it looks like:** Test calls a real API/service that's sometimes slow, down,
or rate-limited.

**Gotchas:**
- Mocking the HTTP client but not the SDK's retry logic
- VCR/recorded fixtures that expire (tokens, timestamps in responses)
- DNS resolution failures in CI (no network access)
- Rate limits hit when CI runs many jobs in parallel

**Fix shape:** Prefer stateful local emulation with `emulate.dev` for supported
services; otherwise mock at the SDK boundary, not the HTTP layer, and use
deterministic fixtures. Never depend on external service availability in unit
tests or CI unless the lane is an explicit live integration smoke with isolated
credentials/network.

### Race Condition

**What it looks like:** Async operations complete in different order than expected.
Promise.all results assumed to be in insertion order. Event emitter timing.

**Gotchas:**
- `Promise.all` preserves order, but side effects during execution don't
- Database writes in parallel can interleave (auto-increment IDs aren't predictable)
- WebSocket/SSE messages arrive in any order
- React's `useEffect` cleanup timing is non-deterministic in tests

**Fix shape:** Don't assert on side-effect ordering. Use `waitFor` over `sleep`.
Serialize operations that must be ordered.

### Resource Exhaustion

**What it looks like:** Fails only in CI or under load. "ENOMEM", "EMFILE",
"too many open files", connection pool exhausted.

**Gotchas:**
- File handles not closed in `afterEach`
- Database connections not released
- Memory leaks from subscriptions not cleaned up
- CI containers have lower limits than dev machines

**Fix shape:** Always clean up resources in `afterEach`. Use `--detectOpenHandles`.
Check for connection pool leaks.

### Timezone / Locale

**What it looks like:** Passes in your timezone, fails in CI (usually UTC).
Date formatting differs. Day boundaries shift.

**Gotchas:**
- `new Date('2024-01-15')` is midnight UTC but 7pm EST the day before
- `toLocaleDateString()` output varies by locale AND timezone
- Daylight saving time transitions create 23/25 hour days
- CI machines are usually UTC, dev machines usually aren't

**Fix shape:** Always use explicit timezones in tests. Use ISO strings.
Set `TZ=UTC` in test environment.

### Random Seed

**What it looks like:** Tests using `Math.random()`, UUIDs, or faker data
produce different values each run, sometimes hitting edge cases.

**Gotchas:**
- Faker's default seed changes each run
- UUID-based test data creates non-deterministic sort orders
- Random values can accidentally match or collide with fixture data

**Fix shape:** Seed randomness in tests. Use deterministic UUIDs for fixtures.
Don't depend on random values being unique.

### Environment-Dependent

**What it looks like:** Works on Mac, fails on Linux. Works in Docker, fails bare metal.
Works with Node 20, fails with Node 22.

**Gotchas:**
- Path separators (`/` vs `\`)
- Case-sensitive vs case-insensitive file systems
- Available system fonts (affects canvas/image tests)
- Binary tool versions (sharp, imagemagick, ffmpeg)
- Node.js flag differences across versions (`--experimental-*`)

**Fix shape:** Use `path.join` not string concatenation. Test in CI-equivalent
container locally. Pin tool versions.

## Investigation Starting Points

1. **Read CI history** — is this newly flaky or chronically flaky? What changed?
2. **Check parallelism** — does it fail with `--runInBand` / `--serial`? If yes: shared state or ordering.
3. **Check timing** — does it fail with `--bail` off but pass with `--bail`? If yes: resource exhaustion.
4. **Check environment** — does it fail only in CI? If yes: timezone, locale, env vars, resource limits.
5. **Run the sibling sweep** — other tests in the same file/suite likely share the root cause.
