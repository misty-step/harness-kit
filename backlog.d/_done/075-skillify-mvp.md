# Scaffold the /skillify MVP and core CRUD primitives

Priority: P1
Status: done
Estimate: M

## Goal

Build the foundational `/skillify` pipeline supporting the `--from-current` execution path. It discovers, parses, and classifies active conversational transcripts from Claude Code JSONL files, synthesizes them into compliant first-party skills, and executes system-wide propagation via bootstrap.

## Oracle

- [x] `harness-kit-checks skillify-skill-crud` is implemented and supports deterministic `create`, `read`, `update`, `delete`, and `validate` operations on skill filesystem primitives.
- [x] `harness-kit-checks skillify-parse-transcript` parses Claude Code JSONL transcript streams into clean instruction packets.
- [x] `skills/skillify/SKILL.md` is registered and triggers on triggers like `/skillify`.
- [x] Classification dispatches successfully call two or more roster providers to evaluate the novelty and repeatability of the conversation.
- [x] `harness-kit-checks check-frontmatter` passes on the newly generated skill.
- [x] `./bootstrap.sh` cleanly propagates the generated skill, making it immediately available to active harnesses.
- [x] `dagger call check --source=.` runs green.

## Notes

Strictly enforce the portability contract: reject any generated skill that references harness-specific operations (`SendUserMessage`, `bash`, `Edit`, `Skill` dispatch) without fallback mechanisms. Focus solely on Claude JSONL transcripts first; Gemini and Codex batch ingestion will follow in Phase 4.

## What Was Built

- Added first-party `/skillify` with `--from-current`-oriented workflow prose
  and the portability contract.
- Added deterministic skill CRUD in `crates/harness-kit-checks/src/skillify_skill_crud.rs`.
- Added Claude Code JSONL parsing in `crates/harness-kit-checks/src/skillify_transcript.rs`.
- Added roster-backed novelty/repeatability classification in
  `crates/harness-kit-checks/src/skillify_classify.rs`.
- Added Rust unit tests and Dagger lanes so the commands are enforced in CI.
- Regenerated `index.yaml` and the docs companion.

## Verification

- `cargo test --workspace --locked skillify` - passed, including skill CRUD round trip, classification dispatch command building, `--from-current` transcript resolution, and unrelated-project rejection coverage.
- `cargo test --workspace --locked` - passed.
- `python3 -m py_compile ci/src/harness_kit_ci/main.py` - passed.
- `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .` - passed with existing warnings for
  `karpathy-guidelines` and `model-research`.
- `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .` - passed.
- `python3 skills/harness-engineering/scripts/validate-evals.py` - passed.
- `python3 scripts/check-evidence-blocks.py skills` - passed.
- `bash scripts/check-docs-site.sh --self-test` and `bash scripts/check-docs-site.sh` - passed.
- `harness-kit-checks skillify-classify <packet> --provider codex --provider pi --timeout-s 90` - dispatched two providers successfully.
- `HARNESS_KIT_DIR="$PWD" ./bootstrap.sh` plus
  `test -e "$HOME/.codex/skills/skillify/SKILL.md"` - passed.
- `dagger call check --source=.` - passed, 17 lanes.

## Acceptance Hashes

- `/skillify` skill:
  `sha256:90192f18bfbd4d8245a301023557e641c5b653f49c9cb0b7c7a5af80b7ea94b1`
  `skills/skillify/SKILL.md`
- CRUD command:
  `sha256:85b06b3cf16695b291b83057dffb58b4d80fba7c156903070609870317b4d2c3`
  `crates/harness-kit-checks/src/skillify_skill_crud.rs`
- Transcript parser command:
  `sha256:f5c907e666fa3aa31c2987565ce7618c2cf039359d9784a3459e5e1ac4715534`
  `crates/harness-kit-checks/src/skillify_transcript.rs`
- Classifier command:
  `sha256:1f652d8f981dd014f3a5a4f2cb9e6c34e882e81d4f1a999b85dd9bbd60265fe7`
  `crates/harness-kit-checks/src/skillify_classify.rs`

## Delegation Evidence

- codex planning receipt `fd64266c-8caf-42c9-b711-0e466172a92d` recommended a
  self-contained skillify directory, CRUD/parser scripts, a thin classifier over
  `harness-kit-checks dispatch-agent`, and command tests; accepted.
- pi planning receipt `8f2b463f-3464-4917-84a3-5af0c72f600f` independently
  recommended CRUD, parser, classifier, bootstrap propagation, and portability
  rejection; accepted.
- classifier codex receipt `a762627b-5996-4941-90ee-b954f9a64d62` and pi
  receipt `7567d8a5-632d-4429-9d61-922fc074329b` prove the classification path
  dispatched two roster providers successfully.
- critic codex receipt `9f613827-dea4-46ad-8263-d92669dc9092` found two
  blockers: missing `--from-current` parser behavior and a Dagger ignore bug;
  both were fixed before closeout.
- critic pi receipt `0cedf662-36c4-43ce-8c68-18e6572affa6` found no blockers
  and noted nonblocking hardening follow-ups; accepted as nonblocking.
- final critic codex receipt `6a04d4c1-a24b-49e7-896e-7736ef66026b`
  and grok-build receipt `43f6a7de-18de-4bf9-85c7-e0332a33d921` found no
  blockers after the `--from-current` unrelated-project fallback was fixed.
