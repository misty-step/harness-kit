# /code-review evals

Capability under test: `/code-review` reviews the diff as an artifact, uses
fresh-context reviewers, requires executable-path verification, and records a
review verdict.

Expected failure mode: reviewer ratifies the author's reasoning, misses an
unexercised executable path, or reports style findings without a ship/block
verdict.
