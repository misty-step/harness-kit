# Quality System

Use a quality system when the work is too meaningful for "do the task" to be
the whole prompt. The point is not ceremony; it is to make "good enough" visible
before execution and to keep proof independent of the author's confidence.

## Shape

Every quality system names:

- **Outcome:** the user, developer, or operator behavior that will be true.
- **Standards:** the local project properties that define good work here.
- **Proof methods:** distinct evidence sources that can refute the done claim.
- **Critic topology:** who reviews what artifact, through which lens, at what
  risk tier.
- **Stop rules:** conditions that halt execution instead of improvising.

Good systems are artifact-backed. Bad systems are vibes with more bullets.
For the proof-loop contract, load
`harnesses/shared/references/verification-system-first.md`.

## Proof Methods

Prefer independent proof families for substantive work:

| proof | answers | examples |
|---|---|---|
| Live oracle | Does the changed behavior work in reality? | browser walk, CLI run, request replay, migration dry run, consumer build |
| Structural gate | Did repo-owned automation catch regressions? | tests, lint, typecheck, harness gate, generated-doc drift check |
| Eval / benchmark | Does the agent/model/performance claim survive held-out tasks or workload pressure? | skill eval, raw-vs-harness comparison, latency/throughput benchmark |
| Fresh critic | Did a cold reviewer refute the claim? | diff + oracle review, design critique, security lens, architecture lens |

Independence matters. Three commands that exercise the same unit test boundary
are one proof method. A critic who inherited the author's reasoning is not fresh.

## Risk Tiers

| tier | when | minimum quality system |
|---|---|---|
| Tiny | mechanical, single-file, low-blast-radius change | Structural gate or exact manual inspection; name why no fresh critic is needed. |
| Substantive | product behavior, shared code, harness primitive, generated output, or multi-file diff | Live oracle + structural gate + one fresh-context critic aimed at the likely production embarrassment. |
| High-stakes | security, data loss, auth, migrations, architecture boundary, public launch surface, or hard-to-rollback infra | Live oracle + structural gate + multiple distinct critic lenses or model families; re-review blocker fixes. |
| Event-driven | PR-ready review, incident response, scheduled audits, or recurring production checks | Mode B / bitterblossom owns orchestration; Harness Kit supplies the packet and lane contracts. |

Scale down aggressively when the work is small. Scale up only when the failure
would be expensive, public, irreversible, or hard to observe.

Fast examples:

- **Tiny:** one-file wording fix, generated copy refresh, or dependency metadata
  update with no behavior path.
- **Substantive:** skill behavior change, shared helper change, user-facing UI
  adjustment, or generated artifact contract change.
- **High-stakes:** auth, secrets, migrations, destructive file operations,
  release automation, or public launch claims.
- **Event-driven:** recurring PR review, scheduled audit, incident response, or
  production monitor.

## Critic Topology

Do not simulate a council inside one context. Separate execution from critique.

- **Artifact-only:** critics get the diff, plan, oracle, screenshots, logs, or
  rendered surface. They do not get the author's reasoning trail.
- **Lens-specific:** one critic hunts one class of failure. Use
  `harnesses/shared/references/lenses.md` for compact rubrics.
- **Decorrelated when it matters:** a different model family is most useful for
  adversarial review of substantive author-written work.
- **Lead-owned synthesis:** reviewer output is evidence, not authority. The
  lead fixes, rejects with reason, tickets, or escalates.

## HTML Plan Slot

For non-trivial work, the plan's first viewport should show the quality system
without requiring chat context:

1. Target outcome and chosen design.
2. Standards that define success for this repo and task.
3. Proof methods and exact evidence surfaces.
4. Stop rules and review focus.

Support sections can then carry alternatives, acceptance, verification detail,
communication cadence, risks, and reviewer instructions.

## Failure Modes

- **Checklist theater:** fields filled plausibly but no live path exercised.
- **Monoculture:** several same-context reviewers agreeing with the author.
- **Over-review:** wide benches for tiny diffs, wasting tokens and attention.
- **Late review:** foundational mistakes found only after a big-bang build.
- **Vendor myth:** encoding a model brand instead of the capability needed.
