---
name: tailor
description: |
  Install or refresh a repo-local Spellbook harness. Detect the repo, choose
  useful primitives, rewrite workflow skills to fit the repo, preserve
  existing user harness work, and verify the installed harness. Use for
  "tailor this repo", "configure agents for this codebase", "set up a
  harness", "what skills apply here".
  Trigger: /tailor.
---

# /tailor

Install a bespoke repo harness. As small as it should be, no smaller. Not a
catalog dump. Not a generic AGENTS generator.

## Contract

1. **Inventory.** Read the repo: stack, domain, gate, lifecycle, tracker, hot
   paths, prior harness. The shared skill root is `.agents/skills/`. If a repo
   has legacy `.agent/skills/`, migrate deliberately or ask before touching it.
2. **Context.** Dispatch exploration lanes, then synthesize the repo's purpose,
   stack, gate, lifecycle, invariants, debt, terminology, and user corrections.
   Persist `.spellbook/repo-brief.md` only when it will be reused by subagents
   or future runs. If `.spellbook/` is ignored and the brief is load-bearing,
   also write tracked `.claude/.tailor/repo-brief.md`.
3. **Pick.** Use roster lanes: planner proposes portfolio; critic rejects
   shallow or incoherent picks. Domain inventions need concrete repo evidence.
4. **Reconcile.** Marker-owned artifacts may be replaced. Unknown/unmarked
   artifacts are user-owned: preserve or ask `preserve / replace / diff`.
5. **Install.** Write shared skills once to `.agents/skills/`; bridge `.claude/skills/`,
   `.codex/skills/`, `.pi/skills/` back to the shared root. Copy shared scripts
   only when absent or tailor-owned. Merge settings additively.
6. **Adapt.** Workflow skills cover the SDLC. Preserve universal judgment where
   the source already fits; tailor command-bearing and high-variance surfaces
   with this repo's gate, tracker, lifecycle, signals, refusal conditions, and
   file paths. Source skill is reference, not template.
7. **Audit.** Run `skills/tailor/scripts/collect-post-tailor-evidence.py`.
   Dispatch critic on evidence + installed harness. Persist verdict. Repair
   blockers and rerun. Do not declare success before clean verdict.
8. **Close.** Run the repo gate. End with clean git status.

## Buckets

- **Workflow:** `research`, `groom`, `shape`, `implement`, `qa`, `demo`,
  `code-review`, `refactor`, `ci`, `diagnose`, `monitor`, `deliver`,
  `settle`, `ship`, `yeet`, `flywheel`. Install by default. Tailor only where
  repo-specific commands, lifecycle, gates, or domain language materially
  change the contract.
- **Universal:** cross-repo judgment protocols such as `reflect`. Copy
  verbatim unless repo evidence says otherwise.
- **External:** registry aliases under `$SPELLBOOK/skills/.external/<alias>`.
  Install as absolute symlink at shared root plus sibling `<alias>.spellbook`.
  Marker fields: `source`, `alias`, `target`, `category: external`. Marker is
  never inside target. Never write inside the external target/cache.
  Zero frontend externals for non-frontend repos.
- **Agent:** copy only when a real agent directory exists or the repo needs a
  tool-restricted static agent. Prefer ad-hoc roster lanes inside skills.

Kept external: re-resolve/refresh symlink. Dropped external: remove symlink and marker. Harness bridges are relative symlinks back to shared root.

## AGENTS.md Output

Root `AGENTS.md` is a router:

- `Stack & boundaries`
- `Gate contract`
- `Lifecycle`
- `Known debt`
- `Harness index`
- `Invariants`

Keep under six top-level headings and about 650 words unless the user asks for
more. No `(unfiled)`. No generic skill inventory. No "what is a skill" prose.
Mention non-harness-native mechanisms: provider roster, custom gates, local
tracker, clean-tree closeout.

## Acceptance

- No byte-identical workflow skills unless tailoring Spellbook itself.
- QA/demo/monitor are never skipped because browser/deploy is absent.
- CLI or library repos still get CLI-specific QA, demo, and monitor surfaces.
- `/deliver` without args selects highest-priority ready work when the tracker
  has deterministic priority/status.
- `/groom` preserves source contract when tracker storage changes.
- `/ship`, `/settle`, `/groom`, `/flywheel`, `/implement`, `/deliver` agree on
  lifecycle detector and closure signal.
- Evidence collector facts are not a verdict. Critic decides bespoke fit.
- Generated AGENTS is concise, repo-specific, and current.

## References

- `references/focus-postmortem.md` — critic rejection checklist.
- `scripts/collect-post-tailor-evidence.py` — deterministic audit facts.
