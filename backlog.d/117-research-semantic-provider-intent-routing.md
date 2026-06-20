# Add semantic provider-intent routing to research

Priority: P1
Status: pending
Estimate: M

## Goal

Route `/research` provider selection from semantic intent and required
capability instead of relying on regex-only query classes.

## Current Evidence

- A Harness Primitives review on 2026-06-20 found regex-only query intent in
  `skills/research/query-utils.ts`, `skills/research/cli.ts`, and
  `skills/research/orchestrator.ts`.
- The same review found provider-routing tests cover exact pattern classes but
  not paraphrased realtime/speech/model-provider intent.
- The Standby failure mode was the product-level version of this same bug:
  semantic model work was collapsed into phrase heuristics.

## Non-Goals

- Do not build a semantic workflow engine around providers.
- Do not remove deterministic validation, provider fallback, or receipts.
- Do not make provider CLIs own synthesis or final authority.

## Repo Anchors

- `skills/research/SKILL.md`
- `skills/research/query-utils.ts`
- `skills/research/cli.ts`
- `skills/research/orchestrator.ts`
- `skills/research/__tests__/provider-routing.test.ts`
- `harnesses/shared/references/model-native-product-primitives.md`

## Design

Add a narrow `research-route-intent` stage:

```text
query
  -> provider-intent classifier or structured model prompt
  -> { intent, confidence, required_capability }
  -> deterministic provider chain builder
  -> bounded providers + receipts
```

Regexes may remain as fast-path hints and low-confidence fallback, but they
should not be the only path for semantically clear paraphrases.

## Oracle

- `bun test skills/research/__tests__/provider-routing.test.ts` passes with new
  paraphrase cases for realtime speech, model/provider comparison, library docs,
  social/discourse, and ordinary web research.
- A low-confidence classifier result falls back to the existing deterministic
  provider chain and records the fallback reason.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Verification System

- Claim: `/research` routes semantically equivalent provider-intent prompts to
  the right acquisition path without brittle exact phrasing.
- Falsifier: a paraphrased realtime/speech/model-provider query bypasses the
  intended provider chain while an exact phrase succeeds.
- Driver: research provider-routing tests plus full Harness Kit check.
- Grader: expected provider chain, intent metadata, confidence/fallback reason,
  and no regression of existing routing tests.
- Evidence packet: test output and any fixture snapshots added with the change.
- Cadence: run with every research skill/harness change.

## Notes

Why: this is the Harness Kit counterpart to Standby's AI-first correction.
Deterministic routing remains useful, but semantic intent should be model-native
when the route itself depends on meaning rather than string shape.
