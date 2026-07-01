# Phase 2 Cull Receipt

## Scope

Implemented backlog `127-cull-zero-use-vendored-skills`: removed the zero-use
vendored external skill imports named by the 2026-07-01 factory groom decision.

## Before

- First-party skills: 24
- Vendored externals: 56
- Total skill files: 80
- Total skill description bytes: 25,639
- Cull-set directories: 29
- Cull-set description bytes: 4,667
- Cull-set telemetry rows: 0

## After

- First-party skills: 24
- Vendored externals: 27
- Total skill files: 51
- Total skill description bytes: 20,972
- Cull-set directories: 0
- Cull-set description bytes: 0

## Removed

- all `mattpocock-*` synced skills
- `steipete-skill-cleaner`
- `openai-gh-address-comments`
- `openai-gh-fix-ci`
- `petekp-grill-me`
- `every-ce-dogfood-beta`

## Commands

- `cargo run --locked -p harness-kit-checks -- sync-external --repo .`
- `cargo run --locked -p harness-kit-checks -- generate-index --repo .`
- `cargo run --locked -p harness-kit-checks -- build-docs-site --repo .`
- `cargo run --locked -p harness-kit-checks -- sync-external --repo . --check`
- `cargo run --locked -p harness-kit-checks -- lint-external-skills --strict`
- `cargo run --locked -p harness-kit-checks -- check --repo .`

## Result

`sync-external --check` reported clean, strict external lint reported
`27 / 27 aliases self-contained`, and the aggregate Harness Kit gate passed.
