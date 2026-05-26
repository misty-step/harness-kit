# Rebrand the shared harness library

Priority: P0 - highest
Status: ready
Estimate: M

## Goal

Choose and execute a new name for this repo before the next architecture pass
lands. The current "Spellbook" name still points at a fantasy metaphor and at
static catalogs of spells/agents. The project is becoming a practical shared
harness library: skills, repo guidance, projections, and bootstrap support for multiple
agent runtimes.

The rebrand should make that product shape obvious.

## Non-Goals

- Do not change the architecture while renaming. This ticket is naming,
  repository identity, docs, command names, package names, and public wording.
- Do not rename every internal `.spellbook/` state path until the replacement
  name is chosen and migration compatibility is specified.
- Do not pick a clever name that obscures the tool's job. Prefer practical,
  harness-oriented language over metaphor.
- Do not rewrite backlog history. Closed tickets may keep old names.

## Oracle

- [ ] A replacement name is chosen with a short rationale and rejected-name
      notes. The rationale explicitly says why it fits a shared harness repo.
- [ ] `README.md`, `project.md`, `AGENTS.md`, `CLAUDE.md`, `bootstrap.sh`,
      `registry.yaml`, `index.yaml`, and active skill descriptions no longer
      present "Spellbook" as the product name, except in compatibility notes.
- [ ] The bootstrap entrypoint preserves a compatibility path for the old repo
      checkout and any existing `.spellbook/` local state.
- [ ] The repo identity, package names, generated index headers, hook messages,
      and Dagger module names are renamed or explicitly deferred with a tracked
      migration reason.
- [ ] At least one downstream repo is bootstrapped or seeded after the rename to
      prove the old name is not required by harness artifacts.
- [ ] `dagger call check --source=.` passes.

## Notes

### Why this is first

Naming is now blocking architecture. If we keep landing new work under
"Spellbook," every new file bakes in the old metaphor and makes the eventual
rename larger. The highest-leverage first move is to choose the product identity
that matches the actual direction: a shared harness primitive library.

### Naming criteria

- Says "harness," "primitive," "workflow," "craft," "kit," or another practical
  systems word without needing explanation.
- Works as a repo name, command prefix, config directory, and sentence in docs.
- Does not imply orchestration, fleet management, memory, or policy enforcement.
- Feels compatible with Claude Code, Codex, Antigravity, and Pi instead of
  belonging to one vendor.

### Compatibility contract

Keep `.spellbook/` as the legacy state directory until a later migration ticket
proves it can be renamed without breaking existing repos. The product can
rebrand before local state paths do.
