# End-to-end offline validation test

Priority: low
Status: fixed
Estimate: S

Unblocked by 024 (`.evidence/` storage) and 025 (Dagger pre-merge gate).
This ticket now lands the executable manual protocol; it remains a manual
validation path, not an automated CI gate.

## Goal

Prove the git-native, offline-first workflow actually works end-to-end without
any network access. Run the full cycle in airplane mode:

1. Create a feature branch
2. Make a local-only change
3. Run Dagger CI locally from warm cache
4. Store local `.evidence`
5. Store and validate review verdict in Git
6. Exercise the pre-merge gate
7. Document fallbacks when network-bound pieces are unavailable

## Why

Individual pieces may work offline, but the integrated workflow hasn't been
validated. Docker images need to be pre-pulled. retired-bench needs network for
external providers. What's the actual offline boundary?

## Oracle

- [x] Manual runbook exists at `meta/OFFLINE_VALIDATION.md`.
- [x] Preflight helper exists at `scripts/offline-validation-preflight.sh`.
- [x] Full workflow steps are executable with network disabled after warm-up.
- [x] Components that require network and offline alternatives are documented.
- [x] Dagger warm-cache requirements and runner image facts are documented.
- [x] Fallback: single local/fresh-context review when multi-provider review is unavailable.

## Non-Goals

- Making everything work offline (some things need network — document them)
- Automated CI for this test

## Implementation

- `meta/OFFLINE_VALIDATION.md` is the protocol.
- `scripts/offline-validation-preflight.sh` checks local prerequisites and
  prints the `.evidence/<branch>/<date>/` path.
- `scripts/test-offline-validation-preflight.sh` covers the helper without
  invoking real Dagger or network.
