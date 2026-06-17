# Lenses

Compact critic rubrics. The primary reads a lens here and commissions an
**ad-hoc** critic subagent that embodies it — no static persona file is
required (backlog 061, "subagent roles, not files"). Name the
lens, give it scope + an evidence contract, synthesize its findings yourself.
Used by `harnesses/shared/references/routing.md`, `/code-review`, `/critique`,
`/groom`, `/refactor`, and `/shape`.

## critic
**Essence:** adversarial freshness — test the claim against the artifact and
oracle, not the author's rationale.
**Looks for:**
- Missing acceptance evidence, stale SHA evidence, or unexercised paths.
- Diff claims that do not match the changed files.
- Risk hidden by generated files, broad summaries, or self-review.
- Blocking ambiguity in the oracle, scope, or residual risk.
**Catches:** plausible "done" claims that would fail production because no
fresh reviewer tried to refute the actual artifact.

## ousterhout
**Essence:** deep modules — a simple interface over a powerful implementation;
information hiding manages complexity.
**Looks for:**
- Interface complexity high relative to functionality delivered (shallow module).
- Leaked implementation details: data structures, dependencies, algorithms.
- Pass-through methods; generic names (`Manager`, `Util`, `Helper`).
- Change amplification and cognitive load in the diff.
**Catches:** shallow modules and leaked internals that force change
amplification and make safe modification impossible.

## carmack
**Essence:** direct implementation and shippability — focus is deciding what
NOT to do.
**Looks for:**
- Simplest concrete solution first; no abstraction without 2+ real uses.
- Every commit deployable (tests pass, no broken build).
- Optimization or scope expansion only after measurement.
- Speculative features / "we'll need it later" framework-building.
**Catches:** premature abstraction or unmeasured optimization that produces
over-engineered, unshippable code.

## grug
**Essence:** complexity is the enemy — say "no" to abstraction theater early
and often.
**Looks for:**
- Abstraction before two concrete uses or a clear cut-point.
- Too many layers, clever code, patterns that hurt debugging.
- Chesterton's-fence violations: removing code whose reason isn't understood.
- Frameworks/microservices where a monolith or direct call works.
**Catches:** early abstraction and complexity that make the code impossible to
debug or change without cascading breakage.

## beck
**Essence:** red-green-refactor TDD + simple design (passes tests, reveals
intention, no duplication, fewest elements).
**Looks for:**
- Tests written before implementation for new behavior or a bug fix.
- The four design rules applied in priority order; YAGNI enforced.
- Small evolutionary steps, not big-bang changes.
- Abstraction only after 2+ concrete implementations exist.
**Catches:** code written before its tests, leaving untestable design that
can't be refactored safely.

## cooper
**Essence:** classicist TDD — mock only at system boundaries, never internal
collaborators.
**Looks for:**
- Internal mocks: `vi.mock` / `jest.mock` on relative paths or owned packages.
- Tests exercising real seams vs. mocking owned modules.
- Use of a real or in-memory fake instead of a mock at the boundary.
- Missing integration coverage where modules meet.
**Catches:** internal mocks that let contract/edge-case integration bugs ship
while the whole suite stays green.

## security
**Essence:** trust-boundary discipline — untrusted input, authority, secrets,
and network or filesystem effects must preserve explicit invariants.
**Looks for:**
- Missing authentication, authorization, origin, CSRF, or tenant checks on new
  routes, middleware, jobs, or command paths.
- Secrets, tokens, credentials, or sensitive payloads reaching logs, errors,
  traces, fixtures, prompts, commits, or client-visible output.
- SSRF, path traversal, open redirect, injection, unsafe deserialization, or
  input-laundering through fetch, URL, filesystem, shell, SQL, or template
  boundaries.
- Cryptography, signing, session, token, or expiry logic built from ad-hoc
  string handling or unauthenticated state.
**Catches:** confused-deputy and trust-boundary bugs that pass happy-path tests
because no adversarial path exercised the authority or input boundary.

## works
**Essence:** tests are not the whole definition of working — public surface,
human workflow, performance tradeoffs, compatibility, and operations matter.
**Looks for:** see `harnesses/shared/references/works-critique.md`.
**Catches:** changes that pass tests while the API/CLI/UI, operator path, or
production signal is incoherent.

## delete-first
**Essence:** question, delete, simplify, speed up, automate — in that order.
**Looks for:** see `harnesses/shared/references/delete-first.md`.
**Catches:** optimizing or automating a requirement, dependency, process, mode,
or abstraction that should not exist.

## Adding a lens
A lens is name + essence + "looks for" + "the failure it catches." Keep it to
that shape — this is a dispatch-time rubric, not an essay. Security, perf, and
API-contract lenses can be added the same way as the need arises.
