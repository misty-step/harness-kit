# Harden shared lifecycle scripts

Priority: P1
Status: merge-ready
Estimate: S

## Goal

Make the shared shell helpers fail closed and preserve exact JSON payloads when
they are used directly or vendored into target repos.

The Conviction repo-local harness review found two script-level hazards that are
Harness Kit bugs, not downstream repo bugs:

- `scripts/lib/verdicts.sh` falls back to `HEAD` when the requested branch or
  ref does not resolve, so a missing review target can silently validate the
  wrong commit.
- `scripts/lib/backlog.sh` changes into the repo root before `git mv` without
  guarding the `cd`, so a failed directory change can run the move from the
  wrong working directory.

## Non-Goals

- Do not change backlog trailer semantics.
- Do not add a semantic workflow layer around the shell helpers.
- Do not lower the merge verdict gate.

## Oracle

- [x] `verdict_validate` fails with a non-zero exit and actionable message when
      the requested branch, ref, or commit cannot be resolved.
- [x] `verdict_validate` never substitutes `HEAD` for an unresolved caller
      argument.
- [x] Verdict JSON is passed to Python with exact bytes via `printf '%s'` or an
      equivalent mechanism; `echo "$json"` is not used for JSON transport.
- [x] Verdict validation covers unresolved branch input and valid JSON with
      edge characters.
- [x] `backlog_close` guards the root-directory change before `git mv`; failure
      aborts before any move is attempted.
- [x] `dagger call check --source=.` passes.

## Notes

This should be fixed in Harness Kit because bootstrap and `/seed` expose these
helpers to target repos. Patching only the generated Conviction copies would
leave the canonical source with the same fail-open behavior.

## Progress

- `verdict_validate` now resolves the requested target first, reports
  unresolved targets, and never falls back to `HEAD`.
- Verdict JSON is piped with `printf '%s'` everywhere payload bytes enter
  Python or Git blob storage.
- `backlog_archive` now fails closed when archive directory creation, repo-root
  `cd`, or `git mv` fails.
- Added regression tests for unresolved verdict targets, exact JSON byte
  storage with edge characters, failed repo-root `cd`, and failed `git mv`.

## Verification

- `bash scripts/lib/test_verdicts.sh`
- `bash scripts/lib/test_backlog.sh`
- `shellcheck --severity=error scripts/lib/backlog.sh scripts/lib/test_backlog.sh scripts/lib/verdicts.sh scripts/lib/test_verdicts.sh`
- `dagger call check --source=.`
