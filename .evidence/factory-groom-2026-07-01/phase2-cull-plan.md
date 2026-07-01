# Phase 2: Cull Zero-Use Vendored Skills

## Target Outcome

Deliver backlog `127-cull-zero-use-vendored-skills`: remove the zero-use vendored external imports named by the groom decision, regenerate the synced catalog outputs, and ship a focused PR with before/after evidence.

## Baseline

- First-party skills: 24
- Vendored externals: 56
- Total skill files: 80
- Total skill description bytes: 25,639
- Cull-set directories: 29
- Cull-set description bytes: 4,667
- Cull-set telemetry rows: 0

## Cull Set

- all `mattpocock-*` synced skills
- `steipete-skill-cleaner`
- `openai-gh-address-comments`
- `openai-gh-fix-ci`
- `petekp-grill-me`
- `every-ce-dogfood-beta`

## Execution

1. Remove the kill-list sources from `registry.yaml`.
2. Update first-party references that still point to removed aliases.
3. Run full `cargo run --locked -p harness-kit-checks -- sync-external --repo .`.
4. Regenerate `index.yaml` and `docs/site`.
5. Record after counts and run `cargo run --locked -p harness-kit-checks -- check --repo .`.

## Stop Conditions

- A retained first-party skill still requires a removed alias.
- `sync-external --check` would re-create deleted directories.
- The aggregate gate fails after regeneration.
