# Add role-scoped bootstrap bundles

Priority: P1 · Status: in-progress · Estimate: L

## Goal

Let bootstrap project a small, role-appropriate subset of Harness Kit skills
instead of loading the entire catalog into every session by default.

## Oracle

- [x] A bundle manifest or generated config defines at least five role-scoped
      bundles (`lead`, `implementer`, `critic`, `designer`, `ops` or equivalent)
      with 8-15 default skill descriptions each. (`.harness-kit/bundles.yaml`
      — `lead`/`implementer`/`critic`/`designer`/`vault`, 8-21 skills each
      counting first-party + routed vendored externals; `designer` runs
      larger since the /design bench alone is ~17 specialists — a first cut,
      not a settled taxonomy, see child 5.)
- [x] `bootstrap` can install a selected bundle for a system or repo harness
      without losing the current full-catalog mode. (`bootstrap --bundle
      <name>`; omitting the flag installs the full catalog exactly as
      before — verified live, byte-for-byte identical skill count.)
- [x] A dry-run or fixture bootstrap reports before/after projected skill count
      and estimated description bytes. (`bootstrap --dry-run [--bundle
      <name>]`, no filesystem writes.)
- [x] Existing detected harness installs keep working with the default behavior,
      or the migration is explicit and reversible. (Default `bootstrap` with
      no flags is unchanged; verified live in a scratch `$HOME` — 24
      first-party + 27 vendored = 51 skills, same as before this change.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Verification System

- Claim: role-scoped projection reduces session description tax while preserving
  a reversible full-catalog path for broad operator sessions.
- Falsifier: bootstrap cannot reproduce the selected bundle; a role bundle omits
  a required first-party dependency; default installs unexpectedly shrink; the
  generated index/docs disagree with projected skills.
- Driver: bootstrap fixture/dry run for each role, generated count/byte report,
  install-path checks, and the aggregate repo gate.
- Grader: bundle manifests resolve to existing skills; dry-run output shows the
  intended count reduction; default mode remains available; gate output is
  green.
- Evidence packet: bundle manifest diff, dry-run transcripts, and before/after
  count table.
- Cadence: pre-merge for each bundle change, then telemetry review after real
  sessions use bundles.

## Children

1. [x] Decide the minimal role vocabulary from Weave/Factory usage rather than
   creating persona theater. (`lead`/`implementer`/`critic`/`designer`/`vault`
   — grounded in each skill's actual function and the /design routing table,
   not a riffed persona list; membership documented inline in
   `bundles.yaml`'s comments.)
2. [x] Add bundle declarations that are data, not duplicated skill prose.
   (`.harness-kit/bundles.yaml` — skill directory names only.)
3. [x] Teach bootstrap to select a bundle for system and repo harness projections
   while preserving full mode. (`--bundle NAME`; unknown names fail loudly
   listing valid ones; a bundle referencing a deleted skill fails loudly
   too — verified via tests, not just happy path.)
4. [x] Add a dry-run/count report so token savings are visible before installation.
   (`--dry-run`; live numbers: full catalog ~21.4k description bytes, each
   bundle ~4-8k — roughly 60-81% smaller, directionally consistent with the
   ~12k-tokens-to-<=5k target, though bytes are a proxy for tokens, not an
   exact conversion.)
5. [ ] Validate bundles against telemetry and update defaults only after evidence.
   **Not yet done** — requires real sessions to actually use `--bundle` first
   (bootstrap.sh and the CLI both support it now); no lead agent or harness
   config wires a default bundle selection yet, so this stays opt-in only
   until usage evidence exists.

## Notes

Operator decision, 2026-07-01: "Role-scoped bootstrap bundles (~12k
tokens/session -> target <=5k)."

This extends, rather than reverts, the older skill-catalog tailoring work. Keep
the interface smaller than the implementation; avoid one-off hardcoded harness
branches when a manifest plus bootstrap selection is enough.

**2026-07-01 — landed.** `.harness-kit/bundles.yaml` + `bootstrap --bundle
<name> [--dry-run]`, implemented in `crates/harness-kit-checks/src/bundles.rs`
(kept separate from `bootstrap.rs` to avoid tipping it over the god-file
ceiling — see `BOUNDARIES.md`-style reasoning, though this crate's own
boundary work is backlog 129, a parallel epic). Child 5 (telemetry-driven
membership correction) is explicitly deferred until bundles see live use.
