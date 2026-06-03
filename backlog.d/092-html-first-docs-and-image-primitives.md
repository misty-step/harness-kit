# HTML-first docs companion with sparse generated image primitives

Priority: P1
Status: pending
Estimate: M

## Goal

Explore whether Harness Kit should prefer generated static HTML as the
reader-facing documentation artifact, while keeping source docs agent-readable
and allowing sparse generated images only when they make a workflow, primitive,
or acceptance contract easier to understand.

This is an exploration and implementation ticket, not an immediate shared
doctrine change. The first slice should prove a bounded docs primitive before
teaching agents to prefer it broadly.

## Context

Harness Kit already has a static HTML docs companion generated from repo
sources. The interesting next step is not "HTML instead of Markdown" as an
absolute rule; it is a disciplined source-to-HTML workflow where Markdown,
JSON, or structured manifests remain useful source surfaces, and generated HTML
becomes the polished reader-facing artifact.

Generated images may add value when used sparingly: diagrams of harness
lifecycles, workflow state, acceptance evidence, or provider/delegation routing.
They are also a high-slop risk if they become decoration, unstated marketing
claims, binary churn, or inaccessible stand-ins for actual documentation.

## Constraints / Invariants

- Do not hand-edit generated `docs/site/**` output.
- Keep docs static and deterministic at build/check time.
- Do not add a new frontend framework, hosted docs product, CDN dependency, or
  image-generation service runtime.
- Images are never the canonical explanation of a primitive, workflow, gate, or
  acceptance contract; the HTML text and agent-readable source must still stand
  alone.
- Generated images require purpose, provenance, alt text, deterministic
  filenames, size/count budgets, and a clear regeneration note.
- Generated images are committed static assets after human/operator review. CI
  must check declared assets; it must not call an image-generation API or
  regenerate images.
- Image generation model/API names are time-sensitive. Implementation must
  verify current official OpenAI image-generation docs before pinning model or
  API details.
- No decorative hero art, mood boards, or one-image-per-skill defaults.
- Default budget: at most one generated image per generated docs page, at most
  one generated image in the first implementation slice, and at most 250 KiB
  per generated image unless the ticket explicitly raises the budget.
- Do not weaken `scripts/check-docs-site.sh`,
  `python3 scripts/check-agent-roster.py`, or `dagger call check --source=.`
  to make docs generation pass.

## Oracle

- [ ] A source manifest such as `docs/copy/site.json` or a small sibling docs
      asset manifest can declare every generated image with: purpose, page
      target, prompt, model/API version, generation timestamp, operator,
      relevant generation parameters, provenance note, alt text, text-equivalent
      explanation link, source policy, file path, max size, and regeneration
      command/reference.
- [ ] `scripts/build-docs-site.py` can render static HTML pages that include
      declared image assets without requiring hand edits to generated output.
- [ ] `scripts/check-docs-site.py` fails when a generated image is missing,
      unused, lacks alt text, lacks provenance, exceeds the configured budget,
      or appears in generated HTML without manifest coverage.
- [ ] CI/check scripts never call an image-generation API. They validate the
      committed asset and manifest only.
- [ ] The first implementation slice names one specific target page and image
      type before generation, preferably a delegation lifecycle or completion
      evidence flow diagram where an image clarifies a real repo mechanic.
- [ ] The docs companion still emits agent-readable text surfaces, including
      `llms.txt` or equivalent, so image-backed pages remain understandable to
      humans and agents without image inspection.
- [ ] Existing docs-site checks still cover stale generated output, primitive
      coverage, workflow pages, icon coverage, private-text bans, and source
      links.
- [ ] The ticket includes a no-image path for pages where text, tables, or
      simple HTML structure are clearer than generated imagery.
- [ ] Official OpenAI documentation is checked during implementation before any
      model/API name or prompt template is persisted.
- [ ] A rollback path is documented and tested: removing the image manifest row
      and asset, rebuilding docs, and rerunning the docs check returns the page
      to a text-only form.
- [ ] Negative checker tests prove that missing image files, missing alt text,
      missing provenance, and over-budget assets fail.
- [ ] `bash scripts/build-docs-site.sh`,
      `bash scripts/check-docs-site.sh --self-test`,
      `python3 scripts/check-agent-roster.py`, and
      `dagger call check --source=.` pass.

## Implementation Sequence

1. Audit the current docs companion source/generator/checker surfaces and
   decide whether image declarations belong in `docs/copy/site.json` or a
   sibling manifest.
2. Add the smallest manifest and generator support for one documentation image
   class, preferably a workflow or evidence-lifecycle diagram.
3. Add checker coverage for missing/unused/unattributed/no-alt/over-budget
   image assets and for generated HTML that references undeclared images.
4. Verify the current official OpenAI image-generation API/model surface before
   writing any persisted generation recipe.
5. Generate or stage at most one useful image asset for the first slice, with a
   text-equivalent explanation in the page body.
6. Add the rollback note and negative checker tests.
7. Run the docs checks, roster check, and full Harness Kit gate.

## Delegation Evidence

- `codex` GPT-5.5 low investigator lane:
  `019e8e5c-be4d-7570-ae5a-c0ead6b33cbf`; receipt
  `4a147b32-5636-4cdd-ab60-19f59233d673`. Accepted recommendation: create
  this ticket, keep doctrine unchanged for now, make HTML the reader-facing
  artifact while preserving deterministic source/generator/check surfaces.
- `codex` GPT-5.5 low adversarial critic lane:
  `019e8e5c-be51-7920-b3f3-4f9974e9b72a`; receipt
  `a8ff243f-54f1-4537-9129-7fdf73290f1b`. Accepted risks: image generation can
  create slop, binary churn, accessibility gaps, and stale generated docs
  unless the manifest/checker enforces provenance, alt text, count/size
  budgets, and text-equivalent explanations.
- `claude` low read-only critic lane; receipt
  `d62d6dbe-52e7-4c85-8da9-38cb0dd0db4b`. Accepted revisions: make the image
  budget numeric, define provenance fields, commit generated images instead of
  regenerating in CI, name a first-slice target page/image type, and require a
  rollback path plus negative checker tests.

## Residual Risk

The first implementation will need current official OpenAI docs verification
because "latest image generation model" is intentionally time-sensitive. This
ticket should not pin a model name from memory.
