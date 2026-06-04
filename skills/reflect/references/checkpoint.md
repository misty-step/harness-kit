# /reflect checkpoint

Opt-in teach-back checkpoint for complex sessions where the operator should
prove they understand the load-bearing decision, failure mode, and next action
before the session advances or closes.

## Trigger

Run this mode only when one of these is present:

- explicit operator request such as `/reflect checkpoint <topic>`;
- a packet or backlog marker `Comprehension-required: <topic>`;
- a skill reference that explicitly opts into the same marker.

Absent the marker or explicit request, there is no comprehension gate. Do not
quiz the operator by default, and do not block mechanical fixes, small changes,
or emergency state preservation.

## Workflow

1. Name the checkpoint topic and the evidence refs the operator should
   understand.
2. Ask one focused question that requires the operator to restate the decision,
   failure mode, and next action in their own words.
3. Fill gaps briefly when the restatement is incomplete.
4. Record a checkpoint artifact with a verdict of `pass`, `partial`, or `fail`.
5. When a packet explicitly requires the topic, run the validator in gate mode.

The checker proves artifact structure and gate behavior. It cannot prove actual
cognition; the lead remains responsible for judging whether the restatement is
substantive.

## Artifact Schema

Store checkpoint artifacts as local JSON when they need to be validated or used
as evidence. Recommended path:
`.harness-kit/reflect/checkpoints/<topic>.json`.

Required fields:

```json
{
  "topic": "decision-name",
  "source_refs": ["backlog.d/096-reflect-teach-back-checkpoints.md"],
  "question": "What decision did we make, what can fail, and what happens next?",
  "operator_restatement": "Short operator-authored restatement.",
  "lead_verdict": "pass",
  "gaps": [],
  "next_action": "Continue the session.",
  "timestamp": "2026-06-04T00:00:00Z"
}
```

Rules:

- `lead_verdict` is exactly one of `pass`, `partial`, or `fail`.
- `operator_restatement` is required; do not infer understanding from silence,
  acknowledgements, or agent summaries.
- `pass` requires empty `gaps`.
- `partial` and `fail` require one or more specific technical gaps.
- Keep `operator_restatement` short. Record refs, decisions, gaps, and next
  action, not raw transcripts, raw tool output, or private learning profiles.

## Gate

Use the validator:

```bash
python3 skills/reflect/scripts/checkpoint.py validate \
  .harness-kit/reflect/checkpoints/<topic>.json \
  --gate <topic> \
  --packet <packet-or-backlog.md>
```

If `--packet` does not contain `Comprehension-required: <topic>`, the gate is a
no-op and exits zero. If the packet requires the topic, the gate exits non-zero
unless a matching checkpoint artifact exists with `lead_verdict: pass`, an
operator restatement, and empty gaps.

Run the self-test before relying on this mode:

```bash
python3 skills/reflect/scripts/checkpoint.py --self-test
```

## Privacy Boundary

Checkpoint artifacts are not transcripts and not a learning analytics store.
They may include evidence refs, a short restatement, technical gaps, and a next
action. They must not include credentials, raw session logs, private customer
data, broad psychological claims, or cross-session learning profiles.
