# Harness pivot: minimal globals, per-repo primitives

Priority: P0 (shipped)
Status: done
Estimate: M
Shipped: 2026-04-18
Supersedes: 029 (MVP redesigned), 043 (`/tailor-skills` deleted)

## What changed

Harness Kit's distribution model flipped from "install the full catalog
globally, allow per-project filters" to "install nearly nothing
globally, let per-repo skills populate `.claude/` on demand."

**Before the pivot:**

- `~/.claude/skills/` held 80+ symlinks (first-party + externals).
- `/tailor-skills` wrote a per-project `.harness-kit.yaml` allowlist.
- `bootstrap.sh` re-ran and filtered `~/.claude/skills/` globally,
  meaning the last-pruned repo silently dictated skill availability
  in every OTHER repo.
- `/tailor` MVP wrote `AGENTS.md` + `settings.local.json` only, with
  a Phase 4 A/B killswitch and a `tailor-lint.sh` pre-commit hook.

**After the pivot:**

- `~/.claude/skills/` holds exactly two symlinks: `/tailor` and
  `/seed`. Agents remain globally available (they're lightweight and
  used by many workflows).
- `/tailor-skills` is deleted. Premise is subsumed: if a repo wants a
  scoped set, running `/tailor` installs only the primitives that
  fit.
- `/tailor` is a natural-language skill (~85 lines, no state machine,
  no killswitch). Explores the repo, reads prior session history,
  browses harness-kit, picks relevant primitives, copies them into
  `.claude/skills/` and `.claude/agents/`, specializes where
  concrete, writes `AGENTS.md` and `settings.local.json`.
- `/seed` is a new ~35-line skill. Dumb copy of most harness-kit
  primitives into the current repo. For when you want something
  working fast and will curate by hand.
- `bootstrap.sh` hard-codes `GLOBAL_SKILLS=(tailor seed)`. The
  allowlist parser, `HARNESS_KIT_TEST_MODE` probe, and
  `EXTERNAL_SKILLS[]` machinery are gone.
- A/B killswitch (`tailor-ab-spike.sh`, `tailor-ab.sh`) and
  pre-commit lint (`tailor-lint.sh`) deleted. Replaced by agent
  judgment — the critic's `focus-postmortem.md` checklist is
  applied at pick-time, no post-hoc verification.

## Why

Three forces converged:

1. **The "global blown up" bug.** Per-project allowlists mutating
   shared `~/.claude/skills/` state was the wrong mechanism on Claude
   Code's current skill model (no native per-project scoping).
2. **Bitter-lesson audit.** Audit showed the MVP had ~60% over-
   engineering — deterministic scoring, flowcharts, state machines —
   where agent judgment was simpler and better. Cross-referenced
   against Anthropic/OpenAI/Vercel exemplar skills (all natural-
   language, mostly bullets + guardrails).
3. **Architectural cleanup.** Minimal globals + per-repo population
   is the only design where context bloat mitigation is automatic
   (nothing loads that the repo didn't opt into) AND no shared
   state pollutes other projects.

## Migration notes for existing users

After this lands, `~/.claude/skills/` shrinks from ~80 to 2 symlinks
on next bootstrap. Repos that already had `.claude/skills/` content
(e.g., via prior `/qa scaffold`) are unaffected. Repos without
`.claude/` get nothing until the user runs `/seed` or `/tailor`.

Stale `.harness-kit.yaml` files in existing repos are now no-ops — safe
to delete.

## References

- `skills/tailor/SKILL.md` — the rewritten skill
- `skills/seed/SKILL.md` — the new dumb-copy skill
- `skills/harness/SKILL.md` — principle 10 (prose over programs)
- `bootstrap.sh` — simplified installer
- `backlog.d/_done/029-tailor-per-repo-harness-generator.md` —
  original `/tailor` spec (most superseded; overlay-pattern v2
  section retained as design sketch)
- `backlog.d/_done/043-skill-catalog-tailoring.md` — original
  `/tailor-skills` (deleted)
