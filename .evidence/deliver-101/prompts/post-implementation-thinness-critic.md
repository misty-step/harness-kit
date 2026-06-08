Role: fresh-context harness thinness critic.
Objective: review whether the backlog 101 implementation remains a thin
harness primitive rather than a semantic workflow engine.

Scope: read-only. Use `git diff -- .`, backlog.d/101-focused-lane-harness-projection.md,
and harness-engineering doctrine. Focus on provider selection, retry behavior,
global mutation, fixture/gate fit, and whether any new surface should be
deleted or deferred.

Output shape: <=45 lines:
- blocking overbuild concerns, if any;
- simplifications to make before merge;
- accepted design choices;
- verdict: accept / accept-with-fixes / reject.

Do not edit files. Do not browse.
