# Lane Card

Role:
Objective:
Provider target:
Model override: none
Scope:
Inputs / oracle:
Allowed skills:
Allowed tools:
Output shape:
Do not touch:
Receipt expectation:
Lane harness: none

## Launch

```sh
cargo run --locked -p harness-kit-checks -- dispatch-agent \
  --provider-target <provider> \
  --model-override <variant-or-id> \
  --objective "<objective>" \
  --input-ref "<path-or-ticket>" \
  --prompt-file <lane-card-path> \
  --backlog-ref <work-ref>
```

Add `--lane-harness <manifest>` only when the lane needs focused visible skills.
Remove `--model-override` when the card says `none`.
