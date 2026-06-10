# Skill Design Principles

Apply when `/harness-engineering` is improving first-party skills, creating a
new skill, or auditing the catalog after external skill-system research.

Source prompt: Anthropic's "Lessons from building Claude Code: How we use
skills" (2026-06-03), adapted to Harness Kit's cross-harness filesystem-first
contract.

## Translate External Principles

| Principle | Harness Kit rule |
|---|---|
| Skill is a folder | Prefer `references/`, `scripts/`, `examples/`, `assets/`, `templates/`, and `evals/` over long inline prose. |
| Clean category | Each skill owns one workflow category; multi-category skills compose other skills or split. |
| Verification skills matter | Verification behavior gets scripts, assertions, or evals before extra prose. |
| Do not state the obvious | Delete generic SWE advice unless it names a Harness Kit-specific failure mode. |
| Gotchas carry signal | Add gotchas from observed failures, receipts, audits, or failing gates; avoid speculative warnings. |
| Progressive disclosure | `SKILL.md` routes; references hold depth; scripts and assets hold repeatable mechanics. |
| Avoid railroading | Give constraints, choices, and oracles; do not force one procedure when repo evidence should choose. |
| Description is trigger classifier | Frontmatter must include concrete `Use when:` phrases plus `Trigger:` aliases. |
| Help the skill remember | Repeated workflows may use append-only JSONL, ledgers, or invocation data under approved state roots. |
| Store scripts | If the model would rebuild boilerplate twice, add a helper script or template. |
| On-demand hooks | Use skill-active hooks only for bounded, high-friction guardrails that would be annoying globally. |
| Distribution matters | Global first-party skills are default; repo-local skills are for substantial repo-specific context. |
| Compose explicitly | Name the owner skill instead of copying its method. |
| Measure | Use invocation and work-ledger data to find hot, cold, undertriggering, stale, and overlapping skills. |

## Upgrade Loop

1. Classify the target skill's single primary category.
2. Read live usage, recent receipts, active backlog, and failure evidence if
   available.
3. Delete generic instructions the model already knows.
4. Move detail into references/scripts/assets/templates when it is repeatable.
5. Tighten description triggers and aliases before changing body prose.
6. Convert any repeated gotcha into a script, hook, eval, or gate when feasible.
7. Run `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`
   and the full repo gate before shipment.

## Catalog-Wide Application

Start with machine-checkable hygiene before subjective rewrites:

- no missing `Trigger:` definitions;
- no trigger collisions;
- no stale local references in routes or examples;
- no skill over 500 lines without progressive disclosure;
- no substantial workflow skill without the shared roster floor pointer;
- no generated docs/index drift after a skill change.

Only then spend attention on taste: category fit, gotcha quality, excess prose,
or whether a workflow should split, merge, or compose.
