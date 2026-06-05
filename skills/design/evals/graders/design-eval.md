# Design Eval Grader

Run the Rust grader from the Harness Kit source root:

```sh
cargo run --locked -p harness-kit-checks -- design-eval [rendered-critique|scaffold-contract|design-contract-maintenance|token-only-critique] <candidate-output>
```

The default mode is `rendered-critique` when only `<candidate-output>` is
provided.
