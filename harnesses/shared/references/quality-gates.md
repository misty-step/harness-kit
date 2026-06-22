# Quality Gates

The repo's STANDING quality floor — the gates that run on every change, forever.
Distinct from `quality-system.md` (the bar for one piece of work) and
`verification-system-first.md` (the proof loop for one change). This is what the
repo enforces automatically so neither humans nor agents have to remember it.

Every gate here obeys the repo's oldest gate rule: **it names the real failure
it catches, or it is deleted.** A gate that cries wolf gets tuned out (Google's
Tricorder held analyzer false-positives under ~5% for exactly this reason); a
gate nobody trusts is worse than no gate.

## Gate the diff, not the codebase

The one principle real orgs converge on: enforce strict standards on
**new/changed code**, let legacy improve by ratchet. This is SonarSource's
"Clean as You Code" (the origin of the term *quality gate*), Google's presubmit,
and Meta's diff-time analysis — all gate the change, not the baseline. It is the
only model that retrofits a high bar onto a brownfield repo without a
boil-the-ocean cleanup. Free to do: `diff-cover` for patch-scoped coverage, a
committed baseline for everything else. Never gate a global average — it is
either unmeetable on legacy or trivially passed.

## Three tiers

Goodhart's law sets the tier: the moment a measurement becomes a target, people
— and agents, harder — optimize the number and hollow out what it measured. So
most metrics are diagnostics, not gates.

| Tier | Use for | Mechanism |
|---|---|---|
| **Hard-block** | Goodhart-resistant, behavioral, diff-scoped invariants | Fail the build: tests green / no-merge-on-red, diff-coverage floor, mutation score on core logic, supply-chain (vuln / license / dep-existence), secret scan over source + git/PR metadata. |
| **Ratchet** | Structural debt with a real but gameable signal | Baseline the current state; block new violations and any regression; let the baseline only shrink. God-files, duplication, dead code, lint-warning count, binary size. |
| **Report** | Diagnostics that guide judgment but get gamed as gates | Emit a reviewable artifact (trend, per-file, hotspots) under `.evidence/`; never block. Cyclomatic/cognitive complexity, maintainability index, coupling, churn × complexity hotspots. |

A gate moves UP a tier by ratchet — land it report-only, tighten to ratchet,
promote to hard-block once the baseline is clean. It never moves down to go
green (red line: do not lower gates).

## Meaningful, not arbitrary

The number must trace to a failure mode, or it is ceremony:

- **Coverage → diff coverage + mutation, not global %.** Line coverage proves a
  line ran, not that a test would catch a bug in it — and once agents write the
  tests, the % is actively corrupt. Gate coverage on *changed* lines; gate
  *mutation score* on core logic (does the suite kill an injected bug?).
- **God-file → multi-signal, LOC as the cheap tripwire.** A god-file is high
  responsibility × churn × fan-in; length is the proxy you can measure for free.
  Ship the LOC ratchet as the first gate; reach for churn × complexity (Tornhill
  hotspots) to rank what to split first.
- **Line/complexity limits → per-function, ratcheted, reported.** Absolute caps
  get gamed by extraction — logic scatters into tiny functions, the gate passes,
  the reader chases call chains. Cap the worst outliers, ratchet the rest, keep
  the metric a report.
- **Duplication → token-level, diff-scoped.** Fire on new clones, not on the
  Rule of Three's second copy.

## Gating agent-authored code

The distinguishing case: when an agent both writes the code and runs the check,
it optimizes the check, not the quality — measured at 47–74% of
self-improving-agent optimizations showing proxy gains without real gains, and
*widening* the longer it iterates. Therefore:

- **Tamper-evident and externally enforced.** A gate the author can weaken,
  delete, or self-attest is theater. Thresholds ratchet (monotonic, committed);
  the verifier is not the producer (fresh context beats self-review).
- **Behavior-anchored, not metric-anchored.** Anchor to live oracles, tests, and
  mutation — what an agent cannot satisfy by gaming a number.
- **Dependency existence before install.** ~1 in 5 LLM-suggested packages do not
  exist and agents auto-install them ("slopsquatting"); resolve deps against the
  registry before any install, not at PR time.
- **Watch the agent tells.** Clones, `as any` / `@ts-ignore`, swallowed errors,
  `todo!()` stubs, dead scaffolding — they compile and pass tests, so review
  tuned for human mistakes misses them. Gate them mechanically.

## The menu (illustrative — compose for the repo, free/OSS only)

Each entry is a *kind* of gate, not a checklist to install wholesale. Pick what
names a real failure in this repo and language. Default to free + open-source,
self-hostable, or a ~20-line homebrew tripwire; never force a paid SaaS on a
consumer.

- **Behavioral:** tests green / no-merge-on-red; executable acceptance specs
  (Gherkin) when business-owned; the suite *is* the gate.
- **Coverage quality:** `diff-cover` (patch coverage from any LCOV/Cobertura);
  `cargo-mutants --in-diff` / Stryker / mutmut (mutation).
- **Structure (ratchet):** homebrew god-file LOC tripwire; jscpd / PMD-CPD
  (duplication); knip / cargo-machete / vulture (dead code, unused deps).
- **Supply chain (hard-block):** cargo-deny / osv-scanner / pip-audit /
  govulncheck (vuln + license + bans); cosign + SLSA provenance; OpenSSF
  Scorecard.
- **Surface:** cargo-semver-checks / cargo-public-api / api-extractor;
  size-limit / cargo-bloat budgets.
- **Architecture (fitness functions):** ArchUnit / dependency-cruiser /
  import-linter — layering and dependency-direction rules as a build failure.
- **Hygiene:** secret scan over source AND commit/PR metadata; forbidden
  markers; warnings-as-errors (`-D warnings`, strict typecheck).
- **Homebrew wins:** the god-file ratchet, an orphan-marker grep, and a
  baseline-ratchet wrapper (run any count-emitting tool, fail on growth) each
  beat adopting a dependency.

Tooling note: prefer `ast-grep` (MIT) over Semgrep (maintained rules relicensed
2024); replace Codecov/Coveralls/SonarCloud (hosted) with `diff-cover` + raw
coverage artifacts or self-hosted SonarQube Community.

## Adoption

- A repo missing a meaningful floor is epic-scoped backlog work for `/groom`;
  start report-only and ratchet up.
- A structural win from `/refactor` (a god-file split, a killed dependency) gets
  ratcheted into a gate so it can't regrow.
- `/ci` audits the floor and strengthens it; the two-tier fast-local / full-CI
  split decides where each gate runs (fast & offline → local; networked or
  expensive → CI/ship).

## Prior art

Clean as You Code (SonarSource); fitness functions (Thoughtworks, *Building
Evolutionary Architectures*); the Tricorder false-positive bar and the Beyoncé
Rule (*Software Engineering at Google*); hotspots = churn × complexity
(Tornhill); characterization tests (Feathers); reward-hacking in self-improving
code agents; slopsquatting (hallucinated-dependency research). Treat
coverage-as-target and single-number metric thresholds as Goodhart-prone.
