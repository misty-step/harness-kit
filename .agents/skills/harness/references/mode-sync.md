# /harness sync (in spellbook)

Pull external skill sources into `skills/.external/<alias>/` via
`registry.yaml`. This is repo-internal sync, not per-repo install.
(Per-repo install is `/tailor` and `/seed`, not `/harness sync`.)

## The commands

```
./scripts/sync-external.sh                      # sync all declared sources
./scripts/sync-external.sh --check              # drift check — fails if sync would change tree
./scripts/sync-external.sh --only <org>/<repo>  # sync one source
./scripts/sync-external.sh --allow-floating     # permit branch/HEAD refs
./scripts/lint-external-skills.sh               # advisory: hardcoded paths, tree escapes
./scripts/lint-external-skills.sh --strict      # gate-mode
```

`skills/.external/` is gitignored. Reproducibility lives in
`registry.yaml` — the pinned `sha` (or `pin:` field) is the contract.

## Registry editing

`registry.yaml` is the source of truth for external skill sources.
Load-bearing schema fields:

- `repo: <org>/<name>` — required. GitHub path.
- `default: true` — marks the local spellbook checkout; sync skips.
- `ref: <branch|tag|sha>` — intent. `main`/`HEAD`/branch is floating.
- `pin: <full-sha>` — preferred. Immutable. Sync uses `pin` when set,
  else `ref`.
- `skills_path: <dir>` — sparse-checkout path (default `.`). When
  upstream scatters skills (`skills/` + `skill-data/`), declare
  multiple entries for the same `repo`.
- `include:`/`exclude:` — allow/deny specific skill names.
- `alias_prefix: "tag-"` — **required** for every non-default source.
  Short tag (`anthropic-`, `vercel-`). Installed skill's
  invocation becomes `/<prefix><name>`.
- `allow_floating: true` — opt-in to a non-pinned `ref`. Default is
  refuse.
- `active: false` — declared for embeddings only; sync skips.
- `embeddings: false` — exclude from the embedding index.

## Alias-prefix doctrine (load-bearing)

Same prefix + same skill name = fatal collision at sync. Two sources
may share a prefix only if their skill-name sets are provably
disjoint — e.g. `vercel-labs/agent-skills` and
`vercel-labs/agent-browser` both carry `alias_prefix: "vercel-"`
because their included skill names never overlap.

Silent collisions are load-bearing bugs. `sync-external.sh` dies:

> alias collision: '<name>' declared by both '<repo-a>' and '<repo-b>'
> — set alias_prefix on the later source

When adding a new source, pick a unique prefix and never rely on
`include:` alone to dodge the collision — upstream may add a skill
tomorrow.

## Add a new source (concrete sequence)

1. Edit `registry.yaml`. Add:
   ```
   - repo: <org>/<repo>
     ref: main
     alias_prefix: "<tag>-"
     skills_path: <dir>      # default "."
     include: [<names>]      # curate aggressively
     allow_floating: true    # temporary — see step 4
   ```
2. Run `./scripts/sync-external.sh --only <org>/<repo>`. Verify the
   skills land at `skills/.external/<tag>-<name>/`.
3. Run `./scripts/lint-external-skills.sh --strict`. If the upstream
   skill hardcodes `~/.claude/…` or `/Users/…` or tree-escapes, either
   `exclude:` that skill or document the containment caveat in
   `registry.yaml`.
4. After first successful sync, read the installed
   `skills/.external/<alias>/.sync-meta.json`, copy the `sha` to
   `pin:` in `registry.yaml`, and remove `allow_floating: true`.
   Rerun `sync-external.sh --check` to confirm the pin is stable.
5. `/harness eval` the new skills against representative prompts.
   External skills pay the same description tax as first-party.

## When NOT to use the registry

- First-party skills are filesystem-discovered; no registry entry.
- Per-repo installs are handled by `/tailor` and `/seed`, not by
  syncing externals. The registry populates `skills/.external/`;
  `/tailor` chooses what to bring into a consumer repo's
  `.agents/skills/`.

## External-skill design axes (classification before adoption)

Before pulling an external source in, classify it:

- **Portable** — self-contained, resolves via `$SCRIPT_DIR`, no
  global installs. Copy as-is. Lint clean.
- **Toolkit** — depends on a native CLI or daemon. Reject the source unless
  the native install path is part of the contract, or reimplement the
  doctrine-critical pattern as a first-party skill.
- **Runtime-dependent** — relies on a harness-specific feature.
  Reject unless you can replace with filesystem-layer equivalents.

## Gotchas

- `allow_floating: true` is a TODO, not a shipping state. Every
  entry with it bears a "pin after first sync" comment; resolve
  before the next release.
- `scripts/sync-external.sh` GCs orphans aggressively. Renaming a
  source's alias without cleanup leaves a hole; rerun after any
  edit.
- `skills/.external/` is per-machine. CI does not sync it
  (production gates ignore `skills/.external` via the
  `Ignore([...])` arg in `ci/src/spellbook_ci/main.py`). Skills you
  write MUST NOT depend on an external being present at runtime.
