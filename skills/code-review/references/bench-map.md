# Bench Map — Static Reviewer-Lens Selection

The marshal picks reviewer lenses via a declarative path-glob map in
`bench-map.yaml`. Deterministic, greppable, eval-able. No LLM classifier.

## How It Works

```
changed files  ──►  match globs  ──►  union `add` lenses with `default`
                                   ──►  de-dupe
                                   ──►  cap at 5 (critic pinned)
                                   ──►  bench
```

1. **Get changed files:** `git diff --name-only <base>...HEAD`
2. **Start from `default`:** always 3 lenses, always includes `critic`.
3. **Match rules:** for each rule, if ANY changed file matches ANY glob in
   `paths`, union the rule's `add` lens labels into the bench.
4. **De-duplicate** — lens labels appear at most once.
5. **Cap at 5.** If over, drop lenses contributed by the rule with the
   fewest file matches. `critic` is never dropped.
6. **Bench size stays in [3, 5]** for every diff.

## Fallback (No Rule Matches)

The `default` list is the fallback. If no rule matches (e.g. a diff touching
only unknown extensions), the bench is exactly `default`. The review still
runs — it never errors on unclassified diffs.

## How To Add a Rule

Edit `bench-map.yaml`. Each rule has a `name`, a `paths` list of globs, and
an `add` list of reviewer-lens labels.

```yaml
- name: graphql
  paths: ["**/*.graphql", "**/schema.gql"]
  add: [ousterhout, beck]
```

Constraints:

- Labels in `add` MUST exist in `internal-bench.md` or be defined as an
  explicit ad-hoc critic in the review synthesis.
- Keep rules specific. Overly broad globs inflate the bench and force the cap
  to drop useful reviewers.
- Prefer 1-2 `add` agents per rule. `default` already carries 3.

## Override Mechanics

There is no per-repo override file yet.

Manual overrides for a single review are fair game: the marshal may swap a
reviewer lens or add an ad-hoc critic if the diff has concerns the map doesn't
capture (e.g. a one-off perf-critical hot loop). Document the swap in the
synthesis output so it stays auditable.

## Reviewer Lenses Referenced

Built-in lens labels:

- `critic`, `ousterhout`, `carmack`, `grug`, `beck`
- `a11y-auditor` (web UI accessibility)

If you want a new specialty (e.g. security, performance), define it as an
ad-hoc critic in the review synthesis before referencing it in a rule.

## Determinism

Same diff + same `bench-map.yaml` → same bench. No randomness, no LLM call
in selection. This is a load-bearing property: it makes `/code-review`
reproducible and lets us write eval fixtures against known bench outputs.
