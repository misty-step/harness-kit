# Internal Review Bench

Philosophy agents running on the same model as the marshal. Different lenses,
same model — useful for depth, not diversity. The marshal selects 3-5 based
on the diff and crafts tailored prompts for each.

All reviewers run as **Explore type** (read-only).

Each philosophy entry below is a **lens, not a static agent file**: read its
rubric from `harnesses/shared/references/lenses.md` and dispatch an ad-hoc
read-only critic embodying it (backlog 061, roles-not-files). Each returns
findings as `finding · evidence file:line · impact`.

## Agent Catalog

| Agent | Lens | Best For |
|-------|------|----------|
| **critic** | Grading rubric: correctness, depth, simplicity, craft | Every review — the baseline evaluator |
| **ousterhout** | Deep modules, information hiding, complexity management | API changes, module boundaries, abstractions |
| **grug** | Complexity hunting, over-abstraction, premature generality | Large diffs, new abstractions, framework-heavy code |
| **carmack** | Shippability, pragmatism, direct implementation | Feature work, performance-sensitive code |
| **beck** | TDD discipline, simple design, YAGNI | Test changes, untested code, test quality |

## Selection Heuristics

- **Always include critic** — the baseline scoring agent.
- **API/module changes** → ousterhout (module depth) + carmack (shippability)
- **Large diffs with new abstractions** → grug (complexity) + ousterhout (depth)
- **Test-heavy or untested code** → beck (TDD) + critic
- **Performance-sensitive** → carmack (direct implementation) + critic
- **Security-sensitive** → critic + define an ad-hoc security-focused agent

The marshal may also define **ad-hoc agents** with custom prompts for
concerns specific to the repo or diff that the named agents don't cover.

## Prompting

Tell each agent:
1. What to review (the diff scope)
2. What to focus on (their lens applied to this specific diff)
3. What verdict options they have: **Ship**, **Conditional**, **Don't Ship**
4. To cite file:line for every finding

The marshal crafts these prompts — this reference describes the lenses, not
the exact words.
