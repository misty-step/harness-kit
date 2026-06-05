# Clean copied skill reference quality

Priority: P2
Status: merge-ready
Estimate: S

## Goal

Fix small but real defects in skill references so bootstrap and `/seed` do not
expose stale commands, unsafe examples, malformed snippets, or inconsistent
phase names to downstream repos.

## Non-Goals

- Do not redesign the skill reference system.
- Do not remove references that are still useful once corrected.
- Do not change workflow semantics beyond the documented defects.

## Oracle

- [x] `skills/demo/references/pr-evidence-upload.md` replaces the literal
      `{NUMBER}` grep placeholder with a shell variable or another executable
      example that actually filters the intended PR number.
- [x] `skills/demo/references/tts-narration.md` avoids unsafe shell
      interpolation into JSON and shows a robust JSON construction pattern.
- [x] `skills/reflect/scripts/gather_evidence.sh` uses `grep --` before any
      pattern that could be interpreted as an option.
- [x] `skills/research/references/delegate.md` completes the truncated conflict
      resolution bullet and uses fenced code blocks with language tags.
- [x] Copied markdown references use language-tagged fences where the language
      is known.
- [x] `skills/deliver/references/receipt.md` uses the canonical `/code-review`
      phase name instead of a stale `review` alias where phase names are
      enumerated.
- [x] `skills/qa/evals/graders/check.sh` uses an appropriate fixed-string or
      escaped extended-regex check; unescaped alternation must not make the
      grader pass on the wrong text.
- [x] `dagger call check --source=.` passes.

## Notes

These are upstream quality defects because downstream repos consume the same
references through the system-wide harness or explicit `/seed` vendoring.
Downstream repos may patch local copies for an active PR, but the durable fix is
in Harness Kit.

## Progress

- Replaced PR evidence upload placeholders with `PR_NUMBER` and `REPO`
  variables, plus fixed-string `grep -F --` release-tag filtering.
- Replaced TTS narration JSON string splicing with `jq -n --rawfile`.
- Hardened shell grep invocations with `--` and explicit `grep -E` where
  alternation is intended.
- Completed the `/research` delegation conflict-resolution bullet and tagged
  known text/code fences in the touched copied references.
- Updated `/deliver` receipt examples to use `code-review` phase naming.
- Added Rust `eval_graders` tests to lock the fixed reference
  defects.

## Delegation Evidence

- `agy`: accepted `receipt.md` lifecycle fence finding; rejected broad
  catalog-wide fence sweep as out of scope for this S ticket.
- `grok-build`: accepted overlapping lifecycle fence finding; core shell/JSON/
  phase-name issues were already covered by the implementation patch.

## Verification

- `cargo test --workspace --locked eval_graders`
- `shellcheck --severity=error skills/reflect/scripts/gather_evidence.sh skills/qa/evals/graders/check.sh`
- `cargo test --workspace --locked`
- `dagger call check --source=.`
