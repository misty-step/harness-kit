---
name: settle
description: |
  DEPRECATED redirect. /settle, /pr-fix, and /pr-polish now route to
  /deliver --polish-only <branch|PR> — the single owner of "existing branch ->
  merge-ready" (backlog 080 collapsed settle into deliver). Slated for deletion
  next release.
  Use when: muscle-memory "polish this", "address PR reviews",
  "get this merge-ready" — then run /deliver --polish-only.
  Trigger: /settle, /pr-fix, /pr-polish.
argument-hint: "[PR-number|branch-name]"
---

# /settle → /deliver --polish-only  (deprecated redirect)

`/settle` collapsed into `/deliver` (backlog 080). One skill now owns
"existing branch → merge-ready," whether the branch came from `/implement` or
from a human. **This file is a redirect, slated for deletion next release** —
update muscle memory now.

## Delegation Floor

`/settle` has no separate delegation behavior. Use
`/deliver --polish-only`, whose Delegation Floor and specialized phase lanes
own existing-branch merge-readiness.

## Mapping

| You ran | Run instead |
|---|---|
| `/settle` | `/deliver --polish-only` |
| `/settle <branch\|PR>` | `/deliver --polish-only <branch\|PR>` |
| `/pr-fix <PR>` | `/deliver --polish-only <PR>` |
| `/pr-polish <PR>` | `/deliver --polish-only <PR>` |

## What moved where

- The polish loop (`/ci` → `/code-review` → `/refactor` → conditional
  `/design` + `/a11y` → `/qa`) is `/deliver`'s clean loop —
  `skills/deliver/references/clean-loop.md`.
- The fresh-eyes gate is the clean loop's **hindsight sanity pass**.
- PR-mode comment triage → `skills/deliver/references/pr-fix.md`; the
  full-comment fetcher →
  `cargo run --locked -p harness-kit-checks -- fetch-pr-reviews`.
- Deep hindsight/confidence reference → `skills/deliver/references/pr-polish.md`.
- Complexity reduction stays with `/refactor`
  (`skills/refactor/references/simplify.md`).

## Behavior change to know

`/deliver --polish-only` ends with the full `/deliver` closeout — a brief
**plus `/reflect`**. The old `/settle` stopped at merge-ready and left
reflection to `/ship`. Same merge-readiness gates, deliberately heavier
closeout. Full protocol: `skills/deliver/references/polish-only.md`.

## Verification

Semantic waiver: `/settle` is a deprecated redirect, so its behavior is proven
by `/deliver --polish-only` gates rather than a separate eval. Validate this
stub with `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`.
