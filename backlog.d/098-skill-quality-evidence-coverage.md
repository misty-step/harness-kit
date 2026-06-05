# Skill quality evidence coverage pass

Priority: high
Status: ready
Estimate: M

## Problem

`/groom audit` now uses generated catalog truth instead of shared-AGENTS
mirroring, which removed the false routing failures. The remaining broad gap is
real: 22 first-party skills still lack objective test/eval evidence according
to `python3 skills/groom/scripts/audit-skills.py`.

This should not be fixed by sprinkling empty `evals/` folders or generic
"Testing" sections. The skill-design rule is that verification behavior earns
scripts, assertions, or explicit semantic waivers.

## Goal

Raise the first-party skill catalog's verification coverage without adding
shallow artifacts. Every remediated skill gets one of:

- a deterministic test/eval/script that proves its load-bearing behavior;
- a small verification section that names the exact existing gate proving it;
- an explicit waiver explaining why the skill is reference-only or intentionally
  untestable today.

## Evidence

Current audit command:

```sh
python3 skills/groom/scripts/audit-skills.py
```

Current summary after the audit-script repair:

```text
Skills audited: 33
Failed dimensions:
  2: 0
  1: 22
  0: 11
```

The remaining single failure is `tests` for the 22 non-passing skills.

## Acceptance Oracle

- [ ] `python3 skills/groom/scripts/audit-skills.py` reports materially fewer
      `tests: FAIL` rows, with no empty placeholder eval/test directories.
- [ ] Each remediated skill's evidence is either executable, points to an
      existing gate, or carries an explicit semantic waiver.
- [ ] `python3 scripts/check-frontmatter.py`
- [ ] `python3 scripts/check-agent-roster.py`
- [ ] `bash skills/groom/scripts/test_audit_skills.sh`
- [ ] `dagger call check --source=.`

## Non-Goals

- Do not require every skill to have a full eval suite in one pass.
- Do not turn `/groom audit` into a hard Dagger gate until the signal is clean.
- Do not add generic boilerplate "Testing" sections that only satisfy regexes.

## Related

- `skills/harness-engineering/references/skill-design-principles.md`
- `skills/groom/scripts/audit-skills.py`
- `skills/harness-engineering/references/mode-eval.md`
