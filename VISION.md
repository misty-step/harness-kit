# Harness Kit Vision

Harness Kit exists to make agent work better by making the harness explicit,
portable, and empirically improvable.

It started with a practical irritation: Codex, Claude Code, Pi, Antigravity,
and the next agent all look for instructions, skills, configs, and safety
rules in different places. Harness Kit gives them one shared source of truth:
one doctrine, one skill catalog, one provider roster, one local gate, one way
to leave receipts.

That origin still matters. The project should feel easy, simple, current, and
sharp. A senior operator should be able to install it, improve a primitive,
delete a stale one, re-bootstrap, and immediately feel the harness get better
across every agent they actually use.

The larger bet is that high-quality agent work can be made scientific. Harness
changes should not survive because they sound wise, match a favored model, or
feel productive in one session. They should survive because matched tasks,
blinded comparisons, deterministic checks, model judges, and human judgment
show that they produce better artifacts, better decisions, safer edits, faster
loops, or more trustworthy evidence.

Harness Kit is the place where the operator harness stays alive enough to test
that bet.

## What Must Stay True

- **One source, many harnesses.** The source repo owns the primitives; bootstrap
  projects them into whatever agent surfaces exist today. Harness-specific
  behavior is an adapter, not a second truth.
- **Thin harness, strong models.** Do not build a semantic workflow engine
  around provider CLIs. Load judgment into capable models; let models do model
  work; keep deterministic code responsible for gates, receipts, bootstrap,
  redaction, and proof.
- **Supervised first.** Harness Kit is the ad-hoc operator layer. It should
  make human-supervised work excellent before it claims unattended autonomy.
  Event-driven and unsupervised loops belong in the Mode B plane until a
  shaped boundary proves otherwise.
- **Evidence beats taste.** Skills, doctrine, provider defaults, and local
  workflows earn their place through telemetry, evals, live QA, fresh-context
  critique, and user outcomes. A beautiful instruction with no effect is
  baggage.
- **Bleeding edge, not brittle.** The catalog should track the best current
  agent surfaces, models, and practices, while staying easy to rollback, prune,
  and re-audit after model shifts.
- **Delete as progress.** Stale skills, duplicated prose, obsolete harness
  bridges, and gates that only enforce ceremony should disappear. The best
  harness is smaller after learning.
- **Operator substrate, not buyer theater.** Harness Kit is for technical
  operators who can work with repos, gates, skills, and traces. Client-facing
  packaging, procurement stories, admin control planes, and spend governance
  can use Harness Kit underneath, but they are not this repo.

## The Open Question

The right harness may be one elegant global operating system. It may be a
family of harnesses by agent, substrate, and supervision level. It may be
just-in-time, task-specific harness generation that beats any static catalog.

Harness Kit should not pretend to know the answer. It should make the
experiments cheap enough, rigorous enough, and visible enough that the answer
emerges from use.

## Near Term

Keep the shared harness excellent: source skills, shared `AGENTS.md`, provider
roster, bootstrap, local gate, docs, and receipts. Make it easy to use across
Codex, Claude Code, Pi, Antigravity, OpenCode, Goose, and whatever earns a
place next.

Cut the catalog hard. Improve the hot primitives. Delete the dead ones. Keep
repo-local specialization exceptional unless evidence says it wins.

## Medium Term

Build the evidence loop. Run matched evals across raw agent runs, Harness Kit
primitives, and credible alternatives. Compare Codex to Claude to OMP to open
model harnesses. Compare model families. Compare task families. Grade with
deterministic checks first, then model judges, then human preference when the
claim is subjective.

Use those results to decide what to keep, adapt, import, rewrite, or delete.

## Long Term

Harness Kit should become the always-current reference for what a high-quality
agent harness looks like in practice.

If the science says the future is a single portable harness, make that harness
excellent. If it says the future is bespoke harnesses generated per repo or
per task, make Harness Kit the source of the primitives, evals, and taste that
generate them. If supervised and unsupervised agents need different harness
families, make the boundary explicit and prove each family on its own terms.

The goal is not to own agent work. The goal is to make agent work observable,
comparable, and steadily better.
