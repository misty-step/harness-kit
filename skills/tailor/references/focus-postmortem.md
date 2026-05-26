# Critic Checklist — /focus postmortem

The critic has two roles in `/tailor`:

1. **Pick review** — runs once after the planner proposes a primitive
   set. One round. No loop. Criteria 1-4 below, scoped by category.
2. **Rewrite review** — runs after workflow rewriters return their
   drafts. Reviews all rewrites together (not individually). Three
   checks: depth, cohesion, completeness. Up to 3 iterative rounds;
   critic sends specific rewriters back with specific objections and
   judges convergence.

This is the structural differentiator from `/focus`, which shipped
71 commits over two months (87 skills, 41 auditors) before being
killed in March 2026 for ceremony-to-value inversion. The criteria
below encode the failure modes that killed it, *scoped to the
categories where they apply*.

## Pick review

## Scope: which criteria apply to which category

`/tailor`'s picking defaults (see `SKILL.md`) distinguish three
categories: universal, workflow, domain. The critic applies
different criteria to each.

| Category | Default | Critic criteria |
|---|---|---|
| Universal (reflect, …) | **include when still useful verbatim** | 1 only |
| Workflow (deliver, ci, qa, groom, flywheel, …) | **include for SDLC coverage; tailor where commands/contracts vary** | 1, 2 |
| Domain (invented: `/convex-migrate` etc.) | **exclude unless concrete** | 1, 2, 3, 4 |

A workflow skill with a real lifecycle surface (tests, review, CI, release,
monitoring, backlog, or deploy) should land unless it would be empty ceremony.
Criterion 4 exists to reject speculative domain inventions, not workflow
coverage.

## 1. Does this duplicate a truly global skill?

**Reject if yes.** Only two skills are globally installed:
`/tailor` and `/seed`. A project-local copy of either is dead weight.

(Other primitives are *meant* to be installed per-repo under the
minimal-globals model. This criterion only flags the two globals.)

## 2. Would a scaffold-on-demand skill cover it?

**Reject if yes.** `/qa scaffold`, `/demo scaffold`, etc. generate
repo-specific scaffolding on invocation. Don't pre-bake their output
into a separate skill.

- Reject: a tailored `/qa` that hardcodes this repo's test golden
  paths (that's what `/qa scaffold` produces on demand)
- Allow: a `settings.local.json` permissions entry that lets `/qa
  scaffold` run without prompts

## 3. Is the domain set focused, or 41-auditor ceremony?

**Reject if invented domain skills sprawl** into near-duplicate
reviewers or kitchen-sink single skills. This criterion *does not
apply to the workflow category* — 12 workflow skills is not
ceremony, it's the inner and outer loops. It applies to domain
inventions only.

- Reject: four near-duplicate Rust reviewers (`unsafe-`, `lifetime-`,
  `trait-bound-`, `generic-reviewer`) when one would do
- Allow: one focused `rust-unsafe-reviewer` if the repo has unusual
  `unsafe` patterns that justify it

## 4. For domain inventions: can you name the concrete characteristic?

**Reject if no.** A proposed domain skill must correspond to a
concrete repo characteristic the planner can state in one sentence.
"This repo uses Convex with live-traffic schema migrations and
`v.optional` upgrade paths" → `/convex-migrate` makes sense.
"This might be useful someday" → reject.

This criterion does not apply to workflow skills — their
justification is already encoded in the default: present
infrastructure means the skill earns its place.

## Pick stopping condition

Critic emits a list of blocking objections (possibly empty). The
pick proceeds iff empty. No round 2. No appeals. User can re-run
`/tailor` with different inputs or adjust the planner's
instructions.

## Rewrite review

After the workflow rewriter subagents return their drafts, the
critic reviews all rewrites together — not individually. Reviewing
as a set is how you catch cohesion failures (one skill calls the
enforcing layer `pnpm ci:prepush`, another calls it
`pnpm ci:dagger:all`, a third doesn't mention it — each rewrite
is locally plausible, the aggregate is incoherent).

Three checks:

### A. Depth (subtractive test)

**Reject if no.** A rewritten SKILL.md should be *wrong* if applied
to another repo with the same stack. If its gotchas could describe
any Next.js+Convex repo, any Rust monorepo, any Python data
pipeline — it's find-and-replace, not tailoring.

- Reject: "Use your project's test command" rewritten to "Use `pnpm
  test`" and nothing else changed.
- Allow: "When a guest-flow test fails, check mutation ordering in
  `useGamePhase` before suspecting Convex — #146 was the last time
  a canary-observer race bit us there."

### B. Cohesion (repo brief alignment)

**Reject if a rewrite contradicts the repo brief or a sibling
rewrite.** The repo brief is the shared spine. Every gate-adjacent
skill cites the same load-bearing gate. Every debt reference uses
the repo brief's issue IDs. Terminology matches.

- Reject: `/ci` says the gate is `pnpm ci:prepush`, `/deliver` says
  the gate is Dagger, `/settle` says the gate is semantic-release.
- Allow: all three cite `pnpm ci:prepush` as the gate and describe
  their role within that gate's execution.

### C. Completeness (no byte-identical drops)

**Reject if the rewritten SKILL.md is byte-identical to its
spellbook source.** A workflow skill was treated as universal. The
rewriter dropped the ball.

## Rewrite stopping condition

Critic emits per-skill objection lists. Rewriters with non-empty
objections revise. Up to 3 rounds. Stop when all lists are empty
or the critic judges the remaining objections non-blocking. If
round 3 still has blocking objections, the lead agent decides
whether to ship with known gaps (noted in AGENTS.md debt map) or
surface the conflict to the user.
