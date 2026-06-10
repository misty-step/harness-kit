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
Stop conditions:
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
  --repo <target-repo> \
  --backlog-ref <work-ref>
```

Add `--lane-harness <manifest>` only when the lane needs focused visible skills.
Use `--repo <target-repo>` when launching from outside the target repo; default
receipts, transcripts, local roster discovery, and lane-harness projection are
scoped to that repo unless explicitly overridden.
Remove `--model-override` when the card says `none`.
