---
title: <Page title — the system or concept this page owns>
generated-at-sha: <repo HEAD short sha at generation, e.g. a3f1c2d>
covers:
  - <source glob this page describes, e.g. src/auth/**>
  - <add one per load-bearing path; keep precise — over-broad globs read stale forever>
verified: <YYYY-MM-DD of last accuracy-oracle pass>
model: <generating model + verifier family, e.g. opus-4.8 / verified-by gpt-5.5>
---

<!--
World-class page skeleton. Delete sections that do not apply to this page's
Diátaxis mode — do not pad. Lead with the job-to-be-done, link to source lines,
capture intent and flow, never paraphrase function bodies.
-->

# <Title>

> One sentence: what this system does and why a reader is here.

## In one read

The job-to-be-done framing: what a reader can do or understand after this page.
Two or three sentences, not a definition dump.

## How it works

The flow and the *why*. Link claims to source: see `src/auth/session.ts:42`.
Every architectural assertion here must survive the accuracy oracle — if you
cannot point a skeptic at the lines, cut it or mark it `unverified`.

```mermaid
%% Only if a diagram answers a question prose answers worse.
%% Every node/edge must be grounded in source.
sequenceDiagram
    Client->>API: request
    API->>Auth: validate session
    Auth-->>API: ok / 401
```

## Key pieces

- **`<Symbol / module>`** (`path:line`) — what it owns, its boundary.

## Why it is shaped this way

Non-obvious decisions and trade-offs, each with a source (commit, ADR, comment).
Unsourced rationale is a guess — drop it or find the source. This is the
highest-value, lowest-supply section; do not skip it when the *why* is real.

## See also

- [<Related page>](../<section>/<page>.md) — cross-links the navigability oracle
  exercises; point the reader where they go next.
