# Bench Map — Static Reviewer Lens Selection

The marshal picks reviewer lenses via a declarative path-glob map in
`bench-map.yaml`. Deterministic, greppable, eval-able. No LLM classifier.

## How It Works

```
changed files  ──►  match globs  ──►  apply `replace`
                                   ──►  union `add` with `default`
                                   ──►  de-dupe
                                   ──►  cap at 5 (critic pinned)
                                   ──►  bench
```

1. **Get changed files:** `git diff --name-only <base>...HEAD`
2. **Start from `default`:** always 3 reviewers, always includes `critic`.
3. **Match rules:** for each rule, if ANY changed file matches ANY glob in
   `paths`, remove optional `replace` reviewers, then union the rule's `add`
   list into the bench.
4. **De-duplicate** — reviewer ids appear at most once.
5. **Cap at 5.** Keep `critic` pinned, then keep the earliest selected
   reviewers until the cap is met.
6. **Bench size stays in [3, 5]** for every diff.

## Fallback (No Rule Matches)

The `default` list is the fallback. If no rule matches, the bench is exactly
`default`. The review still runs — it never errors on unclassified diffs.

## How To Add a Rule

Edit `bench-map.yaml`. Each rule has a `name`, a `paths` list of globs, an
`add` list of reviewer ids, and optional `replace` ids for cases where a
specialty lens should take a general lens's slot.

```yaml
- name: graphql
  paths: ["**/*.graphql", "**/schema.gql"]
  add: [ousterhout, beck]
```

Constraints:

- Reviewer ids in `add` and `replace` MUST be lens names from
  `harnesses/shared/references/lenses.md` or explicit Explore-type agent files
  under `agents/<name>.md`.
- Keep rules specific. Overly broad globs inflate the bench and force the cap
  to drop useful reviewers.
- Prefer 1-2 `add` reviewers per rule. `default` already carries 3.

## Override Mechanics

There is no per-repo override file yet.

Manual overrides for a single review are fair game: the marshal may swap a
reviewer or add an ad-hoc lens if the diff has concerns the map doesn't
capture. Document the swap in the synthesis output so it stays auditable.

## Reviewer IDs Referenced

Lens ids come from `harnesses/shared/references/lenses.md`:

- `critic`, `ousterhout`, `carmack`, `grug`, `beck`, `cooper`, `security`

Explicit Explore-type agent files may also appear when they are not just a
lens rubric:

- `a11y-auditor` (web UI accessibility)

If you want a new philosophy specialty, add a lens first, then reference it
here. Add a static agent file only when tool or permission isolation needs a
separate runtime type.

## Determinism

Same diff + same `bench-map.yaml` → same bench. No randomness, no LLM call
in selection. This is a load-bearing property: it makes `/code-review`
reproducible and lets us write eval fixtures against known bench outputs.
