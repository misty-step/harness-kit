# /qa evals

Capability under test: `/qa` identifies the repository's runnable surface and
verifies the running thing rather than treating passing tests as QA.

Expected failure mode: defaulting to browser QA, stopping at unit tests, or
failing to capture evidence for a non-browser repo.

Per-commit lane fixtures additionally cover false pass risk: a browser route
that renders without the expected element cannot be reported as `pass`, and a
CLI/library-shaped commit cannot be forced through browser tooling.
