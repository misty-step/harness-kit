# /harness-engineering eval waiver

expires: 2026-08-30

## Reason

Self-referential: this is the meta-skill used to build and audit the skill catalog's own eval coverage (this waiver pass included). Fixtures need to be authored from outside its own working session to avoid circularity.

## Disposition

Not exempt from the eval-coverage contract — this waiver is a time-boxed
deferral, not a permanent opt-out. When it expires, either an eval spec lands
at `skills/harness-engineering/evals/harness-engineering-eval.md` (see `skills/skill-eval/templates/eval-spec.md`)
or this waiver is renewed with a fresh reason and date. A stale, silently
renewed waiver with no new reasoning is itself a finding for `/harness-engineering`'s
next skill-health audit.

Tracked under `backlog.d/128-scale-skill-eval-integration.md` (EVALS-PER-SKILL).
