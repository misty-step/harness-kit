# Deep Review Lens

Use when a PR or branch needs evidence-first review rather than a broad
nitpick pass. Inspired by Peter Steinberger's `github-deep-review` and
`autoreview` contracts.

## Review Shape

For each important finding, ask:

- **Behavior:** What user/operator behavior is wrong or at risk?
- **Root cause:** Where is the ownership boundary, and why is this the cause?
- **Provenance:** Was it introduced by this diff, made visible by this diff,
  or carried forward from earlier code?
- **Best fix:** Is the proposed fix at the right layer and small enough?
- **Proof:** What command, test, route, artifact, or source proves the claim?
- **Residual risk:** What remains unverified?

Use confidence words precisely: `clear`, `likely`, or `unknown`. `Not proven`
is an acceptable review result when the evidence does not support a finding.

## Long-Running Review Discipline

- Treat review output as advisory evidence, not authority.
- Verify every accepted finding by reading the real code path.
- Reject speculative risks and broad rewrites that do not name a concrete
  failure mode.
- If a review-triggered fix changes code, rerun focused tests and review the
  changed diff again.
- Do not kill a structured review because it is quiet for a few minutes.
  Heartbeat or elapsed-time output is healthy progress.
- Stop when the structured helper or final review exits clean with no
  accepted/actionable findings. Do not run redundant reviews for nicer wording.

## Finding Template

```markdown
### Finding
- Severity:
- Behavior at risk:
- Evidence:
- Provenance: introduced by | made visible by | carried forward by | unknown
- Best fix:
- Proof required:
```
