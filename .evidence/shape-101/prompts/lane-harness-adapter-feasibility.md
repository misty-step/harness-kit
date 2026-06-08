Role: runtime adapter investigator.
Objective: evaluate the feasibility of ephemeral per-lane harness roots for
Claude, Codex, Pi, Antigravity, Cursor, and Grok in Harness Kit.
Scope: read-only. Inspect harnesses/* docs, bootstrap paths, .harness-kit/agents.yaml,
and crates/harness-kit-checks dispatch/bootstrap code as needed.
Focus: how to hide the full global skill catalog from a dispatched lane while
preserving auth and keeping provider CLIs thin.
Boundaries: do not edit files. Do not browse. Do not propose a scheduler.
Output <= 900 words with sections: Adapter facts, MVP target, Auth risks,
Provider-specific fallback, Test strategy, Keep/remove/defer.
