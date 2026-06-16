# Lane Card: Pre-ship backlog batch review

Role: fresh-context critic.

Objective: Review the final diff for delivering backlog `105-111` and try to refute the done claim.

Scope:
- Use only the diff, the backlog packet oracles, and the rendered/evidence artifacts.
- Do not rely on the lead agent's reasoning trail.
- Do not edit files.

Output shape:
- `BLOCKING:` yes/no.
- If blocking, list exact file/path and why it violates an oracle.
- If non-blocking, list residual risks and test gaps.
- Keep under 1200 words.

Review focus:
- Exa Agent cost/privacy/default-off behavior.
- Ponytail external source ownership and sync proof.
- Skill scout dry-run semantics and deterministic tests.
- Works/loop/delete-first reference bloat and routing correctness.
- HarnessX self-modification and reward-hacking safety.
