# /ci eval waiver

expires: 2026-08-30

## Reason

Payoff is a real CI/gate state change across arbitrary repos; no single fixture repo captures the space yet. Needs a small seeded suite of intentionally-broken CI configs before an A/B is meaningful.

## Disposition

Not exempt from the eval-coverage contract — this waiver is a time-boxed
deferral, not a permanent opt-out. When it expires, either an eval spec lands
at `skills/ci/evals/ci-eval.md` (see `skills/skill-eval/templates/eval-spec.md`)
or this waiver is renewed with a fresh reason and date. A stale, silently
renewed waiver with no new reasoning is itself a finding for `/harness-engineering`'s
next skill-health audit.

Tracked under `backlog.d/128-scale-skill-eval-integration.md` (EVALS-PER-SKILL).
