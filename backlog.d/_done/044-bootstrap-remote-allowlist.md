# Bootstrap remote-install path — honor `.harness-kit.yaml` allowlist

Priority: P2
Status: **closed — ghost bug**
Estimate: XS
Closed: 2026-04-17

## Outcome

No code change required. Verified by inspection: the allowlist filter
at `bootstrap.sh:337` is discovery-agnostic — it rewrites
`GLOBAL_SKILLS[]` / `EXTERNAL_SKILLS[]` once, after either
`discover_local` or `discover_remote` has populated them, and before
either `link_local` or `install_remote` consume them.
`install_remote()` at `bootstrap.sh:562` iterates the already-filtered
`GLOBAL_SKILLS[]`, so remote-install honors `.harness-kit.yaml`
automatically.

## What actually shipped

- Clarifying comment above the filter block explaining discovery-agnostic
  design.
- Header paragraph in `scripts/test-bootstrap-filter.sh` documenting
  why the local-discovery fixtures prove remote-path correctness by
  structural equivalence (the "or documents why" branch of the original
  oracle).

## Original framing (preserved for history)

Follow-up to 043. The allowlist filter landed in 043 intersects
`GLOBAL_SKILLS[]` and `EXTERNAL_SKILLS[]` after `discover_local()` —
local-checkout path only. The remote-download path (`discover_remote()`
→ `install_remote()`) does not read `.harness-kit.yaml` and always
installs the full catalog from GitHub.

→ Incorrect premise. The filter was already post-discovery-union;
041's "after discover_local" implication was wrong. The oracle's own
hedge ("the allowlist parsing already happens once globally before
either path runs ... Confirm that in the diff and widen it.") pointed
at this — confirming the diff showed no widening was needed.

## Oracle

- [x] On a machine without a harness-kit checkout, running
      `curl -sL ...bootstrap.sh | bash` from a project directory with
      `.harness-kit.yaml` installs only the allowlisted subset.
      (Proven by inspection; filter is pure w.r.t. array contents.)
- [x] Unknown names in allowlist warn and drop (same as local).
      (Same code path; covered by existing `zzz-does-not-exist` test.)
- [x] `scripts/test-bootstrap-filter.sh` gains a case that exercises
      the remote path (or documents why the existing cases suffice
      because the filter is shared).
      (Documented the shared-filter invariant in the test header.)
