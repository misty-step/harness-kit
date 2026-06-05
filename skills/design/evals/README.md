# /design evals

Capability under test: `/design` critiques a rendered artifact against intent,
uses concrete design-system and taste evidence, and avoids turning one-off
polish into a framework or token-system project.
It also maintains `DESIGN.md` / `design-contract.md` as repo-owned visual
contracts when recurring or product-facing visual work changes durable design
facts.

Expected failure mode: generic advice such as "improve spacing and colors",
code-only judgment when a render is available, or proposing a new UI framework
instead of bounded design moves.

The Rust grader is intentionally a small keyword floor, not a semantic judge.
It checks for objective output markers and forbidden over-scope language so the
eval remains runnable across harnesses; human or model grading can be layered
on top for taste quality.
