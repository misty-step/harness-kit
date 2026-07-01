# /next eval waiver

expires: 2026-08-01

## Reason

Shares its routing-decision shape with `/orient` (both recommend one next move from live state). Planned as a follow-on eval that reuses `/orient`'s fixture harness (`skills/orient/evals/orient-eval.md`) once that first run lands, rather than duplicating fixture design from scratch.

## Disposition

Not exempt from the eval-coverage contract — this waiver is a time-boxed
deferral, not a permanent opt-out. When it expires, either an eval spec lands
at `skills/next/evals/next-eval.md` (see `skills/skill-eval/templates/eval-spec.md`)
or this waiver is renewed with a fresh reason and date. A stale, silently
renewed waiver with no new reasoning is itself a finding for `/harness-engineering`'s
next skill-health audit.

Tracked under `backlog.d/128-scale-skill-eval-integration.md` (EVALS-PER-SKILL).
