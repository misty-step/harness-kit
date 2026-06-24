# Encourage portable, reversible system design in the shared doctrine

Priority: P2 · Status: pending · Estimate: M

## Goal
Agents building or grooming any HK-bootstrapped repo are nudged, by default, toward
**infrastructure-agnostic, reversible-by-config** system design — vendors behind
capability ports, standard protocols over proprietary SDKs, and a vendor-neutral data
model — so swapping host / storage / DB / auth / embeddings is a deploy-manifest + env
change, not a rewrite. The harness should treat "is this choice reversible?" as a
first-class design value alongside deep-modules and delete-first.

## Context
The sploot stack-sovereignty spike (sploot ADR-009; epic sploot `backlog.d/044`)
surfaced a general lesson. The recurring question — "should we run on Vercel / Fly /
Railway / Cloudflare / DO?" — has no durable answer, because the best vendor in three
months is unknowable. The mistake is trying to *pick*; the fix is to make the choice
*reversible*. A full coupling-map of sploot found it was ~70% portable almost by
accident (stock Postgres + pgvector, an HTTP telemetry sink, a thin auth-principal
seam), and the real lock-in was concentrated in a few avoidable decisions: persisting
absolute vendor URLs behind a CHECK constraint, and using the auth provider's user-id
as the database primary key. Both are exactly what an agent reaches for unless the
doctrine says otherwise.

HK already encodes adjacent values (deep modules / small surface, delete-first,
model-native primitives, verification-first) but nothing that names **vendor
reversibility** — so an agent shaping a new feature has no nudge to put the S3 SDK
behind a port instead of importing it in a route handler. This is a design *value*, not
a workflow: it belongs in Layer 1 + an on-demand reference, NOT a new phase or
mandatory checklist (thin-harness invariant — don't railroad).

The portability kit (seven levers, condensed for doctrine):
1. **Ports & adapters per capability** — the vendor SDK lives only inside an adapter,
   selected by env (Storage / Cache / Embeddings / Identity / Scheduler / Telemetry).
2. **Target protocols, not products** — S3 API, Postgres wire + OSS pgvector, Redis
   protocol, OIDC/JWT, OCI containers, HTTP-triggered cron, OTLP telemetry. Adapt to the
   lingua franca → the provider is fungible.
3. **Keep vendor identity out of the data model** — the DB is the most expensive thing
   to move; store opaque/relative refs, never a vendor URL or a provider user-id as a key.
4. **One typed env contract (12-factor)** — provider selection + creds via env.
5. **Declarative deploy per target** — Dockerfile + a small per-host manifest +
   `release_command`; standing up a new host is a recipe, not an adventure.
6. **A portability CI gate** — run the container against generic backends (plain
   Postgres + MinIO + Redis) with no vendor env; green = proof of no hidden lock-in.
7. **Quarantine proprietary runtime features** (edge middleware, ISR, provider image
   optimization, `waitUntil`) behind flags.

## Repo Anchors
- `harnesses/shared/AGENTS.md` Layer 1 — add one principle (e.g. "### Design for
  reversibility") beside "Strategic design: deep modules", "Delete before adding",
  "Match the implementation to the product premise".
- `harnesses/shared/references/` — add `portable-substrate.md` (the seven levers + the
  protocol map + the no-vendor-identity-in-the-data-model rule), a sibling to
  `model-native-product-primitives.md` and `delete-first.md`.
- `skills/shape/SKILL.md`, `skills/groom/SKILL.md`, `skills/deliver/SKILL.md` — consult
  the reference when a change introduces a vendor SDK, persists a vendor identifier, or
  picks an infra target. On-demand pointer lines, not a forced step.
- Motivating worked example (external repo): sploot `apps/web/docs/adr/009-...md` +
  sploot epic 044's "Portability doctrine" section.

## Oracle
- [ ] A `portable-substrate.md` reference exists in `harnesses/shared/references/` with
      the seven levers, the protocol map, and the "no vendor identity in the data model"
      rule.
- [ ] One Layer-1 principle in `harnesses/shared/AGENTS.md` names reversibility, links
      the reference, and stays consistent with the thin-harness vision (a value, not a
      phase).
- [ ] `/shape` and `/groom` point at the reference for infra/vendor-touching work
      (pointer lines, no duplicated prose).
- [ ] `cargo run --locked -p harness-kit-checks -- check --repo .` green.
- [ ] (Stretch) a reviewable smell named for consumer repos — "vendor SDK imported
      outside an adapter layer" / "vendor URL or provider user-id persisted" — as a
      `/deliver` review-lens callout (no bespoke linter required).

## Verification System
- **Claim:** an agent shaping or grooming vendor-touching work in a consumer repo
  surfaces the reversibility lens (ports, protocol-not-product, neutral data model)
  without being told.
- **Falsifier:** running `/shape` or `/groom` on a task that adds a new vendor SDK
  yields a plan that imports the SDK directly in business logic or persists a vendor
  identifier, with no portability nudge.
- **Driver:** dry-run `/shape` on a synthetic "add <vendor> object storage" task against
  the updated doctrine; inspect the shaped design.
- **Grader:** the reference is cited and the three core moves (capability port, protocol
  target, neutral data model) appear in the plan.
- **Evidence packet:** the shape transcript before/after the doctrine change.
- **Cadence:** once at landing; re-check opportunistically when a consumer repo does
  infra work.

## Notes
- Sibling doctrine: `model-native-product-primitives.md` (make the model boundary
  explicit) and `delete-first.md` — same "name the design value, let strong models apply
  it" shape. Reversibility is the infra analogue.
- **Vision guard:** must NOT become a mandatory portability phase or a heavy IaC
  mandate. Small projects and spikes are allowed to be vendor-coupled on purpose; the
  doctrine raises *awareness of the cost* and makes the neutral choice the default when
  it's ~free (a port is cheap; un-migrating a data model is not).
- Provenance: filed from the sploot session that ran the Fly spike (ADR-009) and the
  2026-06-24 full coupling-map. Cross-repo — the worked example lives in sploot.
