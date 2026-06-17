Run the harness-kit ship gate and report pass/fail.

Run this from the repo root:

```
cargo run --locked -p harness-kit-checks -- check --repo .
```

This is the Rust-owned local gate. Green means it passed. The lane list is
defined in `crates/harness-kit-checks/src/ci_check.rs`.

If it fails, report the exact failing lane and the first real error. Do NOT
propose lowering a gate to get green — diagnose the root cause. The gate names
real failures it catches; if a lane is wrong, fix the lane, not the code it
flags.
