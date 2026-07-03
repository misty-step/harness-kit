---
name: harness-engineering
description: |
  Harness engineering for Harness Kit primitives: skills, shared doctrine,
  provider roster, harness configs, gates, evals, bootstrap, and sync logic.
  Use for "improve the harness", "harness engineering", "bootstrap is wrong",
  "AGENTS.md is stale", "skill health", "skill usage", "undertriggering skill",
  "description tax", "eval skill", "sync primitives", "roster defaults",
  "preferred stack", "stack defaults", "hosting defaults", "CI defaults",
  "observability defaults", "release defaults", "design system defaults",
  "storage defaults", "agent substrate defaults", "one core many faces",
  "API CLI MCP SDK skill template", "factory product template",
  "generate repo-local skill", "bespoke skill subset", "domain agent skill".
  Trigger: /harness-engineering, /harness, /skill.
argument-hint: "[create|eval|lint|convert|sync|engineer|audit|models] [target]"
---

# /harness-engineering

Engineer the harness. Keep it thin.

## Route

| Need | Load |
|---|---|
| create skill or prompt | `references/mode-create.md` |
| eval skill | `references/mode-eval.md` |
| lint skill | `references/mode-lint.md` |
| apply skill-design lessons | `references/skill-design-principles.md` |
| clean skill catalog | `references/mode-audit.md` + `harness-kit-checks telemetry` |
| convert agent/skill | `references/mode-convert.md` |
| sync externals | `references/mode-sync.md` |
| engineer doctrine/gates/hooks | `references/mode-engineer.md` |
| measure skill usage/health/staleness | `references/mode-audit.md` |
| current model/provider/harness facts | the roster skill's `references/model-provider-harness-index.md` |
| open-model defaults | `references/open-model-roster.md` |
| preferred stack / infrastructure defaults | `references/preferred-stack.md` |
| factory app capability routing | `../factory-apps/SKILL.md` |
| one core / many faces product template | `references/one-core-many-faces.md` + `templates/one-core-many-faces/` |
| generate a focused repo-local skill for a domain agent | `references/repo-local-skill-generation.md` + `templates/repo-local-skill/` |
| public-surface "works" critique | `../../harnesses/shared/references/works-critique.md` |
| model-native product boundary | `../../harnesses/shared/references/model-native-product-primitives.md` |
| loop readiness / Mode B handoff | `../../harnesses/shared/references/loop-readiness.md` |
| verification system first | `../../harnesses/shared/references/verification-system-first.md` |
| delete-first simplification lens | `../../harnesses/shared/references/delete-first.md` |
| Ponytail anti-overengineering ladder | `../.external/dietrich-ponytail/SKILL.md` |

Repo-local skills for consumer repos (bespoke QA drivers, deploy runbooks,
persona probes) are written directly into that repo's `.agents/skills/`
with its real routes and commands; this skill owns the craft either way.
Process, provenance header, and eval-stub shape:
`references/repo-local-skill-generation.md` +
`templates/repo-local-skill/`. For a repo's verification skill, interview
the operator first: the manual checks they run after the agent responds and
before merge are the spec — encode each check that has a tool. Turning a
proven session pattern into a first-party primitive starts at the primitive
test below — most patterns are prompts, not skills.

## Contract

- Fix root cause in the highest-leverage layer: type/test/hook/gate/skill/
  AGENTS, in that order.
- Prefer deletion. Harness prose is context tax.
- Use Ponytail before adding or expanding primitives, provider layers,
  harness-specific accelerators, gates, wrappers, or skill prose. The lazy
  viable path has to lose on evidence before a larger harness mechanism earns
  its place.
- Cross-harness first means filesystem + `SKILL.md` portability. Prefer
  smoke-tested open-model peer lanes through Pi, Goose, and OpenCode on
  OpenRouter; keep Claude, Antigravity, Cursor, and Grok conditional unless
  their specific surface answers the task.
- Skills stay self-contained: scripts/libs/references under the skill; state
  roots from invoking repo.
- Code outside a skill serves only Harness Kit source-repo maintenance,
  generated artifacts, bootstrap/install, or harness configuration. It is not a
  place for skill behavior.
- Treat a skill as a folder, not a markdown file. Use scripts, references,
  examples, templates, assets, evals, or append-only data when prose would
  make the agent reconstruct repeatable work.
- Model/provider/harness selection facts live in the roster skill
  (`skills/roster/references/model-provider-harness-index.md`). Keep that
  file factual: model ids, context, price, latency/smoke evidence, tool
  support, benchmark sources, deprecations, and freshness. Do not encode
  role-fit policy there; the lead agent composes task-specific teams from
  current evidence.
- Harness Kit source skills live in `skills/`; repo-local `.agents/skills/` and
  harness-specific skill bridge dirs are `/seed` output for consumer repos.
- Generated/root `AGENTS.md` is a router, not a manual. Keep non-obvious facts
  only.
- System bootstrap exposes the full first-party skill catalog; repo-local
  vendoring is exceptional and must earn its complexity.
- Provider CLIs are tools. Do not wrap them in semantic orchestration unless a
  shaped ticket explicitly asks.
- Harness-specific accelerators (e.g. orchestration-workflow templates) may
  ride inside a skill folder as assets the prose names as optional — a
  harness without the feature must lose nothing by ignoring them. Build one
  only after telemetry shows the pattern recurring; never pre-author.

## Delegation Judgment

delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use lanes for doctrine critique, runtime compatibility, gate design, and regression risk. Do not treat a missing repo-local roster as a waiver; use the resolver-backed probe.

## Primitive Test

Before creating or growing anything, classify it (2026-06 audit, backlog 103):

- **Local prompt** — "is this just what the operator would retype to a strong
  model?" Keep it in chat, local scratch, or a skill template only when a skill
  truly needs reusable wording. Do not add a repo-level prompt layer. If the
  operator expects Codex app slash or `$` discovery, use a skill instead.
- **Skill** — "does this change what a frontier model does, for the better,
  repeatedly?" Judgment + context the model can't derive.
- **Doctrine line** — "worth paying for in every session?" Goes in AGENTS.md,
  not a folder.
- **Mode B** — event-triggered (on PR-ready, on production error, on
  schedule)? It belongs in the event plane (bitterblossom), not this harness.
  This repo is the ad-hoc operator layer plus the shared disk contracts.
  Load `harnesses/shared/references/loop-readiness.md` before proposing any
  unattended loop.

History: slash commands were collapsed into skills when skills arrived, so
saved prompts masqueraded as skills and the catalog tripled. Do not recreate
that. Counter-history: Codex app discovery is skill-shaped, so high-frequency
operator commands that must appear in the app are skills even when their body is
lightweight.

## Quality Bar

- `SKILL.md` encodes judgment, not a procedure the model already knows.
- Frontmatter descriptions are model trigger classifiers, not human summaries:
  include explicit `Use when:` phrases and `Trigger:` aliases.
- Instruction prose is compression, not literature. Sacrifice grammar before
  clarity; keep terse imperatives, named failure modes, and concrete oracles.
- Put long mode detail in `references/`; keep the entry file short.
- Build gotchas from repeated agent failures. If a gotcha can be asserted by a
  script, hook, or eval, codify it there and point the skill at the artifact.
- Ad-hoc roster lanes beat static project subagents unless tool permissions
  must be isolated.
- New mechanisms include a verification system: gate, eval, benchmark, QA
  path, smoke path, or probe that can fail for the real error.
- Every run ends clean: no untracked or modified files.

## Post-Sync Acceptance

After changing skills, shared doctrine, generated docs, bootstrap, roster, or
harness projections, prove the output is repo-fit, not merely structurally
valid.

```markdown
## Acceptance Evidence
- Live repo evidence read: source skill, shared doctrine, generated docs, bootstrap output, roster, or harness projection inspected.
- Acceptance source: backlog oracle, skill contract, generated index/docs contract, bootstrap contract, or explicit absence.
- Evidence that proves it: command output, diff, generated artifact, bootstrap transcript, or gate output.
- Exact command/path/route exercised: check, generator, bootstrap, smoke path, projection path, or route run.
- Oracle / acceptance artifact hash: sha256 digest for any fixture, generated artifact, transcript, or contract used as the oracle, or state that no artifact-backed oracle exists.
- Contract-change acknowledgment: reason when the change alters an acceptance contract, generated source, or assertion surface, or state that no contract changed.
- Repo-fit check: source/generator/projection agree; no stale generated docs, wrong skill root, stale command, or copied bridge remains.
- Structural gate: `check --repo .` result, or the specific sub-gate exercised.
- Residual risk: skipped harness, external dependency, or none with reason.
```

## Gotchas

- Phase prose is not judgment. Frontier models know the SDLC; a skill that
  restates implement/refactor/review steps is railroading (Anthropic's own
  top skill lesson: don't state the obvious, avoid railroading). Encode the
  bespoke part — oracles, repo facts, taste — or nothing.
- Process bureaucracy trains checkbox compliance, not quality. Multi-field
  completion gates, oracle hashes, and learning-packet ceremonies get filled
  in plausibly by strong models. Verification is tests, CI, and driving the
  live surface.
- Evals, benchmarks, and QA are verification systems only when they have a
  driver, grader, evidence packet, and cadence. A directory, prompt, or
  transcript with no falsifier is not proof.
- Deterministic scaffold is the historical failure mode here: agents unsure
  of harness engineering fall back to Rust/scripts that enforce prose. Every
  gate must answer "what real failure did you catch in the last 90 days?" —
  no answer, delete it.
- Check telemetry before adding or keeping a skill. The 2026-06 audit found
  ~15 of 36 skills unused; usage is a power law. Low usage with high
  value-when-used is fine (say so); low usage with no story is deletion.
- A new frontier model release silently converts some skill prose from
  judgment into railroading: instructions tuned for the last model anchor
  the new one to stale patterns. After a major model ships, re-audit skill
  and doctrine prose; prefer deleting an instruction over updating it.
- Name harness features by capability, not vendor. "Use the harness's
  large-scale orchestration feature when it has one" degrades gracefully
  across harnesses; "use dynamic workflows" confuses every harness that
  lacks them.
- Meta-work ratio: if this repo's commit rate rivals the product repos',
  the flywheel is feeding itself.
- Stale AGENTS prose is worse than missing prose.
- Duplicated repo-local skill copies are usually stale context unless a repo
  needs checked-in vendored harness state.
- Generated catalog/docs drift means the source skill changed but the harness
  projection did not.
- Unsupported invocation hooks mean usage telemetry is structurally shaped but
  not empirically proven for that harness.
- Structural eval trees are not semantic proof; objective graders must assert
  behavior or carry an explicit waiver.
- Helper scripts that are not wired into a gate become optional folklore.
- Regexes over agent prose are usually the wrong boundary.
- If a rule matters, enforce it outside prose.

## Verification

Run `cargo run --locked -p harness-kit-checks -- check --repo .` after
changing harness primitives, gates, roster, bootstrap, or sync logic. For
bootstrap changes, also re-run bootstrap and confirm installed skills/configs
match the source tree and retired prompt/example links are pruned.
