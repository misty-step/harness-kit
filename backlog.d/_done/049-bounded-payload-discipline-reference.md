# Bounded-payload discipline reference (shared skill note)

Priority: medium
Status: pending
Estimate: S

## Goal

Ship a cross-repo reference at
`skills/code-review/references/bounded-payload-discipline.md` that
documents the generic antipattern — "declare a cap in the response
shape, enforce it in memory after the unbounded load" — with
language-agnostic rules, concrete examples from the Elixir/Ecto and
TypeScript/Prisma ecosystems, and pointers back to per-repo lint or
CI enforcement. Loads automatically via `#048`'s catalog-consultation
convention whenever a reviewer sees pagination/caps/bounded-list
shapes in a diff.

## Non-Goals

- Ship a lint rule. Per-language implementations live in consuming
  repos (canary: `#027` `Canary.Checks.PreloadThenTake`).
- Enumerate every ORM. Cover the two highest-frequency stacks in the
  operator's portfolio (Ecto, Prisma/Drizzle); leave room for
  additions.
- Mandate a single remediation shape. Two fixes are both valid
  (push cap into the ORM query; or separate COUNT + LIMIT calls). The
  reference explains when each is preferable.

## Oracle

- [ ] `skills/code-review/references/bounded-payload-discipline.md`
      exists with: (a) the generic rule in one sentence; (b) a
      decision tree — "when does the cap belong in SQL vs. memory?";
      (c) at least two language-specific examples (Elixir+Ecto with
      `preload` vs SQL `LIMIT`; TypeScript+Prisma with `take` inside
      `findMany` vs slicing an array); (d) pointers to the two
      remediation shapes (in-query limit vs. separate count+fetch)
      with a rule of thumb for when each is correct; (e) a
      telemetry-attach test pattern for asserting the query count is
      constant in N
- [ ] The reference is linked from
      `skills/code-review/references/review-patterns-template.md`
      (shipped via `#048`) as a canonical example of a
      cross-repo-useful pattern reference, so consuming repos know
      they can link out to shared docs rather than copy-pasting
- [ ] Canary's local `#029` catalog entry `P-07` (preload-then-take)
      cross-references this harness-kit doc as its "why it matters"
      expansion
- [ ] `./bin/validate` green (docs-only)

## Notes

**Why now.** CodeRabbit caught two instances of this pattern in canary
over two cycles — the incident detail endpoint preloaded every
`IncidentSignal` row before truncating to 25, and the earlier
`errors_by_class` response computed `total_errors` from a truncated
groups list. The *specific* fixes are Ecto-specific. The *discipline*
is not: any paginated/capped API on any ORM has the same risk shape.
Writing the generic form once in harness-kit means every consuming repo
inherits the review-time rule without re-deriving it.

**Reference shape (draft outline).**

```markdown
# Bounded-payload discipline

Any API response that advertises a cap (top N signals, newest M
annotations, "truncated" flag) must enforce the cap at the data
layer, not in memory after an unbounded read. Violating this silently
degrades under scale and understates totals in summary fields.

## The antipattern

\`\`\`
fetch everything → truncate in memory → return "top N"
\`\`\`

Memory grows linearly with the un-capped count. Tail latency degrades
with N. The "total" fields in summaries end up reflecting the capped
view, not reality.

## The discipline

Two valid shapes, both push the cap into the data layer:

**Shape A — cap inside the query.**

- Elixir + Ecto: `preload: [items: ^from(i in Item, limit: ^max)]`
  or a separate `from(… limit: ^max)` query.
- TypeScript + Prisma: `include: { items: { take: max, orderBy: … } }`
  or a standalone `findMany({ where, take: max })`.

Right when the caller only needs the top-N and does not compute
totals or `truncated` flags from the same fetch.

**Shape B — separate count + bounded fetch.**

- Elixir + Ecto: `count_items(parent_id)` returning an integer +
  `fetch_top_items(parent_id, max)` returning the bounded list.
- TypeScript + Prisma: `prisma.item.count({ where })` +
  `prisma.item.findMany({ where, take: max, orderBy: … })`.

Right when the response payload needs both a bounded list AND a
true-total field (`total_count`, `signals_truncated: boolean`, etc.).
Two queries, both cheap; total memory independent of N.

## The assertion

Bounded-payload read models must have a telemetry-attached test that
asserts the physical query count is constant in the number of rows.
Scale the fixture (N=5, N=50, N=500) and assert the query count does
not grow.

Canary reference implementation:
\`lib/canary/query/incidents.ex:fetch_top_signals/3\` + the
`[:canary, :repo, :query]` telemetry assertion in
`test/canary_web/controllers/incident_controller_test.exs:stays within a small query budget`.

## Enforcement

- **Static (Credo / ESLint / etc.)** — per-repo. Canary's
  `Canary.Checks.PreloadThenTake` (see canary `#027`) flags the
  Ecto shape at lint time.
- **Review checklist** — `P-07` (or equivalent) in the repo's local
  `review-patterns.md`, backed by this shared reference.
- **Runtime** — telemetry-attach test in the read model's test file.

## Further reading

- Canary memory note:
  `~/.claude/projects/.../memory/feedback_bounded_payloads.md`
- Canary canonical implementation:
  `lib/canary/query/incidents.ex`
- Original finding: CodeRabbit comment on canary PR #133
```

**Execution sketch (one PR, one commit).**

*Commit 1 — `docs(code-review): add bounded-payload discipline reference`.*
One file, the draft above fleshed out. Cross-link from
`review-patterns-template.md` (ships via `#048`). Cross-reference in
canary's `#029` follow-up.

**Risk list.**

- *Reference becomes stale as ORMs evolve.* Link to canonical upstream
  docs (Ecto `preload`, Prisma `include: { take }`) rather than
  reproducing API shape details inline. Update on the next `/deps`
  upgrade cycle that touches either ORM.
- *Consuming repos skip it.* Mitigated by `#048`'s SKILL.md clause
  requiring reviewers to consult local + cross-referenced pattern
  docs; this one shows up as a linked "why it matters" expansion.

**Lane.** Cross-harness convention. Depends only on `#048` shipping
first (so the catalog template can link to this reference).

Source: `/reflect prevent-coderabbit-patterns` session against canary
on 2026-04-21; generalizing the bounded-payload rule that ``#027``
enforces locally.
