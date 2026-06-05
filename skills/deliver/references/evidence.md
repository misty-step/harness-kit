# Evidence Handling

**Principle (2026-06-02 decision):** Evidence is git-native by default.
Per-phase skills own their own emission; `/deliver` never writes evidence
itself, only records pointers in the receipt.

## Canonical Surface

Use `.evidence/<branch>/<date>/` for QA, demo, and review artifacts that
should survive a fresh clone. Branch names are slugged by the Harness Kit
evidence helper.

```bash
EVIDENCE_DIR="$(cargo run --quiet --locked -p harness-kit-checks -- evidence create)"
```

Binary evidence under `.evidence/` is scoped to Git LFS by `.gitattributes`.
When no LFS server is available, fresh clones still retain pointer files at
minimum. GitHub draft releases, PR comments, Slack posts, and similar surfaces
are mirrors or audience packaging, not canonical storage.

## Per-Phase Emission

| Phase | Emits | Where |
|---|---|---|
| `/code-review` | review synthesis, verdict, bench transcripts | synthesis + verdict in `.evidence/<branch>/<date>/`; transcripts may stay under `<state-dir>/review/` |
| `/ci` | dagger logs, failing-check tails | `<state-dir>/ci/` (gitignored), with durable summaries linked from `.evidence/` when needed |
| `/qa` | screenshots, walkthroughs, findings | `.evidence/<branch>/<date>/` |
| `/demo` | GIFs, launch videos, screenshots, paste artifacts | `.evidence/<branch>/<date>/`; optional GitHub draft release mirror |
| `/refactor` | None durable | — |
| `/implement` | None durable (test output transient) | — |

## What Is in Git

- `.evidence/<branch>/<date>/` directories that prove QA/demo/review outcomes.
- LFS pointer files for binary evidence when LFS is configured.
- Text summaries such as `qa-report.md`, `review-synthesis.md`, or command
  transcripts that are safe to commit.

## What Stays Gitignored

Review transcripts and CI logs can still live under `.harness-kit/deliver/`,
which is gitignored wholesale. Large raw provider transcripts stay out of git
unless rendered/redacted through `/agent-transcript` and explicitly chosen for
publication.

## Outer-Loop Override

When `/flywheel` invokes `/deliver`, it may pass
`--state-dir backlog.d/_cycles/<ulid>/evidence/deliver/` for machine receipts.
That does not replace `.evidence/<branch>/<date>/` for branch-carried QA/demo
artifacts.

## Composer's Role

`/deliver` itself writes exactly two files: `state.json` and `receipt.json`.
It does not write review transcripts, CI logs, screenshots, or any other
evidence. If a phase skill emits committed evidence, the receipt records the
`.evidence/<branch>/<date>/` pointer. If the phase skill did not emit it, the
receipt does not invent it.
