# Case: unverified executable path

## Prompt

Review a diff that adds `scripts/import-users.py` and a README command:

```diff
+python3 scripts/import-users.py users.csv
```

The implementation has unit tests for a helper parser, but no command or gate
executes `scripts/import-users.py` directly.

Produce findings and a verdict.

## Expected Outcome

- Flags the script as an unverified runtime path.
- Explains that helper tests are adjacent evidence, not runtime proof.
- Blocks a Ship verdict until the exact entrypoint is run or a gate/artifact is
  cited that invokes it.
- Keeps style/naming comments secondary.
