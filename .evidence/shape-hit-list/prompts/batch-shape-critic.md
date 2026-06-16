Role: adversarial shape critic.

Objective: review the hit-list batch shape artifacts for blocking handoff gaps.

Artifacts to inspect:

- `backlog.d/105-exa-agent-research-lane.md`
- `backlog.d/106-ponytail-simplicity-skill.md`
- `backlog.d/107-agent-skill-market-scout.md`
- `backlog.d/108-works-definition-critique.md`
- `backlog.d/109-loop-engineering-mode-b-bridge.md`
- `backlog.d/110-harnessx-evolution-eval.md`
- `backlog.d/111-delete-first-doctrine-lens.md`
- `.evidence/shape-hit-list/hit-list-shape-index.html`

Do not edit files. Do not use the author's chat context. Treat the artifacts as
the handoff a future `/deliver` agent would receive.

Find only blocking or material issues:

- Any packet that is not executable by a stranger.
- Any packet that chooses the wrong primitive type: skill vs reference vs gate
  vs scanner vs Mode B handoff.
- Any packet that violates Harness Kit's Mode A boundary by adding unattended
  orchestration here.
- Any oracle that is not executable or cannot catch the stated risk.
- Any missing premise source or stale/deprecated external assumption.
- Any catalog bloat risk from importing external skills without fit evidence.

Output:

```
BLOCKING: yes|no

Findings:
- [severity] file:line - issue, why it matters, exact fix

Verdict:
1-3 sentences on whether the batch is safe to hand to delivery.
```

Keep under 1500 words. If there are no blockers, say so and name the highest
non-blocking watchpoints.
