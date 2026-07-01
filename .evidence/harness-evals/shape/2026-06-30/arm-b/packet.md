# Spec: `--top <N>` flag for `harness-kit-checks telemetry`

## 1. Goal & premise

Add a `--top <N>` flag to the `telemetry` subcommand (alias `skill-invocation-analytics`) that caps the **Skill Frequency** section of the report to the N most-used skills, ranked by invocation `count` (descending). When omitted, behavior is unchanged (all skills shown). The flag is the natural complement to the existing `--since` / `--repo` / `--project` / `--skill` filters: it narrows *how much* of the ranked frequency table is emitted without re-running anything.

This is a small, additive CLI change. It rides the existing rank ordering that `analyze` already computes, so there is no new sorting logic — only a truncation.

## 2. Scope decision (read this first)

`--top N` limits **only the `skills` array** (the "Skill Frequency" table). It does **not** filter transitions, work sequences, source/harness coverage, delegation usage, or warnings.

Rationale:
- The task says "N most-used skills (by invocation count)." Only `report["skills"]` is the per-skill, count-ranked list. The other sections are relational (transitions are skill→skill edges; a transition involving a non-top-N skill has no well-defined "rank") or infrastructural (coverage/warnings are about data sources, not skill popularity). Filtering them would be a larger, ambiguous semantic change and is out of scope.
- In the code, `skills` is built independently of `transitions`/`sequences`/`coverage` (all of those read from `skill_rows` / `sessions`, not from the `skills` vec), so truncating `skills` is naturally isolated and affects no other section. See `skill_invocation_analytics.rs:70-194`.

If a future ticket wants "top-N everywhere," that is a separate, deliberate change — call it out, do not silently widen this one.

## 3. Exact change set

Two files. All line numbers are against commit `3bf0b46`.

### 3.1 `crates/harness-kit-checks/src/skill_invocation_analytics.rs`

**(a) Add the field to `AnalyzeOptions`** (struct at lines 18-27). Place it last, after `skill`:

```rust
pub skill: String,
pub top: Option<usize>,
```

`Option<usize>` (not a plain `usize` sentinel) so "no limit" is unambiguous and `None` is the trivial default. `usize` because it feeds `Vec::truncate`.

**(b) Truncate after the existing rank sort** (the `skills.sort_by(...)` block ends at line 93). Immediately after it, before `transition_counts` is built at line 95, add:

```rust
if let Some(top) = options.top {
    skills.truncate(top);
}
```

`Vec::truncate` is the whole implementation: it is a no-op when `top >= skills.len()`, and yields an empty vec when `top == 0`. The sort at lines 87-93 already establishes the exact ranking (count desc, then skill name asc as a stable tiebreak), so "first N after sort" *is* "N most-used." No other code in `analyze` reads `skills`, so this is the only touch point for the report body.

**(c) Update the `self_test()` fixtures and add a contract assertion.** `self_test` constructs `AnalyzeOptions` twice (lines 291-299 and 337-345); both need the new field:

```rust
skill: String::new(),
top: None,
```

Then add a third `analyze` call plus assertions, reusing the already-written `skill_log` fixture (which has `shape` count=2, `implement` count=1, so the expected top-1 result is deterministic). Insert after the existing `skills[0] == "shape"` assertion (around line 303) or alongside the other `analyze` calls:

```rust
let top_report = analyze(&AnalyzeOptions {
    skill_log: skill_log.clone(),
    work_ledger: work_ledger.clone(),
    delegations: delegations.clone(),
    since: String::new(),
    repo: String::new(),
    project: String::new(),
    skill: String::new(),
    top: Some(1),
})?;
ensure(
    top_report["skills"].as_array().map(Vec::len) == Some(1),
    "--top 1 should yield exactly one skill",
)?;
ensure(
    top_report["skills"][0]["skill"] == "shape",
    "--top 1 should keep the most-used skill",
)?;
```

Note `skill_log`/`work_ledger`/`delegations` are `PathBuf` moved into the first `analyze` call today; add `.clone()` at the earlier call sites (or reorder) so the paths remain usable for this third call. The existing first `analyze` at line 291 currently moves them — switch those three to `.clone()` there as well.

### 3.2 `crates/harness-kit-checks/src/main.rs`

**(a) Default in the parser** (`parse_skill_invocation_analytics_args`, struct literal at lines 1161-1169). Add:

```rust
skill: String::new(),
top: None,
```

**(b) Match arm** (the `match flag` block, lines 1177-1197). Add alongside the other value-taking flags (e.g. after `--skill` at line 1188):

```rust
"--top" => options.top = Some(parse_u64("--top", &value()) as usize),
```

Reuse the existing `parse_u64` helper (lines 842-847) — it already prints `"--top must be a non-negative integer"` and exits `2` on bad input, matching the repo's flag-error convention. Cast to `usize` (safe on the 64-bit targets this builds for). This keeps the value-consuming control flow identical to `--since`/`--repo`/etc.: the closure `value()` reads `args[index]`, and the loop's trailing `index += 1` at line 1199 advances past the consumed value. Do **not** add a `continue` (that path is only for the valueless `--self-test` flag).

**(c) Usage string** (line 938). Add `[--top N]` to the telemetry line, e.g. after `[--skill NAME]`:

```
harness-kit-checks telemetry [--skill-log PATH] [--since 7d|12h] [--repo NAME] [--project NAME] [--skill NAME] [--top N] [--format json|text|markdown] [--self-test]
```

## 4. Behavior table / edge cases

| Input | Result |
|---|---|
| flag omitted | `top = None` → all skills (unchanged behavior) |
| `--top 2` with 5 skills | first 2 by rank |
| `--top 99` with 5 skills | all 5 (`truncate` no-op) |
| `--top 0` | empty `skills` array; markdown/text render the existing "none" fallback row (`skill_invocation_analytics.rs:661-663`). Allowed — harmless, no special-casing. |
| `--top abc` / `--top -1` | `parse_u64` prints `--top must be a non-negative integer`, exit 2 |
| `--top` with no value | `value()` closure hits `usage()`, exit 2 (same as other value flags) |
| combined with `--since`/`--repo`/`--skill` | filters apply first (they shrink `skill_rows`), then ranking, then `--top` truncates the ranked result — correct composition |
| `--format json\|text\|markdown` | all three read `report["skills"]`, so the cap applies uniformly to every format |

## 5. Verification loop (the proof, not just green)

1. **Unit/contract:** `cargo test --locked -p harness-kit-checks` — the extended `self_test_contract_passes` now asserts top-1 truncation against the in-repo fixture. This is the primary oracle.
2. **Self-test path:** `cargo run --locked -p harness-kit-checks -- telemetry --self-test` → prints `analyze-skill-invocations self-test ok`, exit 0.
3. **Live manual walk** — write a throwaway JSONL with ≥3 distinct skills at distinct counts (e.g. `shape`×3, `deliver`×2, `diagnose`×1), then read the evidence, not just the exit code:
   - `telemetry --skill-log /tmp/fx.jsonl --format json --top 2` → assert `.skills | length == 2` and the two entries are the highest-count skills (pipe through `jq '.skills | map(.skill)'`).
   - `--top 0` → `.skills == []`; markdown shows the `| none | 0 | dead | ... |` row.
   - `--top 99` → equals the un-capped run.
   - `--top abc` → stderr `--top must be a non-negative integer`, exit 2 (`echo $?`).
   - `--top 1 --format markdown` and `--format text` → frequency section shows exactly one row, other sections intact.
4. **Repo gate (must stay green):** `cargo run --locked -p harness-kit-checks -- check --repo .` plus `cargo fmt --check` and `cargo clippy --locked -- -D warnings` (the standard Rust floor for this crate).

Evidence to capture in the PR/receipt: the `jq` output for `--top 2` and `--top 0`, the exit-2 transcript for `--top abc`, and the `cargo test` summary line.

## 6. Out of scope / non-goals

- No filtering of transitions, work sequences, coverage, harness coverage, delegation usage, or warnings (see §2).
- No new sort modes (no `--top` by tokens or cost — `count` is the only ranking key; the existing sort is reused verbatim).
- No `--bottom`/`--tail` inverse flag.
- No change to default output (omitting `--top` must be byte-identical to today).
- No new dependencies, no new helper functions if `parse_u64` suffices (delete-before-add: prefer the cast over a bespoke `parse_usize`).

## 7. Why this is the minimal correct shape

The ranking already exists and is already stable (`sort_by` at `skill_invocation_analytics.rs:87-93`); `--top` is one `Vec::truncate` plumbed through one new `Option<usize>` field, mirroring the four filters that already live on `AnalyzeOptions`. Parsing reuses `parse_u64`; errors reuse the existing exit-2 convention; all three render formats inherit the cap for free because they read the same `report["skills"]`. Total surface: ~6 added lines of logic across two files, plus the usage string and a self-test assertion. The interface ( `--top N` ) is far simpler than asking the operator to post-process JSON with `jq`, which is the deep-module test this passes.

**Touch list:** `skill_invocation_analytics.rs` (struct field, truncate, self_test fixtures+assert) and `main.rs` (struct default, match arm, usage string). No other files. The eval fixture (`.evidence/harness-evals/shape/2026-06-30/fixtures/01-telemetry-top/`) confirms the deliverable is this spec packet only — `crates/**` edits are forbidden for this exercise.
