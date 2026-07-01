# /diagnose eval waiver

expires: 2026-08-15

## Reason

Needs a seeded, reproducible bug fixture (a real failing test plus a known root cause) authored per incident class; the first fixture class (a flaky-test repro) is scoped but not yet built.

## Disposition

Not exempt from the eval-coverage contract — this waiver is a time-boxed
deferral, not a permanent opt-out. When it expires, either an eval spec lands
at `skills/diagnose/evals/diagnose-eval.md` (see `skills/skill-eval/templates/eval-spec.md`)
or this waiver is renewed with a fresh reason and date. A stale, silently
renewed waiver with no new reasoning is itself a finding for `/harness-engineering`'s
next skill-health audit.

Tracked under `backlog.d/128-scale-skill-eval-integration.md` (EVALS-PER-SKILL).
