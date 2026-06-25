---
name: council
description: |
  Convene a council/thinktank: fan one question out to several DISTINCT
  high-quality OpenRouter model families (via opencode/pi), each carrying a
  different generative persona/perspective, then synthesize the divergent
  thinking as chair. Generative deliberation — brainstorm, explore the option
  space, weigh tradeoffs, decorrelated ideation, lock a contested direction.
  Distinct from /roster's adversarial critique bench (that finds bugs in an
  artifact; this generates and reframes). Use when: "convene a council",
  "thinktank", "council of models", "brainstorm with different models", "get
  diverse perspectives", "panel of AIs", "what would different experts think",
  "divergence pass", "ideate broadly", "stress-test this direction with other
  models". Trigger: /council, /thinktank.
argument-hint: "[question or task]"
---

# /council

You are the chair, not a participant. Convene a small bench of distinct model
families — each in a different generative persona — to think *divergently* about
one question, then synthesize. The point is decorrelated thinking the single
orchestrator can't produce alone.

This is the **generative** sibling of `/roster`. Roster's adversarial bench
reviews an artifact to find the bug (one lens each, artifact-only). A council
*generates*: options, framings, first-principles takes, the non-obvious move,
the real disagreement. Reuse roster's dispatch mechanics and model index; this
skill owns composition + synthesis.

## When to convene (and when not)

- Convene for: a contested design/direction call, a wide-open "what should we
  build / how should we approach this", a divergence pass before converging on
  a plan, a brainstorm where one model's taste shouldn't decide. The
  divergence-for-design-decisions mandate (≥4 structurally distinct voices on
  non-trivial architecture) is exactly this.
- Don't convene for: a factual lookup, a settled call, anything a native
  subagent or a moment's thought answers. A council costs real money (each lane
  is a paid inference run) and your own synthesis time. Options, not obligation.

## Compose the bench (the judgment)

Two axes, both must vary — decorrelation comes from family × lens:

1. **Distinct model families.** 4–6 members, each a *different* family
   (Kimi/Moonshot, DeepSeek, Qwen, GLM/Zhipu, MiniMax, …). Same-family variants
   don't decorrelate — a wide bench of one family is waste. Optionally add a
   frontier closed model on its own CLI (`codex`, `grok`, `agy`) for extra
   family spread.
   - **Slugs rot in days — never hardcode them.** Pull current top models live:
     `roster`'s `references/model-provider-harness-index.md`, or the **OpenRouter
     MCP** (`models-list`, `benchmarks`, `model-endpoints` for current quality +
     pricing). `pi --provider openrouter --list-models <family>` lists live slugs.
2. **Distinct generative personas.** One per member, pulling in different
   directions (builder vs simplifier vs user-advocate vs contrarian …). Library:
   `references/personas.md`. Compose a bespoke lens for the real question rather
   than forcing a stock role.

Scale to stakes: a quick divergence is 3 members; a load-bearing design call is
5–6 plus a native subagent or two on further lenses.

## Run it

`scripts/council.sh` fans the bench out in parallel, caps each lane, and
collects every output — failed lanes reported, not hidden.

```
# members.tsv:  label <TAB> cli <TAB> model <TAB> persona   ('#' comments ok)
scripts/council.sh --task /tmp/q.txt --members /tmp/members.tsv --outdir /tmp/council
```

Write the shared task to one file (inline ALL context — lanes run cold, no
shared history). Default `cli` is `opencode` (`opencode run --model
openrouter/<slug>`); `pi` is the lighter no-tools alternative. See `/roster` for
the exact headless forms.

Timeout judgment is part of composition. The script default is 1200s per lane
because strong reasoning models often need real wall time for cold, tool-using
architecture questions. Use shorter caps only for smoke checks, slug probes, or
cheap quick-divergence runs. For load-bearing design calls, keep the default or
raise `--timeout` to 1800s; if a lane hits the cap, report it as caller-capped
and rerun or exclude it before drawing model-quality conclusions.

## Synthesize as chair (the other half of the judgment)

Reading the lanes is the work — don't just paste them.

- **Surface the non-obvious.** The value is the idea or framing you wouldn't have
  reached alone, not the consensus.
- **Name the real disagreement.** Where lanes genuinely diverge is signal —
  present the live tension and your call, not an average.
- **Don't vote/tally.** N models agreeing is weak evidence (shared training);
  one model's sharp dissent can be the right answer. Weigh, don't count.
- **Own the result.** Council output is evidence; you decide and are
  accountable. A rambling or failed lane is a result too — say so.

## Gotchas

- **Monoculture = theater.** Same family across lanes, or all lanes on the same
  lens, produces correlated mush. Vary both axes or don't convene.
- **Cold lanes.** Members share nothing but the task file. Inline constraints,
  goal, and what "good" looks like into the task — a member missing context
  invents it.
- **Stale slugs.** A lane failing instantly is usually a dead/renamed slug or an
  auth lapse, not a verdict — re-check live (roster index / OpenRouter MCP) and
  re-run that lane.
- **Brainstorm ≠ converge in one shot.** Use the council to widen; then you (or
  a focused follow-up) narrow. Don't ask the bench to also pick the winner —
  that's the chair's job.
- **Timeouts are caller evidence.** A reasoning lane that hits `--timeout` was
  capped by the chair. Use short caps only intentionally, and don't blame the
  model for a cap chosen too low for the task.
- **Cost is real.** Bound the bench to the stakes; `--timeout` is still the
  runaway guard. The OpenRouter MCP `credits-get` shows remaining balance.

## Composes with

- `/roster` — dispatch mechanics, live model index, the adversarial-critique
  counterpart (use that to *review*, this to *generate*).
- `nous-creative-ideation` — a routed library of named ideation methods. Seed a
  member's persona with a specific method (OuLiPo, TRIZ, lateral provocations),
  or run the question through one first when the bench risks converging on the
  obvious.
- The OpenRouter MCP (user-scoped) — live model catalog, benchmarks, pricing,
  and balance for choosing the bench.
