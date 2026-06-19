# /harness-engineering engineer

Design harness improvements: hooks, enforcement, context, codification.

## Codification hierarchy

When encoding knowledge, target the highest-leverage mechanism:

```
Type system > Lint rule > Hook > Test > CI > Skill > AGENTS.md > Memory
```

## The Design Test (Norman Principle)

For any harness component, apply the Norman test:

1. **Can an agent make this error?** — The harness allows it. Add prevention.
2. **Does the harness make this error likely?** — The harness induces it. Fix urgently.
3. **After an error, does the response fix the system?** — If not, you're teaching
   burner mappings. Redesign the stove.

Prevention hierarchy: Type system > Hook > Lint > Test > Skill > Prose.
Prose is the burner label. Hooks are the redesigned stove.

## Local CI

For Harness Kit itself, run the Rust-owned local gate:

```sh
cargo run --locked -p harness-kit-checks -- check --repo .
```

When adding gate coverage, put durable checks in `harness-kit-checks` first.
Do not make Dagger, Docker, GitHub Actions YAML, or provider CLIs the default
inner-loop gate for Harness Kit.

## Consumer repo gate velocity

When engineering CI defaults for other repos, encode a two-tier loop:

- **Inner loop:** local hooks run fast deterministic checks agents will actually
  tolerate during amend/push cycles.
- **Outer loop:** full Dagger/Docker/browser/network/live-readiness gates remain
  required before merge, main deploy, or an explicit ship command.

Slow pre-push gates are a harness defect when CI repeats the same expensive
proof. Fix by splitting `check-fast` from `ship-check`, adding stale-PR
concurrency cancellation, or giving Dagger a persistent/cloud engine cache.
Do not simply path-skip the only required workflow; skipped required GitHub
checks can leave a PR pending.

## Verification systems

When a harness change affects agent behavior, runtime behavior, generated
artifacts, or operator trust, load
`../../../harnesses/shared/references/verification-system-first.md` and name
the driver, grader, evidence packet, and cadence before editing. A new
primitive without a gate, eval, benchmark, QA path, smoke path, or probe is
unfinished.

## Hooks are the highest-leverage investment

Hooks run on every tool use. CLAUDE.md is read once. A hook that blocks
`rm -rf` is infinitely more reliable than a CLAUDE.md line saying
"don't delete files." Invest in hooks over prose.

Source of truth: `harnesses/claude/hooks/`

## AGENTS.md is a map, not a manual

Keep AGENTS.md under 100 lines. It should point to deeper sources of truth
(skills, references, docs/) rather than containing all instructions inline.
A monolithic AGENTS.md becomes a graveyard of stale rules.

## Stress-test assumptions

Every harness component encodes an assumption about model limitations.
When a new model drops, audit: is this skill still needed? Is this hook
still catching real problems? Strip what's not load-bearing.

## Thin harness default

Default to a thin harness:

- define agents, tools, prompts, and boundaries
- launch them
- capture raw artifacts
- optionally synthesize with another agent

Do not default to semantic workflow engines, regex recovery of agent structure,
or heavy handoff machinery. If the harness is reasoning about the repo or
recovering meaning from free-form agent prose, that is a strong smell.

## Workflow layering

When multiple skills touch the same delivery lane, enforce strict layering:

- **Leaf skills own one domain and are runnable standalone.** Examples:
  `/ci`, `/qa`, `/code-review`.
- **Composer skills orchestrate leaves around one bounded objective.**
  Example: `/deliver`.
- **Outer-loop / event workflows are Mode B** (bitterblossom), not new
  skills here — see `meta/CONTRACTS.md`.
- **Aliases are vocabulary, not new domains.** Do not add a skill when a
  trigger alias on an existing one covers the request.

Redundancy test:
- If a composer explains a leaf skill's internal methodology in detail, that is
  drift. The composer should invoke or reference the leaf, then add only the
  boundary judgment it owns.
- If two skills can both plausibly claim to be the authoritative owner of the
  same concern, the boundary is wrong. Pick one owner and make the other compose it.
