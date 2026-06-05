# External skill management — sync, pin, install

Priority: high
Status: proposed
Estimate: M (~2-3 dev-days)

## Goal

External skills (`anthropics/skills`, `vercel-labs/agent-skills`,
`garrytan/gstack`, etc.) are declared in `registry.yaml`, pulled by a
single idempotent command, installed into `skills/.external/<alias>/`,
and discovered by `bootstrap.sh` and `generate-index.sh` automatically.
No manual copies. No drift. Symlink-mode users see externals identically
to first-party skills; remote-mode users get them downloaded alongside
first-party.

"Good" = `harness-kit-checks sync-external` is the ONLY path a new external
lands in the tree, and `git status` after running it is reproducible
from the registry alone.

## Why Now

`registry.yaml` has existed since early Harness Kit but only feeds
embeddings. External skills (e.g. `agent-browser`) get manually copied
into `skills/`, drift from upstream, and pollute `index.yaml`. The first
incident (agent-browser → index drift breaking CI on `feat/iterate-mvp-phase1`)
is the forcing function.

## Source Inventory (research 2026-04-15)

Adopt now:
- **`anthropics/skills`** — 17 skills. `frontend-design` already loaded
  globally → dedupe. Curate via `include:` per workflow needs.
- **`vercel-labs/agent-skills`** — 6 focused skills (`react-best-practices`,
  `web-design-guidelines`, `react-native-guidelines`,
  `react-view-transitions`, `composition-patterns`,
  `vercel-deploy-claimable`). **Open question:** `agent-browser` and
  `dogfood` were named by the operator but don't appear in this repo —
  confirm source before adding. `agent-browser` is likely an
  `npx`-installable package with a hand-authored SKILL.md; would land
  as its own registry entry.
- **`openai/skills`** (`.curated/` only) — 42 skills. Skip
  SaaS-credentialed ones (Figma, Linear, Notion, Sentry, Vercel,
  Cloudflare) unless the user actually uses the service.

Defer (curation cost > value):
- **`alirezarezvani/claude-skills`** — 235 skills across 9 domains;
  mixed quality. If adopted, pick a subset via `include:`; do not
  wholesale-ingest.
- **`garrytan/gstack`** — heavy install (Bun, setup script),
  non-standard layout, namespace overlap with existing Harness Kit skills
  (`qa`, `investigate`, `review`, `ship`, `retro`). Cherry-pick
  individual SKILL.md files via `include:` if desired; don't install
  the whole kit.
- **`Leonxlnx/taste-skill`** — 7 frontend variants, conceptually overlaps
  `frontend-design` and `web-design-guidelines`. Reconsider after using
  the others.

Skip (discovery only, not sync targets):
- `VoltAgent/awesome-agent-skills`, `travisvn/awesome-claude-skills` —
  awesome-lists pointing at upstream repos.

## Storage Layout

```
skills/.external/
├── <org>/<repo>/              # raw checkout (gitignored, sparse, shallow)
│   ├── .sync-meta.json        # {commit, fetched_at, registry_sha}
│   └── <subdir>/…             # upstream tree, sparse-checked-out to skills_path
└── <alias>/                   # one symlink per installed skill (committed)
    └── SKILL.md → ../<org>/<repo>/<skills_path>/<name>/SKILL.md
```

Two layers:
- `<org>/<repo>/` is the raw checkout. Bulky; gitignored.
- `skills/.external/<alias>/` is a flat symlink farm. Small; committed.

`bootstrap.sh` and `generate-index.sh` iterate the alias farm only. They
never touch the raw checkouts.

**Commit policy:** gitignore the checkouts; commit the symlinks plus
`.sync-meta.json` per source. Rationale: committing live checkouts
bloats history and fights git; committing nothing breaks offline clones
and CI. Symlinks + meta is the thin middle. Fresh clones must run
`harness-kit-checks sync-external` once before symlinks resolve — documented
in README and enforced by CI (`--check` mode).

## Registry Schema Upgrade

```yaml
sources:
  - repo: vercel-labs/agent-skills
    ref: main                      # branch | tag | sha; default "main"
    pin: a1b2c3d                   # resolved sha; written by sync
    layout: flat                   # flat | multi-root
    skills_path: skills            # sparse path under repo
    include: [react-best-practices, composition-patterns]  # allowlist
    exclude: [vercel-deploy-claimable]                     # blocklist
    alias_prefix: ""               # prepended on collision (e.g. "vercel-")
    embeddings: true               # whether harness-kit-checks generate-embeddings includes
```

`pin` is written by sync and committed. `ref` is operator intent. Floating =
`ref: main` and re-run sync; pinned = also commit `pin`. Hybrid by construction.

## Sync Mechanism

`crates/harness-kit-checks/src/external_sync.rs` — Rust + git. No `gh`, no curl-tarball,
no pip deps. `git clone --filter=blob:none --sparse --no-checkout` gives
shallow + partial-tree in one primitive; we already parse YAML minimally
in `harness-kit-checks generate-embeddings` (reuse).

Per source:
1. `git clone --filter=blob:none --sparse --no-checkout` into
   `skills/.external/<org>/<repo>/` if missing; `git fetch` otherwise.
2. `git sparse-checkout set <skills_path>`.
3. Checkout `pin` if set; else resolve `ref` → sha, write back to
   `registry.yaml` as `pin:`, then checkout.
4. Rebuild alias farm: remove stale symlinks under
   `skills/.external/<alias>/`, create one per included skill.
5. Write `.sync-meta.json` with commit + timestamp + registry content hash.

## Update Cadence

Manual only. `harness-kit-checks sync-external` is the one entrypoint. No CI
cron, no pre-pull hook. Rationale: floating-ref automation silently
drifts the harness; a manual command keeps updates as explicit commits
(operator reviews the `pin:` diff). Weekly `/groom`-time habit suffices;
add cron later only if signal warrants.

## Conflict / Collision Policy

Resolved at alias-farm build time, in registry-declaration order:
- **First-party always wins.** `bootstrap.sh` links `skills/<foo>/`
  before walking `skills/.external/*`; any external `foo` is silently
  shadowed (warning printed once at sync).
- **External vs external:** second declaration MUST set `alias_prefix:`
  or sync exits non-zero with both conflicting paths named. No silent
  last-writer-wins.
- **Upstream rename:** `.sync-meta.json` records each installed skill's
  source path; on next sync, a missing source path aborts with
  "upstream removed X — update `include:` or remove source."

## Index & Embeddings Integration

- `generate-index.sh` walks BOTH `skills/*/` and `skills/.external/<alias>/`,
  dereferencing symlinks. Single `skills:` section; per-entry `source:
  external|first-party` field.
- `harness-kit-checks generate-embeddings` switches from GitHub-API fetch to local
  alias-farm read (faster, offline-capable, removes rate-limit path).
  `embeddings: true|false` per source controls inclusion.

## Bootstrap Interaction

Separate concerns. Sync runs in the checkout; bootstrap runs on the
user's machine.
- LOCAL bootstrap: link `skills/.external/<alias>/*` into
  `~/.claude/skills/` with the same per-dir symlink logic used for
  `skills/*`.
- REMOTE bootstrap (`curl | bash`): externals ride along for free
  because the alias farm is committed; downloaded via the same GitHub
  contents API.
- Fresh clone: README documents `harness-kit-checks sync-external` as a
  one-time setup before `./bootstrap.sh`.

## Removal / GC

Sync computes `declared_aliases` from registry, `installed_aliases`
from filesystem; the diff is removed (symlink farm + checkout + meta).
Single source of truth: registry. No orphans.

## Harness Enforcement

Structural prevention beats prose:

- [ ] `.gitignore`: `skills/.external/*/` (checkouts), with
      `!skills/.external/*/.sync-meta.json` and
      `!skills/.external/<alias>/` (symlink farm tracked)
- [ ] Pre-commit hook rejects new files under
      `skills/.external/<org>/<repo>/` that aren't `.sync-meta.json`
      (no manual copies sneaking in)
- [ ] `scripts/check-vendored-copies.sh` extended: any
      `skills/.external/<alias>/` entry not backed by a symlink into a
      checked-out source = drift, fail
- [ ] CI gate: `harness-kit-checks sync-external --check` exits non-zero if
      running sync would produce a diff (catches registry edits without
      resync — exactly the failure mode that triggered this ticket)
- [ ] `generate-index.sh` skips `skills/.external/<org>/<repo>/`
      checkouts (already does — globs `skills/*/` only) AND walks
      `skills/.external/*/` for the alias farm

## Phase Plan

1. **Schema + script skeleton.** Extend `registry.yaml` with
   `ref`/`pin`/`include`/`exclude`/`alias_prefix`/`embeddings`.
   Write `harness-kit-checks sync-external` handling one source end-to-end.
   Dogfood on the parked `agent-browser` (replace the hand-made
   directory with a proper sourced symlink — confirm source first).
2. **Multi-source + collision.** Loop over all registry sources. Alias
   conflict detection.
3. **Index + embeddings wiring.** Update `generate-index.sh` to walk
   alias farm. Update `harness-kit-checks generate-embeddings` to read local alias farm
   instead of remote API.
4. **Bootstrap pickup.** Teach `bootstrap.sh` LOCAL and REMOTE discovery
   to include `skills/.external/*/` alongside `skills/*`. First-party
   precedence on collision.
5. **Enforcement.** `.gitignore` entries, pre-commit hook,
   `--check` mode, CI wiring.
6. **Adopt the recommended sources.** Add `anthropics/skills`,
   `vercel-labs/agent-skills`, `openai/skills:.curated`. Curate
   `include:` lists.

Commits 1-2 are standalone-useful; 3-5 complete the integration; 6 is
the proof-of-utility.

## Oracle

- [ ] `rm -rf skills/.external/*/` then `harness-kit-checks sync-external`
      restores a bit-identical tree (modulo `.sync-meta.json`
      timestamps); `git diff` is empty
- [ ] A registry entry with `ref: main` and no `pin:` → after sync,
      `pin:` is written with a resolved sha, ready to commit
- [ ] Removing a source from registry → next sync removes both checkout
      and alias symlinks, leaves no orphans
- [ ] Two sources both exposing `agent-browser` without `alias_prefix`
      on the second → sync exits non-zero with both paths named
- [ ] First-party `skills/foo/` + external `skills/.external/foo/` →
      `bootstrap.sh` links only the first-party; warning printed
- [ ] `harness-kit-checks sync-external --check` on a clean tree → exit 0;
      after manual registry edit without sync → exit non-zero
- [ ] `generate-index.sh` lists externals with `source: external`
- [ ] `harness-kit-checks generate-embeddings --offline` reads from the alias farm
      without GitHub API calls
- [ ] Worktree-B running sync concurrently with worktree-A does not
      corrupt either — checkouts are worktree-local via
      `git rev-parse --show-toplevel`

## Non-Goals

- Automated PR-opening against upstream repos on local fixes
- Semantic version resolution, dependency graphs between skills,
  cross-source skill composition
- Mirroring upstream CI or skill tests
- Transitive sources (an external source declaring its own
  `registry.yaml`)
- Private GitHub auth flows beyond the `GITHUB_TOKEN` env var
  `harness-kit-checks generate-embeddings` already honors
- Per-skill config overlays (patch files on top of upstream)
- Any daemon, watcher, or cron — sync is manual-only in MVP
- Wholesale ingestion of mega-collections (`alirezarezvani`,
  `gstack`); curated allowlists only

## Related

- Depends on: nothing (works against current `registry.yaml` +
  `bootstrap.sh`)
- Touches: `registry.yaml`, `bootstrap.sh`, `harness-kit-checks generate-index`,
  `harness-kit-checks generate-embeddings`, `harness-kit-checks check-vendored-copies`,
  `.gitignore`, `.githooks/`
- Forcing function: index-drift CI failure on
  `feat/iterate-mvp-phase1` caused by manually-copied
  `skills/agent-browser/` (now parked at `skills/.external/agent-browser/`,
  to be replaced by a proper sourced install in Phase 1)
- Unblocks: treating `anthropics/skills`, `vercel-labs/agent-skills`,
  `openai/skills` as harness inputs instead of inspiration
- Siblings: 026 (multi-machine sync) shares the bootstrap-remote-mode path
