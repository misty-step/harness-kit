# /shape evals

Capability under test: M+ `/shape` context packets name the premise source
artifact that led to the shaped decision, or carry an explicit waiver with
residual risk.

Expected failure mode: a packet includes acceptance evidence but no upstream
premise source, references a missing local file, carries a stale digest, or
treats raw/private transcript storage as the default path instead of a redacted
excerpt or waiver.

Run:

```bash
bash skills/shape/evals/check-premise-source.sh
```
