# `/tailor` installs externally-managed skills (link, don't rewrite)

Priority: high
Status: pending
Estimate: M (1-2 days)

## Goal

Extend `/tailor` so that consuming repos get a **three-mode** skill
install, not just the current two:

| Mode | Source | Action | Who owns the content |
|---|---|---|---|
| Universal | `$HARNESS_KIT/skills/<name>/` | Copy verbatim | Harness Kit |
| **Workflow** | `$HARNESS_KIT/skills/<name>/` | **Rewrite** with repo specifics | Consuming repo (per run) |
| **External (NEW)** | `$HARNESS_KIT/skills/.external/<alias>/` (populated by `harness-kit-checks sync-external`) | **Symlink** to shared global cache | Upstream (unchanged) |

The rewrite rule **must not apply to externals.** An external skill in a
consuming repo is a pointer to upstream, not a fork. `/tailor` re-runs
never mutate external content.

## Why this matters

Today, externals declared in `registry.yaml` (anthropic/skills,
vercel-labs/*, jakubkrehel, emilkowalski, forrestchang/karpathy,
JuliusBrussee/caveman, garrytan/gstack, openai/skills) are:

1. Synced into `$HARNESS_KIT/skills/.external/<alias>/` via `harness-kit-checks sync-external`.
2. Indexed by `harness-kit-checks generate-embeddings` for semantic search.

Some are **not installed onto any harness's skill-discovery path.**
`bootstrap.sh:271` only globally symlinks `GLOBAL_SKILLS=(tailor seed)`
plus agents. `/tailor` currently picks only from the canonical `skills/`
catalog. Net effect: `/jakub-make-interfaces-feel-better`,
`/emil-emil-design-eng`, `/karpathy-*` don't resolve as slash commands
in any repo today. They exist for embeddings-based discovery only.

Note: `garrytan/gstack` is intentionally inactive in `registry.yaml`.
Do not re-enable it as part of this ticket; the Harness Kit-native
distillations are the canonical source for those planning patterns.

This shape closes that gap per-repo — not globally — and preserves the
"externally managed" contract: externals update upstream, not by hand.

## Design

### 1. Install-mode taxonomy

`/tailor`'s pick produces a four-bucket plan:

- **Core (workflow) skills** — `skills/<name>/` (harness-kit canonical).
  Rewritten per-repo. `category: workflow` in `.harness-kit` marker.
- **Universal skills** — `skills/<name>/` (harness-kit canonical, explicit
  list: research, groom, office-hours, ceo-review, reflect).
  Copied verbatim. `category: universal`.
- **External skills** — from `registry.yaml`. Installed as
  symlinks pointing into `$HARNESS_KIT/skills/.external/<alias>/`.
  `category: external`.
- **Agents** — `agents/<name>.md`. Copied. `category: agent`.

### 2. External-install mechanics

In the consuming repo's shared root:

```
.agents/skills/<alias>              → absolute symlink → $HARNESS_KIT/skills/.external/<alias>/
.agents/skills/<alias>.harness-kit    → marker (sibling, since we can't write inside the symlinked upstream)
```

Per-harness bridges stay the relative-symlink pattern that already works:

```
.claude/skills/<alias>              → ../../.agents/skills/<alias>   (relative; resolves via the absolute link)
.codex/skills/<alias>               → ../../.agents/skills/<alias>
.pi/skills/<alias>                  → ../../.agents/skills/<alias>
```

Marker file `<alias>.harness-kit` (sibling, not inside the target) contains:

```yaml
source: <org>/<repo>           # registry.yaml repo
alias: <alias>                 # e.g. jakub-make-interfaces-feel-better
installed: <ISO-8601>
installed-by: tailor
tailor-version: <sha>
category: external
target: $HARNESS_KIT/skills/.external/<alias>/
```

Why sibling-marker: placing `.harness-kit` inside the symlink target would
corrupt the shared global external cache — every consuming repo writes
its own timestamp into the one upstream snapshot. Sibling works because
the consuming repo owns the symlink; harness-kit owns the target.

### 3. Picking externals per-repo

Same picking doctrine applies. A web-app repo (has `package.json` with
React/Next.js/Vue/Svelte signals, or `index.html`, or a frontend bundler
config) auto-picks:

- `vercel-agent-browser` (browser automation)
- `vercel-web-design-guidelines`, `vercel-react-best-practices`,
  `vercel-composition-patterns`, `vercel-react-view-transitions`,
  `vercel-react-native-skills` (as the stack allows)
- `jakub-make-interfaces-feel-better` (design polish)
- `emil-emil-design-eng` (taste flowcharts)
- `anthropic-frontend-design` (bundled with Claude Code globally —
  already excluded in registry.yaml; skip to avoid double-install)

A CLI library / harness library / ML-ops repo with no frontend surface
picks **zero frontend externals**. Always-on externals (stack-neutral):

- `karpathy-karpathy-guidelines` (LLM coding anti-patterns — universal)
- `julius-caveman` (terse-output compression — universal)

The critic's subtractive test extends: "would this external be wrong
for this repo's stack?"

### 4. Reconciliation on re-run

Extend the existing reconcile step:

- Tailor-owned **workflow** entries (marker: `category: workflow`):
  still in new pick → replace with fresh rewrite; not in pick → remove.
- Tailor-owned **universal** entries: refresh (copy current source; may
  have changed in harness-kit).
- Tailor-owned **external** entries (marker: `category: external`):
  - Still in pick → re-resolve absolute symlink (handles harness-kit moves).
  - Not in pick → remove symlink + sibling marker.
  - **Never overwrite the symlink target** — that's the global cache.
- Unmarked entries: preserve, flag.

### 5. Refresh / "automatic updates"

- User runs `harness-kit-checks sync-external` in harness-kit → updates
  `$HARNESS_KIT/skills/.external/<alias>/` in place.
- Consuming repos with absolute symlinks automatically see the update —
  next skill-discovery scan reads the new content.
- No per-repo action needed; updates flow through the shared cache.

Coupling caveat (document explicitly): consuming repos are coupled to
the harness-kit checkout's local path. On a different machine, externals
resolve only if harness-kit is at the same path (or the user re-runs
`/tailor` to re-resolve). For solo-developer workflows, this is fine.
For shared-clone team workflows, document the limitation.

### 6. New `/tailor --refresh-externals`

Lightweight subcommand: re-resolve every `.harness-kit` marker with
`category: external`. Useful when harness-kit's checkout path changes
(machine migration, directory rename). No picking, no rewrites — just
symlink repair. Optional; core install logic handles this via reconcile.

## Oracle

- [ ] `/tailor` in the harness-kit repo itself (CLI library) picks zero
      frontend externals. Always-on externals (karpathy-*) installed.
- [ ] `/tailor` in a Next.js app picks and installs (as symlinks) at
      minimum: `vercel-agent-browser`, `jakub-make-interfaces-feel-better`,
      `emil-emil-design-eng`, `karpathy-karpathy-guidelines`.
- [ ] Installed externals are absolute symlinks verifiable via
      `readlink .agents/skills/<alias>` → `$HARNESS_KIT/skills/.external/<alias>`.
- [ ] Sibling marker `.agents/skills/<alias>.harness-kit` exists per
      external, with `category: external` and `target: …`.
- [ ] Per-harness bridges (`.claude/skills/<alias>`, etc.) resolve
      transitively to the external content.
- [ ] Running `harness-kit-checks sync-external` in harness-kit updates the content
      visible via `cat .agents/skills/<alias>/SKILL.md` in a consuming
      repo (single source of truth).
- [ ] `/tailor` re-run with no pick changes is a no-op for externals
      (idempotent).
- [ ] `/tailor` re-run where an external was dropped from the pick
      removes the symlink + sibling marker; does not touch upstream cache.
- [ ] Self-audit step (existing check #2 in `/tailor`) extended: every
      external symlink resolves to a live target.
- [ ] Running the cross-harness install-paths check
      (`scripts/check-harness-agnostic-installs.sh`) still passes — no
      Claude-only wording introduced.

## Non-goals

- **Per-repo external cache.** Would duplicate content across repos
  and lose the automatic-update property. Stay with shared global cache.
- **Registry mutation from consuming repos.** `registry.yaml` is
  harness-kit-owned. Consuming repos are consumers.
- **Rewriting externals.** Point of "externally managed" is to NOT
  rewrite. Composition lint (future gate) can enforce this.
- **Globally symlinking every external into `~/.claude/skills/`.** That
  would reverse the `f91f1c4` minimal-globals pivot; per-repo is the
  right layer.
- **Automatic scheduled `harness-kit-checks sync-external` runs.** Out of scope; user
  runs it manually or via a cron-ish skill (`/schedule`) later.

## Why this isn't /focus rebuilt

`/focus` (killed 2026-03) was discovery-ceremony-heavy with 87 candidate
skills and no install killswitch. This shape is narrower:
- Picking is fast — `registry.yaml` is authoritative, not embeddings.
- Install is mechanical — symlink + sibling marker, no content generation.
- Reconciliation is deterministic — marker category drives action.
- Scope is bounded — externals per-repo, not per-session.

## Implementation notes

- Pure `/tailor` SKILL.md change; no new scripts. The install recipe is
  added to the skill body (~50-80 lines), reusing existing patterns.
- The critic's adjudication extends to checking that external-marked
  entries are symlinks, not copies.
- No changes needed to `registry.yaml`, `harness-kit-checks sync-external`, or
  `bootstrap.sh`. The existing external pipeline is load-bearing and
  correct; this shape only adds the consumer side.

## Dependencies

- None blocking. Builds on current `/tailor`, `registry.yaml`,
  `harness-kit-checks sync-external`, and `.harness-kit` marker convention.
