# /research eval

The oracle for the `/research` skill. Tests the one claim a research skill
must earn: **given a drift-prone or comparison-shaped question, `/research`
returns a synthesized answer where every load-bearing claim is tied to a
dated source or a live-repo fact, with residual uncertainty and stale/skipped
sources named — that a bare "look this up for me" prompt on the same model
does not (it answers from training-data memory and states no dates, no
source list, and no uncertainty).**

This is a `mode-eval` A/B run, not a directory shape. Arms: A = `/research`
installed and invoked; B = raw same-model ("research this and tell me the
answer", no skill, same tool access); C = single-WebSearch-call baseline
(tests whether multi-source fan-out earns its cost over one lookup). Grade
blind, objective checks first, judge a different model family than the
workers.

## Fixtures

| # | Prompt | Repo/context | Forbidden edits | What it stresses |
|---|---|---|---|---|
| 1 | "Which model is best for cheap, high-volume OpenRouter calls right now — compare 2–3 candidates on price, context, and tool-calling support." | none (pure web research) | any file edit | drift-prone fact, dated sourcing, ranked recommendation (`references/exemplars.md`/model-provider index pattern) |
| 2 | "What are people actually saying about Anthropic's skills feature since the June 2026 blog post — is the reception positive?" | none (social/discourse) | any file edit | discourse/social synthesis, source diversity, distinguishing signal from noise |
| 3 | "Does this repo's `check-eval-coverage` gate duplicate anything already shipped upstream in a well-known OSS skill-eval framework? Check the repo first, then the web." | `harness-kit@c6e01b9` (`crates/harness-kit-checks/src/eval_coverage.rs`) | any source edit | repo-truth-first ordering (read local before external), mixed repo+web synthesis |

Two of three must show A>B for a pass; the fixtures span a pure external
lookup, a discourse/social scan, and a repo-grounded question that should
start with local evidence per the skill's own contract.

## Objective checks (scriptable, pass/fail, ~free — run on every `skills/research/**` edit)

- [ ] Every load-bearing factual claim carries a citation (URL, file path, or
      command output) — no bare assertion.
- [ ] At least one cited source carries an explicit date, and the report
      states the research date/recency window.
- [ ] Fixture 1 lists ≥2 candidates with a ranked recommendation, not a menu
      with no pick.
- [ ] Fixture 3: the report reads the live repo (`eval_coverage.rs` or its
      test file) before or alongside the web search — repo-truth-first is
      violated if the report answers from web sources alone.
- [ ] Skipped, failed, or stale sources are named explicitly, not silently
      omitted.
- [ ] A residual-uncertainty line is present (what the report could not
      confirm).
- [ ] No claim is sourced only from the model's training-data memory when a
      current source was reasonably available.

## Rubric (1–5, blind, one-line justification each — judgment-heavy delta only)

| Dimension | 5 | 1 |
|---|---|---|
| Source grounding | every claim traceable to a dated source or repo fact | confident answer, no citations |
| Recommendation quality (fixture 1) | ranked pick with stated tradeoffs | describes options, picks nothing |
| Repo-truth ordering (fixture 3) | reads local repo before concluding | answers purely from general knowledge of "typical" eval frameworks |
| Honesty about gaps | names what's unverified or stale | smooths over missing evidence |

## Pass condition

Arm A beats arm B on source grounding and residual-uncertainty reporting
across **≥2 of 3** fixtures, AND ties-or-wins every objective check. A
no-op "research" (equivalent to raw prompting) fails because the raw arm
reliably answers from memory with no dates, no source list, and no stated
uncertainty — exactly the gap the objective checks catch without a judge.

## Human anchor

The operator blind-grades fixture 1 (model comparison — verifiable against
the operator's own current knowledge of pricing/availability). Record the
verdict and match/mismatch here once run. **PENDING — no run yet.**

## Cadence

- Edit-time: 1-fixture native-subagent smoke (fixture 1) on any
  `skills/research/**` change.
- Contract change (the source-ordering rule, the fan-out defaults, or the
  residual-uncertainty requirement moves): full A/B, all 3 fixtures,
  decorrelated families.
- Major model release: re-audit — a stronger bare model with better built-in
  search grounding may close `/research`'s edge on drift-prone facts first.

## Run log

**No run yet.** Spec seeded 2026-07-01 under backlog.d/128 (EVALS-PER-SKILL);
`/research` is the third-highest-usage first-party skill (27 recorded
invocations per the 2026-07-01 groom telemetry read) and had no eval coverage
before this. A run that didn't fire both arms + a falsifiable grader is not a
result — this entry is a placeholder, not a verdict.
