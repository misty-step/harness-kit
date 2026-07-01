# Harness Kit Factory Groom Plan

## Target Outcome

Phase 1 lands a groom PR titled `groom(2026-07-01): backlog + vision + license`, based on current `origin/master`, with the factory decisions reflected in the active backlog and `VISION.md`. The existing MIT license is verified and left unchanged.

## Chosen Shape

- Add ordered epic tickets in `backlog.d/` for the Harness Kit decision stack: skill catalog cull, skill-eval integration, checks-crate diet, role-scoped bootstrap bundles, skill-generated bespoke bundles, and greenfield-only template verification.
- Keep existing active tickets unless explicitly superseded; write consolidations and absorption notes in the new epics instead of deleting user work.
- Update `VISION.md` to name Harness Kit as Weave's skill pile and sync/eval primitive source: whole or subset installs into system and repo harnesses, with each durable skill earning its place through eval evidence.
- Treat `LICENSE` as already satisfying the MIT/Misty Step requirement.

## Proof Surface

- `test -f VISION.md`
- `paths=(AGENTS.md .antigravitycli .codex .pi skills); rg -n "VISION\\.md" "$paths[@]" 2>/dev/null`
- `cargo run --locked -p harness-kit-checks -- check --repo .`
- PR created, pushed, merged after the repo gate passes.

## Phase 2 Start

After the Phase 1 merge, begin the top-priority epic from the operator decisions: cull the zero-use vendored skills recoverable from upstream. The first implementation slice should remove the explicitly listed external imports and generated vendored copies, refresh generated indexes/docs, run the same repo gate, and ship a focused PR.
