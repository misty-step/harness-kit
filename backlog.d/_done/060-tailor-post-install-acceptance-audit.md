# `/tailor` post-install acceptance audit

Priority: P0
Status: done
Estimate: M
Shipped: 2026-05-21

## Problem

`/tailor` can produce a structurally valid harness that is still not bespoke.
The Spellbook self-tailor run exposed the failure mode: green deterministic
checks missed stale gate counts, hardcoded checkout paths, missing
always-install `/trace`, deploy-lifecycle leakage, stale repo-brief debt, and
workflow skills that were effectively copied instead of rewritten.

The fix is not deterministic "tailoring depth" scoring. Bespoke fit is a
judgment problem. Deterministic code should gather facts for reviewers; named
subagents should decide whether the installed harness actually fits the repo.

## Goal

Add a final `/tailor` phase that runs after install:

```
collect evidence -> subagent acceptance audit -> repair blockers -> re-audit
```

`/tailor` may not declare success until the acceptance audit passes or records
an explicit user-approved residual gap. The default path runs this phase; an
explicit `/tailor audit` rerun mode may reuse it against an already-installed
harness.

## Design

### 1. Deterministic evidence collector

Add `skills/tailor/scripts/collect-post-tailor-evidence.py`.

Inputs:

- target repo root
- resolved Spellbook root
- shared skill root (`.agent/skills` or `.agents/skills`)
- installed agent root
- repo brief path

Output:

- `.spellbook/tailor/audit/<run-id>/evidence.json`
- optional sibling artifacts such as `diffs/*.diff`, `grep/*.txt`,
  `readlinks.txt`

The collector has no semantic verdict. It exits nonzero only when it cannot
collect evidence. It records objective facts:

- installed skills, categories, markers, source names, and byte identity
  against Spellbook sources
- always-install presence, including `/trace`
- per-harness bridge topology and broken symlinks
- external-skill symlink targets plus sibling markers
- installed agent refs mentioned by skills and whether the agents resolve
- portable-path grep hits such as `/Users/<name>/`
- stale lifecycle vocabulary hits, especially deploy terms in repos whose
  brief says no deploy target
- gate phrase inventory across `/ci`, `/implement`, `/deliver`, `/settle`,
  `/ship`, `/qa`, `/monitor`, and `AGENTS.md`
- lifecycle fact table for `/groom`, `/ship`, `/settle`, `/trace`,
  `/flywheel`, `/implement`, and `/deliver`: active tracker, closed tracker,
  closure signal, archive operation, work-record store, detector command
- repo-brief freshness facts: generated date, active backlog count, debt IDs,
  branch/base branch, load-bearing gate
- AGENTS.md known-debt IDs and `(unfiled)` hits
- shared script presence and sync status for `scripts/lib/backlog.sh` and
  `scripts/lib/verdicts.sh`

The evidence schema should be small and typed enough that reviewers do not
parse the filesystem repeatedly. It is a fact packet, not a manifest to own
future state.

### 2. Subagent acceptance audit

After evidence collection, `/tailor` dispatches a mandatory acceptance critic
with:

- `.spellbook/repo-brief.md`
- `AGENTS.md`
- the evidence packet and artifact directory
- rewritten workflow `SKILL.md` files
- source Spellbook workflow paths for comparison
- the planner's install/rewrite brief when available

The critic reviews the installed harness as a set. It returns:

```json
{
  "status": "pass | fail",
  "blockers": [
    {
      "kind": "depth | cohesion | contract | install | stale-context",
      "path": ".agents/skills/monitor/SKILL.md",
      "evidence_ref": ".spellbook/tailor/audit/<run-id>/grep/lifecycle.txt",
      "finding": "Monitor still treats /deploy receipts as the primary input despite repo brief saying no deploy target.",
      "repair": "Rewrite monitor around CI, index drift, skill eval drift, bootstrap/symlink drift, and agent-session audit trails."
    }
  ],
  "nonblocking": [],
  "residual_risk": []
}
```

Persist that judgment to
`.spellbook/tailor/audit/<run-id>/verdict.json`. This is a run receipt, not a
future ownership manifest. It records only the reviewer decision, blockers,
evidence refs, repair directives, residual risk, and reviewer identities. A
rerun creates a new audit directory; it does not mutate prior verdicts.

No numeric score, percentage, LLM-output regex, or deterministic quality
threshold is allowed. Byte-identical workflow content is sufficient evidence
for the critic to fail; it is never sufficient evidence to pass.

Route additional reviewers when evidence suggests their lens:

- `ousterhout` for lifecycle/gate cohesion, pass-through rewrites, and shallow
  module boundaries
- `carmack` for scope bloat, speculative domain skills, or heavyweight audit
  machinery
- `beck` for test/eval contract drift or TDD-surface mismatches

The lead agent repairs only blocking findings, reruns the collector, and
re-dispatches critique. Limit to three rounds before surfacing the conflict to
the user; do not silently ship a failed acceptance audit.

### 3. `/tailor` contract changes

Replace the prose-only self-audit ending with two explicit contracts:

- **Objective self-audit:** deterministic collector gathers facts and catches
  collection/tooling failure.
- **Acceptance audit:** subagents judge bespoke fit from those facts and the
  repo context.

The existing seven self-audit checks become collector dimensions or critic
prompts. They should not remain as free-floating orchestration prose.

## Oracle

- [ ] `skills/tailor/scripts/collect-post-tailor-evidence.py` writes a stable
      evidence packet for a target repo without producing a quality verdict.
- [ ] `/tailor` runs evidence collection after install and includes the artifact
      path in its final report.
- [ ] `/tailor` dispatches a mandatory post-install acceptance critic before
      declaring success.
- [ ] Critic output is pass/fail with blocking findings, evidence refs, and
      repair directives; no numeric score or deterministic semantic threshold.
- [ ] Critic output is persisted to
      `.spellbook/tailor/audit/<run-id>/verdict.json`; `/tailor audit` reruns
      create a new audit directory instead of mutating old verdicts.
- [ ] A fixture or eval case with stale gate count, hardcoded user path, missing
      `/trace`, deploy lifecycle leakage, stale debt map, and byte-identical
      workflow content fails the acceptance audit with concrete blockers.
- [ ] A valid tailored fixture passes with zero blockers while still preserving
      the evidence packet.
- [ ] The collector is covered by focused tests for stable JSON shape, broken
      symlink capture, byte-identity detection, lifecycle fact extraction, and
      portable-path hit capture.
- [ ] `dagger call check --source=.` green.

## Non-Goals

- No deterministic bespoke-quality score.
- No heavyweight owned manifest or rollback system.
- No new standalone `audit` skill.
- No CI gate for every tailored downstream repo in v1.
- No broad replanning unless the critic finds a portfolio-level defect.

## Notes

This belongs inside `/tailor`, not `/harness audit`, because it judges
`/tailor`'s terminal artifact. A rerunnable `/tailor audit` mode is useful, but
the default tailor path must invoke the acceptance audit automatically.

The design is intentionally evidence-driven but not deterministic in its final
judgment: scripts reduce reviewer load and prevent stale-context blindness;
subagents decide whether the result is actually bespoke.
