# Verification: 101 Focused Lane Harness Projection

## Commands
- `cargo fmt --all --check` - passed.
- `cargo test --workspace --locked` - passed; 278 passed, 0 failed, 1 ignored.
- `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .` -
  passed, including `.harness-kit/examples/lane-harness.yaml` fixture
  validation.
- `cargo run --locked -p harness-kit-checks -- check-runtime-primitives --repo .`
  - passed.
- `cargo run --locked -p harness-kit-checks -- materialize-lane-harness --manifest .harness-kit/examples/lane-harness.yaml --root .harness-kit/tmp/lane-harness/manual-smoke`
  - passed with `lane_harness_sha256:
  fd57fde038cdeb5252a64a70921a04c4ab55d579a35d78ac599a144cd16a64f0` and
  `visible_skills: ci`.
- `test -e .harness-kit/tmp/lane-harness/manual-smoke/.codex/skills/ci/SKILL.md && test ! -e .harness-kit/tmp/lane-harness/manual-smoke/.codex/skills/shape && test ! -e .harness-kit/tmp/lane-harness/manual-smoke/.codex/skills/groom`
  - passed.
- `rm -rf .harness-kit/tmp/lane-harness/manual-smoke && find .harness-kit/tmp/lane-harness -mindepth 1 -maxdepth 2 -print`
  - passed with no remaining runtime roots.
- `dagger call check --source=.` - passed; Harness Kit CI Results reported
  31 passed, 0 failed.

## Manual Smoke Result
The projected harness root contained `ci` under provider discovery roots and did
not contain `shape` or `groom` under the Codex projected root. The runtime root
was removed after inspection.

## Residual Risk
- Real-provider skill-discovery isolation is still conditional on each provider
  respecting the supplied `HOME`/config environment variables. The fake-provider
  smoke proves the filesystem projection contract, and real provider dispatch
  failures are typed receipts rather than successful evidence.
- `allowed_tools` and `allowed_external_aliases` are schema/registry validated
  but not yet a complete cross-provider tool or external-skill enforcement
  surface.
