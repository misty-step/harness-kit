# Root-skill vendoring follow-ups (from the herdr cross-model review)

Priority: P3 · Status: in-progress · Estimate: S

## Goal
Three improvements to the root-level external-skill vendoring shipped in 120
(`skill_name` + `stage_root_skill`), surfaced by the grok-4.3 review. None blocks
herdr — its skill is self-contained and the vendored output is correct — but each
earns its own small change.

## Items
1. **Companion-file support for root skills.** `stage_root_skill` vendors only
   `SKILL.md` + the upstream LICENSE. herdr is self-contained, but a future
   root skill that references a sibling `references/`/`scripts/` file would
   silently break. Add an explicit `skill_includes: [paths]` field for
   `skill_name` sources rather than a name-based heuristic (the first cut wrongly
   swept herdr's app `assets/`/`scripts/`). Until then, root skills must be
   single-file + license. **Deliberately deferred, not attempted.** Speculative
   schema addition for a root skill that doesn't exist yet — no current source
   needs `skill_includes`, and the ticket's own fallback ("until then, root
   skills must be single-file + license") is a genuinely acceptable interim
   state, not a real gap. Building the field now would be designing a schema
   around a hypothetical future consumer. Revisit when a real root-skill source
   actually needs companion files.

2. [x] **Sparse-checkout the skill files, not the whole app.** For
   `skills_path: "."`, `set_sparse` disables sparse, so each root-skill sync
   clones the full upstream working tree into the (gitignored) `_checkouts`
   cache — for herdr, the entire Rust app. Sparse-checkout just `SKILL.md`
   (+ LICENSE), or fetch via `git show`, to keep the cache minimal.
   Correctness is unaffected; disk/time only.
   (Root cause found live: `git clone --filter=blob:none --sparse` already
   defaults cone-mode sparse-checkout to top-level-files-only — the bug was
   that `set_sparse`'s `"."` branch actively called `sparse-checkout disable`,
   undoing that default and pulling the whole tree. Fix: `sparse-checkout set`
   with no path args, which re-asserts top-level-only explicitly and
   idempotently regardless of prior sparse state on a reused cached checkout.
   Live-verified against the real herdr checkout, not a synthetic fixture:
   re-ran `sync-external --only ogulcancelik/herdr` against the existing
   already-fully-cloned 44M cache — shrank to 12M, working tree now shows only
   top-level files, no `src/`/`.github/`/subdirectory content. Vendored output
   (`skills/.external/herdr-herdr/SKILL.md` + `LICENSE`) confirmed
   byte-identical via `git status` before/after.)
3. [x] **Decompose `external_sync.rs` production logic.** The file mixes registry
   parsing, git plumbing (`ensure_checkout`/`set_sparse`/`resolve_ref_to_sha`/
   `checkout_sha`), install, and cleanup. The 120 change tripped the god-file
   ratchet; extracting the inline tests to `external_sync_tests.rs` bought
   headroom but did not decompose the production logic (~800 lines, near the 850
   baseline). Extract the git-plumbing helpers into a submodule as a real cut.
   (New `external_sync_git.rs` submodule — `ensure_checkout`, `set_sparse`,
   `resolve_ref_to_sha`, `checkout_sha`, and their shared `run_checked` helper
   — registered via `#[path = ...] mod git;`, same convention this file
   already used for its own test submodule. `external_sync.rs` dropped from
   821 to 716 lines — comfortably under the general 800-line ceiling, so its
   god-file baseline entry was removed entirely rather than just updated;
   `check-godfiles` no longer needs to grandfather it. All 8 existing
   `external_sync` tests pass unmodified.)

## Non-Goals
- Reworking the vendoring of herdr itself (shipped, correct, gate-green).

## Provenance
Filed from the `Closes-backlog: 120` cross-model review (grok-4.3, 2026-06-24):
items 1, 2, and the production-decomposition half of item 4. License/idempotency/
bail-and-collision logic were reviewed and confirmed sound.

**2026-07-02 — items 2 and 3 shipped; item 1 stays deliberately deferred.**
Epic stays `in-progress` — item 1 has no current consumer and its own
fallback behavior is acceptable, so there's nothing blocking it, just
nothing yet motivating it. Re-open the item when a real root-skill source
needs companion files.
