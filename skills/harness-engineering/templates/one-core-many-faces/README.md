# One Core, Many Faces Template

Copy this folder into a new product repo when the product needs API, CLI, MCP,
SDK, skill, and human surfaces over one Rust-owned core.

This is a scaffold, not a generator. Replace tokens, delete unearned faces, and
make the first verified slice work before adding surface.

## Tokens

- `{{project}}` - product name, for humans.
- `{{client_class}}` - TypeScript-safe client class prefix, for example
  `Landmark`.
- `{{crate_prefix}}` - Rust crate prefix, snake case.
- `{{binary}}` - CLI binary name.
- `{{repo}}` - GitHub repository, for example `misty-step/example`.
- `{{base_branch}}` - repository base branch, for example `main` or `master`.
- `{{npm_scope}}` - npm scope without `@`, for example `misty-step`.
- `{{fly_app}}` - Fly.io app name, for example `example-prod`.
- `{{fly_region}}` - Fly.io primary region, for example `iad`.
- `{{description}}` - one concrete product sentence.

## Target Tree

```text
.
├── AGENTS.md
├── Cargo.toml
├── Dockerfile
├── fly.toml
├── litestream.yml
├── .env.example
├── bin
│   └── entrypoint.sh
├── .landmark.yml
├── crates
│   ├── core
│   ├── shell
│   ├── api
│   ├── cli
│   └── mcp
├── sdk
│   └── typescript
└── skills
    └── {{binary}}
        └── SKILL.md
```

## First Slice

1. Fill `core` with one real domain rule and a failing-then-passing test.
2. Fill `shell` with one use case and fake external ports.
3. Add only the first adapter that has a real consumer.
4. Add the verification path for that adapter.
5. Run `cargo generate-lockfile` before locked gates or Docker builds.
6. Delete the adapter folders not exercised by the first acceptance oracle.

## Proof Before Expansion

- API face: request replay.
- CLI face: stdout/stderr smoke.
- MCP face: protocol replay and structured-error check.
- SDK face: throwaway consumer build.
- Skill face: cold-agent use smoke.
- Web face: browser path with screenshot or trace.
- Deploy face: Docker image build, `fly.toml` validation, `/healthz` and
  `/readyz` smoke, and Litestream restore drill or explicit pre-production
  waiver.

## Guardrails

- Business rules do not import adapters.
- Adapter crates do not branch on product policy.
- MCP tools are intent-shaped, not endpoint-shaped.
- Non-Rust SDKs stay tiny unless a consumer needs them.
- Landmark release intelligence is part of the shipped product surface.
- Litestream runs only at the process edge; business logic never shells out to
  backup tooling.
