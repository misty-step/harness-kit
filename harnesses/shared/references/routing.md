# Routing Reference

Concrete trigger -> action guidance lives here so shared `AGENTS.md` stays
limited to always-applicable philosophy and fundamentals. Use this reference
when choosing a workflow route, delegation shape, search/research boundary, or
critic lens. Skills may point here, but should keep their own workflow-specific
judgment in the skill.

## Subagent Type

| Trigger | Action |
|---|---|
| More than three exploratory tool calls, unknown scope | Explore lane with an explicit question |
| Non-trivial architecture decision not yet shaped | `/shape`, or a plan lane with a scoped design question |
| Shaped ticket, acceptance criteria clear | `/deliver` |
| Fuzzy failure, root cause unknown | `/diagnose`, or an Explore lane with an explicit hypothesis |

## Delegate Or Go Solo

| Trigger | Action |
|---|---|
| Exploration, scoped search, small review | Native subagent, when the harness has one |
| Milestone or pre-merge critique of your own work | Fresh-context critic, different model family preferred (`/roster`) |
| Question a different model family answers better: second opinion, adversarial critique, competing attempt | Peer harness CLI (`/roster`) |
| Tens-to-hundreds of parallel agents, or findings that need adversarial cross-checking at scale | The harness's own large-scale orchestration feature when it has one; otherwise parallel subagents or a sprite fleet |
| Heavy, long-running, detached, or isolation-needing lane | Sprite lane via `/sprites` |
| Recurring event-driven workflow | Mode B: the event plane (bitterblossom), not ad-hoc Harness Kit |
| Mechanical command already chosen; emergency preservation; user forbids delegation | Direct solo |
| Need tool/permission isolation only | Static project subagent |

## Search Vs Research

| Trigger | Action |
|---|---|
| Need current repo truth: contracts, file content, skill definitions | `rg` / read the live file first |
| Need external ecosystem facts: libraries, CVEs, recent changes | `/research` |
| Need model/provider comparison | `/research` |
| Product premise depends on realtime, speech, vision, semantic intent, or model-native agents | Read `model-native-product-primitives.md`, verify current provider docs, then `/shape` the model boundary before implementation |

## Integration Shape

| Trigger | Action |
|---|---|
| Integrating an external system | Read `meta/INTEGRATION_GUIDE.md` before choosing MCP, skill, CLI, or script |
| Choosing language, host, CI, observability, release, design system, storage, or agent substrate | Start from `skills/harness-engineering/references/preferred-stack.md`, then verify current facts with `/research` before committing |

## Critic And Philosophy Lens

| Trigger | Action |
|---|---|
| Reviewing code you just wrote | Fresh critic lane: diff + oracle only, no author context |
| Module-depth / information-hiding concern | Ousterhout lens critique |
| Scope / shippability concern | Carmack lens |
| Complexity / abstraction-theater concern | Grug lens |
| TDD / test-shape concern | Beck or Cooper lens |
| A done claim that could embarrass production | Adversarial verifier: try to refute it |
