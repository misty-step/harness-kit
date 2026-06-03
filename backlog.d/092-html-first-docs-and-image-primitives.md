# Add one trustworthy workflow figure to the docs companion

Priority: P1
Status: ready
Estimate: M

## Goal

Help a Harness Kit operator understand the ship workflow faster with one
committed, checked, text-backed figure in the generated HTML docs.

## PRD Summary

- User: the technical operator reading the "Ship a repo change with review and
  gates" workflow before using or reviewing Harness Kit.
- Problem: the workflow depends on delegation receipts, evidence, gates, and
  clean-tree closeout. In prose alone, that path is easy to skim past and hard
  to inspect quickly.
- Why now: `087` is shaping broader design-contract discipline. This ticket can
  prove the smaller docs-image primitive first.
- UX enabled: the reader gets one visual map of the workflow, plus the same
  source-of-truth text in HTML and `llms.txt`.
- Deliverable type: working code plus generated docs artifact.
- Success signal: docs build and checker self-test prove the image is declared,
  copied, accessible, text-backed, size-bounded, and removable without any API
  call in CI.

## Product Requirements

- P0: add exactly one figure to the existing "Ship a repo change with review
  and gates" workflow page.
- P0: keep text authoritative. The page must still explain the workflow without
  the image.
- P0: keep builds deterministic and offline. CI validates committed files; it
  never calls an image-generation API.
- P0: enforce alt text, a text-equivalent anchor, source policy, review note,
  file path, and a 250 KiB default size cap.
- P0: fail on missing files, undeclared images, unused declarations, bad paths,
  missing alt/text/provenance, size overflow, stale output, and source/generated
  byte drift.
- P1: record enough provenance for a future operator to regenerate or delete the
  figure without guessing.
- Non-goals: no runtime generation, no new docs framework, no image across the
  catalog, no `/design` scaffold, no HTML-as-canonical-source doctrine change.

## Technical Design

- Chosen architecture: add one `images` entry to `docs/copy/site.json`, store
  one reviewed source asset under `docs/copy/images/`, copy it to
  `docs/site/assets/images/`, and render a `<figure>` with a nearby
  text-equivalent block.
- Files/systems touched: `docs/copy/site.json`, `docs/copy/images/`,
  `scripts/build-docs-site.py`, `scripts/check-docs-site.py`, and generated
  `docs/site/**`.
- Data/control flow: source JSON plus source image -> builder validation and
  byte copy -> generated workflow HTML, `manifest.json`, and `llms.txt` ->
  checker validation and stale-site comparison.
- Build/check boundary: the builder validates shape and copies bytes; the
  checker owns negative fixtures, undeclared/unused detection, metadata
  completeness, byte equality, size/path budgets, and rollback proof.
- ADR decision: not required for this slice because it extends the existing
  static docs companion. Require an ADR if image governance expands across
  many pages, HTML becomes source authority, runtime generation enters the
  build, or `site.json` stops being the docs-copy source.
- Design X vs Y: choose a `site.json` image entry over Markdown-only,
  Mermaid-only, sibling manifest, runtime generation, and a full `087` merger.
  It is the smallest interface that proves the primitive and stays easy to
  delete.

## Deliverable

- Output: one manifest-declared source image, one generated workflow figure,
  one text-equivalent section, and builder/checker support.
- Acceptance oracle: `bash scripts/build-docs-site.sh`,
  `bash scripts/check-docs-site.sh --self-test`,
  `python3 scripts/check-agent-roster.py`, and
  `dagger call check --source=.` pass.
- Evidence artifacts: manifest row, source asset hash, generated page hash,
  generated `manifest.json` and `llms.txt` hashes, negative checker fixtures,
  and the browser-reviewed HTML PRD handoff.
- Residual risk: the checker can prove contract coverage; a human still judges
  whether the figure is visually truthful and worth keeping.

## Constraints / Invariants

- Do not hand-edit generated `docs/site/**`.
- Do not weaken `scripts/check-docs-site.sh`,
  `python3 scripts/check-agent-roster.py`, or `dagger call check --source=.`
- Do not persist a model/API name from memory. Verify current official OpenAI
  image-generation docs before recording generation provenance.
- Keep `087` as source-policy and provenance inspiration only. Do not import
  its broader design-contract scaffolding here.
- If a simple text layout explains the workflow better than the image, keep the
  page text-only and close the ticket with that evidence.

## Authority Order

checker self-tests > generated-site parity > committed manifest/assets > docs
copy source > current official API docs > memory

## Repo Anchors

- `docs/copy/site.json` - docs copy source and selected home for the first
  image declaration.
- `scripts/build-docs-site.py` - deterministic builder; validates, copies, and
  renders the figure.
- `scripts/check-docs-site.py` - docs oracle; owns negative tests and stale-site
  comparison.
- `docs/site/manifest.json` and `docs/site/llms.txt` - generated surfaces that
  must remain useful without image inspection.
- `backlog.d/087-open-design-design-contracts.md` - Open Design-inspired
  provenance discipline, not an implementation dependency.

## Alternatives Considered

| Option | Why It Could Work | Why It Fails Here | Verdict |
|---|---|---|---|
| Text-only HTML | safest and simplest | does not test the image primitive | reject |
| Mermaid | source-reviewable diagram | adds renderer drift instead of asset discipline | defer |
| Sibling image manifest | clean separation | extra parser/check surface for one image | reject |
| `site.json` image entry | one source, one checker path, easy rollback | can crowd `site.json` if overused | choose |
| Runtime generation | fresh assets on demand | nondeterministic, credentialed, CI-hostile | reject |
| Full `087` merger | stronger design system story | too broad for this docs slice | reject |

## Tradeoff Matrix

| Option | Fit | Size | Reversible | Testable | Burden |
|---|---:|---:|---:|---:|---:|
| Text-only HTML | 3 | 5 | 5 | 4 | 5 |
| Mermaid | 3 | 4 | 4 | 3 | 4 |
| Sibling manifest | 4 | 3 | 4 | 4 | 3 |
| `site.json` image entry | 5 | 4 | 5 | 5 | 4 |
| Runtime generation | 2 | 2 | 2 | 2 | 1 |
| Full `087` merger | 3 | 1 | 2 | 3 | 2 |

The selected design is intentionally modest. It tests one real workflow page,
adds no runtime dependency, and can be removed by deleting one manifest entry
and one source asset.

## Agent Readiness

- Profile source: missing; use existing Harness Kit script/checker patterns.
- Stack feedback strength: strong. Python stdlib builder/checker, docs
  self-test, Dagger gate, no network runtime.
- Infrastructure path: local scripts plus committed static assets.
- Mock policy impact: preserved. Test public checker behavior through fixture
  mutations; do not mock builder internals.

## Oracle

- [ ] `docs/copy/site.json` declares the single image with page target, purpose,
      source asset path, alt text, text-equivalent anchor, source policy, review
      note, max size, and regeneration reference.
- [ ] `scripts/build-docs-site.py` copies the committed source asset byte for
      byte and renders the figure plus text-equivalent block.
- [ ] `scripts/check-docs-site.py --self-test` fails for missing source files,
      undeclared `<img>` tags, unused declarations, empty alt text, missing
      text-equivalent anchors, missing source policy/review note, over-budget
      assets, invalid asset paths, stale output, and byte drift.
- [ ] Generated `docs/site/manifest.json` and `docs/site/llms.txt` still make
      the workflow understandable without the image.
- [ ] Removing the image entry and asset, rebuilding, and rerunning the docs
      check returns the page to a valid text-only form.
- [ ] `bash scripts/build-docs-site.sh`,
      `bash scripts/check-docs-site.sh --self-test`,
      `python3 scripts/check-agent-roster.py`, and
      `dagger call check --source=.` pass.

## Acceptance Evidence

- Acceptance source: `docs/copy/site.json`, the committed source image,
  generated workflow HTML, `docs/site/manifest.json`, `docs/site/llms.txt`, and
  docs-site checker self-tests.
- Evidence that proves it: negative checker mutations, stale-site rebuild
  comparison, source/generated hash equality, and Dagger's `check-docs-site`
  lane.
- Artifact hashes: implementation must record the final manifest row, source
  asset, generated page, generated manifest, and generated `llms.txt` hashes.
- Contract-change acknowledgment: this expands docs-site acceptance coverage; it
  does not weaken any existing docs check.

## Formal Spec

- Formal Spec Required: yes.
- Trigger: user-facing generated docs behavior plus acceptance that is best
  proven through manifest/HTML fixtures and negative mutations.
- Acceptance oracle: `bash scripts/check-docs-site.sh --self-test`.
- Waiver path: the operator may reject the image itself. Missing checker
  coverage, missing text-equivalent content, and CI/API calls are not waivable.

## Implementation Sequence

1. Confirm the workflow page and draw the smallest useful delegation/evidence
   flow figure.
2. Add the `site.json` image entry and committed source asset.
3. Teach the builder to validate, copy, render the figure, and render the
   text-equivalent block.
4. Teach the checker the image contract and negative fixtures.
5. Verify current official OpenAI image-generation docs before recording
   provenance.
6. Run the build, docs self-test, roster check, and Dagger gate.

## Observability Plan

- Changed behavior to watch: generated docs include one declared, committed
  figure without losing text authority or deterministic builds.
- Evidence surface: workflow HTML, `docs/site/manifest.json`, `docs/site/llms.txt`,
  and `check-docs-site` failure messages.
- Runtime telemetry: none. This is static documentation.

## Risk + Rollout

- Image becomes false authority: require text-equivalent content and human
  review before commit.
- Binary churn: cap at one image and 250 KiB.
- Accessibility regression: enforce alt text and local text equivalent.
- API drift: record current API source only as provenance; never make CI depend
  on model availability.
- Rollback: delete the image entry and source asset, rebuild docs, rerun the
  checker.

## Delegation Evidence

- `codex`, receipt `66de956b-fa78-4e13-b508-1705109a64ba`, product clarity
  critic. Accepted: make the first screen about one decision, merge repeated
  goal/decision/acceptance prose, and keep the hard constraints.
- `agy`, receipt `eb324d00-3dbb-4d8d-a4e5-f5acb961cc13`, Carmack/Ousterhout
  architecture critic. Accepted: reduce schema theater and keep the module
  boundary small. Rejected: removing provenance entirely, because source policy
  and review notes are the guardrail against generated-image slop.
- `pi`, receipt `c1552361-3c83-4e89-b54b-137c3be823b4`, design critic attempt.
  Rejected as evidence: provider returned an empty transcript.
- Earlier shaping receipts preserved the core constraints: one first-slice
  image, `site.json` as the source, committed bytes only, no CI/API calls,
  text-equivalent content, negative checker fixtures, and rollback proof.

## Residual Risk

The implementation still needs current official OpenAI docs verification before
recording generation provenance. The ticket intentionally does not pin a model
name from memory.
