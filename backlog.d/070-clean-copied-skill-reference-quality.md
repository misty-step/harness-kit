# Clean copied skill reference quality

Priority: P2
Status: ready
Estimate: S

## Goal

Fix small but real defects in copied skill references so `/tailor` does not
install stale commands, unsafe examples, malformed snippets, or inconsistent
phase names into downstream repos.

## Non-Goals

- Do not redesign the skill reference system.
- Do not remove references that are still useful once corrected.
- Do not change workflow semantics beyond the documented defects.

## Oracle

- [ ] `skills/demo/references/pr-evidence-upload.md` replaces the literal
      `{NUMBER}` grep placeholder with a shell variable or another executable
      example that actually filters the intended PR number.
- [ ] `skills/demo/references/tts-narration.md` avoids unsafe shell
      interpolation into JSON and shows a robust JSON construction pattern.
- [ ] `skills/reflect/scripts/gather_evidence.sh` uses `grep --` before any
      pattern that could be interpreted as an option.
- [ ] `skills/research/references/delegate.md` completes the truncated conflict
      resolution bullet and uses fenced code blocks with language tags.
- [ ] Copied markdown references use language-tagged fences where the language
      is known.
- [ ] `skills/deliver/references/receipt.md` uses the canonical `/code-review`
      phase name instead of a stale `review` alias where phase names are
      enumerated.
- [ ] `skills/qa/evals/graders/check.sh` uses an appropriate fixed-string or
      escaped extended-regex check; unescaped alternation must not make the
      grader pass on the wrong text.
- [ ] `dagger call check --source=.` passes.

## Notes

These are upstream quality defects because `/tailor` copies the same references
into target repos. Downstream repos may patch generated copies for an active PR,
but the durable fix is in Spellbook.
