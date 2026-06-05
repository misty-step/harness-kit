# /diagnose evals

Capability under test: `/diagnose` must build or request a believable feedback
loop before hypothesizing or fixing. It should reject speculative fixes when no
reproduction, trace, fixture, or instrumentation path exists.

Expected failure mode: the agent proposes a likely code change from symptoms
alone, writes a shallow regression at the wrong seam, or claims verification
without rerunning the original loop.
