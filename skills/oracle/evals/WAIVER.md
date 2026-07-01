# /oracle eval waiver

expires: 2026-09-29

## Reason

Requires a live, authenticated ChatGPT browser session (Oracle browser mode) — an external, operator-owned dependency that can't be fixtured deterministically yet.

## Disposition

Not exempt from the eval-coverage contract — this waiver is a time-boxed
deferral, not a permanent opt-out. When it expires, either an eval spec lands
at `skills/oracle/evals/oracle-eval.md` (see `skills/skill-eval/templates/eval-spec.md`)
or this waiver is renewed with a fresh reason and date. A stale, silently
renewed waiver with no new reasoning is itself a finding for `/harness-engineering`'s
next skill-health audit.

Tracked under `backlog.d/128-scale-skill-eval-integration.md` (EVALS-PER-SKILL).
