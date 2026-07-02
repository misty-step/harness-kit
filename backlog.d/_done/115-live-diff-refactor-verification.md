# Context Packet: Document the "live-diff" verification pattern for behavior-preserving refactors

Priority: P2 (shipped)
Status: done
Estimate: S
Shipped: 2026-07-02

## PRD Summary
- User: any agent running `/deliver` or `/qa` on a behavior-preserving
  refactor (extract-to-module, route-thinning, dedup) whose target has little
  or no automated test coverage — i.e. the refactor must prove it changed
  *nothing observable*, but there is no test suite that pins the current
  behavior.
- Problem: the verification-system-first reference covers building a driver +
  grader for *new* behavior, but the hardest refactors are the ones where the
  oracle is "identical to before" and the codebase gives you no characterization
  net. Agents default to "unit tests pass" — which structurally cannot catch the
  integration bug between the refactored layer and its real dependencies, the
  exact thing a lift is most likely to break.
- Goal: name the **live-diff** technique as a first-class verification pattern so
  it gets reached for by default on coverage-poor refactors: prove equivalence by
  byte-diffing the live surface — the local refactor branch vs the already-deployed
  prod build — both pointed at the **same backing store**. Identical responses
  across a representative set of reads, plus error/404 parity, = behavior
  preserved. Divergence is the bug, located precisely.
- Why now: used live on 2026-06-17/18 to ship Habitat HA-033 (work-items lift)
  and HA-034 (planning lift). The planning routes had **zero** route-level tests;
  repository unit tests + a before/after live diff (local branch vs deployed
  prod, same Supabase DB → byte-identical list/detail/404 responses) was the only
  thing that actually proved the 6-route rewrite preserved behavior. It worked and
  should be codified rather than re-derived.
- Success signal: the reference names the pattern, its precondition (old code
  still running against the same store), its failure mode (drift = located bug),
  and when it does NOT apply — and an agent facing a coverage-poor refactor picks
  it without being told.

## Product Requirements
- P0: add a short subsection to
  `harnesses/shared/references/verification-system-first.md` — call it
  "Live-diff for behavior-preserving refactors" — covering:
  - **When**: the oracle is "identical to before" AND the target lacks
    characterization tests (lifts, route-thinning, convergence, dedup).
  - **How**: exercise the same representative inputs against (a) the local
    refactor branch and (b) the deployed/old build, both against the same
    backing store; diff responses byte-for-byte; include error and not-found
    paths, not just the happy path.
  - **Why it bites**: it exercises the real integration the refactor touches —
    the seam unit tests mock away — which is where lifts actually break.
  - **Precondition / limits**: needs the pre-refactor behavior still runnable
    against the same data (deployed prod, a pinned build, or `git stash`+rerun);
    does NOT apply when the refactor is *meant* to change output (then pin a
    golden instead), and a read-only diff says nothing about write-path
    side-effects — diff those via post-state reads or a transaction-scoped probe.
  - Pair it with repository/unit tests as the structural net; live-diff is the
    integration net, not a replacement.
- P1: cross-link from the `/deliver` and `/qa` skills' refactor guidance so the
  pattern is discoverable at the point of use (or note why a link is not added).
- Verification: `cargo run --locked -p harness-kit-checks -- check --repo .`
  green after the doc edit; the reference renders and the new subsection is
  reachable from its table of contents / heading structure.

## Notes
- Keep it tight — this is one pattern added to an existing reference, not a new
  reference file. The reference already establishes claim/falsifier/driver/
  grader/evidence vocabulary; phrase live-diff in those terms (the deployed build
  is the grader's reference oracle; the diff is the falsifier).
- Related existing references: `verification-system-first.md` (host),
  `quality-system.md` (proof methods), `delete-first.md` (refactor pressure).
- Origin detail for whoever writes it: Habitat HA-034 close-out evidence —
  modules/cycles/sprints list GETs (21/4/10 rows) + a detail GET + a bogus-id
  404, all byte-identical local-branch vs prod against the shared DB.

## Resolution

**2026-07-02 — shipped.** New "Live-Diff For Behavior-Preserving Refactors"
subsection added to `harnesses/shared/references/verification-system-first.md`
(placed after "What Counts", before "Design Rules") covering when/how/why-it-
bites/precondition-and-limits/pair-don't-replace exactly as specified, phrased
in the file's own claim/falsifier/grader vocabulary (deployed build = grader's
reference oracle, the diff = the falsifier), closing with the real Habitat
HA-034 evidence from the ticket's own Notes.

P1 cross-links added to both named skills, not just one: `/deliver`'s
"Refactor at three altitudes" section (the natural point-of-use — refactors
are exactly where this pattern applies) and `/qa`'s Gotchas section (a new
bullet next to the existing "Generic QA is a stopgap" gotcha, since QAing a
coverage-poor refactor is precisely the failure mode both bullets address).
`cargo run --locked -p harness-kit-checks -- check --repo .` green.
