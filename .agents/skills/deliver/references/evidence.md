# Evidence Handling (spellbook)

**Principle:** Evidence is out-of-band and NOT version-controlled.
Per-phase skills own their own emission; `/deliver` never writes
evidence itself, only records pointers in the receipt.

## Per-Phase Emission

| Phase | Emits | Where |
|---|---|---|
| `/code-review` | bench synthesis, thinktank output, cross-harness transcripts, verdict | `<state-dir>/review/` (gitignored). Also appends one JSON line to `.groom/review-scores.ndjson` (this file IS committed — see below). Writes `refs/verdicts/<branch>` via `scripts/lib/verdicts.sh`. |
| `/ci` | `dagger call check` per-gate tails; `dagger call heal` traces when self-heal fired | `<state-dir>/ci/` (gitignored) |
| `/refactor` | None durable (the diff speaks) | — |
| `/qa` | non-browser verification notes: gate receipt excerpts, skill eval output when relevant, symlink bridge checks, command transcripts, unverified-path notes | `<state-dir>/qa/` (gitignored) |
| `/implement` | None durable (test output transient) | — |

There is no browser screenshot stream unless a future Spellbook surface
adds a UI. `/qa` still records library/harness evidence because "tests
passed" and "the changed surface was smoke-verified" are different
claims.

## What IS Committed to Git

Two review-related artifacts are deliberately tracked (per the source
`/code-review` skill):

- `.groom/review-scores.ndjson` — one JSON line per review. `/groom`
  reads it for quality trends; `backlog.d/023` is the open debt
  tracking its feedback loop.
- `refs/verdicts/<branch>` — a git ref (not a file). Pushed/fetched
  with normal git machinery. Consumed by `.githooks/pre-merge-commit`
  as the merge gate.

These are the only exceptions to the "no committed evidence" rule.

## What Is NOT in Git

- No `.evidence/` directory.
- No LFS pointers for anything.
- No committed review transcripts, CI logs, or heal traces.

Everything under `.spellbook/deliver/<ulid>/` is gitignored wholesale.
The `.githooks/pre-commit` hook actively blocks force-adds of
`state.json` / `receipt.json`.

## Gitignore Convention

`.spellbook/` is gitignored repo-wide. `/deliver` state (`state.json`,
`receipt.json`, `review/`, `ci/`, `qa/`) lands under
`.spellbook/deliver/<ulid>/`.

If a phase emits something that should be permanent (a new backlog
file under `backlog.d/NNN-*.md`, a new/modified SKILL.md, a
migration), that emission belongs in the **diff** on the
`<type>/<slug>` branch, not in `<state-dir>`.

## Outer-Loop Override

When `/flywheel` invokes `/deliver`, it passes
`--state-dir backlog.d/_cycles/<ulid>/evidence/deliver/`. The cycle's
evidence directory is also gitignored at its top level; the outer
loop owns its own retention policy.

## Composer's Role

`/deliver` itself writes exactly two files: `state.json` and
`receipt.json`. It does not write review transcripts, CI logs,
heal traces, `.groom/review-scores.ndjson` entries, or verdict refs.
If the phase skill did not emit it, the receipt does not reference
it.
