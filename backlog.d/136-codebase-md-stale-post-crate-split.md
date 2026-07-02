# Update CODEBASE.md for the harness-kit-hooks / harness-kit-roster split

Priority: P2 · Status: ready · Estimate: S

## Goal

Bring `CODEBASE.md`'s "The Rust Crate" section back in sync with the
checks-crate diet (backlog 129, `_done/`, closed tonight): `claude_hooks` and
the `agent_roster`/`lane_harness`/`source_refs`/`summarize_delegations`
cluster no longer live in `crates/harness-kit-checks` — they moved to
`crates/harness-kit-hooks` and `crates/harness-kit-roster` respectively.

## Oracle

- [ ] `CODEBASE.md`'s crate section lists three crates
      (`harness-kit-checks`, `harness-kit-hooks`, `harness-kit-roster`)
      matching `ls crates/`, each with its actual module contents.
- [ ] The `git_hooks` + `claude_hooks` bullet (currently `CODEBASE.md:45-47`)
      is corrected: `git_hooks` stays in `harness-kit-checks`
      (pre-commit/pre-push/etc. dispatch), `claude_hooks` moved to
      `harness-kit-hooks`.
- [ ] The `agent_roster` + `lane_harness` + `source_refs` +
      `summarize_delegations` bullet (currently `CODEBASE.md:53-55`) is moved
      under a `harness-kit-roster` heading.
- [ ] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Notes

Confirmed live: `ls crates/` shows `harness-kit-checks`, `harness-kit-hooks`,
`harness-kit-roster`; `grep -rn "harness_kit_checks::claude_hooks\|harness_kit_checks::agent_roster"`
across `*.rs`/`*.md`/`*.sh` returns nothing, i.e. the split itself is clean —
only the prose map in `CODEBASE.md` still describes the pre-split boundary.
129's own closing note (`backlog.d/_done/129-diet-checks-crate-boundaries.md`)
reports the post-split LOC count (12.3k/24 files, down from 18.1k/26) but the
PR that landed the split (#147, #154 per `git log`) did not touch
`CODEBASE.md`, so the source-of-truth map is now the one place still
describing the old, pre-diet crate boundary.

**Why:** `CODEBASE.md` is the file a cold agent reads first to understand
crate ownership; it currently tells that agent to look for `claude_hooks` and
`agent_roster` inside `harness-kit-checks`, which is now false and will send
the next session down the wrong module path.
