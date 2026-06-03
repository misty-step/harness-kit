# Multi-machine sync for verdicts

Priority: low
Status: done
Estimate: M

## Goal

Enable `refs/verdicts/*` to sync across machines via `git push`/`git fetch`.
Verdicts (from 020) are local refs.

**Note (032):** Claim-based coordination was dropped when `/flywheel` was
renamed to `/deliver`. This ticket was originally scoped to sync claims +
verdicts; it now covers verdicts only.

## Design

- `verdict_push [remote]` pushes local `refs/verdicts/*` to the remote
  (default: `origin`).
- `verdict_fetch [remote]` fetches remote `refs/verdicts/*` into local
  `refs/verdicts/*` (default: `origin`).
- No local or remote verdict refs is a successful no-op.
- Verdict refs remain immutable by convention once written; this helper only
  transports the refs.

## Oracle

- [x] Verdict created on machine A is visible on machine B after fetch
- [x] Works with any git remote (not GitHub-specific)
- [x] Claims are not reintroduced

## Implementation Notes

- Added `verdict_push [remote]` and `verdict_fetch [remote]` to
  `scripts/lib/verdicts.sh`.
- Added shell tests using a local bare remote plus two clones to prove a
  verdict written in one clone is visible in another after push/fetch.
- Added no-op tests for empty local and remote verdict-ref sets.
- Left claim coordination untouched; no `claims.sh`, `claim_acquire`, or
  `claim_release` path was added.

## Non-Goals

- Real-time sync (polling/push is sufficient)
- Distributed consensus protocol
- Claim synchronization
