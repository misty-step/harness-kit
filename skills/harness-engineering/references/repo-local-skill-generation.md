# Repo-Local Skill Generation

Use when a domain agent is being stood up to work IN a specific repo (a
resident lane, an orchestrator-dispatched subagent) and the global catalog is
either too broad or not yet installed there. The move is not "sync the whole
catalog" — it is: read this repo's real facts and write it one or two
skills that could not exist anywhere else, because they name this repo's
actual commands, not a genre of command.

Exemplar this pattern generalizes: a resident lane wrote
`.agents/skills/canary-qa/SKILL.md` by hand — a QA skill built entirely from
Canary's own `bin/validate`, its HTTP routes, its CLI, its webhook rehearsal
script. No invented process, no generic "run the tests" prose. That skill is
the proof the pattern works; this reference is how to do it again
deliberately, with the two things that first pass skipped: a provenance
header (so a future reader knows this came from generation, not hand
authorship, and can judge whether it has drifted) and an eval stub (the
evals-per-skill floor, backlog 128, extended to generated skills).

## Boundary: this is not `/tailor` again

Harness Kit tried whole-repo harness generation twice and retired it both
times (`backlog.d/_done/029-tailor-per-repo-harness-generator.md`,
`backlog.d/_done/045-harness-pivot-minimal-globals.md`): a manifest schema, an
A/B eval with automatic rollback, a pre-commit lint hook enforcing a skill
cap, a planner/critic dialectic loop. The audit that killed it found ~60%
ceremony where agent judgment was simpler and better. Do not rebuild that
machinery here. This pattern generates **one to three focused skills**, not a
harness: no manifest, no killswitch, no lint hook, no state machine. A lead
agent reads a repo, exercises judgment about what's worth generating, writes
it, proves it once with a cold-agent smoke. If a repo needs more than a
handful of bespoke skills, that is a signal to give it a role-scoped bootstrap
bundle (`.harness-kit/bundles.yaml`, backlog 130) instead of hand-generating
a pile — bundles subset the existing catalog; this pattern authors net-new
repo-specific content that has no catalog equivalent.

## Read the repo before writing anything

In priority order, because later sources correct earlier ones when they
disagree:

1. `AGENTS.md` / `CLAUDE.md` — the repo's own stated contract: gate command,
   base branch, red lines, known-debt map.
2. `.github/workflows/*.yml` (or equivalent CI config) — what actually runs
   on a PR, which may differ from what `AGENTS.md` claims. A repo with no CI
   workflow at all is itself a fact worth naming in the generated skill, not
   smoothing over.
3. `bin/`, `scripts/`, root manifests (`Cargo.toml`, `package.json`) — the
   real invocable commands, copied verbatim, never paraphrased.
4. `docs/*.md` runbooks — deploy, DR, migration, onboarding docs that encode
   procedures too specific to derive from code alone.
5. Root `SKILL.md`, if the repo ships one — that is almost always the
   **product** skill, written for *consumers* of the thing this repo builds
   (see Powder's root `SKILL.md`, written for agents that use Powder as a
   work board). Do not confuse it with a skill for agents *building* the
   repo, and do not duplicate its content.
6. `backlog.d/` — open debt and priority context that shapes which domain is
   worth encoding now versus later.

## Pick a domain with a drivable oracle

Rank candidate domains by whether a cold agent can run something real and
observe pass/fail, not by topic importance:

- **Verification/QA** is almost always the strongest first domain — every
  repo has a shape and a real command to exercise it (`skills/qa/SKILL.md`'s
  shape table: browser app, API/service, CLI, library, MCP, hybrid). This is
  child 1 of backlog 131 for exactly this reason.
- **Deploy/release runbooks** are the second-strongest — a real deploy
  command, a real rollback/DR path, usually already written down in
  `docs/*.md` and just needs collecting into agent-executable form.
- Skip domains with no falsifiable check: "architecture", "conventions",
  "style" have no oracle a cold agent can run, so a generated skill there is
  prose bloat wearing a skill's clothes. If the repo needs that captured,
  it belongs in `AGENTS.md`, not a generated skill.

Generating more than two or three skills in one pass is a signal you're
building a bundle, not authoring bespoke content — stop and reconsider
against role-scoped bundles instead.

## Name and place the file

- Path: `<target-repo>/.agents/skills/<repo>-<domain>/SKILL.md` — e.g.
  `canary-qa`, `powder-qa`, `canary-deploy`. The repo prefix is load-bearing:
  it keeps the generated skill from shadowing a first-party Harness Kit skill
  name (`qa`, `deploy`) and makes provenance legible from the directory name
  alone.
- Never write into Harness Kit's own tree, and never write directly into a
  harness-specific bridge directory (`.claude/skills/`, `.codex/skills/`,
  `.pi/skills/`) in the target repo — those are sync/bootstrap output in
  repos that run Harness Kit's own bootstrap, not source. `.agents/skills/`
  is the portable root a harness projects from.
- Copy `templates/repo-local-skill/SKILL.md.tmpl` as the starting shape:
  frontmatter (`name`, `description` with explicit `Use when:`/`Trigger:`
  phrasing, `argument-hint`), the provenance comment block, a surfaces/routes
  table, exact commands, gotchas, and a report contract — same shape as
  `skills/qa/SKILL.md` and the `canary-qa` exemplar, scaled to what this
  repo actually has.

## Provenance header

Every generated `SKILL.md` carries an HTML comment immediately after the
frontmatter closing `---`, before the first heading:

```markdown
<!--
Generated via harness-kit's repo-local skill generation pattern
(skills/harness-engineering/references/repo-local-skill-generation.md).
Source repo: <owner/repo> @ <sha>. Generated: <YYYY-MM-DD>.
Generator ref: harness-kit@<sha used to generate this>.
Facts below are repo-derived at generation time, not invented. Re-verify
commands against the live repo before trusting this if it has aged — a
generated skill is a snapshot, not a live view.
-->
```

This is new; no existing generated repo-local skill in the fleet carries one
(`canary-qa` predates this convention and is not retrofitted by this change —
it stays as committed, its lane's work is not overwritten). Every skill this
pattern generates from now on carries the header so a future reader can tell
generated content from hand-authored content and judge staleness.

## Eval stub — the evals-per-skill floor extended to generated skills

Copy `templates/repo-local-skill/evals/eval-stub.md.tmpl` to
`<target-repo>/.agents/skills/<repo>-<domain>/evals/<repo>-<domain>-eval.md`.
It is a deliberately smaller instrument than
`skills/skill-eval/templates/eval-spec.md` (Harness Kit's first-party
template): one falsifiable claim, one to two fixtures built around a cold-
agent run (not a multi-arm A/B judge panel — the claim being tested is "does
a cold agent execute the real command from this skill alone," not "does this
beat raw prompting by how much"), objective pass/fail checks, and a run log.
Fill the run log with the actual cold-agent smoke transcript reference from
generation, not "PENDING" — the smoke is required before the skill is
committed (next section), so real evidence should already exist.

## Validate before committing

1. **Run the flagship command yourself, once**, exactly as written in the
   generated skill, before it is committed. A generated skill whose one
   command 404s or typos a flag is worse than no skill — it costs a cold
   agent a debugging detour instead of saving one.
2. **Cold-agent smoke.** Dispatch a fresh-context agent (no session memory of
   this generation work) with only the generated skill file and normal repo
   read access. Ask it to use the skill to perform one real, ideally
   read-only, action. Capture the transcript as the eval stub's first run-log
   entry and as the PR's evidence. This is the actual proof the skill
   works — the author driving their own generated skill is not evidence,
   per the shared no-self-review doctrine.
3. Commit only the new `.agents/skills/<repo>-<domain>/` files. Never stage
   or touch unrelated in-flight changes in the target repo — check
   `git status` first; if another lane has uncommitted work on the checkout,
   use a worktree instead of the shared working tree.

## Anti-goals

- No manifest, no A/B eval infrastructure, no rollback, no lint-enforced skill
  cap — see the boundary section above.
- No shadowing a first-party Harness Kit skill name.
- No touching the target repo's root `SKILL.md` (the product skill, if one
  exists) or its `AGENTS.md`/gate contract — this pattern adds a skill, it
  does not rewrite the repo's own doctrine.
- No regenerating or overwriting an existing hand-authored or previously
  generated skill without a reason stated in the PR — additive by default.
