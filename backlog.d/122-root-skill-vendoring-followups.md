# Root-skill vendoring follow-ups (from the herdr cross-model review)

Priority: P3 · Status: open · Estimate: S

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
   single-file + license.

2. **Sparse-checkout the skill files, not the whole app.** For `skills_path: "."`,
   `set_sparse` disables sparse, so each root-skill sync clones the full upstream
   working tree into the (gitignored) `_checkouts` cache — for herdr, the entire
   Rust app. Sparse-checkout just `SKILL.md` (+ LICENSE), or fetch via `git
   show`, to keep the cache minimal. Correctness is unaffected; disk/time only.

3. **Decompose `external_sync.rs` production logic.** The file mixes registry
   parsing, git plumbing (`ensure_checkout`/`set_sparse`/`resolve_ref_to_sha`/
   `checkout_sha`), install, and cleanup. The 120 change tripped the god-file
   ratchet; extracting the inline tests to `external_sync_tests.rs` bought
   headroom but did not decompose the production logic (~800 lines, near the 850
   baseline). Extract the git-plumbing helpers into a submodule as a real cut.

## Non-Goals
- Reworking the vendoring of herdr itself (shipped, correct, gate-green).

## Provenance
Filed from the `Closes-backlog: 120` cross-model review (grok-4.3, 2026-06-24):
items 1, 2, and the production-decomposition half of item 4. License/idempotency/
bail-and-collision logic were reviewed and confirmed sound.
