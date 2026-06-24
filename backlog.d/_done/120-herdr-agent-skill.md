# Add Herdr agent skill as external source

Priority: P3 · Status: open · Estimate: S

## Goal
Vendor the official Herdr `SKILL.md`
(https://github.com/ogulcancelik/herdr/blob/master/SKILL.md) as an externally-managed
skill so any agent running inside a Herdr pane (`HERDR_ENV=1`) auto-loads the
control instructions: split panes, spawn sibling agents, read output, wait on
agent state, and coordinate across the herd.

## Problem
Herdr's `SKILL.md` lives at the **repository root**, not inside a `skills/`
subdirectory. `sync-external` → `discover_skills` (in
`crates/harness-kit-checks/src/external_sync.rs`) only scans for subdirectories
of `skills_path` that contain a `SKILL.md`:

```rust
pub fn discover_skills(root: &Path) -> Result<Vec<String>> {
    for entry in fs::read_dir(root)? {
        if entry.metadata()?.is_dir()
            && entry.path().join("SKILL.md").is_file()
        { skills.push(name.to_string()); }
    }
}
```

With `skills_path: "."` the skill root is the checkout dir itself, which has no
`SKILL.md`-bearing subdirectory — it has `SKILL.md` directly. The tooling bails
with "no skills found."

## Approach
Extend `discover_skills` to also recognize a root-level `SKILL.md` (i.e. when
`skills_path` points at a directory that itself contains `SKILL.md`, treat that
directory as a single skill named from the repo or an explicit `skill_name`
field). Two options:

- **A (preferred):** Add an optional `skill_name` field to `RawSource` /
  `SourceEntry`. When set, `sync_source` skips directory discovery and installs
  the `skills_path` dir directly as that named skill. Minimal, explicit, no
  heuristic.
- **B:** Make `discover_skills` return a synthetic `"."` entry when the root
  itself has `SKILL.md`. More magic, harder to name the alias.

Then add to `registry.yaml`:

```yaml
- repo: ogulcancelik/herdr
  ref: master
  pin: <sha at ticket time>
  layout: flat
  skills_path: "."
  alias_prefix: "herdr-"
  skill_name: herdr          # new field, option A
  include: [herdr]
```

Alias: `herdr-herdr` (doubled, consistent with `emil-emil-design-eng` precedent).

## Acceptance
- `cargo run --locked -p harness-kit-checks -- sync-external` installs
  `skills/.external/herdr-herdr/SKILL.md` from the pinned sha.
- `sync-external --check` reports no drift.
- `cargo run --locked -p harness-kit-checks -- check --repo .` stays green.
- New test in `external_sync.rs` covering `skill_name` override / root-level
  `SKILL.md` discovery.
- `bootstrap` installs the skill system-wide (it already syncs externals).

## Non-Goals
- Forking or editing the upstream `SKILL.md`. Vendored as-is at the pin.
- A Herdr harness bridge or plugin beyond the skill file.
- Wiring `HERDR_ENV` detection into bootstrap — the skill's own guard handles it.
