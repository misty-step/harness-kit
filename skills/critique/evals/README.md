# /critique evals

Capability under test: `/critique` dispatches a single rubric-backed,
read-only lens critic against a target and reports evidence-backed findings
without claiming a merge verdict.

Expected failure mode: the output references static `agents/<lens>.md`,
duplicates `/code-review`'s bench, or returns a Ship/Don't Ship verdict instead
of targeted critique signal.
