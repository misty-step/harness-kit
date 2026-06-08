Role: fresh implementation critic.
Objective: Review the current diff for the prompt-native `/dispatch` skill implementation.
Scope: Read-only. Inspect `git diff`, the new `skills/dispatch/` files, `harnesses/shared/AGENTS.md`, README, generated index/docs, and the roster gate update. Do not edit files.
Oracle: The implementation should keep global roster delegation mandatory, express dynamic subagents as prompt-native lane cards, avoid a scheduler/runtime/provider-ranking engine, and use only small existing gates.
Output: <=700 words with blocking findings first, then non-blocking notes. Include exact file paths or commands for each blocker.
Do not: critique the user premise that global delegation should be strong; critique only the implementation shape and verification gaps.
