# `/groom audit` — skill quality coverage audit

Priority: P2
Status: done
Estimate: M

## Goal

Add an `audit` mode to `/groom` that walks every skill under `skills/` and reports which are missing load-bearing quality gates: SKILL.md frontmatter, a resolver-reachable description, at least one test or eval reference, and a concrete trigger phrase. Output is a coverage report with severity ranking, not auto-fixes.

Inspired by Gary Tan's "Skillify" 10-step checklist; scoped down to the minimum set that actually predicts skill health in harness-kit.

## Non-Goals

- NOT auto-generating tests, evals, or fixes. Audit is read-only; humans decide what to promote.
- NOT the full 10-step Skillify checklist. Gary's version includes resolver triggers, LLM evals, DRY audits, E2E smoke, filing rules — half of those don't apply to harness-kit's architecture or are already covered by Dagger gates. This ticket is the subset that produces new signal.
- NOT a new Dagger gate. Audit is a /groom subcommand; the existing index-drift and frontmatter gates already enforce the minimum. This adds diagnostic depth, not another hard gate.
- NOT covering `agents/` — separate concern; agent quality has different criteria.

## Oracle

- [x] `/groom audit` prints a coverage report listing every skill with a pass/fail verdict on 4 dimensions:
  1. Has SKILL.md with required frontmatter (name, description, triggers)
  2. Description contains concrete trigger phrases (not just "Use this skill to...")
  3. Has at least one `tests/` directory, eval file, or explicit "Testing / verification" section in SKILL.md
  4. Is referenced from `harnesses/shared/AGENTS.md` routing (once 051 lands; until then, flag as soft warning)
- [x] Report is ordered by severity: skills failing 3+ dimensions first; skills failing 1 last.
- [x] Report is reproducible: running twice on unchanged source produces identical output.
- [x] Baseline run captured: audit output against current source is written to
      `.groom/audit-baseline-2026-06-02.txt` so future runs can diff.
- [x] `dagger call check --source=.` green.

## What Was Built

- Added `/groom audit` trigger and mode guidance to `skills/groom/SKILL.md`.
- Added `skills/groom/scripts/audit-skills.py`, a read-only report over all
  first-party skills.
- Added `skills/groom/scripts/test_audit_skills.sh` for ordering,
  reproducibility, and `--output` behavior.
- Captured `.groom/audit-baseline-2026-06-02.txt`.

## Notes

### Provenance

/groom session 2026-04-23 reading Gary Tan's "How to really stop your agents from making the same mistakes" (the Skillify pattern). Archaeologist investigator corroborated with evidence: only 25 of ~245 SKILL.md files mention "eval," "test," or "benchmark"; 84 test files scattered across skills but no uniform manifest. Coverage is real, sparse, and invisible to current gates.

### Why this ships, unlike audit-resolver

The groom synthesis dropped candidate #4 (skills/audit-resolver) because dark/unreachable skills aren't the load-bearing problem — the index gate already detects that. The load-bearing problem is **skill quality is uneven and invisible**: some skills have tests, evals, and clear triggers; others don't, and there's no surface where the operator sees the gap until a skill quietly fails in production. Audit mode gives that surface.

### Why `/groom audit` and not a standalone skill

`/groom` already operates on the backlog + skill layer. Adding a mode fits its existing surface. A standalone `skills/audit/` would be another entry point to maintain, and the first audit question ("what's the health of our skills?") is exactly a /groom concern.

### The 4 dimensions, why these

- **Frontmatter**: table stakes; already enforced but worth re-asserting at report time.
- **Triggers**: AuggieBench + Gary Tan both found skill invocation fails silently when descriptions lack concrete trigger phrases. This is the biggest observable-failure gap.
- **Tests/evals**: directly addresses the 25/245 coverage finding.
- **Routing reference**: once 051 lands, unrouted skills become dark skills. Pre-051 this is a soft warning; post-051 it's a real signal.

### Deliberate non-goals revisited

Resisting the pull to grow this into a 10-step quality system. Harness Kit's axe-sharpening exemption (it IS the axe) doesn't license building every possible audit; it licenses building audits whose output drives real skill promotion/demotion decisions. Four dimensions is enough to produce that output.

### Composition

- Soft dependency on 051 (AGENTS.md L3 routing) — the fourth dimension becomes hard signal once 051 lands.
- Independent of 052 (.harness-kit/ config).
- Should run after 051 so the routing-reference check produces useful output on first run.

### Execution sketch

One PR, two commits:
1. `feat(groom): add audit mode with 4-dimension coverage report` — extend `skills/groom/SKILL.md` + a small helper script under `skills/groom/scripts/` that walks `skills/` and emits the report.
2. `chore(groom): capture audit-baseline against current master` — write `.groom/audit-baseline-<date>.txt` for future diffs.
