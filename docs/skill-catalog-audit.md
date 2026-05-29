# Skill Catalog Audit

Date: 2026-05-28
Backlog: `backlog.d/069-sdlc-workflow-map-audit.md`

## Executive Verdict

The catalog is mostly carved correctly. Do not add a new top-level
instrumentation, observability, repo-QA-generator, or persona-acceptance skill
yet.

Keep `/create-repo-skill` broad as a thin generator router. Split a generator
only after a mode needs a different artifact contract, side-effect model,
oracle, or lifecycle. Repo QA and persona acceptance currently share the same
shape: live repo discovery, user/product truth, local skill output, eval seed,
critic lane, and repo-fit acceptance block.

The main gap is not a missing skill. It is a shared instrument/observe contract
across lifecycle phases.

## Natural Groups

| Group | Skills | Verdict |
|---|---|---|
| Backlog and framing | `/groom`, `/shape` | Right-sized. `/groom` is verbose but owns always-on backlog hygiene; `/shape` owns buildable packets. |
| Atomic implementation | `/implement`, `/hardening` | Right-sized. `/implement` should add observability only when the packet requires it. `/hardening` is a specialist, not the default path. |
| Verification and polish | `/code-review`, `/ci`, `/refactor`, `/qa`, `/settle` | Right-sized. `/deliver` composes them; it should not absorb their contracts. |
| Evidence and final mile | `/demo`, `/yeet`, `/ship` | Right-sized with naming pressure: `/yeet` packages and pushes; `/ship` lands and archives. Avoid aliases that blur "ship" with deploy. |
| Runtime follow-up | `/deploy`, `/monitor`, `/diagnose`, `/reflect` | Right-sized. `/monitor` observes and escalates; `/diagnose` investigates; `/reflect` turns evidence into doctrine/backlog changes. |
| Outer orchestration | `/deliver`, `/flywheel` | Acceptable. `/deliver` is one item to merge-ready. `/flywheel` is queue/outer-loop control. Keep both thin. |
| Harness/meta | `/harness-engineering`, `/create-repo-skill`, `/seed`, `/agent-readiness` | Mostly right-sized. `/create-repo-skill` remains separate because repo-local generation has a different target and safety model than global Harness Kit mutation. |
| Research/domain support | `/research`, `/model-research`, `/browser`, `/a11y`, `/design`, `/deps` | Useful, but `model-research` is the clearest future merge candidate into `/research` if trigger surface grows noisy. |
| Doctrine | `/karpathy-guidelines` | Static guidance, not a workflow. Candidate to fold into shared doctrine or `/reflect coach` later. |

## Lifecycle Table

| Phase | Owner | Inputs | Outputs | Evidence | Boundary |
|---|---|---|---|---|---|
| groom | `/groom` | Repo state, backlog, user intent | Prioritized/tidied backlog, narrowed next item | Updated backlog, rationale, receipts | Right-sized; too verbose. |
| shape | `/shape` | Raw idea or selected backlog item | Context packet with goal, non-goals, oracle, sequence | Packet plus explored repo anchors | Right-sized. Add observability expectations to packet when behavior must be watched. |
| implement | `/implement` | Context packet | Code/tests on branch | Failing-then-green tests, exact commands, clean tree | Right-sized. Does not own broad telemetry design. |
| instrument/observe | Shared contract | Changed behavior and acceptance risk | Observable surface, telemetry/evidence plan, or explicit debt | Logs/events/receipts/healthchecks named in packet, QA, monitor, or residual risk | Underrepresented. Do not make a new skill yet. |
| review | `/code-review` | Diff, base, oracle | Findings and verdict | File/line findings, tested claims, residual risk | Right-sized. |
| refactor | `/refactor` | Branch diff or scoped target | Simpler equivalent code | Diff reduction, gates, no behavior drift | Right-sized. |
| qa | `/qa` | Running target, route/command, feature claim | Pass/fail QA report | Live surface, command/URL/tool call, artifacts | Right-sized. Should verify observability surfaces when touched. |
| demo | `/demo` | Changed behavior and audience | Demo artifact or "no demo needed" receipt | Screenshot/video/blurb/release note path | Right-sized. |
| yeet | `/yeet` | Local changes | Semantic commits, push, PR/draft as requested | Commit list, pushed branch/PR, clean tree | Right-sized. Keep separate from `/ship`. |
| settle | `/settle` | Feature branch | Merge-ready branch | CI/review/refactor/QA loop receipts | Slight overlap with `/deliver`, but useful after branch work exists. |
| ship | `/ship` | Merge-ready branch and evidence | Landed change, archived backlog, reflect packet | Merge/commit refs, backlog move, post-ship evidence | Right-sized. |
| monitor | `/monitor` | Deploy/release/local run signal target | Watch report or escalation | Health/log/CI/telemetry/readiness signals | Right-sized. Observes, does not diagnose. |
| reflect | `/reflect` | Evidence, failures, receipts, operator notes | Doctrine/backlog/skill mutation proposal | Retrospective, accepted lessons, follow-up refs | Right-sized. |

## Instrument/Observe Ownership

Make instrument/observe a lifecycle invariant, not a new skill.

- `/shape`: names behavior that must be observable and the expected signal.
- `/implement`: adds instrumentation only when the shaped packet or changed
  production behavior requires it.
- `/qa`: proves the observable surface works when it is part of the change.
- `/monitor`: watches post-ship signals and flags missing observability as debt.
- `/reflect` and `/groom`: convert repeated gaps into backlog or harness changes.
- `/seed`: carries repo-local monitor guidance only when vendoring requires it.

Creating `/observe` now would add a semantic wrapper around logging,
telemetry, receipts, and monitor config. That conflicts with the thin-harness
rule until the existing phases fail to carry the contract.

## Real Cycle Checks

Harness Kit cycle: `7ee538c feat(harness): add repo skill generator`.

- Covered: harness mutation, repo-skill generator, QA scaffold, generated docs,
  index drift, full Dagger gate, commit closeout.
- Lifecycle fit: `/harness-engineering` and `/create-repo-skill` were the right
  meta layer; provider review found the broad generator shape stronger than
  narrow repo-QA-only skills.
- Gap: the cycle surfaced description-size drift in `skills/browser/SKILL.md`
  through the Codex loader, not the repo gate. Description terseness needs a
  visible audit/check surface.

Downstream app cycle: Misty Step `86b9007 docs: add review kickoff packet`.

- Covered: product offer, proof artifact, sales workflow, human-send stop rule,
  and live-route evidence in `marketing/sales/TODAY.md`.
- Lifecycle fit: this was not mostly implementation. `/shape`, `/demo`, `/qa`,
  `/monitor`, and `/reflect` concepts mattered more than `/implement`.
- Gap: downstream repos need repo-local persona/QA skills when the acceptance
  surface is fuzzy: target buyer workflow, human approval, route evidence, and
  expected friction. That supports broad `/create-repo-skill` with modes, not
  separate global generator skills.

## Duplication and Verbosity

Clear duplication:

- Delegation-floor boilerplate appears across many skills. Keep skill-specific
  lane shapes locally, but move repeated roster doctrine into shared guidance
  or a reference loaded by skills that need detail.
- `yeet`, `settle`, and `ship` all touch Git state. Preserve the split, but
  keep names and aliases sharp: package/push, polish-to-merge-ready, land/archive.
- `deliver` and `flywheel` are both composers. Preserve the split while
  resisting inline phase doctrine inside either one.

Whittle first:

- `skills/browser/SKILL.md` frontmatter description: exceeded loader limits.
- Long bodies: `groom`, `ship`, `settle`, `monitor`, `deliver`, `demo`,
  `diagnose`, `qa`, and `ci`.
- Long descriptions: `demo`, `design`, `ci`, `monitor`, `reflect`, `qa`,
  `deliver`, `agent-readiness`, and `karpathy-guidelines`.

Avoid immediate merges:

- Keep `/create-repo-skill` separate from `/harness-engineering`; source/global
  skill mutation and repo-local generation have different blast radius.
- Do not split `create-repo-skill qa` yet. The mode is a reference and trigger
  inside the same generator contract.
- Do not create `/observe` yet.

## Follow-Up Backlog Items

Use existing backlog rather than creating duplicate work:

1. `backlog.d/070-observability-coverage-loop.md`
   - Oracle: phase skills carry the shared instrument/observe contract; `/monitor`
     and `/seed` expose repo-local signal surfaces without adding `/observe`.
2. `backlog.d/053-skill-quality-audit-mode.md`
   - Oracle: `/groom audit` reports description size, trigger quality,
     body-length outliers, eval/test coverage, and routing reachability.
3. `backlog.d/083-clean-copied-skill-reference-quality.md`
   - Oracle: reference defects and stale examples are corrected before
     downstream `/seed` copies them into consumer repos.

## Provider Evidence

- `codex` lifecycle lane: accepted. Receipt
  `629723ff-a563-42d2-ac78-e25be98fd2ba`.
- `claude` duplication/terseness lane: partially accepted. Receipt
  `ce043681-18d6-4bb2-803a-486eb7d0c3c2`.

