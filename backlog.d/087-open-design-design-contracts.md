# Open Design-informed design contracts for /design

Priority: P1
Status: ready
Estimate: M

## Goal

Fold the portable parts of `nexu-io/open-design` into Harness Kit's design
workflow so `/design scaffold` and `/design critique` produce sharper,
artifact-backed design-system evidence without depending on the Open Design app,
daemon, adapter layer, or catalog payload.

## Non-Goals

- Do not vendor Open Design, its desktop/web app, daemon, MCP server, or agent
  adapter pool.
- Do not add an `od:` frontmatter runtime contract to Harness Kit skills.
- Do not create a design marketplace, plugin system, model router, or semantic
  workflow engine.
- Do not bulk-import Open Design's bundled design systems, skills, templates, or
  generated brand prose.
- Do not let token compliance replace rendered artifact critique.
- Do not weaken `/design`'s rule that final design judgment requires a
  screenshot, URL, rendered artifact, or explicit source plus unverified caveat.

## Constraints / Invariants

- Harness Kit stays cross-harness first: filesystem skill prose and references
  are canonical; runtime integrations are optional evidence, not the primitive.
- Product repos own their visual language. Harness Kit may define evidence
  shape, scaffold questions, and evals, but not universal brand tokens.
- Design-system guidance must be generated from live repo evidence and rendered
  artifacts when possible, not from generic catalog names.
- Open Design-derived concepts must be attribution-safe and links-first unless a
  license review proves copying is allowed.
- Scope must fit existing `/design`, `/demo`, `/qa`, `/create-repo-skill`, and
  `/agent-readiness` boundaries.

## Authority Order

rendered artifact evidence > tests/evals > code > docs > external prior art >
lore

## Repo Anchors

- `skills/design/SKILL.md` - design routing, rendered-artifact requirement, and
  completion gate.
- `skills/design/references/scaffold.md` - project-local design skill scaffold
  contract; the first implementation should land here.
- `skills/design/references/design-system.md` - current token/component-system
  judgment and escalation rules.
- `skills/design/references/anti-slop.md` - existing universal craft/anti-slop
  reference that overlaps Open Design's `craft/` axis.
- `skills/design/references/ui-surface-routing.md` - how design evidence
  composes with `/a11y`, `/qa`, `/demo`, and `/code-review`.
- `skills/design/evals/` - focused eval suite for design-skill regressions.
- `skills/create-repo-skill/SKILL.md` - file-writing lane for repo-local skill
  generation and acceptance evidence.

## Prior Art

- `https://github.com/nexu-io/open-design` - local-first, open-source design
  artifact studio with `SKILL.md` skills, `DESIGN.md` design systems, skills,
  plugins, sandboxed previews, and export workflows.
- `/tmp/open-design/docs/skills-protocol.md` - OD-compatible skill metadata,
  optional `od:` fields, 9-section `DESIGN.md`, and `craft/` reference layering.
- `/tmp/open-design/docs/architecture.md` - app/daemon topology, artifact store,
  sandboxed iframe preview, and adapter boundaries.
- `/tmp/open-design/design-systems/README.md` - design-system project shape,
  legacy `DESIGN.md` shape, attribution notes, and manifest validation.
- `/tmp/open-design/docs/spec.md` - product bets and non-goals: OD delegates the
  agent loop to existing CLIs and treats skills/design systems as files.

## Portable Concepts To Keep

- 9-section `DESIGN.md` as a repo-owned brand/design contract.
- Separate universal craft guidance from product-specific brand guidance.
- Require provenance labels for design-system facts: `observed`, `provided`,
  or `inferred`.
- Use a `keep / change / do-not-copy` split when deriving a design contract
  from screenshots, sites, or brand references.
- Treat skill/design-system/plugin as separate axes, but only adopt the first
  two as Harness Kit references for now.
- Preserve artifact output contracts: design claims should point at files,
  screenshots, renders, or summaries that can be inspected later.

## Relationship to 092

`092` may borrow this ticket's provenance and source-policy discipline for
generated documentation images, but it should not wait for or absorb the full
`/design` scaffold work. `DESIGN.md`, `design-contract.md`, and design evals
remain owned here; docs companion image manifest/checker work remains owned by
`092`.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Runtime integration | Install/use Open Design CLI/MCP/app from Harness Kit | Fastest way to access OD features | Adds external runtime, daemon assumptions, and adapter policy to Harness Kit | Reject |
| Bulk catalog import | Vendor OD design systems, skills, and templates | Huge immediate design surface | Payload bloat, stale prose, license/attribution risk, generic brand laundering | Reject |
| `od:` frontmatter support | Extend Harness Kit skills with OD mode/preview/input metadata | Makes design skills more machine-readable | Creates a second runtime contract and pushes Harness Kit toward workflow engine behavior | Reject |
| Reference-only design contract | Add OD-informed `DESIGN.md`/`design-contract.md` scaffold guidance and evals | Fits current skills; low dependency; easy to delete | Could become prose theater if no eval proves sharper critique | Choose |
| Craft reference layering | Split universal craft from repo-specific brand facts | Reduces duplication and keeps product tokens authoritative | More reference files can become context tax | Choose as part of reference slice |
| Design eval fixture only | Add an eval that catches generic design critique regressions | Smallest implementation | Does not improve scaffold output unless paired with guidance | Defer as supporting work |
| Comment-mode preview feedback | Click rendered element, send targeted design edit instruction | High leverage for UI/demos later | Needs browser/preview tooling and capability gating | Defer |
| Separate design companion package | Build a richer opt-in package outside Harness Kit | Keeps core clean while enabling deeper experiments | Boundary confusion and maintenance cost | Park |

## Tradeoff Matrix

| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| Runtime integration | 2 | 1 | 3 | 2 | 2 | 3 | 1 |
| Bulk catalog import | 2 | 1 | 4 | 2 | 2 | 2 | 1 |
| `od:` frontmatter support | 2 | 2 | 5 | 3 | 2 | 4 | 2 |
| Reference-only design contract | 5 | 4 | 5 | 5 | 5 | 4 | 5 |
| Craft reference layering | 4 | 4 | 5 | 4 | 5 | 4 | 4 |
| Design eval fixture only | 3 | 5 | 5 | 5 | 5 | 4 | 5 |
| Comment-mode preview feedback | 4 | 2 | 4 | 3 | 3 | 3 | 2 |
| Separate design companion package | 3 | 2 | 4 | 3 | 4 | 3 | 2 |

Reference-only design contracts score best because they improve the existing
`/design` surface without adding a daemon, app shell, catalog payload, or new
frontmatter authority. Craft layering is useful when it stays small and linked
from existing references. Comment-mode feedback is promising but should wait
until a rendered-preview workflow proves it has enough repeated demand.

## Delegation Evidence

- Roster providers used:
  - `claude` as Harness Kit repo-fit investigator, receipt
    `a36ab378-a4b8-4bd6-a92b-7e1a7756c1e2`, transcript
    `.harness-kit/traces/provider-lanes/20260603T153924.582566Z-claude-05e17e10.txt`.
  - `pi` as Open Design prior-art mapper, receipt
    `66d61be2-6942-47e7-9bfa-35c15834456b`, transcript
    `.harness-kit/traces/provider-lanes/20260603T153925.976969Z-pi-4e61d7ea.txt`.
  - `codex` as product/architecture premise critic, receipt
    `60360ef7-09d3-4467-8fa8-7fe1198b64d3`, transcript
    `.harness-kit/traces/provider-lanes/20260603T153924.714200Z-codex-bbf160f4.txt`.
- Accepted evidence: all lanes converged on keeping the first slice inside
  `/design` references and evals, adopting `DESIGN.md`/craft/provenance ideas
  while rejecting OD runtime integration.
- Rejected evidence: the Pi lane's "adopt plugin capability gating" is deferred
  because Harness Kit already has roster/provider boundaries and this ticket is
  about design quality, not plugin governance.
- Waivers: native philosophy critic roles were not used because prior native
  role dispatch failed on this account; the roster-backed Codex critic supplied
  the architecture/premise critique. External research was limited to the
  GitHub repo and its shallow clone, plus the live Harness Kit tree.

## Oracle (Definition of Done)

- [ ] `skills/design/references/scaffold.md` teaches `/design scaffold` to emit
      a repo-owned `DESIGN.md` using a 9-section design-contract convention only
      when recurring UI work earns it.
- [ ] The scaffold reference defines a separate `design-contract.md` evidence
      shape with columns for source, fact, provenance (`observed` / `provided` /
      `inferred`), confidence, and `keep` / `change` / `do-not-copy`.
- [ ] `skills/design/references/design-system.md` or a small sibling reference
      documents the distinction between universal craft rules and repo-specific
      brand/design facts, linking to `anti-slop.md` instead of duplicating it.
- [ ] `skills/design/SKILL.md` links the new reference from the scaffold route
      without adding a new trigger or broadening `/design` into a generator app.
- [ ] A design eval fixture under `skills/design/evals/cases/` fails if a
      scaffold output invents design facts without provenance or omits
      `do-not-copy` guidance for reference-brand material.
- [ ] A second eval fixture fails if a critique claims design success from a
      token/design-system document without naming rendered artifact evidence or
      an explicit unverified caveat.
- [ ] The implementation contains no copied Open Design catalog payload and no
      `od:` frontmatter parser.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Implementation Sequence

1. Update `skills/design/references/scaffold.md` with the `DESIGN.md` plus
   `design-contract.md` output shape, provenance labels, and
   `keep/change/do-not-copy` derivation rule.
2. Add or update the smallest design-system reference needed to explain
   universal craft versus repo-specific brand facts, linking existing
   `anti-slop.md`.
3. Add focused eval cases for unsupported invented design facts and
   no-rendered-artifact design-success claims.
4. Link the reference from `skills/design/SKILL.md` scaffold routing only if the
   reference would otherwise be undiscoverable.
5. Run `python3 scripts/check-agent-roster.py`, `bash scripts/check-docs-site.sh
   --self-test` if generated docs move, and `dagger call check --source=.`.

## Risk + Rollout

- Stale design prose becomes context tax: keep the slice small, reference-based,
  and eval-backed.
- Agents launder reference-brand material into product UI: require provenance
  and `do-not-copy` guidance.
- Design-token abstraction replaces rendered critique: keep `/design`'s
  rendered-artifact requirement explicit and test it.
- License/attribution drift from Open Design catalog content: link to upstream
  prior art; do not copy catalog payload.
- Rollback: remove the reference edits and eval fixtures; no runtime state,
  migration, or dependency should remain.
