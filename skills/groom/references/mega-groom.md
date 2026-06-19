# Mega Groom

Strategic `/groom` is an exhaustive project-quality sweep. It should feel
closer to a full product, architecture, ops, and agent-readiness offsite than
to issue triage.

## Mission

Boil the ocean, then distill it.

The output is a detailed plan for making the current project world class:
codebase, documentation, infrastructure, operations, product, system design,
architecture, value proposition, testing, security, and agent interface. The
backlog diff is the durable expression of that plan.

The plan starts from a project vision. If the repo has no durable vision
artifact, a strategic groom should emit one or shape the epic that will create
it; otherwise the sweep has no stable standard for deciding what belongs.

## Coverage Map

Every strategic groom accounts for these surfaces. Mark a surface `complete`,
`partial`, `failed`, or `skipped`; do not silently omit it.

| Surface | Question |
|---|---|
| Product and value prop | Who is this for, why would they care, what would make it indispensable? |
| Project vision | What durable north star guides brainstorming, backlog moves, deletions, and sequencing? |
| Operator/user experience | Can a new or returning user achieve the core job without friction? |
| Architecture and system design | Are the deep modules deep, the boundaries honest, and the interfaces small? |
| Code quality and simplicity | What can be deleted, flattened, clarified, or made harder to misuse? |
| Runtime reliability | What fails under load, restart, queue pressure, retries, or partial outages? |
| Security and privacy | Where can secrets, tokens, user data, metadata, logs, or authority leak or overreach? |
| Observability and ops | Can an operator see state, cost, queue pressure, incidents, and recovery actions? |
| Tests and verification | Do gates prove the live behavior that matters, not just units, and do they fail the likely security mistakes before publication? |
| Documentation and onboarding | Can a cold agent or human understand, run, extend, and debug the project? |
| Agent readiness | Are skills, AGENTS, lane cards, receipts, and CLI JSON surfaces first-class? |
| Infrastructure and delivery | Are deploys, rollbacks, backups, hosted checks, and local dev boring? |
| External exemplars | What do adjacent best-in-class systems prove or warn against? |

## Swarm Contract

Default strategic groom launches a swarm, not a single helper. Use the
harness's native subagents when explicitly allowed and available; otherwise use
peer CLIs or sprite lanes. If delegation is unavailable, run the same coverage
map locally and label the run degraded.

Minimum useful swarm for a normal repo:

- Product/value strategist.
- Project-vision editor.
- Operator/user-experience critic.
- Runtime reliability investigator.
- Security/privacy reviewer.
- Architecture/Ousterhout reviewer.
- Simplification/deletion reviewer.
- Test/verification reviewer.
- Docs/onboarding reviewer.
- Ops/infrastructure reviewer.
- Agent-readiness/harness reviewer.
- External exemplar scout.
- Premise challenger.

For small repos, lanes may be combined only when the same evidence answers both
questions. For important repos, add more lanes rather than fewer.

Each lane returns:

```markdown
## <Lane> Report
### Top Findings
1. <finding> -- Evidence: <file:line, command, URL, artifact>. Impact: high|med|low.
2. ...
### World-Class Delta
<What would have to be true for this surface to be excellent?>
### Backlog Move
<One epic, ticket, deletion, consolidation, or "no emission" with rationale.>
```

## Evidence Standard

Exhaustive does not mean sloppy.

- Cite live files, commands, URLs, artifacts, receipts, screenshots, or
  rendered surfaces.
- Label hypotheses as hypotheses.
- Verify each candidate emission against the repo before writing it.
- Prefer epics with children for coherent ambitions.
- Keep deletion and consolidation candidates visible even when not applied.
- Never use a backlog count as a veto.

## Output Shape

The final groom report should include:

1. **Source Matrix**: every surface/lane, status, evidence, and contribution.
2. **Project Vision**: canonical artifact read or proposed; audience,
   job-to-be-done, category, standards, non-goals, bets, and 6-12 month target.
3. **World-Class Target**: the best version of the project in concrete terms.
4. **Gap Map**: what live evidence says is missing or weak by surface.
5. **Verification Map**: missing or weak gates, QA paths, evals, benchmarks,
   probes, repo-local verification skills, and secret/content/metadata leak
   checks. Treat missing commit-message, outbound-range, PR-body, log, and
   generated-artifact scanning as agent-readiness gaps unless the repo has a
   stronger server-side control.
6. **Strategy Themes**: 4-8 themes, each with recommendation first and evidence
   second.
7. **Backlog Diff**: applied ticket edits/emissions and proposed deletions.
8. **Sequence**: now, next, later, and blocked.
9. **Best Next Pickup**: one concrete next issue and why it outranks the rest.
10. **Residual Risk**: skipped surfaces, failed lanes, missing credentials,
   uncertain external facts, and stale evidence.

## Emission Bar

A strategic groom that emits only a tiny issue set is incomplete unless the
user explicitly asked for narrow triage. A healthy mega groom usually produces
some mix of:

- A few P0/P1 safety or correctness moves.
- Several strategic epics with ordered children.
- Small ready tickets that remove immediate friction.
- Deletion/consolidation proposals.
- A world-class plan artifact when the repo needs shared direction.

Do not pad with low-value tickets. Do not stop at the first credible theme.
Keep searching until the coverage map has been honestly answered.

## Gotchas

- **Three-ticket trap.** Three issues can be the top of the list, not the
  groom. If only three survived, show the matrix that killed every other
  candidate.
- **Vision without receipt.** A beautiful plan with no file, command, URL, or
  artifact evidence is fan fiction.
- **Vision as wallpaper.** A vision that does not change what gets emitted,
  deleted, sequenced, or rejected is decoration, not strategy.
- **Swarm theater.** Many lanes with the same prompt are one lane wearing
  disguises. Compose perspectives for the repo.
- **Issue confetti.** Many tickets without sequencing or shared themes are
  storage, not strategy.
- **Local maxima.** Always ask what would make the project excellent in its
  category, not merely less broken.
