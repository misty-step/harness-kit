# `.harness-kit/*.yaml` repo-local config contract for `/flywheel`

Priority: P1
Status: done
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

- [x] Schema files live in `meta/config-schemas/`: `deploy.schema.yaml`,
      `monitor.schema.yaml`, and `flywheel.schema.yaml`. They document the
      executable shape while the loader enforces malformed configs before use.
- [x] A reference loader script (`scripts/load-harness-kit-config.py`) reads
      `.harness-kit/<name>.yaml`, validates it, prints normalized JSON to
      stdout, exits 2 for missing required config, and exits 1 with actionable
      schema/parse errors.
- [x] `skills/deploy/SKILL.md` and `skills/monitor/SKILL.md` reference the
      loader/schema as their config input and keep heuristic fallback only when
      `.harness-kit/<name>.yaml` is absent.
- [x] `skills/flywheel/SKILL.md` reads `.harness-kit/flywheel.yaml` if present
      for cycle-level tuning; otherwise it uses invocation flags and defaults.
- [x] Proof-of-integration is represented by the executable loader regression
      lane in Harness Kit. The misty-step repo adoption is a downstream follow-up
      because this `/deliver` run is scoped to the Harness Kit source checkout.
- [x] `dagger call check --source=.` green.

## What Was Built

- Added Draft 2020-12 schema files for deploy, monitor, and flywheel config.
- Added `scripts/load-harness-kit-config.py`, a thin root loader that validates
  `.harness-kit/<name>.yaml` and emits normalized JSON.
- Added `scripts/test-load-harness-kit-config.sh` plus a Dagger CI lane so the
  config contract is executable.
- Updated `/deploy`, `/monitor`, and `/flywheel` to name the loader/schema
  boundary explicitly.

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

### Follow-up

Downstream adoption remains separate: misty-step can now populate
`.harness-kit/deploy.yaml` and `.harness-kit/monitor.yaml` against the Harness
Kit schemas, but this source-repo branch does not mutate consumer repos.
