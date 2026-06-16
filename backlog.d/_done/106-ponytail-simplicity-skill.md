# Context Packet: Ponytail simplicity skill adoption

Priority: P2
Status: ready
Estimate: S

## PRD Summary

- User: Harness Kit operators and agents working on implementation-heavy tasks.
- Problem: agents still overbuild by default: wrappers, dependencies,
  abstractions, and prose defenses for code that should not exist.
- Goal: import Ponytail as a pinned external skill so operators can invoke a
  tested YAGNI/native/stdlib-first lens without growing first-party doctrine.
- Why now: the hit list identified Ponytail as a sharp anti-overengineering
  primitive, and live source checks show it is portable, MIT licensed, and
  skill-folder shaped.
- UX enabled: the operator can ask for Ponytail/lazy/minimal mode and get a
  concrete simplification ladder rather than generic "keep it simple" advice.
- Deliverable type: external skill sync adoption.
- Success signal: `dietrich-ponytail` appears as a synced external skill and a
  probe task chooses native/stdlib solutions before custom code.

## Product Requirements

- P0: Add only Ponytail's core skill first; do not import its hooks, plugin
  lifecycle, or always-on mode.
- P0: Keep source ownership external with a `dietrich-` alias prefix and a
  pinned immutable ref.
- P0: The import must pass external skill lint, frontmatter, index drift, docs
  drift, and the full Harness Kit gate.
- P1: Consider `ponytail-review` only if a live review probe proves it catches
  overbuilt diffs better than existing `code-review`/`refactor` lenses.
- Non-goals: no first-party rewrite, no global always-on rule, no plugin hooks,
  no benchmark import, no automatic comments in every simplification.

## Repo Anchors

- `registry.yaml` - external source ledger, alias-prefix doctrine, pins, include
  filters, and license caveat convention.
- `crates/harness-kit-checks/src/external_sync.rs` - sync and drift behavior.
- `skills/harness-engineering/SKILL.md` - primitive test and external-skill
  adoption stance.
- `skills/harness-engineering/references/mode-sync.md` - sync lifecycle.
- `skills/harness-engineering/references/skill-design-principles.md` - imported
  skill fit and catalog hygiene.
- `harnesses/shared/AGENTS.md` - existing delete-before-add doctrine.
- `cargo run --locked -p harness-kit-checks -- check --repo .` - full gate.

## External Evidence

- `DietrichGebert/ponytail` is public, MIT licensed, and describes itself as a
  "laziest senior dev" ruleset that biases agents toward YAGNI, stdlib, native
  platform features, already-installed dependencies, one-liners, then minimal
  custom code.
- The repo's `skills/` directory currently contains `ponytail`,
  `ponytail-review`, `ponytail-audit`, `ponytail-debt`, and `ponytail-help`.
- `skills/ponytail/SKILL.md` has a portable skill shape and explicit trigger
  classifier for "simplest solution", "minimal solution", "YAGNI", "do less",
  "shortest path", and over-engineering complaints.
- The current `main` commit checked during shaping was
  `99139a25d07e3523d3f6871419798dda600db49a`; the latest observed tag was
  `v4.7.0` at `adad50d9b393926b2dd5ed7225dcb1848b9df408`.

## Alternatives

| Option | What it buys | Where it fails | Verdict |
|---|---|---|---|
| Do nothing | No catalog growth. | Keeps a known overbuild mitigation out of the harness despite good fit and license. | Reject. |
| Copy Ponytail into first-party `skills/` | Full control and no external sync. | Forks upstream, creates maintenance burden, and violates "external exemplars stay external" when no bespoke change is needed. | Reject. |
| Sync all Ponytail skills | Captures review/audit/debt variants too. | Catalog growth before evidence; several overlap with existing review/refactor/diagnose skills. | Reject for first slice. |
| Sync core `ponytail` only | Adds the sharp primitive with low surface and source provenance. | Needs a probe to avoid decorative import. | Choose. |
| Add only a doctrine line | Cheaper than external sync. | Loses Ponytail's tested ladder, triggers, and operator-invocable mode. | Reject. |

## Design

Add one `registry.yaml` source:

- `repo: DietrichGebert/ponytail`
- `ref: adad50d9b393926b2dd5ed7225dcb1848b9df408`
- `pin: adad50d9b393926b2dd5ed7225dcb1848b9df408`
- `layout: flat`
- `skills_path: skills`
- `alias_prefix: "dietrich-"`
- `include: [ponytail]`

Then run sync/lint/generate checks. Do not install Ponytail's hooks or plugin
runtime. Harness Kit consumes it as an ordinary external skill exposed through
bootstrap. Add a short registry comment explaining why only the core skill is
included and why companion skills are deferred.

## Oracle

- `cargo run --locked -p harness-kit-checks -- sync-external --repo . --only DietrichGebert/ponytail`
  installs exactly `skills/.external/dietrich-ponytail/SKILL.md`.
- `cargo run --locked -p harness-kit-checks -- lint-external-skills --strict`
  passes.
- `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`
  passes.
- `cargo run --locked -p harness-kit-checks -- generate-index --repo .` followed
  by `cargo run --locked -p harness-kit-checks -- check-index-drift --repo .`
  passes.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.
- Manual probe: invoke the synced skill on "add a date picker" and confirm the
  response selects native `<input type="date">` or explicitly explains why the
  native path is insufficient.

## Premise Source

Premise Source: sha256:ba4b4b28887bd991ea21311c4f9c0c9b38a8d0d4c5f27535b941c62dd50b8663 HIT-LIST.md

External source checked on 2026-06-16:

- https://github.com/DietrichGebert/ponytail
- https://raw.githubusercontent.com/DietrichGebert/ponytail/main/skills/ponytail/SKILL.md

## HTML Plan

HTML Plan: `.evidence/shape-hit-list/hit-list-shape-index.html#ponytail`

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| Catalog bloat | Import only `ponytail`; defer companion skills behind telemetry/probe evidence. |
| Always-on railroading | Do not import plugin hooks or global mode; expose as opt-in skill. |
| Stale upstream | Pin immutable ref; sync changes arrive as reviewable diffs. |
| Simplification becomes negligence | Preserve Ponytail's stated exclusions for security, data-loss, accessibility, and explicit user requirements. |

Rollback: remove the registry source, run `sync-external`, regenerate index/docs,
and rerun the full gate.
