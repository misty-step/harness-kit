# /fix

Audit. Fix. Verify. One issue at a time.

`/fix` uses `/diagnose`'s feedback-loop-first contract for arbitrary errors
and bugs. No speculative patch ships without a reproduction, replay,
instrumentation signal, or explicit no-loop stop.

## Usage

```
/fix stripe          # Audit Stripe, fix top issue
/fix docs            # Audit docs, fix top gap
/fix bitcoin         # Audit Bitcoin, fix top issue
/fix <error msg>     # Diagnose and fix an arbitrary error
```

## Domains

**Dynamic.** Scan `references/*-checklist.md` for available domains.

## Process

### Domain Fix (argument matches a domain)

1. **Audit** -- Read `audit/references/{domain}-checklist.md`, or
   `audit/generated-references/{domain}-checklist.md` for pack-provided domains,
   then run all checks
2. **Prioritize** -- Identify highest priority (P0 first) failing check
3. **Fix** -- Apply the fix for that one issue
4. **Verify** -- Re-run the failing check to confirm resolution
   - If user-facing behavior changed, run `/dogfood http://localhost:3000` and verify critical flows with `agent-browser` / `browser-use`
5. **Report** -- Output what was fixed and what remains

Fix priority order: P0 > P1 > P2 > P3. One fix per invocation.

### Error Fix (argument is an error message / stack trace)

1. **Build feedback loop** -- reproduce, replay, or instrument the symptom. If
   no loop is possible, stop and request the smallest missing artifact or
   access.
2. **Diagnose** -- Read the full error, locate source, understand context
3. **Research** -- Find similar issues, check docs for idiomatic solution
4. **Root cause** -- Ask: "Are we solving the root problem or treating a symptom?"
5. **Fix** -- Apply minimal fix addressing the proven root cause
6. **Verify** -- Re-run the original loop plus relevant tests
   - If user-facing behavior changed, run `/dogfood http://localhost:3000` and validate repro/fix with `agent-browser` / `browser-use`
7. **Commit** -- `fix: description`

`/dogfood` is available as an agent skill in this environment. Do not treat missing shell binaries as missing dogfood capability.

### Domain-Specific Fix Guides

#### Bitcoin
- Node not synced: check sync status, restart if needed
- Wallet not encrypted: `bitcoin-cli encryptwallet`
- UTXO consolidation: create consolidation tx

#### Lightning
- Inactive channels: rebalance or close/reopen
- Liquidity imbalance: loop out/in or open new channels
- Stuck channels: force close if necessary

#### Stripe
- Missing webhook secret: configure STRIPE_WEBHOOK_SECRET
- No signature verification: add `stripe.webhooks.constructEvent`
- Hardcoded keys: move to env vars

#### Docs
- Missing README: scaffold with project info
- Missing .env.example: extract from codebase env var usage
- Stale docs: update with current state

#### Observability
- No incident platform: install and configure the project's primary tracker
  (Canary, Sentry, or equivalent)
- Silent error boundaries: add explicit capture to the project's incident
  tracker in boundary/handler paths
- No health endpoint: create `/api/health`

#### Landing
- No landing page: scaffold marketing page
- No CTA: add primary call-to-action
- No OG tags: add metadata

#### Bun
- Mixed lockfiles: remove non-Bun lockfile
- CI not updated: switch to `oven-sh/setup-bun`
- Scripts using `node`: update to `bun`

## Output

```
Fixed: [P0] Webhook signature not verified
Remaining: P0: 0 | P1: 2 | P2: 3 | P3: 1
Run /fix stripe again for next issue.
```

## Related

- `/audit <domain>` -- Full audit without fixing
- `/log-issues <domain>` -- Create issues instead of fixing
