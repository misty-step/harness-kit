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
**Heavy external version:** the synced Ponytail skill
(`skills/.external/dietrich-ponytail/SKILL.md`) — reach for it when the main
risk is bloat, boilerplate, or speculative engineering and you want the concrete
YAGNI / stdlib / native / existing-dependency ladder.

## fowler
**Essence:** Martin Fowler's *Refactoring* Ch.3 smell vocabulary — dense,
pretraining-anchored names that make a structural problem nameable in a diff.
The name carries its own definition; pair each with a diff-specific cue.
**Looks for** (each is a judgement call, surfaced as a question — never an
auto-block):
- **Mysterious Name** — a new name needs the body read to understand the call site.
- **Duplicated Code** — the diff repeats a structure that already exists, or
  repeats itself across new hunks, instead of extracting it.
- **Feature Envy** — a new method touches another object's data more than its
  own; the logic wants to live on that other type.
- **Data Clumps** — the same 3+ fields/params travel together and want one object.
- **Primitive Obsession** — string/int/map stands in for a domain concept that
  wants its own type (id, money, range, enum).
- **Repeated Switches** — the same switch/if-on-type appears in 2+ places; a new
  case means editing all of them.
- **Shotgun Surgery** — one logical change forced scattered edits across many
  files/functions.
- **Divergent Change** — one module is edited for unrelated reasons in the same
  diff; too many responsibilities.
- **Speculative Generality** — abstraction, hook, param, or "for later"
  flexibility with no current caller.
- **Message Chains** — `a.getB().getC().getD()`; the caller navigates structure
  it shouldn't know.
- **Middle Man** — a new class/method mostly delegates; a hop without value.
- **Refused Bequest** — an impl/subclass ignores or overrides most of what it
  inherits or the interface it claims.
**Binding rules:** a documented repo standard **overrides** this baseline; every
smell is a **judgement call**, reported as a question, never a hard violation.
**Catches:** structural decay that passes tests and review-by-vibes because no
one named it — duplication, wrong-home logic, primitive sprawl, change
amplification. (Curation credit: Matt Pocock's `/review` baseline.)

## thermo-nuclear
**Essence:** maximally strict maintainability — abstraction quality, file size,
and spaghetti-condition growth, with ambition for behavior-preserving "code
judo" restructurings, not local cleanup.
**Looks for:** see `skills/.external/cursor-thermo-nuclear-code-quality-review/SKILL.md`.
**Distinct earn over `ousterhout`/`grug`/`delete-first`/`fowler`:** the concrete
1000-line-file threshold and the "make it inevitable in hindsight" reframing bar.
**Catches:** working-but-messier diffs that rubber-stamp because nobody held the
structure to a restructuring-ambition bar.

## Decorrelate, don't stack
`fowler`, `thermo-nuclear`, `delete-first`, `grug`, `carmack`, and `ousterhout`
overlap heavily on structure-and-simplicity. Running three of them on the same
hunk produces three critics triple-reporting one duplication finding — the
undifferentiated wall this bench exists to prevent. Pick the **sharpest 1–2**
for the change (`fowler` for nameable structural smells, `thermo-nuclear` for
file-size/spaghetti and restructuring ambition, `delete-first`/`ponytail` for
"should this exist at all"); reach for a heavy external skill only when its
distinct earn applies. Diversity across failure modes beats redundancy.

## Adding a lens
A lens is name + essence + "looks for" + "the failure it catches." Keep it to
that shape — this is a dispatch-time rubric, not an essay. Security, perf, and
API-contract lenses can be added the same way as the need arises.
