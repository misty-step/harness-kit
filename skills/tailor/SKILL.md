---
name: tailor
description: |
  Tailor this repository's harness. Explore the repo, read prior
  session history, browse the spellbook catalog, and install a
  per-repo set of skills into a shared repo-local skill layer, with
  harness-specific entrypoints bridged back to that shared copy.
  Workflow skills get rewritten with this repo's commands and
  conventions embedded throughout — not a generic body with a
  repo-notes appendix. Use when: "tailor this repo", "configure the
  agent for this codebase", "set up a harness", "what skills apply
  here". Trigger: /tailor.
---

# /tailor

You're a tailor, not a wardrobe curator. Spellbook is a reference
library; for this repo you cut new garments sized to fit. Sewing an
extra inch onto an off-the-rack jacket is decoration, not tailoring.

## Shape of the work

1. **Explore.** Read enough of this repo to know what it is —
   language, frameworks, test/CI/deploy commands, size, domain.
   `package.json` / `Cargo.toml` / `pyproject.toml`, README, top-level
   structure.

   **Also inventory any existing harness.** Resolve the repo's
   **shared skill root** first: prefer an existing `.agent/skills/`,
   then an existing `.agents/skills/`. If neither exists yet, note
   that you will create one during install. Then inspect that shared
   root plus any existing harness bridges such as `.claude/skills/`,
   `.claude/agents/`, `.codex/skills/`, or `.pi/skills/`. If any
   have content, classify each entry:
   - **Tailor-owned** — has a `.spellbook` marker with `source:
     <name>`, `installed: <timestamp>`, and (newer runs)
     `installed-by: tailor`. Safe to replace or remove.
   - **Scaffolded** — marker says `installed-by: <skill>-scaffold`
     (e.g. `qa-scaffold`, `demo-scaffold`) or the content is a
     scaffold output the repo explicitly authored. Preserve.
   - **Human-authored / unknown** — no marker, or marker from an
     era before current /tailor (no `installed-by` field). Preserve
     by default; flag for user confirmation before overwriting.

   Read the repo-level `.spellbook/repo-brief.md` if present —
   it's the prior run's brief, useful for diffing what changed.

2. **Prior art.** If your harness keeps session history for this repo
   (Claude Code: `~/.claude/projects/<path-hash>/`, Codex: analogous
   state path), read the session JSONL and memory files. What
   commands does the user actually run here? Where have they
   corrected you? Highest-signal input available.

3. **Browse.** Read the spellbook catalog — resolve via `readlink -f`
   on this SKILL.md, walk up to find `$SPELLBOOK/skills/` and
   `$SPELLBOOK/agents/`. Each primitive's frontmatter describes when
   to use it. That's your map.

4. **Synthesize a repo brief.** Write a short prose document
   (1-2 pages) that every downstream subagent will read. It is
   the shared spine that keeps rewrites coherent — without it,
   `/ci` calls the enforcing layer one thing and `/deliver` calls
   it another. Required anchors:

   - **Vision & purpose** — what this repo is building, for whom.
   - **Stack & boundaries** — layers and what each owns.
   - **Load-bearing gate** — *the* single declaration of what must
     pass to ship (e.g. "`pnpm ci:prepush` IS the gate; the Dagger
     lanes are its components"). Every gate-adjacent skill cites
     this verbatim.
   - **Invariants** — repo-wide rules (don't touch X, base branch
     is Y, commits go through Z).
   - **Known debts** — active issues, hot files, recurring failure
     modes, incident IDs. Pull from backlog, session history, and
     recent git log.
   - **Terminology** — what this repo calls its things.
   - **Session signal** — 3-5 recurring user corrections from
     session JSONL + 3-5 validated patterns the user has ratified.

   Short, dense, concrete. This is the single source of truth for
   "what is this repo" that every rewriter anchors to.

   **Persist the brief to `.spellbook/repo-brief.md`** at the repo root.
   Future runs read it to diff changes. Overwrite any existing version.
   If `.spellbook/` is gitignored, also write a tracked compatibility copy
   at `.claude/.tailor/repo-brief.md` (or the repo's documented harness-state
   path) with the same content. A breadcrumb to ignored state is not durable.

5. **Pick.** Dispatch planner + critic subagents with the repo
   brief attached. Planner proposes a set following the picking
   defaults below; critic applies `references/focus-postmortem.md`.

   **This is the planning leg of a planner → executor → critic
   loop.** The planner owns portfolio selection and rewrite briefs.
   It does not rewrite skills itself. The critic judges the plan
   before any install work begins and sends the planner back with
   specific objections when the pick is shallow, contradictory, or
   missing a repo-defining concern.

   **Planner must also propose ≥1 candidate domain invention per
   round**, with the concrete repo characteristic that would justify
   it (e.g., "`/convex-migrate` — this repo has live-traffic Convex
   schema evolution with `v.optional` upgrade paths in
   `convex/schema.ts`"). The critic evaluates via criterion 4. Landing
   on "none justified" is a valid outcome — the point is to force the
   thinking, not the output. Repos with load-bearing domain needs
   shouldn't slip through because the picker never considered them.

   Run the pick loop until critic-clear or 3 rounds, whichever comes
   first. If the planner and critic still disagree after 3 rounds,
   escalate to a second outside voice (`/research`, Thinktank,
   Gemini, or another fresh-context bench) before converging. Do
   not silently accept a fuzzy pick because the first pass looked
   plausible.

6. **Install.** First reconcile, then write. **The shared skill root
   is canonical; `.claude/skills/` is a bridge layer** — see
   cross-harness install invariant below.

   **Reconcile.** For each tailor-owned item inventoried in step 1
   (has `.spellbook` marker with `installed-by: tailor`):
   - Still in the new pick → replace with fresh rewrite.
   - Not in the new pick → remove the skill/agent directory. This
     is how `/tailor` self-heals canary-class fossils where the
     prior pick no longer matches the repo's current needs.

   For items without a `.spellbook` marker (unknown origin): do not
   touch without user confirmation. If a skill you're about to
   install collides with an unmarked existing skill of the same
   name, surface the conflict against the shared root: "existing
   `<shared-skill-root>/qa/` has no tailor marker — is it
   scaffolded, human-authored, or prior-era /tailor output?
   [preserve / replace / diff]".

   **Install shared, bridge per-harness.** Write every spellbook-
   distributed skill to the shared skill root
   (`.agent/skills/<name>/` or `.agents/skills/<name>/`, whichever
   this repo uses). Then create or refresh per-harness skill bridges
   back to that shared copy:

   - `.claude/skills/<name>` should be a symlink to the shared skill.
   - If the repo already has `.codex/skills/` or `.pi/skills/`
     bridges, keep them pointed at the same shared skill root.
   - Do not duplicate spellbook-distributed skill trees across
     multiple harness-specific directories.

   Agents are different: install them into the repo's existing agent
   directory (today that is usually `.claude/agents/`) unless the
   repo already documents a shared-agent convention. Do not invent a
   second agent layout just because the skills are shared.

   **Write `.spellbook` markers.** Every shared skill directory and
   every installed agent directory or file updated by this run gets a
   marker file at `<skill-or-agent>/.spellbook` with:

   ```yaml
   source: <primitive-name>
   installed: <ISO-8601 timestamp>
   installed-by: tailor
   tailor-version: <git commit SHA of spellbook/skills/tailor>
   category: universal | workflow | domain-invented
   ```

   Markers are how re-runs tell your output apart from human-
   authored content.

   **Shared scripts.** Tailored skills source shared shell helpers
   at a stable path. Install verbatim from `$SPELLBOOK/scripts/lib/`
   to `<repo-root>/scripts/lib/` (create if missing). Do not rewrite
   — divergence breaks sourcing in every consumer at once.

   | Path | Required by | Overwrite policy |
   |---|---|---|
   | `scripts/lib/backlog.sh` | `/ship`, `/groom tidy`, any skill reconciling master against `backlog.d/` | Copy verbatim when absent or tailor-marked; if unmarked and different, stop and ask `preserve / replace / diff` |
   | `scripts/lib/verdicts.sh` | `/ship`, `/settle` (git-native mode) | Copy verbatim when absent or tailor-marked; if unmarked and different, stop and ask `preserve / replace / diff` |

   Emit `.spellbook` markers alongside copied scripts when the repo
   accepts marker files there. An unmarked divergent script is an
   ownership conflict, not a failed self-audit you silently fix.

   Three categories, different install rules:

   - **Universal skills** — `office-hours`, `ceo-review`, `reflect`,
     and similar judgment protocols that carry no repo-specific command
     surface. Copy their directories verbatim. Tailoring them would be
     artificial.
   - **Workflow skills** — the software development lifecycle:
     `research`, `groom`, `shape`, `implement`, `qa`, `demo`,
     `code-review`, `refactor`, `ci`, `diagnose`, `monitor`,
     `deliver`, `settle`, `ship`, `yeet`, `flywheel`, and
     deploy-oriented skills when the repo has a deploy surface.
     **Rewrite each SKILL.md with this repo's commands, gates,
     conventions, and file paths embedded throughout.** Use the
     spellbook version as structural reference; fill every example,
     every command, every gotcha with repo-specific content. Preserve
     `references/` and `scripts/` from the source — they travel with
     the skill.

     **Dispatch one executor/rewriter subagent per workflow skill,
     in parallel. These executors are the executor leg of the loop.**
     Monolithic rewriting of 10+ skills in one context causes
     attention decay — the model pattern-matches "just copy this
     one too" and ships byte-identical output. Each executor
     receives: (a) the repo brief from step 4 (shared spine —
     cite these anchors, don't invent parallel vocabulary),
     (b) the planner's rewrite brief for that skill,
     (c) the spellbook source, (d) a reading assignment (see
     table below), (e) the mandate above.

     **Executor ownership is narrow and explicit.** Executors do not
     re-decide the portfolio and they do not argue policy in the
     abstract. Their job is to produce the best repo-specific rewrite
     for their assigned skill, then revise that rewrite in response
     to critic objections until it clears. Quality lives in this
     loop, not in post-hoc semantic lint rules.

     **Preserve workflow contracts, not source mechanisms.** Before
     rewriting a workflow skill, extract purpose, mandatory phases,
     terminal artifact, refusal conditions, and destructive-action
     boundaries. Mechanisms may change; dropping the source success
     condition requires explicit planner + critic acceptance.

     **Every workflow skill gets a reading assignment.** Skills
     with clear command anchors get rewritten easily; abstract-
     process skills (shape, refactor, code-review, flywheel,
     groom, implement, diagnose) are the ones that silently pass
     through because the source looks "already generic enough."
     They aren't — they're the skills where rewriters have to
     encode *this repo's judgment* as content, since there's no
     command to swap in. Assignments:

     | Skill | Read |
     |---|---|
     | ci | `.github/workflows/`, `dagger/`, `Dockerfile*`, CI status history |
     | deliver | `backlog.d/`, recent merged PRs, CONTRIBUTING.md |
     | deploy | `vercel.json`, `fly.*.toml`, `Dockerfile*`, deploy workflows |
     | settle | merge history, branch-protection rules, release tooling |
     | ship | squash-merge history (`git log --merges master`), drift contract (e.g. `docs/context/DRIFT-WATCHLIST.md`), PR templates, branch-naming conventions, `backlog.d/` + `backlog.d/_done/` layout |
     | yeet | commit history, `commitlint.config.*`, semver tooling |
     | qa | test configs, smoke paths, critical user/API/CLI/library surfaces, existing manual QA evidence |
     | demo | evidence scripts, README examples, release-note patterns, screenshot/terminal/API capture tools |
     | monitor | signal surfaces: prod observability when present; otherwise logs, CI history, flaky tests, benchmark drift, local daemon output, or release health |
     | **shape** | `backlog.d/*.md`, recent shape docs, planning conventions, design-review lineage |
     | **refactor** | git churn (hot files), debt map, ARCHITECTURE.md, recent refactor PRs, named hotspots from repo brief |
     | **code-review** | AGENTS.md red flags, recent review threads, repo's philosophy bench, known anti-patterns |
     | **flywheel** | milestone tracker, issue IDs from repo brief, recent cycle reflections, deploy observables |
     | **groom** | `backlog.d/` (or equivalent tracker), GitHub issues if in use, grooming cadence, prioritization scheme |
     | **implement** | test runner command, mocking boundaries, test config, recent TDD cycles, branch-naming convention (`<type>/<id>-<slug>`) |
     | **diagnose** | signal surfaces, `.evidence/`, postmortems, observability tooling |
     | **research** | prior research artifacts (`.claude/research/`, `docs/research/`, `.groom/`), stack manifest (`package.json` / `Cargo.toml` / `pyproject.toml`) for domain cues, open research threads in `backlog.d/`, repo-specific source/doc lists |

     **Loop-core skills must not ship shallow.** The seven bolded
     rows above (shape, refactor, code-review, flywheel, groom,
     implement, diagnose) are the development loop's connective
     tissue — they encode *how this team builds here*. Their source
     text looks abstract because its job is generic wisdom. *Your*
     rewrite's job is to name what each skill MEANS in this
     codebase: the actual hotspots, the actual red flags, the
     actual backlog conventions, the actual milestone structure,
     the actual test command, the actual signal surfaces. "Generic
     refactoring discipline" is the source's contribution; your
     contribution is "in this repo, refactor first targets
     `getRevealPhaseState` (N+1, #146) and theme-token leakage."
     If your rewrite could describe any Next.js+Convex repo, any
     Elixir/Phoenix repo, any Rust service — it's shallow. Redo.

     **Backlog-lifecycle load-bearing mandates.** The ship/groom/
     settle/flywheel/implement rewriters MUST preserve one mechanical
     closure contract across all five skills. The portable concept is:
     active work lives in `backlog.d/`, closed work lives in
     `backlog.d/_done/`, and a structured closing signal lets a local
     command or CI check detect a shipped-but-still-active ticket.
     Spellbook's default closing signal is the git trailer set below,
     but a tailored repo may use PR metadata, branch naming, or another
     structured signal only if the rewrite names the exact enforcement
     command and every lifecycle skill uses the same signal. Prose-only
     closure instructions are a failed tailor run.

     | Skill | Must preserve |
     |---|---|
     | `/ship` | Names the repo's structured closing signal; archives each closing `backlog.d/<id>-*.md` file into `_done/` before merge or verifies it is already archived; preserves the closing signal into the landed commit/PR record when the repo's detector reads history; verifies the archive landed after merge; invokes `/reflect` with bounded scope (branch, merged SHA, closing IDs, reference IDs); routes harness edits to `harness/reflect-outputs`, never the protected branch; embeds the repo's merge strategy and doc-sync convention verbatim. Spellbook default: branch regex `^(feat\|fix\|chore\|refactor\|docs\|test\|perf)/(\d+)-`, trailers `Closes-backlog:` / `Ships-backlog:` + `Refs-backlog:`, `backlog_archive`, explicit squash body carrying trailers. |
     | `/groom` | Always-on `tidy` step invokes the same lifecycle detector `/ship` relies on, reconciles active tracker items against shipped closing signals, closes or archives stale-active work through the tracker-native operation, and flags contradictions. Strategic-layer trigger vocabulary can stay universal; prioritization scheme, cadence, tracker surfaces, lifecycle command, and output artifact are repo-specific. No retired third-party trackers. |
     | `/settle` | Polish-loop framing (CI → code-review → refactor until merge-ready); stops at merge-ready; hands off to `/ship`; checks or names the lifecycle gate that prevents a ready PR from referencing active-but-unarchived backlog work. |
     | `/flywheel` | Composition `pick → /shape → /implement → /yeet → /settle → /ship → /monitor → loop`; `/ship` owns closure (archive + reflect + harness routing); `/flywheel` does not archive tickets or invoke `/reflect` directly; it only consumes `/ship`'s final lifecycle result. |
     | `/implement` | Branch or context-packet creation produces whatever structured item reference `/ship` can later resolve. If the repo weakens Spellbook's default branch regex, the rewrite must update `/ship`, `/groom`, `/settle`, and `/flywheel` in the same tailor run and name the replacement detector. |

     **Rewriters may add sections, delete irrelevant structure,
     and restructure — the spellbook source is a reference, not
     a template to fill in.** Stack-native content (Elixir
     `:observer` + `:telemetry`, Rust `cargo miri`, Go `-race`,
     OTP supervision patterns) often has no slot in the JS/TS-
     shaped source. Add the section. Conversely, if the source
     has content that doesn't apply (matrix builds for a
     single-target repo, Playwright config for a CLI), delete it.
     Find-and-replace at the noun level produces B+ output; real
     tailoring restructures where the repo demands it.

     **Critic reviews all rewrites together, not individually.**
     Three checks: (1) **depth** — subtractive test, would this
     rewrite be wrong if applied to another repo with the same
     stack? If no, it's shallow — reject; (2) **cohesion** — does
     every gate-adjacent skill cite the same load-bearing gate
     from the repo brief? The critic must name at least one concrete
     contradiction check per gate-adjacent skill against the repo
     brief's gate statement or a sibling skill's statement. Any
     contradiction against repo brief or sibling skills → reject;
     (3) **contract conservation** — after translating mechanisms,
     does the rewrite still produce the terminal artifact and honor
     mandatory phases? If no, reject; (4) **completeness** — byte-identical to spellbook source →
     reject. Critic sends only the failing executors back with
     specific objections. Passed skills freeze. Only changed skills go
     back to executors. Re-enter the planner only when the critic has
     found a portfolio problem, missing scope, or cross-skill
     contradiction that the executor brief itself cannot resolve.
     Iterate until coherent. Up to 3 rounds; critic judges
     convergence.
   - **Domain skills (invented)** — greenfield additions like
     `/convex-migrate`, `/rust-unsafe-reviewer`. Only invent when you
     can name the concrete repo characteristic demanding it. Every
     invented skill ships with an eval seed under its own tree
     (`evals/` or `tests/`) that captures at least one representative
     invocation, expected artifact, and pass/fail grader. A skill that
     cannot name how it will be evaluated is not ready to install.

7. **Write `AGENTS.md`.** Project the repo brief + the coherent
   rewrite set into a router (not a manual). Suggested structure:
   - **Stack & boundaries** — stack names and what each layer owns.
   - **Ground-truth pointers** — files that ARE the API (e.g.
     `convex/_generated/api.d.ts`); stale training data lies.
   - **Invariants** — hard rules specific to this repo (functions,
     env vars, schema constraints, auth flows).
   - **Gate contract** — cite the load-bearing gate from the repo
     brief; enumerate pre-commit hooks, what humans do, what's
     enforced where.
   - **Known-debt map** — concrete file/line pointers. Every P0
     debt item gets a filed issue ID, not `(unfiled)`. If the repo
     brief surfaced a P0 with no tracker ID, file it (`gh issue
     create` or `backlog.d/NNN-*.md`) before writing this section.
     Debt-map entries like `INCIDENT-*.md` filenames are pointers,
     not IDs — attach a tracker ID.
   - **Harness index** — table: installed skill → what it does
     *here* (not the generic description). Agents in a sibling
     table, not a prose sentence.

8. **Write per-harness settings.** Each harness has its own
   settings format and path; emit one variant per harness:
   - `.claude/settings.local.json` (JSON, Claude Code) — command
     allowlist under `permissions.allow` (e.g. `"Bash(go test:*)"`).
   - `.codex/config.toml` (TOML, Codex) — **no command
     allowlist.** Codex's `[permissions]` key is a
     `FilesystemPermissionsToml` struct (path scopes), not a
     command array. Emitting `[permissions] allow = ["go run ..."]`
     fails config load with `invalid type: string ..., expected
     struct FilesystemPermissionsToml`. Command authorization is
     driven by top-level `approval_policy` + `sandbox_mode` in
     `~/.codex/config.toml`, not per-repo. Leave the per-repo
     `.codex/config.toml` to MCP servers, `[skills]` roots, and
     similar — and add a short comment block documenting why
     command permissions are absent (see
     `spellbook/.codex/config.toml` and
     `worktrees/699b/teams-docs-bot/.codex/config.toml` for the
     canonical phrasing).
   - `.pi/settings.json` (JSON, Pi) — `skills[]` glob allowlist.

   Permissions allowlist (and any other per-harness toggles)
   derive from the tools actually in use. Merge additively with
   any existing file — don't nuke user-added entries. Emit all
   three so the harness can be swapped without re-tailoring.

## Invariants

- Never write outside the current repo. No `$SPELLBOOK` mutation,
  no `~/.claude` / `~/.codex` / `~/.pi` mutation.
- **Workflow skills split into two tiers.** Universal skills
  (`office-hours`, `ceo-review`, `reflect`) are always-present via
  step 6's universal category; they install verbatim, not tailored.
  `/groom` is not universal: backlog shape, lifecycle enforcement,
  priority scheme, and tracker surfaces are repo-specific.
  - **Always install** (orchestrators, foundational loop skills,
    and judgment-only skills): `research`, `groom`, `shape`,
    `implement`, `qa`, `demo`, `code-review`, `refactor`, `ci`,
    `diagnose`, `monitor`, `deliver`, `settle`, `ship`, `yeet`,
    `flywheel`. **Never skipped.** `/groom` owns backlog lifecycle;
    `/settle` leaves branches merge-ready; `/ship` closes and lands
    them; `/flywheel` composes the loop. `/ci`, `/diagnose`,
    `/research`, `/qa`, `/demo`, and `/monitor` are still tailored
    when their default infrastructure is absent: name the repo's
    actual gate, debug surface, research sources, QA path, demo
    artifact, and observable signal path. Exact-copy is valid only
    when tailoring Spellbook itself.
  - **Deploy-surface skills are evidence-tied.** `deploy` is installed
    only when the repo has a real deploy target (`vercel.json`,
    `fly.*.toml`, `Dockerfile*`, `.github/workflows/*deploy*`, or an
    equivalent release mechanism named in the repo brief). No deploy
    target does not justify skipping `/monitor`: every repo still has
    signals to watch, even if they are CI failures, local daemon logs,
    benchmark drift, flaky tests, release regressions, or agent-session
    audit trails rather than production telemetry.
- **Domain skills default to exclude.** Invent only when you can
  name the concrete repo characteristic demanding it. "We might want
  X" is not a name.
- **No `references/<repo-name>.md` sidecar files.** If a skill has
  repo-specific content, it belongs in the SKILL.md body. A sidecar
  named after the repo is the sewn-on-sleeve anti-pattern — the
  generic jacket with an appendix. Forbidden.

  *Permitted:* stack-specific references under their own topic
  (`references/elixir-observability.md`, `references/convex-
  patterns.md`, `references/otp-supervision.md`). These carry
  reusable content that's too deep for SKILL.md progressive
  disclosure — not a repo appendix. If the name could apply to
  another repo on the same stack, it's fine. If the name is this
  repo, rewrite it into SKILL.md.
- **Self-audit before declaring done.** Seven checks:
  1. **Workflow rewrites:** `diff` each installed workflow SKILL.md
     against the spellbook source. Byte-identical = rewriter
     dropped the ball. Go back and redo.
  2. **Always-install coverage:** every skill in the always-install
     tier (`deliver`, `shape`, `implement`, `code-review`, `ci`,
     `refactor`, `groom`, `flywheel`, `settle`, `ship`, `yeet`,
     `diagnose`, `monitor`, `research`, `qa`, `demo`) resolves to a directory
     under the shared skill root. Zero missing. If one is absent, the run
     failed — reinstall before declaring done.
  3. **Excluded workflows:** only `deploy` may be skipped, and the skip
     names the concrete missing deploy surface (no `vercel.json`, no
     `fly.*.toml`, no `Dockerfile*`, no `.github/workflows/*deploy*`,
     no release script or equivalent). An always-install skill appearing
     in the skipped list is a critical regression. `qa`, `demo`, and
     `monitor` are always-install — never appear here.
  4. **Agent installation:** grep every installed skill for
     `subagent_type:` references. Every referenced agent must resolve
     to a file in the repo's installed agent directory (usually
     `.claude/agents/`). A `/code-review` that dispatches
     `ousterhout` + `carmack` + `grug` against nonexistent agent
     files is a silent regression — the skill fails at call time.
  5. **Backlog lifecycle coherence:** `/ship`, `/groom`, `/settle`,
     `/flywheel`, and `/implement` all name the same active tracker,
     closed tracker, structured closing signal, archive operation, and
     detector command. Contradictions like "no trailers" in `/ship`
     while `/groom` sweeps trailer history fail the run. If the repo
     lacks a detector today, the tailor run must file a high-priority
     backlog item and every lifecycle skill must name that gap instead
     of pretending prose is enforcement.
  6. **Shared scripts present:** `scripts/lib/backlog.sh` and
     `scripts/lib/verdicts.sh` exist at the repo root and match
     spellbook content, unless an unmarked divergent file was
     classified as `preserve` by the user and the final report names
     the residual drift. `/ship` and `/groom tidy` source `backlog.sh`;
     `/ship` and `/settle` source `verdicts.sh`. A missing library is
     a silent-sourcing failure at call time. Also grep installed skills
     for retired third-party tracker names — zero hits. Any residual
     reference to a retired tracker is a stale-rewrite regression; the
     canonical tracker set is `backlog.d/` plus GitHub issues.
  7. **AGENTS.md debt map:** zero `(unfiled)` entries. Every P0 has
     a filed tracker ID.

  Silent skip is the failure mode that ships B+ output when A is
  the bar.
- **Keep deterministic checks objective.** Self-audit is for
  presence/absence regressions, exact-copy detection, missing
  agents/scripts, and debt-map hygiene. Do not invent semantic
  scorecards, file-size thresholds, or "rewrite depth" heuristics.
  Planner → executor → critic owns semantic quality.
- Preserve self-containment. When you copy or rewrite a skill, its
  `references/` and `scripts/` stay with it.
- **Cross-harness install.** Spellbook-distributed skills live in a
  shared repo-local skill root (`.agent/skills/` or
  `.agents/skills/`). Harness-specific skill dirs such as
  `.claude/skills/` are symlink bridges back to that shared root,
  not duplicate copies. A repo installed Claude-only is harness-
  locked by construction. AGENTS.md at the repo root is already
  harness-neutral. Per-harness settings files (`.claude/settings.
  local.json`, `.codex/config.toml`, `.pi/settings.json`) are still
  emitted per-harness because their formats differ. Agents may stay
  in a harness-native agent directory until the repo has a documented
  shared-agent convention.
- **Non-destructive by default.** Never delete content in the shared
  skill root or any harness dir that lacks a `.spellbook` marker
  with `installed-by: tailor`. If reconciliation wants to remove
  something unmarked, ask the user first. Tailor owns `AGENTS.md`
  (overwrite is fine; pre-existing AGENTS.md without a prior
  tailor run should prompt confirmation) and
  `.spellbook/repo-brief.md` (always overwrite, plus tracked
  compatibility copy when `.spellbook/` is ignored).
- **Settings merge, not overwrite.** When writing any harness
  settings file, merge with the existing file additively —
  user-added permissions entries survive. Only the entries this
  run derived from actual tool usage are your contribution.

## What "tailored" means

At the SKILL.md level: examples are repo-specific, commands name the
actual command the user runs, gotchas point to real files, and the
skill reads like it was written for this codebase. A generic body plus
a "repo notes" appendix is not tailored; rewrite the body itself.

## References

- `references/focus-postmortem.md` — critic's rejection checklist.
