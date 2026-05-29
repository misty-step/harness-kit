# Scaffold the /skillify MVP and core CRUD primitives

Priority: P1
Status: ready
Estimate: M

## Goal

Build the foundational `/skillify` pipeline supporting the `--from-current` execution path. It discovers, parses, and classifies active conversational transcripts from Claude Code JSONL files, synthesizes them into compliant first-party skills, and executes system-wide propagation via bootstrap.

## Oracle

- [ ] `skills/skillify/scripts/skill-crud.py` is implemented and supports deterministic `create`, `read`, `update`, `delete`, and `validate` operations on skill filesystem primitives.
- [ ] `skills/skillify/scripts/parse-transcript.py` parses Claude Code JSONL transcript streams into clean instruction packets.
- [ ] `skills/skillify/SKILL.md` is registered and triggers on triggers like `/skillify`.
- [ ] Classification dispatches successfully call two or more roster providers to evaluate the novelty and repeatability of the conversation.
- [ ] `scripts/check-frontmatter.py` passes on the newly generated skill.
- [ ] `./bootstrap.sh` cleanly propagates the generated skill, making it immediately available to active harnesses.
- [ ] `dagger call check --source=.` runs green.

## Notes

Strictly enforce the portability contract: reject any generated skill that references harness-specific operations (`SendUserMessage`, `bash`, `Edit`, `Skill` dispatch) without fallback mechanisms. Focus solely on Claude JSONL transcripts first; Gemini and Codex batch ingestion will follow in Phase 4.
