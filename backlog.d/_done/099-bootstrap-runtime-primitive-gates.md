# Bootstrap and runtime primitive projection gates

Priority: high
Status: done
Estimate: M

## Problem

Harness Kit treats bootstrap projections, shared doctrine, runtime hooks, and
provider receipts as first-class primitives, but the current gate coverage is
uneven. Existing checks prove pieces of the shape, not the whole projection
contract across Claude, Codex, Pi, and Antigravity.

Provider audit evidence pointed at three concrete risks:

- bootstrap tests focus on agent allowlists, not full harness projections;
- runtime hook behavior is Claude-centered while the doctrine is cross-harness;
- skill invocation and provider receipt evidence can look structurally valid
  without proving that the runtime capture or smoke output is real.

## Goal

Add script-backed gates that prove runtime primitives are projected and
reported honestly across supported harnesses.

## Candidate Sequence

1. Add a bootstrap projection test that installs into temp harness homes and
   asserts the expected shared `AGENTS.md`, skill links, agent links, Claude
   hooks/settings, Codex config, Pi settings, and Antigravity shared guidance.
2. Add a runtime hook/settings check that validates committed Claude hook tests
   run and every settings hook target exists after bootstrap.
3. Tighten skill-invocation validation so `(harness, source_protocol)` pairs
   distinguish live hook capture from imported or unsupported data.
4. Add optional provider smoke-output sentinels for roster dispatch receipts
   without requiring networked model calls in Dagger.

## Acceptance Oracle

- [x] A new or extended bootstrap test fails if a projected harness is missing
      shared doctrine/config that bootstrap claims to install.
- [x] A hook/settings gate fails on stale hook targets.
- [x] Skill invocation fixtures reject aspirational live-hook records for
      unsupported harness/protocol pairs.
- [x] Provider smoke receipts distinguish process exit from output compliance.
- [x] `dagger call check --source=.` includes the new or extended gates.

## Non-Goals

- Do not port Claude hooks blindly into Codex, Pi, or Antigravity without a
  stable event contract.
- Do not add live provider dispatch or network calls to the main Dagger gate.
- Do not build a semantic workflow engine around provider CLIs.

## Related

- `bootstrap.sh`
- `scripts/test-bootstrap-agent-allowlist.sh`
- `scripts/check-agent-roster.py`
- `scripts/analyze-skill-invocations.py`
- `scripts/lib/agent_roster.py`
