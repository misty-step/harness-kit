# Provenance & freshness — the committed-docs contract

Committing docs into `docs/` versions them with the code, makes doc changes show
up in PR diffs, and lets the docs travel with forks and offline clones. It also
means they drift the instant code changes. Drift in committed, authoritative
docs is the *silent killer* — no error, just steady divergence into confident
lies. The mitigation is non-negotiable for this choice: make drift **detectable**
with provenance, and **checkable** with a falsifier.

## Per-page provenance stamp

Every generated page carries front-matter (see `templates/page.md`):

```yaml
---
title: Authentication
generated-at-sha: a3f1c2d          # repo HEAD when this page was written
covers:                            # source globs this page describes
  - src/auth/**
  - src/middleware/session.ts
verified: 2026-06-25               # last accuracy-oracle pass
model: <generating model + verifier family>
---
```

`covers:` is load-bearing: it is how coverage, incremental scope, and freshness
all key off a page. Be precise — over-broad globs make every page perpetually
stale; over-narrow globs let real drift slip through.

## Freshness falsifier

- **Claim:** every committed page is true of the current HEAD.
- **Falsifier:** a page whose `covers:` globs match files changed since its
  `generated-at-sha`.
- **Driver:** `scripts/freshness.py [docs_dir]` — parses each page's stamp, runs
  `git diff --name-only <sha>..HEAD`, and reports pages whose covered files
  moved. Exits non-zero if any page is stale, so it can gate.
- **Grader:** zero stale pages = synced. Stale pages are the incremental-scope
  work list.
- **Cadence:** start of every run (to scope the work) and on `--check`.

## Incremental scope

The quality bar is constant — the full verify loop runs every time. The *scope*
is not: on a re-run, regenerate only

1. pages the freshness driver flagged stale, plus
2. their cross-link neighbors (a changed system can falsify a claim on a page
   that points at it).

`--full` overrides this and regenerates everything (use after large refactors or
IA changes). This is how "always world-class" stays affordable on a one-line
change instead of re-documenting the monorepo from scratch.

## The Mode B handoff

Keeping committed docs fresh *on every push* is an event-triggered loop — Mode B
— which by `meta/CONTRACTS.md` lives in **bitterblossom**, not here. `/document`
is the on-demand (Mode A) generator. The freshness driver is the trigger
contract that future loop consumes: a push that makes pages stale is the event;
running `/document` (incremental) is the action.

Before proposing that loop, load
`harnesses/shared/references/loop-readiness.md` and name the three hard stops.
Do not build the push-triggered automation inside this skill — ship the Mode A
generator and the detectable-staleness signal; hand the refresh loop to the
event plane.

Until that loop exists, staleness is *visible* (stamped + `--check`-able)
between manual runs — far better than the invisible drift of an unstamped wiki.
