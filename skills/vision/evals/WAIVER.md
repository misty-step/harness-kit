# /vision eval waiver

expires: 2026-09-15

## Reason

Conversational, multi-turn interrogation to produce VISION.md; a single scripted fixture can't faithfully replay the back-and-forth. Needs a human-anchor design, same class as `council`/`human-writing`.

## Disposition

Not exempt from the eval-coverage contract — this waiver is a time-boxed
deferral, not a permanent opt-out. When it expires, either an eval spec lands
at `skills/vision/evals/vision-eval.md` (see `skills/skill-eval/templates/eval-spec.md`)
or this waiver is renewed with a fresh reason and date. A stale, silently
renewed waiver with no new reasoning is itself a finding for `/harness-engineering`'s
next skill-health audit.

Tracked under `backlog.d/128-scale-skill-eval-integration.md` (EVALS-PER-SKILL).
