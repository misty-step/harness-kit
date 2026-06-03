# <Repo name> Review Patterns

Living catalog of recurring review findings for this repository. `/code-review`
loads `references/review-patterns.md` before every review, checks each diff
hunk against applicable entries, and cites entry IDs (`P-NN`) on findings.

This file is a template. When tailoring a repo, copy it to
`references/review-patterns.md`, replace placeholders with real local history,
and keep the IDs stable once reviewers have cited them.

## Seeding

- Start from the last 5-10 merged PRs and external-reviewer comments
  (CodeRabbit, Gemini Code Assist, human review, incident retros).
- Group by root cause before numbering. Do not add one entry per comment.
- Prefer entries with a concrete violating example from real repo history.
- Leave this file out if the repo has no recurring review patterns yet; the
  `/code-review` warning is enough until the first real pattern appears.

## Lifecycle

New PR review surfaces a novel recurring finding -> add or update an entry.
When the pattern later becomes structural, add an `Enforcement:` line naming
the lint rule, hook, CI lane, test, or checklist that catches it. Keep the
entry in the catalog after enforcement lands so reviewers understand the rule's
origin and can cite the stable ID.

Manual curation is intentional. Do not scrape every PR comment into this file;
operators decide which findings are pattern-worthy.

## Entry Format

~~~markdown
### P-NN - <title>

**Rule.** <one sentence, imperative.>

**Violating example** (from <PR URL / file:line / incident ref>):
```<language>
# bad
```

**Fix.**
```<language>
# good
```

**Why it matters.** <2-3 sentences on consequence and review signal.>

**Enforcement.** <lint rule / hook / CI lane / test / review checklist only>
~~~

## Example Entries

These examples show the intended specificity. They use `EX-NN` IDs so they
cannot be cited as active `P-NN` repo rules by accident. Replace them with real
repo-specific `P-NN` entries before treating the catalog as active.

### EX-01 - Preserve domain invariants in generated summaries

**Rule.** Every generated or formatted summary must preserve the domain
invariant that makes the number meaningful.

**Violating example** (placeholder):
```text
Showing 10 of 50
```

**Fix.**
```text
Showing 10 incidents of 50 total incidents
```

**Why it matters.** Bounded counts, totals, and derived labels are common places
for semantic drift. Static lint rarely knows whether a noun or total came from
the right aggregate, so reviewers need a local pattern when this has happened
before.

**Enforcement.** Review checklist only until a repo-specific test or lint lane
can assert the invariant.

### EX-02 - Keep contract examples aligned with public schemas

**Rule.** Public docs, fixtures, and examples must use the same types and field
shapes as the exported contract.

**Violating example** (placeholder):
```json
{"id": 123}
```

**Fix.**
```json
{"id": "abc123"}
```

**Why it matters.** Contract-hygiene drift trains clients and agents on the
wrong interface. When the repo has generated schemas or OpenAPI output, review
examples against that source of truth.

**Enforcement.** Add the schema parity check, generated-doc test, or contract
lint lane when available; otherwise cite this entry in review.

### EX-03 - Tests must fail loudly on setup errors

**Rule.** Test scaffolding must not silently filter or skip failed setup
results.

**Violating example** (placeholder):
```elixir
for {:ok, item} <- responses do
  assert item.ready?
end
```

**Fix.**
```elixir
for response <- responses do
  assert {:ok, item} = response
  assert item.ready?
end
```

**Why it matters.** Pattern-matching generators, broad catches, and helper
filters can make tests pass while the system under test returns errors.
Reviewers should check whether the test proves the failure path is impossible
or merely ignores it.

**Enforcement.** Review checklist only unless the repo has a targeted test
style lint.
