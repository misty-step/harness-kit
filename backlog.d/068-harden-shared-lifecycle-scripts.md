# Harden shared lifecycle scripts

Priority: P1
Status: ready
Estimate: S

## Goal

Make the shared shell helpers fail closed and preserve exact JSON payloads when
they are copied into target repos by `/tailor`.

The Conviction tailor PR review found two script-level hazards that are
Spellbook bugs, not downstream repo bugs:

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

- [ ] `verdict_validate` fails with a non-zero exit and actionable message when
      the requested branch, ref, or commit cannot be resolved.
- [ ] `verdict_validate` never substitutes `HEAD` for an unresolved caller
      argument.
- [ ] Verdict JSON is passed to Python with exact bytes via `printf '%s'` or an
      equivalent mechanism; `echo "$json"` is not used for JSON transport.
- [ ] Verdict validation covers unresolved branch input and valid JSON with
      edge characters.
- [ ] `backlog_close` guards the root-directory change before `git mv`; failure
      aborts before any move is attempted.
- [ ] `dagger call check --source=.` passes.

## Notes

This should be fixed in Spellbook because `/tailor` copies these helpers into
target repos. Patching only the generated Conviction copies would leave the
next tailored repo with the same fail-open behavior.
