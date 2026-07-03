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
| Verification skills matter | Verification behavior gets a real system: driver, grader, evidence packet, and cadence before extra prose. |
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

## Skill-Prose Compression

Source: Matt Pocock, "Writing Great Skills" (mattpocock/skills,
`writing-great-skills`) — vocabulary for the pruning discipline the Upgrade
Loop already demands.

- **Leading words.** A pretrained token the agent thinks with (*ratchet*,
  *mundane harvest*, *plausible-but-wrong*, *unknown unknowns*) anchors a
  region of behavior in the fewest tokens — and doubles as an invocation
  hook when it recurs in descriptions and prompts. Upgrade-loop move: hunt
  restatements ("fast, deterministic, low-overhead") that collapse into one
  word (*tight*). Fewer tokens and a sharper behavioral hook, at once.
- **No-op test, per sentence.** Does this sentence change behavior versus
  the model's default? No → delete the sentence, don't trim it. This is the
  per-sentence form of the standing gotcha that each new frontier model
  converts judgment prose into railroading.
- **Premature completion.** The named failure the oracle doctrine defends
  against: attention slips to *being done* before the completion criterion
  is met. Defense order: sharpen the criterion first (checkable,
  exhaustive); split the sequence only when the rush is actually observed.
- **Invocation cost accounting.** Every model-facing description pays
  context load every session — the description tax. A skill only ever fired
  by hand can drop its model-facing description in harness projections that
  support it (e.g. Claude Code `disable-model-invocation`); telemetry
  (`/groom audit`) names the candidates. Cross-harness caveat: this is a
  per-harness projection decision, never a source-skill deletion.

## Upgrade Loop

1. Classify the target skill's single primary category.
2. Read live usage, recent receipts, active backlog, and failure evidence if
   available.
3. Delete generic instructions the model already knows.
4. Move detail into references/scripts/assets/templates when it is repeatable.
5. Tighten description triggers and aliases before changing body prose.
6. Convert any repeated gotcha into a script, hook, eval, or gate when feasible.
   Use `../../../harnesses/shared/references/verification-system-first.md` for
   eval, benchmark, QA, or smoke-path design.
7. Run `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`
   and the full repo gate before shipment.

## New Skill: Eval Scaffold Is Not Optional

Every new first-party skill ships with eval coverage from the first commit,
not as a follow-up. The `check-eval-coverage` gate
(`cargo run --locked -p harness-kit-checks -- check-eval-coverage --repo .`,
folded into the repo gate) fails the build for any `skills/<name>/SKILL.md`
with no `skills/<name>/evals/*.md` and no live `skills/<name>/evals/WAIVER.md`
— so this step is enforced, not just recommended.

When scaffolding a new skill:

1. Copy `../../skill-eval/templates/eval-spec.md` to
   `skills/<name>/evals/<name>-eval.md` and fill in the one claim the skill
   must earn, 2–3 fixtures, objective checks, and a pass condition — even if
   the run itself hasn't happened yet (see `skills/orient/evals/orient-eval.md`
   for a seeded-but-unrun example).
2. If the skill's claim genuinely can't be evaled yet (external live
   dependency, taste-heavy rubric needing a human anchor, no fixture budget
   yet), write `skills/<name>/evals/WAIVER.md` instead: a reason plus an
   `expires: YYYY-MM-DD` line. See any file under `skills/*/evals/WAIVER.md`
   for the expected shape. A waiver is a time-boxed deferral, not a permanent
   opt-out — an expired waiver fails the gate exactly like a missing eval.
3. Never satisfy the gate with an empty or placeholder eval file just to pass
   it — that is worse than an honest waiver, because it looks covered to
   `/harness-engineering`'s health audits and to telemetry-driven pruning.

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
