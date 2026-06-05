# /harness-engineering sync

Synchronize external skill exemplars into Harness Kit without making them
first-party source.

## Source Of Truth

`registry.yaml` declares external sources, pins, paths, include/exclude filters,
and alias prefixes. Do not hand-edit `skills/.external/`; it is gitignored
machine state owned by `harness-kit-checks sync-external`.

## Lifecycle

1. `harness-kit-checks sync-external` fetches pinned sources and installs selected
   skills under `skills/.external/<alias>/`.
2. `bootstrap.sh` projects first-party `skills/*` and synced
   `skills/.external/*` into each detected harness as ordinary skill names.
3. `harness-kit-checks lint-external-skills --strict` checks imported skills
   are self-contained enough to expose.

Partial sync with `--only <repo>` is scoped: it may add/update aliases for that
source, but it must not prune unrelated external skills. Full sync owns orphan
cleanup.

## Design Rule

Imported skills are external exemplars/tools. Keep source ownership in the alias
(`vercel-*`, `every-*`) unless a shaped ticket proves the behavior belongs in a
first-party Harness Kit primitive. Bootstrap must fail on global skill-name
collisions instead of relying on projection order.
