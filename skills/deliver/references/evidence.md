# Evidence Handling

**Principle (2026-04-15 decision):** Evidence is out-of-band and NOT
version-controlled. Per-phase skills own their own emission; `/deliver`
never writes evidence itself, only records pointers in the receipt.

## Per-Phase Emission

| Phase | Emits | Where |
|---|---|---|
| `/code-review` | review synthesis, verdict, bench transcripts | `<state-dir>/review/` (gitignored) |
| `/ci` | dagger logs, failing-check tails | `<state-dir>/ci/` (gitignored) |
| `/qa` | screenshots, walkthroughs, findings | Its own scaffolded output dir (e.g. `/tmp/qa-<slug>/`); receipt records pointer |
| `/demo` | GIFs, launch videos | GitHub draft release via `/demo upload` (already works) |
| `/refactor` | None durable | — |
| `/implement` | None durable (test output transient) | — |

## What Is NOT in Git

- No `.evidence/` directory
- No LFS pointers for QA artifacts
- No committed screenshots, videos, or test transcripts

Review transcripts and CI logs live under `.harness-kit/deliver/` which is
gitignored wholesale. Demo artifacts live on GitHub releases.

## Gitignore Convention

`.harness-kit/` is gitignored repo-wide. `/deliver` state (`state.json`,
`receipt.json`, `review/`, `ci/`) lands under
`.harness-kit/deliver/<ulid>/`. Demo artifacts land on GitHub releases.

Nothing `/deliver` or its phase skills emit should be tracked by git.
If a phase emits something that should be permanent (commit-worthy
docs, migrations, etc.), that emission belongs in the **diff** on the
feature branch, not in `<state-dir>`.

## Outer-Loop Override

When `/flywheel` invokes `/deliver`, it passes
`--state-dir backlog.d/_cycles/<ulid>/evidence/deliver/`. The cycle's
evidence directory is also gitignored at its top level; the outer loop
owns its own retention policy.

## Composer's Role

`/deliver` itself writes exactly two files: `state.json` and
`receipt.json`. It does not write review transcripts, CI logs, screenshots,
or any other evidence. If the phase skill did not emit it, the receipt
does not reference it.
