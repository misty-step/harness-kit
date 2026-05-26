# Harden research skill runtime behavior

Priority: P1
Status: ready
Estimate: M

## Goal

Make `/research` resilient under normal runtime faults: concurrent cache
writes, hung providers, optional synthesis failures, and empty dedupe results.

The Conviction repo-local harness review surfaced `skills/research/` defects
that are upstream Spellbook defects because target repos consume the canonical
`skills/research/` implementation.

## Non-Goals

- Do not replace the current provider model.
- Do not require every provider to be configured.
- Do not make optional AI synthesis mandatory for command success.

## Oracle

- [ ] Concurrent cache writes in `skills/research/src/cache.ts` cannot lose
      unrelated entries; writes are atomic and serialize or retry safely.
- [ ] Provider fetches in `skills/research/src/providers.ts` use bounded
      timeouts and return structured provider failures instead of hanging the
      command indefinitely.
- [ ] Optional Perplexity or synthesis failure in `skills/research/src/cli.ts`
      does not abort an otherwise successful research run; the output records
      the synthesis failure as degraded evidence.
- [ ] `skills/research/src/orchestrator.ts` treats empty post-dedupe results as
      a degraded or failed research result instead of a successful cacheable
      result.
- [ ] Regression tests cover cache concurrency, provider timeout, synthesis
      degradation, and empty post-dedupe behavior.
- [ ] `dagger call check --source=.` passes.

## Notes

Downstream repos should not patch vendored TypeScript copies. The canonical
runtime behavior belongs in Spellbook's `skills/research/` source and its Bun
test suite.
