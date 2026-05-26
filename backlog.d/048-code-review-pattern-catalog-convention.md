# `/code-review` pattern-catalog convention

Priority: medium
Status: pending
Estimate: S

## Goal

Codify a cross-repo convention: every consuming repo maintains a local
`review-patterns.md` (path:
`${REPO}/<skills-dir>/code-review/references/review-patterns.md`)
populated from the recurring findings its external reviewers (CodeRabbit,
Gemini Code Assist, human reviewers) surface on merged PRs. The
`/code-review` skill is updated to require loading this file into the
reviewer subagent's context before any review runs, and to cite catalog
entry IDs on every finding.

## Non-Goals

- Prescribe the catalog's *content* — that is per-repo and evolves.
- Ship a pre-populated catalog for every downstream repo. The
  initial population happens during explicit repo-local vendoring or manually
  after the first few `/reflect cycle` runs.
- Replace language-specific lint rules. The catalog captures
  *semantic* rules (template invariants, test-scaffolding pitfalls,
  contract-hygiene reminders) where static checks don't fit.
- Build an automated ingestion pipeline that scrapes GitHub PR
  comments. Manual curation is the feature — it forces the operator
  to judge whether a one-off find is pattern-worthy.

## Oracle

- [ ] `skills/code-review/SKILL.md` in spellbook documents the
      convention: the critic subagent's prompt template includes a
      line like *"Load `references/review-patterns.md` if present.
      Check each diff hunk against the catalog and cite entry IDs on
      findings."*
- [ ] `skills/code-review/references/review-patterns-template.md`
      exists as a blank scaffold with the entry format, three
      illustrative placeholder entries (one semantic, one
      contract-hygiene, one test-scaffolding), and instructions on how
      a repo seeds its own catalog
- [ ] `skills/code-review/references/review-patterns-template.md`
      documents the lifecycle: "new PR review surfaces a novel
      finding → add entry; eventually the pattern gets codified as a
      lint/check/CI lane → entry gains an `Enforcement:` line but
      remains in the catalog as reference"
- [ ] `/seed` is updated to offer, as part of vendoring `/code-review` into a
      consuming repo, the option to copy `review-patterns-template.md` renamed
      to `review-patterns.md` — and to leave the file out if the repo already
      has one
- [ ] The canary repo's `#029` ("code-review pattern catalog") lands
      as the first consumer implementation; the spellbook convention
      is validated against it
- [ ] `./bin/validate` green in spellbook (docs-only change)

## Notes

**Why now.** Two cycles of `/flywheel` against canary just shipped. In
total, CodeRabbit and Gemini caught ~15 findings across two PRs;
roughly a third were semantic (summary-template drift, bounded-count
vs total-count, for-comprehension `{:ok, _}`-unwrap silently filtering
errors) and could not be captured by a static lint. Canary's response
— item `#029` in its backlog — is a local pattern catalog that the
reviewer subagent consults. The pattern is not canary-specific: every
repo that runs `/code-review` can benefit from a locally-curated list
of "things external reviewers flag repeatedly here," and every repo
that doesn't have one re-discovers the same issues each cycle.

**What spellbook owns vs. what the repo owns.**

| Spellbook provides | Consuming repo provides |
|---|---|
| The template file (`review-patterns-template.md`) | The actual catalog entries, authored from that repo's history |
| The SKILL.md clause that makes consultation mandatory | The per-entry rule / violation / fix / enforcement details |
| The numbering convention (`P-01`, `P-02`, …) | Their own P-IDs, stable across time |
| The lifecycle narrative (seed → enforce → keep as docs) | The judgment on which finds are pattern-worthy |

This split is load-bearing. Spellbook standardizes the shape so an
operator moving between repos recognizes the surface. The *content*
has to be per-repo — what's a pattern in canary (RFC 9457 problem
details; deterministic template summaries; single-writer
`Canary.Repo`) is noise in a different stack.

**Template shape.**

```markdown
# <Repo name> review patterns

Living catalog of recurring review findings. Loaded by `/code-review`
before every review. Entries earn an `Enforcement:` line when a lint
rule / hook / CI lane catches the pattern structurally; they stay in
the catalog as reference documentation even after enforcement lands.

## Seeding

- Copy surface finds from the last 5-10 PRs' external-reviewer
  comments.
- Group by root cause before numbering.
- Prefer entries with a concrete violating example from real
  history over synthetic snippets.

## Entry format

### P-NN — <title>

**Rule.** <one sentence, imperative>

**Violating example** (from <PR URL / file:line>):
\`\`\`<language>
# bad
\`\`\`

**Fix.**
\`\`\`<language>
# good
\`\`\`

**Why it matters.** <2-3 sentences>

**Enforcement.** <lint rule module / hook / CI lane / review checklist only>

### P-01 — <first real entry>

…
```

**SKILL.md clause (draft).**

Under `skills/code-review/SKILL.md`, add (or expand) a `Context
loading` section:

```markdown
## Context loading

Before reviewing any diff, the critic subagent MUST:

1. Attempt to read `references/review-patterns.md` from the current
   repo's `/code-review` skill directory. If present, load into
   context and treat each entry as a rule applicable to diffs in its
   domain.
2. Attempt to read `CLAUDE.md` (or equivalent repo conventions doc)
   from the repo root for any `## Footguns` / `## Invariants`
   sections.
3. On every finding the reviewer emits, cite which catalog entry
   (`P-NN`) or CLAUDE.md section the rule comes from. If none apply,
   say so explicitly — absence of a catalog hit is itself a
   reviewable judgment.

If `review-patterns.md` is missing, emit one warning in the review
output (`"no review-patterns.md present — consider seeding one per
/reflect prevent-patterns"`) and proceed.
```

**Execution sketch (one PR, three commits).**

*Commit 1 — `docs(code-review): add review-patterns-template.md`.*
New file at
`skills/code-review/references/review-patterns-template.md` with the
scaffold and three illustrative placeholder entries.

*Commit 2 — `refactor(code-review): require catalog consultation in SKILL.md`.*
Add the `Context loading` section above. Update any critic-dispatch
prompt templates embedded in the skill to pass the catalog through.

*Commit 3 — `refactor(seed): offer review-patterns.md when vendoring`.*
Small edit to `skills/seed/SKILL.md` so `/seed` copies the template (renamed)
on first vendored install if absent, and leaves existing catalogs untouched.

**Risk list.**

- *Operators skip the convention.* Mitigated by the SKILL.md clause
  that emits a warning when `review-patterns.md` is missing — visible
  in every review output until seeded.
- *Drift between canary's `#029` shape and the spellbook template.*
  Ship `#029` first, then backport the lessons into the template.
  Canary is the reference consumer; the template should match what
  actually worked there.
- *Consumer repos have no external reviewer.* The catalog still has
  value for human reviewers and for self-review via critic
  subagents — same mechanism, different source of finds.

**Lane.** Cross-harness convention work. Depends on canary's `#029`
landing first to validate the shape.

Source: `/reflect prevent-coderabbit-patterns` session against canary
on 2026-04-21, generalizing the pattern that `#029` captures locally.
