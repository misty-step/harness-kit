# Model-Capability Ledger

Operator standing directive (2026-07-05): failures are routing data. Every
material lane failure or standout gets distilled here — what the model tier
is actually good and bad at, and what the spec needed to say for a weaker
model to have succeeded. Update this file when an incident teaches something
about a TIER; keep per-incident detail in cards and traces, not here. Read
this before choosing a lane's model and before writing its brief; read
`prompting-frontier.md` for HOW to brief each tier.

## The routing rule the evidence keeps proving

Every 2026-07-04/05 failure was an **unwritten-invariant failure, not a
capability failure**: the CSP card never named the production Clerk domain;
the schema card never said expand→migrate→contract; the releases page read a
store no card mentioned. Weaker models fail exactly where the spec ends.
So: **dispatch strong models when the invariant set is unknown (discovery,
overhaul, root-cause); dispatch weaker models when the invariants are
written down — and write them down.** Spec specificity is not prose volume;
it is invariant coverage plus an oracle that exercises the live environment.

## GPT-5.5 / codex lanes (default implementation workhorse; xhigh is strong)

- **Good at:** faithful, high-volume spec execution. Same night it introduced
  the CSP gap it also cleanly shipped multi-card queues across linejam,
  crucible, and bb with green gates and honest reports.
- **Failure mode (twice in one day):** does not reach beyond the written card
  to *live-environment* invariants or *cross-deploy sequencing*. linejam
  PR #291 hand-listed Clerk CSP domains and missed the prod custom domain
  (preview env diverged from prod; gate couldn't see it). PR #298 contracted
  a Convex schema in the same deploy as its migration, wedging all
  production deploys (expand-migrate-contract was nowhere in the card).
- **Spec implication:** codex cards touching config/infra/schema seams must
  carry the environmental oracle explicitly ("verify against the LIVE prod
  config; enumerate env divergence between preview and prod") and the ops
  invariant by name. If you cannot write the invariant, the card is
  discovery work — route it up-tier instead.

## Sonnet 5 (Claude Code teammate lanes)

- **Good at:** verification discipline when the brief demands it — the
  powder-930 loop (law-gated Playwright, diff-only critic, live-bundle
  proof), the bastion-910 independent re-verification, and the canary-912
  catch of Landmark fabricating release notes were all Sonnet lanes holding
  an evidence bar under pressure. Runs multi-hour queues near-autonomously.
- **Failure modes:** silent stalls after assignment (thrice on 2026-07-04);
  absorbing surprises instead of surfacing them (found its card decomposed →
  idled wordlessly); independent verifiers converging on the same layer
  (two lanes both proved the server while the client was dead).
- **Spec implication:** briefs end with a concrete first-deliverable-before-
  idle; a surface-your-pivot clause; and verification *perspectives assigned
  explicitly* (one verifier stationed at the user boundary — mobile viewport,
  real click). With those three clauses, no Sonnet lane stalled.

## Fable (lead + rare high-effort lanes)

- **Good at:** cross-layer root-cause under ambiguity (the linejam outage
  chain: CSP → env divergence → deploy wedge → expand/migrate/contract dance,
  each layer verified live); noticing stale or wrong briefs instead of
  executing them (crucible-champion caught a compaction-stale card id and
  flagged before acting); groom-then-execute with discrepancy handling.
- **Use for:** deep cleanup/overhaul passes, incident command, grooming,
  anywhere the invariant set must be *discovered*. Brief per
  `prompting-frontier.md`: goal-not-steps, house rules, executable bar,
  builder-never-grades, status artifact.

## Small models in content pipelines (gpt-4o-mini via Landmark)

- **Failure mode:** fabricates confident structure from sparse input — one
  real PR in, invented "Breaking Changes" and "Bug Fixes" sections out; the
  structural quality gate scored it valid (landmark-907).
- **Implication:** claims-bearing output from small models requires a
  deterministic grounding gate (claim→source entailment), never an
  LLM-judge-only or structure-only check. Prompt instructions ("omit empty
  sections") are not a gate.

## Operator-noted lanes to smoke-test (2026-07-04)

Composer 2.5 via cursor-agent: operator-endorsed as a subagent lane. Grok
via Grok Build, Gemini 3.5 Flash via Antigravity: try on low-stakes lanes
and record the verdict here.
