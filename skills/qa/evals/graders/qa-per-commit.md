# QA Per-Commit Graders

Run the Rust graders from the Harness Kit source root:

```sh
cargo run --locked -p harness-kit-checks -- eval-grader qa-browser-missing-selector <candidate-output>
cargo run --locked -p harness-kit-checks -- eval-grader qa-non-browser <candidate-output>
```
