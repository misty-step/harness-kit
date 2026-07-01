# Harness Kit Vision

Harness Kit exists to make agent work explicit, portable, and empirically
improvable across fast-changing agent surfaces.

It is the version-controlled primitive layer for agent work: a pile of skills,
shared `AGENTS.md` doctrine, references, provider and harness configuration,
bootstrap/install logic, receipts, gates, and eval contracts. Codex, Claude
Code, OMP, OpenCode, Pi, Antigravity, Goose, and whatever earns a place next
should not each require a separate philosophy and maintenance loop. Improve the
primitive once, sync it in whole or subset into system and repo harnesses, and
every useful harness gets sharper.

Harness Kit is operator-first. It is built for this operator's workflows,
preferences, tools, standards, and taste. It should be general enough that
another serious agent operator can pull it, understand it, adapt it, and benefit
from it, but it does not need to pretend to be neutral infrastructure. Favorite
models, CLIs, stacks, providers, patterns, and operating doctrines are allowed
to show up when they are real advantages. Avoid machine-private paths and
unportable assumptions; do not sand off the opinionated edge that makes the
harness useful.

## The Bet

Agent tools move too quickly for each harness to become its own hand-maintained
world. The stable layer is not the vendor UI. The stable layer is the set of
primitives that tell agents what good work looks like, what proof is required,
which tools are worth using, how to delegate, how to stop, and how to leave
evidence.

The near-term product is the skill pile and easy sync layer for supervised agent
work. In the Weave, Harness Kit is not the compute plane or work app; it is the
source of agent primitives that those tools install, subset, evaluate, and hand
to domain agents. The central next bet is orchestration: master agents that can
manage projects, dispatch bespoke subagents, assemble ad hoc harnesses, and
coordinate large or specialized agent teams without turning the operator into a
babysitter.

Harness Kit should move toward that world without prematurely hardening the
wrong architecture. It may remain a shared global primitive catalog, but that
cannot stay the only install shape. It should become a source of primitive
buckets for specific roles and repo-local domain agents. It may become the
doctrine and component library a master agent uses to generate task-specific
harnesses. It may split primitives by supervision level, task family, or agent
role. The project should preserve optionality while making the orchestration
experiments concrete enough to learn from.

## What Must Stay True

- **One source, many harnesses.** The source repo owns the primitives;
  bootstrap projects all or selected subsets into the agent surfaces that
  matter. Harness-specific behavior is an adapter, not a second truth.
- **Operator-bespoke, not machine-private.** Lean into this operator's chosen
  tools, vendors, stacks, and doctrine. Do not bake in private local paths,
  accidental machine state, or assumptions that make the primitives impossible
  to reuse elsewhere.
- **Primitives before platforms.** Most durable value lives in doctrine, skills,
  references, gates, receipts, eval definitions, and dispatch contracts. Do not
  build a semantic workflow engine around provider CLIs just because the harness
  has enough shell access to try.
- **Master-agent ready.** New primitives should be readable by a lead agent
  that frames work, dispatches lanes, compares evidence, and decides. Skills
  should support role-specific and ad hoc harness assembly, not only one human
  manually invoking slash commands.
- **Subsettable by design.** The whole catalog stays available, but the default
  direction is smaller role and repo bundles. A critic, designer, implementer,
  or project-specific QA agent should load the skills it needs, not every skill
  Harness Kit has ever seen.
- **Evidence beats taste.** Skills, doctrine, provider defaults, orchestration
  patterns, and model choices earn their place through telemetry, evals,
  benchmarks, fresh-context critique, live QA, and user outcomes. Durable
  first-party skills ship with an eval or an explicit waiver; vendored skills
  survive by use, routed reference, or measured win.
- **Delete as progress.** Stale skills, duplicated prose, obsolete harness
  bridges, and gates that only enforce ceremony should disappear. The best
  harness is often smaller after learning.
- **Operator substrate, not buyer theater.** Harness Kit is for technical
  operators who can work with repos, gates, skills, traces, and agentic
  workflows. Client-facing packaging, procurement stories, admin control planes,
  and spend governance can use Harness Kit underneath, but they are not this
  repo.

## Evals And Benchmarks

Harness Kit should not be a place where plausible agent doctrine wins by
sounding wise. It needs real evidence about whether a primitive improves agent
outcomes, by how much, under what task families, and at what cost.

The local responsibility is a compact eval contract and enough machinery to run
or consume evals when Harness Kit changes meaningfully: task definition,
context, transcript or receipt, outcome, deterministic graders where possible,
rubric/model/human graders where judgment matters, baselines, variance notes,
and decision labels such as keep, adapt, import, delete, or needs more tasks.

Serious repeated eval design and run management may belong outside this repo.
A dedicated eval product or lab surface, possibly `crucible`, can own designing
evals, defining task sets, running model comparisons, surfacing human-judgment
queues in a delightful UI, and publishing or iterating on results. Project-
specific evals should often live with the project that cares about them. Harness
Kit should be able to import, export, run, or consume those evals when its
primitives are under test.

The local eval bar is pragmatic: every first-party skill needs a falsifiable
claim, at least one run or seeded eval path, and a cadence for model-upgrade
re-audit. New skills should start with the eval scaffold rather than inheriting
trust from the catalog. External skills remain recoverable from upstream; if
telemetry and references do not justify default loading, they should leave the
active sync set.

Daedalus has a different job if that split holds: optimization over agent
configurations. Given an eval or task specification, Daedalus can vary prompts,
skills, available tools, subagent topology, models, reasoning budgets, runtime
budgets, and other knobs to search for better agent configurations. Harness Kit
supplies primitives and contracts; eval tooling measures outcomes; Daedalus
searches the configuration space.

## Near Term

Keep the shared harness excellent: source skills, shared `AGENTS.md`, provider
roster, bootstrap, local gate, docs, receipts, and backlog discipline. Make it
easy to use across Codex, Claude Code, OMP, OpenCode, Pi, Antigravity, Goose,
and whatever earns a place next.

The immediate product work is deletion and proof:

- cull zero-use, zero-reference vendored skills from the default catalog;
- wire skill evals into first-party skill creation and maintenance;
- fix telemetry blind spots for routed invocations;
- diet the checks crate back toward gates, bootstrap, generated artifacts, and
  repo maintenance;
- build role-scoped bootstrap bundles that target <=5k standing prompt tokens
  for common session roles;
- prove the one-core-many-faces template as greenfield-only by instantiating it
  in CI.

Push lightly but deliberately toward the master-orchestrator pattern:

- make lead-agent responsibilities crisp;
- make lane cards and receipts reliable enough for many-agent delegation;
- keep skills useful as role-specific primitive buckets;
- preserve enough structure for ad hoc harness generation;
- collect evidence about when orchestration actually beats one strong agent.

Active work and debt live in `backlog.d/`, with closed work archived under
`backlog.d/_done/`. Those files are how the vision turns into shaped,
reviewable experiments. When a backlog item changes a primitive, gate, provider
default, orchestration pattern, or doctrine line, it should name the proof
surface that decides whether the change survives.

## Medium Term

Build the evidence loop. Compare raw agent runs, Harness Kit primitives, and
credible alternatives on matched tasks. Compare model families and harness
surfaces. Compare single-agent work to lead-agent plus subagent work. Compare
static shared primitives to bespoke role harnesses. Grade with deterministic
checks first, then calibrated model or human judgment when the claim is
subjective.

Use those results to decide what to keep, adapt, import, rewrite, split, or
delete.

## Long Term

Harness Kit should become the always-current primitive source for excellent
agent-assisted work.

If the science says the future is a single portable harness, make that harness
excellent. If it says the future is bespoke harnesses generated per repo, role,
or task, make Harness Kit the source of the primitives, eval contracts, and taste
that generate them. If supervised work, event-driven loops, and autonomous
orchestrator swarms need different primitive families, make the boundaries
explicit and prove each family on its own terms.

The goal is not to own agent work. The goal is to make agent work observable,
comparable, steerable, and steadily better.

## Where The Depth Lives

- `AGENTS.md` is the repo operating contract and gate map.
- `harnesses/shared/AGENTS.md` is the shared cross-harness doctrine.
- `docs/positioning.md` defines the operator-substrate vs buyer-facing-package
  boundary.
- `meta/CONTRACTS.md` defines the Mode A/Mode B boundary, lane cards, receipts,
  backlog contracts, and evidence paths.
- `skills/harness-engineering/SKILL.md` defines the primitive test.
- `skills/harness-engineering/references/mode-eval.md` defines the local eval
  contract for agent-behavior claims.
- `backlog.d/` and `.harness-kit/traces/` hold active improvement work and
  receipt-grounded evidence.
