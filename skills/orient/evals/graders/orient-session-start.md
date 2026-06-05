# Orient Session Start Grader

Run the Rust grader from the Harness Kit source root:

```sh
cargo run --locked -p harness-kit-checks -- eval-grader orient-session-start <candidate-output>
```

The grader accepts concise orientation reports that name live sources, workspace
state, backlog signal, roster state, likely next skill, and residual
uncertainty. It rejects readiness scoring, transcript mining, hidden session
state, broad debrief/reflection, and generic preambles without evidence.

