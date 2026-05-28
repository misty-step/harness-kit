# `.harness-kit/*.yaml` repo-local config contract for `/flywheel`

Priority: P1
Status: pending
Estimate: L

## Goal

Define the repo-local `.harness-kit/*.yaml` schema + loader so `/flywheel`, `/deploy`, and `/monitor` read declarative config instead of falling back to heuristics (`.vercel/project.json`, git metadata, env sniffing). Unblocks downstream adoption of the outer loop.

Minimum viable shape:

- `.harness-kit/deploy.yaml` — target environment, deploy command, health-check URL, rollback command, pre/post hooks.
- `.harness-kit/monitor.yaml` — signal sources (logs, error rates, latency dashboards), alert thresholds, what `/diagnose` should reach for when an alert fires.
- `.harness-kit/flywheel.yaml` (optional) — cycle cadence, budget, which backlog items are in scope, stop conditions.

## Non-Goals

- NOT building new `/deploy` or `/monitor` skills. Those exist; this gives them a config input instead of inference.
- NOT requiring the schema for inner-loop work (`/deliver`, `/shape`, `/implement`, `/code-review`). Inner loop continues to work without `.harness-kit/`.
- NOT auto-generating `.harness-kit/` at bootstrap. Downstream repos opt in;
  `/seed` may grow to scaffold it during explicit repo-local vendoring
  (separate concern).
- NOT a runtime config system with reloads, namespaces, or env layering. Single flat read at skill-invocation time.

## Oracle

- [ ] Schema files live in `meta/config-schemas/` (or equivalent location that `/diagnose` can point at): `deploy.schema.yaml`, `monitor.schema.yaml`, and optionally `flywheel.schema.yaml`. JSON Schema Draft 2020-12 or similar executable form so malformed configs fail at parse, not at use.
- [ ] A reference loader script (`scripts/load-harness-kit-config.sh` or `.py`) reads `.harness-kit/<name>.yaml`, validates against schema, prints normalized JSON to stdout. Non-zero exit with actionable error on schema violation.
- [ ] `skills/deploy/SKILL.md` and `skills/monitor/SKILL.md` reference the schema as their config input and fall back to heuristic mode only when `.harness-kit/<name>.yaml` is absent — documented behavior.
- [ ] `skills/flywheel/SKILL.md` reads `.harness-kit/flywheel.yaml` if present for cycle-level tuning; otherwise uses sensible defaults.
- [ ] Proof-of-integration: misty-step populates `.harness-kit/deploy.yaml` + `.harness-kit/monitor.yaml` using the schema; `/flywheel` run there consumes them instead of heuristics. Close misty-step's `backlog.d/006` as shipped by this work.
- [ ] `dagger call check --source=.` green.

## Notes

### Provenance

Surfaced by the Strategist investigator in the /groom session 2026-04-23. Evidence: misty-step `backlog.d/006` explicitly asks for `.harness-kit/deploy.yaml` + `.harness-kit/monitor.yaml` to unblock `/flywheel`. Canary `AGENTS.md:14-15` references `.harness-kit` as a config source. Zero downstream repos except canary populate `.harness-kit/` today, because the schema has never been defined.

### Why schema-first, not skill-first

The skills (`/deploy`, `/monitor`, `/flywheel`) already exist. What's missing is the input contract. If each skill keeps inferring from environmental clues, every downstream repo hits different failure modes and has to rediscover the same workarounds. A schema makes the contract explicit, which the loader can then validate uniformly.

### Deliberate minimalism

Three files, flat keys, no nesting layers, no environment-specific overlays. If the schema grows tentacles (deploy.prod.yaml, deploy.staging.yaml, env-specific merges), that's a sign the problem was framed wrong and should be re-shaped, not extended.

### Downstream leverage rationale

misty-step is explicitly blocked on this (own backlog). canary would drop its custom harness-dispatch config if the schema covered its use cases. bitterblossom's sprite orchestration has adjacent config pain that could consume the same primitive. Three downstream consumers unblocked in one ticket. This is the proving-ground pattern the /groom backlog doctrine asks for.

### Risk: over-engineering the schema

Schemas attract bikeshed. Mitigation: land the minimum keys that misty-step and canary currently need, then add keys only when a concrete downstream repo blocks on their absence. Schema versioning via a top-level `schema_version:` field so future breaking changes are detectable.

### Composition with other backlog items

- Complements the repo-local harness config work: schemas define explicit inputs
  without requiring generated repo-specific skill copies.
- Independent of 051 (AGENTS.md restructure) but lands better after 051 so `.harness-kit/` rationale can be documented cleanly in the restructured L3 routing layer.
- Does NOT depend on 023–027 (review-score, offline evidence, Dagger merge gate). Those are orthogonal.

### Execution sketch (not binding)

One PR, three commits:
1. `feat(config): define .harness-kit/ schemas for deploy, monitor, flywheel` — schema files + one reference loader.
2. `feat(deploy,monitor,flywheel): consume .harness-kit/ configs when present` — wire skills to use the loader; keep heuristic fallback with a deprecation note.
3. `feat(misty-step): adopt .harness-kit/ configs` — downstream integration in the actual repo; validates the schema survives real-world contact before we call it done.

Ships as 052 closes and misty-step/006 closes in parallel.
