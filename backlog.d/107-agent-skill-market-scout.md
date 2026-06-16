# Context Packet: Agent skill market scout

Priority: P2
Status: ready
Estimate: M

## PRD Summary

- User: Harness Kit maintainers deciding which external skills to import,
  adapt, or reject.
- Problem: star lists and marketplaces are noisy but useful. Today evaluation
  is manual and easy to bias toward popularity, hype, or catalog bloat.
- Goal: add a dry-run scouting report that evaluates popular skill repos
  against Harness Kit's registry, license, layout, trigger, and fit contracts.
- Why now: the hit list includes a top-skills leaderboard and explicitly asks
  to consider every top skill without blindly importing them.
- UX enabled: maintainers can run one command and get candidate-specific
  verdicts: recommend, defer, reject, or already covered.
- Deliverable type: Rust-owned report/gate-like helper plus docs.
- Success signal: the report evaluates the hit-list top repos and emits
  backlog-ready recommendations without modifying `registry.yaml`.

## Product Requirements

- P0: Dry-run first; no automatic registry edits.
- P0: Each candidate row must include repo, resolved default branch SHA,
  license, skill layout, aliases that would be installed, duplicate overlap
  with existing `registry.yaml`, and a fit verdict.
- P0: Verdicts must distinguish "popular but not a Harness Kit skill" from
  "good exemplar" and "actionable import".
- P0: The report must be commit-safe: no external secrets, no raw tarballs, no
  uncontrolled clone cache in the repo.
- P1: Accept an input markdown list or URL list so `HIT-LIST.md` and future
  marketplace exports can be evaluated.
- Non-goals: no scraping star rankings as the source of truth, no auto-install,
  no semantic rewrite of third-party skills, no claim that GitHub stars measure
  quality.

## Repo Anchors

- `registry.yaml` - existing imports and collision/alias doctrine.
- `crates/harness-kit-checks/src/external_sync.rs` - registry parsing,
  pinning, sparse path, and include/exclude model.
- `crates/harness-kit-checks/src/skill_invocation_analytics.rs` - existing
  report shape and telemetry patterns.
- `skills/harness-engineering/SKILL.md` - primitive test and measure-before-
  adding guidance.
- `skills/harness-engineering/references/skill-design-principles.md` - fit
  rubric for skills as folders, trigger classifiers, progressive disclosure,
  and no generic prose.
- `registry.yaml` comments for no-license caveats and source ownership.

## External Evidence

- The hit list's leaderboard mixes official skills, skill frameworks, personal
  setups, design skills, job-search workflows, trend scanners, and knowledge
  graph tools. These are not equivalent adoption candidates.
- `VoltAgent/awesome-agent-skills` currently describes itself as a curated
  1400+ skill collection across official teams and community skills and links
  to `officialskills.sh`.
- Harness Kit already imports selected external skills through pinned registry
  entries rather than copying broad catalogs.

## Alternatives

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Import the top-10 list manually | Fast catalog expansion. | Stars are not fit; imports personal setups and role-specific skills; high context tax. | Reject. |
| Ignore marketplaces | No bloat. | Misses useful external primitives and ecosystem lessons. | Reject. |
| Write one-off research report | Useful once. | Does not compound; future lists repeat the same work. | Reject. |
| Build dry-run scout in `harness-kit-checks` | Repeatable, evidence-backed, registry-aware, and safe by default. | Requires modest Rust/report work. | Choose. |
| Use only `skill-cleaner` | Reuses external catalog audit. | It audits installed/loaded skills, not new marketplace candidates. | Compose, not replace. |

## Design

Add a `harness-kit-checks scout-skills` command under the external-sync area. It
accepts a markdown/file list of GitHub URLs and emits Markdown and JSON reports.
For each repo:

1. Resolve metadata through GitHub API or `git ls-remote`.
2. Detect license and default branch SHA.
3. Detect likely skill layouts: `skills/*/SKILL.md`, root `SKILL.md`, plugin
   path conventions, or no compatible layout.
4. Compare candidate skill names and aliases against existing `registry.yaml`.
5. Apply a small rubric from `skill-design-principles.md`: single workflow
   category, trigger classifier, self-contained folder, progressive disclosure,
   no obvious vendor/persona-only mismatch, and license status.
6. Emit one verdict per candidate: `recommend-import`, `defer-exemplar`,
   `already-covered`, `reject-not-skill`, `reject-license`, or
   `needs-human-review`.

The first delivered report should use `HIT-LIST.md` as input and include the
ten listed repos plus Ponytail, while preserving the raw list as premise.

## Oracle

- `cargo test --locked -p harness-kit-checks scout_skills` or the focused test
  target added by delivery covers markdown URL extraction, registry duplicate
  detection, license classification, layout classification, and verdict
  rendering with mocked GitHub responses.
- `cargo run --locked -p harness-kit-checks -- scout-skills --input HIT-LIST.md --format markdown`
  writes `.evidence/skill-scout/hit-list-scout.md` containing one row per top
  skill repo and no registry edits.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
- Report spot check: at least one hit-list repo is classified as
  `recommend-import` or `defer-exemplar`, and at least one is classified as
  `reject-not-skill` or `already-covered`; a report that recommends everything
  fails review.

## Premise Source

Premise Source: sha256:ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663 HIT-LIST.md

External sources checked on 2026-06-16:

- https://github.com/VoltAgent/awesome-agent-skills
- https://generativeprogrammer.com/p/20-agent-skills-repos-and-marketplaces

## HTML Plan

HTML Plan: `.evidence/shape-hit-list/hit-list-shape-index.html#skill-market`

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| Marketplace hype becomes catalog bloat | Dry-run only; verdict rubric must reject/defer, not just recommend. |
| GitHub rate limits or network flakes | Mock tests for core behavior; live scout is an evidence artifact, not CI default. |
| Licensing mistakes | License field required; no-license candidates must be defer/reject unless existing policy covers local-only sync. |
| Scanner becomes semantic workflow engine | Keep it a report generator; registry edits remain human-delivered. |

Rollback: delete the scout command and report docs; existing registry behavior is
unchanged.
