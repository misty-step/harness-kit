# Harden research skill runtime behavior

Priority: P1
Status: ready
Estimate: M

## Goal

Make `/research` resilient under normal runtime faults: concurrent cache
writes, hung providers, optional synthesis failures, and empty dedupe results.

The Conviction repo-local harness review surfaced `skills/research/` defects
that are upstream Harness Kit defects because target repos consume the canonical
`skills/research/` implementation.

## Non-Goals

- Do not replace the current provider model.
- Do not require every provider to be configured.
- Do not make optional AI synthesis mandatory for command success.

## Oracle

- [x] Concurrent cache writes in `skills/research/src/cache.ts` cannot lose
      unrelated entries; writes are atomic and serialize or retry safely.
- [x] Provider fetches in `skills/research/src/providers.ts` use bounded
      timeouts and return structured provider failures instead of hanging the
      command indefinitely.
- [x] Optional Perplexity or synthesis failure in `skills/research/src/cli.ts`
      does not abort an otherwise successful research run; the output records
      the synthesis failure as degraded evidence.
- [x] `skills/research/src/orchestrator.ts` treats empty post-dedupe results as
      a degraded or failed research result instead of a successful cacheable
      result.
- [x] Regression tests cover cache concurrency, provider timeout, synthesis
      degradation, and empty post-dedupe behavior.
- [x] `dagger call check --source=.` passes.

## Notes

Downstream repos should not patch vendored TypeScript copies. The canonical
runtime behavior belongs in Harness Kit's `skills/research/` source and its Bun
test suite.

## Progress

- Added `skills/research/__tests__/runtime-hardening.test.ts` covering cache
  concurrent writes, provider timeout structure, synthesis degradation, and
  empty post-dedupe cache avoidance/failure.
- Made `QueryCache` serialize same-process writes per file, re-read inside the
  write section, and save via temp-file then atomic rename.
- Added `ProviderRequestError` and `fetchWithTimeout` so provider HTTP calls
  are bounded and failures carry provider/kind/status fields.
- Exported `runResearch` from the CLI for testable response assembly and added
  `meta.degraded` to record optional synthesis failures without aborting
  successful retrieval.
- Changed empty post-dedupe provider results to continue to another provider
  and ultimately fail with `all providers returned no usable results` instead
  of caching an empty success.

## Delegation Evidence

- Planning lanes: `claude` receipt `a9c46d61-995e-4d40-8eff-093ad6afb36d`
  and `grok-build` receipt `f6b17c53-e268-46f9-9bc7-f4ae63845335`.
  Accepted cache re-read/atomic-save, bounded fetch, synthesis degradation,
  and empty-dedupe warnings. Rejected cross-process file locks, retries,
  provider model replacement, and full CLI DI as over-scope.
- Final diff critics: `grok-build` receipt
  `868a91b0-0fbe-42b4-9492-efeac324f818` and `claude` receipt
  `403489f4-ee43-4f52-be62-c9d895ca9ac9`, both `BLOCKING: no`.
  Accepted the orphaned timeout rejection risk and added a catch sink before
  rerunning verification.

## Verification

- `bun test` from `skills/research` -> 13 pass, 0 fail.
- `git diff --check`
- `bash scripts/build-docs-site.sh`
- `bash scripts/check-docs-site.sh`
- `dagger call check --source=.` -> 15 passed, 0 failed
