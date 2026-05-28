# Skill catalog tailoring — per-project subset selection

Priority: P0
Status: pending
Estimate: S (MVP ~1-2 days)

100 skills symlinked globally. ~12K tokens injected per Claude Code
turn; comparable on Codex/Pi. Most are irrelevant to any given
project. The priority is per-project selection: enable a scoped
subset, disable the rest.

This is urgent. Every new skill we add makes every unrelated session
more expensive.

## Shape

A skill — `/tailor-skills`, or extend the existing `/tailor` (029) —
where the agent:

1. Reads the target repo (language, frameworks, test setup,
   package.json / Cargo.toml / etc.).
2. Reads the catalog of skill descriptions on disk (the frontmatter,
   not full SKILL.md).
3. Picks the 10–20 skills that actually fit this repo's likely tasks.
4. Writes the selection to `.harness-kit.yaml` (committed to the repo).
5. Re-runs bootstrap. Bootstrap honors the selection and only
   symlinks those skills into this repo's harness dirs.

That's it. No bundle manifests. No tier metadata on every skill. No
render scripts. No plugin JSON emission. No lint rules for
compliance. The agent judges; the filesystem records.

## Cross-harness by construction

Every harness scans its skills dir. Symlink 15 skills, every harness
sees 15. Claude, Codex, Pi — identical.

Runtime-toggle layers (Claude `enabledPlugins`, Codex `/plugins`)
exist and might enable faster re-select without re-bootstrap later.
Not MVP. Filesystem is the base.

## What bootstrap needs

One small change: if `$PROJECT/.harness-kit.yaml` declares a `skills:`
list, bootstrap symlinks only those (plus any the agent names as
dependencies). Otherwise current behavior. ~20 lines of bash at most.

## Stretch (separate, later)

Once selection works: repo-specific *versions* of generic skills,
written by the agent using the generic as a template. QA is the
canonical case — generic `/qa` is nearly useless; a repo-specific
`/qa` that knows this project's test runner, fixtures, flaky specs,
and QA gates is a force multiplier. This overlaps with 029 `/tailor`
which already targets per-repo artifact generation; fold in there
when time comes.

## Oracle

- [ ] On an unfamiliar repo, the skill picks a scoped subset (e.g.
      10-15 from 100) and can justify each pick in plain text.
- [ ] Catalog injection on that repo measurably drops (ballpark
      ≥50%) on Claude AND Codex.
- [ ] Same repo behaves identically across Claude, Codex, Pi — the
      subset is filesystem-recorded, so all three harnesses converge.
- [ ] Opt-out is trivial: delete `.harness-kit.yaml` → global install
      behavior restored.

## Non-Goals

- Bundle manifests, tier-metadata frontmatter, render scripts,
  plugin JSON emission, lint rules for tier compliance. Those are the
  scripted-scaffolding pattern the `/flywheel` refactor (commit
  7ccd00d) rejected: agent judgment, not deterministic orchestration.
- Building runtime-toggle plumbing before filesystem-level is proven
  sufficient.
- Template → repo-specific skill generation (stretch; separate
  initiative, adjacent to 029).

## Related

- 029 `/tailor` — per-repo artifact generation (AGENTS.md, settings,
  eventually skills). Adjacent; the stretch goal folds there.
- Thin-harness refactor of /flywheel (7ccd00d) is the form this
  ticket should match. Scripts only where they genuinely beat agent
  judgment.

## What Was Built

Shipped via merge commit `3b9b4b8` to master, pushed to origin
2026-04-16.

- `skills/tailor-skills/SKILL.md` — 90-line thin-form skill. Reads
  current repo, reads `index.yaml`, picks 10-20 skills, writes
  `.harness-kit.yaml` to project root. Invariants: committed file,
  reversible, agent judges, justify each pick.
- `bootstrap.sh` — allowlist filter after `discover_local()`. Reads
  `.harness-kit.yaml` from `git rev-parse --show-toplevel` (subdir-safe).
  Sentinel parser scheme (`PRESENT` / `PARSE_FAIL`) cleanly distinguishes
  empty-list (fail-loud) from missing/malformed (fall through to global).
  Extends `force_per_entry` so the allowlist-active state escapes the
  parent-dir symlink shortcut. ~40 lines of net logic.
- `scripts/test-bootstrap-filter.sh` — 8 cases covering: no file, valid
  allowlist, malformed YAML, unknown names, empty list, missing key,
  null value, subdir invocation. Uses `HARNESS_KIT_TEST_MODE=1` probe
  to dump post-filter state without touching harness dirs.
- Removed pre-existing stale `.harness-kit.yaml` stub in harness-kit root
  (`skills: []` documentation placeholder that became fail-loud after
  the sentinel fix).

Dogfood: ran on `~/Development/bitterblossom`, 101 skills → 17 (83%
reduction). Cross-harness parity verified across Claude/Codex/Pi.
Dagger CI 11/11 green. Bench verdict: ship (critic, ousterhout, grug,
carmack; critic needed a second pass — two real blockers caught and
fixed).

Stretch goal (template → repo-specific skill generation) deferred to a
follow-on ticket; adjacent to 029.

Remote-install allowlist support explicitly deferred — see new
follow-up ticket.
