Role: failure-mode designer.
Objective: design graceful degradation for lane-harness dispatch when a selected
provider is out of credits, unauthenticated, missing, times out, or cannot run
with a projected home/config root.
Scope: read-only. Inspect agent_roster dispatch behavior, receipt schema, and
provider roster docs if useful.
Boundaries: do not edit files. Do not browse. Keep provider CLIs as tools and
avoid semantic scheduling.
Output <= 900 words with sections: Failure taxonomy, What dispatch should do,
What the lead should do, Receipt fields/evidence, What not to automate, Tests.
