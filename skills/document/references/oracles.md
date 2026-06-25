# Oracles — the verification system

"Comprehensive, well-organized, high-quality" is vibes until it is falsifiers.
Four oracles turn each adjective into a system that can fail for the real error.
Shape follows `harnesses/shared/references/verification-system-first.md`:
claim → falsifier → driver → grader → evidence → cadence.

The accuracy oracle is the wedge. Auto-wiki products optimize coverage and
freshness; none adversarially refute each claim against source. That gap is the
reason this skill exists — do not cut it to save tokens.

## 1. Coverage — "comprehensive"

- **Claim:** every public/load-bearing surface is documented.
- **Falsifier:** an exported symbol, route, CLI command, entry point, or service
  with no page or section.
- **Driver:** extract the real surface from source — `rg` for exports/handlers,
  ctags/LSP for symbols, framework-specific route/command enumeration — and diff
  against what the docs cover (the `covers:` globs make this mechanical).
- **Grader:** documented ÷ real surface, with the undocumented set named. A
  number, not a feeling. Deliberate omissions (internal/experimental) are listed
  as waived, not silently dropped.
- **Evidence:** the coverage diff (covered / uncovered / waived).
- **Cadence:** after IA plan (did the plan miss a system?) and pre-commit.

## 2. Accuracy — "high-quality / true" (the wedge)

- **Claim:** every architectural assertion is true of the live code.
- **Falsifier:** a claim that cannot be grounded in specific source lines, or
  that the source contradicts.
- **Driver:** each page's claims fan out to fresh-context skeptic agents on a
  **decorrelated model family**, each prompted to *refute* — "the page says auth
  happens in X; read X and confirm or refute, defaulting to refuted if you cannot
  ground it." Perspective-diverse for claims that can fail multiple ways
  (does-it-compile, does-the-flow-actually-route-there, is-the-invariant-real).
- **Grader:** a claim survives only if grounded in cited source lines.
  Ungrounded claims are cut or flagged `unverified`. Majority-refute kills.
- **Evidence:** per-claim verdicts with the source line each survivor cites.
- **Cadence:** per page, every run. Never skipped on incremental re-runs — a
  changed neighbor can falsify an unchanged page's claim.

## 3. Navigability — "well-organized"

- **Claim:** a reader can find where to make a change using only the docs.
- **Falsifier:** a cold reader, given a real task, lands on the wrong files.
- **Driver:** a **cold-reader agent** sees *only the generated docs* (not the
  code) plus 2–3 representative tasks drawn from real backlog/issues ("where
  would you add a rate limiter?", "where does request validation live?"). It
  names the files it would open.
- **Grader:** did it land on the files a maintainer would? Right = good IA.
  Wrong = the structure or cross-links failed; fix the IA, not the task.
- **Evidence:** task → predicted files → actual files, with hits/misses.
- **Cadence:** after generation, before commit. A cheap behavioral proxy for IA
  quality that no rubric captures.

## 4. Render — "builds and renders"

- **Claim:** the docs render cleanly across surfaces.
- **Falsifier:** HTML build error, unparseable mermaid, dead internal link, or
  missing embedded asset.
- **Driver:** build the HTML render; parse every mermaid block; resolve every
  internal link and image reference.
- **Grader:** zero build errors, zero unparseable diagrams, zero dead links.
  Mechanical, cheap, run last.
- **Evidence:** build log + link-check output.
- **Cadence:** pre-commit, and on `--check`.

## Freshness — "synced to the code"

Owned by `references/provenance-and-freshness.md` and driven by
`scripts/freshness.py`. It answers a different question than the four above:
not "are the docs good?" but "are they still true of HEAD?" Run it on `--check`
and at the start of every run to decide incremental scope.

## Loop-until-dry stop rule

Iterate generation → oracles until **K consecutive rounds raise no blocking
failure** (default K=2). A blocking failure is any refuted claim, uncovered
load-bearing surface, cold-reader miss, or render break. Non-blocking polish
(wording, optional diagram) does not reset the counter. Simple "one pass and
ship" misses the tail; a fixed iteration count over- or under-runs. Converge on
the oracles, not a clock.

Dedup against claims already verified-and-killed so a rejected claim does not
reappear every round and stall convergence.
