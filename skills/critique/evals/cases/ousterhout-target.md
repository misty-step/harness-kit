# Case: Ousterhout Target Critique

Run `/critique --lens ousterhout --target src/auth`.

Expected output:
- names `ousterhout` as the lens;
- reads the lens from `harnesses/shared/references/lenses.md`;
- dispatches one fresh read-only critic, or explicitly marks
  `lens-adopted-by-primary` fallback;
- reports findings as `Finding`, `Evidence: file:line`, and `Impact`;
- does not reference `agents/ousterhout.md`;
- does not return a Ship, Conditional, or Don't Ship verdict.
