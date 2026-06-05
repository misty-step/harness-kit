# Application Workflow

For application work in any repo, the human should usually invoke one of three
entry points:

| Situation | Invoke | Why |
|---|---|---|
| Raw idea, unclear scope, architectural choice, or missing oracle | `/shape` | Produces the context packet, acceptance oracle, design choice, and evidence plan. |
| One shaped backlog item should become merge-ready | `/deliver <ticket>` | Composes implementation, review, CI, refactor, QA, clean-tree closeout, receipt, and reflection. |
| Existing branch or PR already has code and needs cleanup | `/deliver --polish-only <branch|PR>` | Skips fresh shaping/implementation and enters the same merge-ready clean loop. |

Most humans should not invoke every phase skill manually. The phase skills are
leaf tools and escape hatches:

- `/implement` only when the packet is already shaped and the operator wants a
  TDD build phase without full closeout;
- `/code-review` or `skills/code-review/scripts/autoreview` for review-only
  checks, second-model review, or a frozen structured artifact;
- `/ci` for gate triage;
- `/refactor` for behavior-preserving simplification;
- `/design` + `/a11y` when the diff changes UI surfaces;
- `/qa` for running-surface evidence;
- `/hardening` when review, CI, QA, or the packet names a blocking
  test-strength gap;
- `/ship` after merge-ready work is ready for the final merge/archive/update
  decision;
- `/flywheel` only for a broader outer-loop backlog/shipping cycle.

Optimal default:

```text
/shape -> /deliver -> /ship
```

For already-started work:

```text
/deliver --polish-only <branch|PR> -> /ship
```

For review-only closeout:

```text
/code-review
```

The autoreview helper improves the `/code-review` leaf by making one
structured, target-normalized review cheap and repeatable. It does not replace
the roster floor, `/deliver` clean loop, Dagger gate, or clean-tree closeout.
