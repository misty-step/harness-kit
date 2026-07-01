# Context Packet: `telemetry --top <N>` — cap the Skill Frequency table to the N most-used skills

## Goal

An operator running `harness-kit-checks telemetry` can pass `--top <N>` to limit the **Skill Frequency** section of the report to the N skills with the highest invocation counts, in every output format, deterministically.

## Non-Goals

- **Not** filtering the other report dimensions. `transitions`, `work_sequences`, `coverage`, `harness_coverage`, and `delegation_usage` stay full-fidelity. `--top` caps the skills table only. (Whole-report top-N filtering is a different, larger feature — see Alternative D and the Risks stop-condition.)
- **Not** changing the existing sort key or tiebreak (count desc, then skill-name asc).
- **Not** adding `--top` to any other subcommand.
- **Not** adding offset/pagination, percent thresholds, or a "bottom N" inverse.
- **Not** touching the renderers (`render_markdown`, `render_text`, JSON) — they inherit the cap because truncation happens upstream in `analyze()`.

## Constraints

- `AnalyzeOptions` derives `Debug, Clone, PartialEq, Eq` (`skill_invocation_analytics.rs:18`). Any new field must satisfy all four (`Option<usize>` does).
- `AnalyzeOptions` is constructed in **three** places — the CLI parser (`main.rs:1161`) and **twice** inside `self_test` (`skill_invocation_analytics.rs:291` and `:337`). Adding a field touches all three or it will not compile; the compiler is the backstop.
- **Backward compatibility:** when `--top` is absent, output must be byte-identical to today. Existing callers (`AGENTS.md:75`, `skills/groom/SKILL.md:236`, `skills/harness-engineering/references/mode-audit.md:8`, `README.md:100`) invoke `telemetry` with no `--top`.
- The module owns report shape; `main.rs` only dispatches and parses (`run_skill_invocation_analytics` at `:628`). Keep that separation — do not move report-mutation logic into `main.rs`.
- `telemetry --self-test` is the standing executable oracle for this module (`run_skill_invocation_analytics` exits 0/1 on it). The new behavior must be covered by it.

## Repo Anchors

1. `crates/harness-kit-checks/src/skill_invocation_analytics.rs:18-27` — `struct AnalyzeOptions` (add the `top` field here).
2. `crates/harness-kit-checks/src/skill_invocation_analytics.rs:70-93` — `skills` is built then `sort_by(...)` count-desc, skill-name-asc. **Truncate immediately after the sort (after line 93), before transition computation at line 95.**
3. `crates/harness-kit-checks/src/skill_invocation_analytics.rs:205-357` — `self_test`: two `AnalyzeOptions` constructions (`:291`, `:337`) need the new field; add one assertion proving the cap.
4. `crates/harness-kit-checks/src/skill_invocation_analytics.rs:631-663` — `render_markdown` already emits a `| none | 0 | dead | ... |` placeholder row when `skills` is empty (the behavior `--top 0` would otherwise expose; see Risks).
5. `crates/harness-kit-checks/src/main.rs:1154-1202` — `parse_skill_invocation_analytics_args` (add the `--top` match arm).
6. `crates/harness-kit-checks/src/main.rs:842-847` — `parse_u64(flag, value)` helper (parses u64, prints `"{flag} must be a non-negative integer"` and exits 2 on failure) — reuse it for `--top`.
7. `crates/harness-kit-checks/src/main.rs:938` — the `telemetry` usage line (add `[--top N]`).

## Alternatives

| Option | Why it helps | How it fails | Verdict |
|---|---|---|---|
| **A — field on `AnalyzeOptions`, truncate in `analyze()`** | `--top` lives alongside the other report filters (`since`, `repo`, `project`, `skill`); module stays the single source of report shape; all three formats get the cap for free; the existing `--self-test` oracle can cover it. | Touches 3 `AnalyzeOptions` construction sites. | **CHOSEN** |
| B — parse `--top` in `main.rs`, mutate `report["skills"]` array before `render()` | No struct change, no self-test construction churn. | `main.rs` reaches into the report's JSON internals — a responsibility it does not currently have (hidden coupling between dispatch and report schema); the cap is **invisible to `telemetry --self-test`**, so the feature's oracle is weaker. | reject |
| C — no flag; document `telemetry --format json | jq '.skills[:N]'` (the boring/manual path) | Zero code; works today. | Only the JSON format is cappable; `markdown`/`text` operators get nothing; no `-h` discoverability; in non-json the transitions/coverage tables interleave with the skills table so a naive `head` is wrong. The lazy path is genuinely insufficient for the stated outcome (markdown is the default format). | reject |
| D — filter the **entire** report (transitions, sequences, coverage) to the top-N skill set | One mental model: "focus the whole report on N skills." | Semantically murky — drop or keep a transition from a top-N skill *to* a non-top-N skill? Much larger surface for no asked-for benefit; "limits the report to the N most-used skills" reads as the skills list, not a graph-pruning operation. | reject (scope creep) |

**Recommendation: Option A.** It is the smallest change that keeps the cap (a) covered by the existing oracle and (b) uniform across formats, and it places `--top` with its natural peers — the other `AnalyzeOptions` filters.

## Design

**Premise check.** The literal request ("limit the report to the N most-used skills") and the underlying operator outcome ("when I have dozens of skills, show me only the busiest ones so the table is scannable") agree. The one real ambiguity is the word "report": does it mean the **Skill Frequency table** or **every section**? The honest underlying need — a scannable top-of-the-leaderboard view — is served by capping the skills table; pruning the transition graph and coverage tables is neither asked for nor coherent. This packet locks scope to the skills table and flags the ambiguity as a stop-condition.

**Surfaces touched (control flow):**

1. **`AnalyzeOptions`** (`skill_invocation_analytics.rs:18`): add `pub top: Option<usize>` (None = unlimited). `Option<usize>` is the idiomatic representation and satisfies the existing derives; a `usize` 0-sentinel is an acceptable equivalent (builder's choice — see Review "ignore").
2. **`analyze()`**: the `skills` vec is fully built and sorted by `:93`. Insert, right after the `sort_by`:
   ```rust
   if let Some(top) = options.top {
       skills.truncate(top);
   }
   ```
   `Vec::truncate` is a no-op when `top >= skills.len()`, so "N larger than available" needs no special case. Because truncation precedes the transitions/sequences/coverage computation (which all read from `skill_rows`, not from `skills`), those sections remain comprehensive — exactly the Non-Goal boundary. Because it is upstream of `render`, all three formats reflect the cap.
3. **`self_test()`**: set `top: None` in both existing constructions (`:291`, `:337`). Add a third `analyze` call with `top: Some(1)` over the same fixture and assert `report["skills"]` has length 1 and `report["skills"][0]["skill"] == "shape"`. (Fixture counts: `shape`=2, `implement`=1, so sorted-then-capped-to-1 is deterministically `[shape]`.)
4. **`parse_skill_invocation_analytics_args`** (`main.rs:1154`): add a match arm. Reuse `parse_u64("--top", &value())`, then reject `0`:
   ```rust
   "--top" => {
       let n = parse_u64("--top", &value());
       if n == 0 { eprintln!("--top must be a positive integer"); std::process::exit(2); }
       options.top = Some(n as usize);
   }
   ```
   The existing `value` closure already exits 2 (`usage()`) when the value is missing, and `parse_u64` already exits 2 on a non-integer — so the only new guard is the `0` rejection.
5. **`usage()`** (`main.rs:938`): insert `[--top N]` into the telemetry line, e.g. after `[--since 7d|12h]`.

**Data flow:** `--top` is config-only; it changes which rows of an already-computed, already-sorted list survive. No new I/O, no schema change to the input JSONL, no change to `usage`/cost aggregation.

**Decision (ADR-grade):** `--top` is a **report filter**, modeled like `--since`/`--skill`, applied *after* those row filters and *after* the sort. Its scope is the Skill Frequency table only. It is rejected for `0` and composes with all other filters (e.g. `--skill X --top 5` yields at most one skill anyway — harmless).

## CLI Surface

(per `references/cli-design.md`)

- **Command tree:** `harness-kit-checks telemetry [...] [--top N]` (alias `skill-invocation-analytics`, `main.rs:140`).
- **Usage (updated line 938):**
  `harness-kit-checks telemetry [--skill-log PATH] [--since 7d|12h] [--top N] [--repo NAME] [--project NAME] [--skill NAME] [--format json|text|markdown] [--self-test]`
- **Args/flags:** `--top N` — `N` is a positive integer (≥ 1). Optional; absent ⇒ no limit. Order-independent (parser is a `while` loop).
- **Primary user:** both — humans read `--format markdown` (default) / `text`; scripts read `--format json`. All reflect the cap.
- **Output contract:** caps `skills` (JSON array) / the Skill Frequency table (markdown/text) to the first `N` post-sort rows. Other sections unchanged. Primary output → stdout; diagnostics → stderr (unchanged).
- **Error / exit-code map:**
  - `--top 3` → exit 0, ≤3 skills shown.
  - `--top` with no value → exit 2 (existing `value()`→`usage()`).
  - `--top abc` (non-integer) → exit 2, stderr `--top must be a non-negative integer` (from `parse_u64`).
  - `--top 0` → exit 2, stderr `--top must be a positive integer`.
  - `--top 999` (> distinct skills) → exit 0, all skills shown (no error).
- **Config/env precedence:** flag-only; no env or config knob (consistent with the other telemetry filters).
- **Safety controls:** read-only command; no destructive operations; `--top` adds none.
- **Examples:**
  ```bash
  # top 5 busiest skills, human-readable (default markdown)
  harness-kit-checks telemetry --top 5
  # top 3 as JSON for a script
  harness-kit-checks telemetry --top 3 --format json | jq -r '.skills[].skill'
  # compose with a window
  harness-kit-checks telemetry --since 30d --top 10 --format markdown
  ```

## Oracle

(executable definition of done — per `references/executable-oracles.md`)

**Automated (all must exit 0):**

```bash
cd /Users/phaedrus/Development/harness-kit

# 0) Standing module oracle, now covering the cap (self_test asserts top:Some(1) -> 1 skill "shape")
cargo run --locked -p harness-kit-checks -- telemetry --self-test            # prints "...self-test ok", exit 0

# Build a 3-skill fixture (a,a,a / b,b / c) to make ordering deterministic
F=/tmp/top-fixture.jsonl
printf '%s\n' \
  '{"ts":"2026-06-04T00:00:00Z","harness":"claude","source_protocol":"post_tool_use","skill":"alpha","session_id":"s1","project":"p"}' \
  '{"ts":"2026-06-04T00:01:00Z","harness":"claude","source_protocol":"post_tool_use","skill":"alpha","session_id":"s1","project":"p"}' \
  '{"ts":"2026-06-04T00:02:00Z","harness":"claude","source_protocol":"post_tool_use","skill":"alpha","session_id":"s1","project":"p"}' \
  '{"ts":"2026-06-04T00:03:00Z","harness":"claude","source_protocol":"post_tool_use","skill":"beta","session_id":"s1","project":"p"}' \
  '{"ts":"2026-06-04T00:04:00Z","harness":"claude","source_protocol":"post_tool_use","skill":"beta","session_id":"s1","project":"p"}' \
  '{"ts":"2026-06-04T00:05:00Z","harness":"claude","source_protocol":"post_tool_use","skill":"gamma","session_id":"s1","project":"p"}' > "$F"

# 1) cap to 2 -> exactly two skills, the two highest-count ones, in order
test "$(cargo run --locked -q -p harness-kit-checks -- telemetry --skill-log "$F" --top 2 --format json | jq '.skills | length')" = "2"
test "$(cargo run --locked -q -p harness-kit-checks -- telemetry --skill-log "$F" --top 2 --format json | jq -r '.skills | map(.skill) | join(",")')" = "alpha,beta"

# 2) N greater than available -> all three, no error
test "$(cargo run --locked -q -p harness-kit-checks -- telemetry --skill-log "$F" --top 99 --format json | jq '.skills | length')" = "3"

# 3) default (no --top) unchanged -> all three
test "$(cargo run --locked -q -p harness-kit-checks -- telemetry --skill-log "$F" --format json | jq '.skills | length')" = "3"

# 4) cap reaches non-JSON formats too: markdown Skill Frequency table has exactly 1 data row under --top 1
cargo run --locked -q -p harness-kit-checks -- telemetry --skill-log "$F" --top 1 --format markdown | grep -q '| alpha |'

# 5) bad inputs exit 2
cargo run --locked -q -p harness-kit-checks -- telemetry --skill-log "$F" --top 0   ; test $? -eq 2
cargo run --locked -q -p harness-kit-checks -- telemetry --skill-log "$F" --top abc ; test $? -eq 2

# 6) unit + integration suite and repo gate stay green
cargo test --locked -p harness-kit-checks
cargo run --locked -p harness-kit-checks -- check --repo .
```

**Observable (human/`/qa`):** `telemetry --top 5 --format markdown` renders a Skill Frequency table with at most five data rows while the Skill Transitions, Work Sequences, Source Coverage, and Harness Coverage tables remain unchanged versus an uncapped run.

## Premise Source

**Waiver.** No committed backlog item or ADR drives this; the premise is the operator task prompt, which corresponds verbatim to **eval fixture 1** at `skills/shape/evals/shape-eval.md:24` (repo pinned `harness-kit@3bf0b46`). That eval names the expected deliverables (Repo Anchors, executable oracle, CLI-design block), which this packet satisfies. **Residual risk:** the product intent of the word "report" (skills-table-only vs whole-report) is inferred, not operator-confirmed — locked here to skills-table-only with a stop-condition for the alternative.

## HTML Plan

`/tmp/telemetry-top-plan.html` — authored from `skills/shape/templates/html-plan.html` (214 lines, verified present). Hero states the chosen design (field-on-`AnalyzeOptions` + truncate-after-sort) and the proof path (`--self-test` + fixture `jq`); the alternatives table contrasts A/B/C/D on how each fails differently; the execution-plan lanes and risk grid carry the construction-site enumeration and the `--top 0` / scope-ambiguity stop-conditions. Open with `open /tmp/telemetry-top-plan.html` (macOS).

## Risks + Rollout

| Risk | Likelihood | Mitigation |
|---|---|---|
| Field added but one of the two `self_test` constructions missed → won't compile. | Low | The compiler catches it; both sites enumerated in Repo Anchor 3. |
| `--top 0` silently emits the empty `| none | 0 | dead | ... |` row (markdown) and is misread as "no telemetry data." | Medium | Reject `0` with exit 2 and a clear message (Design step 4) rather than rendering an empty table. |
| Operator actually wanted the whole report filtered to the top-N skill set (Alternative D). | Low–Med | **Stop condition:** if that is the intent, halt before building — it is a different, larger feature with murky transition semantics. |
| Cap interacts surprisingly with `--since`/`--repo`/`--project`/`--skill`. | Low | By construction `--top` applies *after* those row filters and the sort, so it is "top-N of the filtered set" — the intuitive composition; Oracle check 1 exercises the sorted ordering. |
| Non-determinism at the N-boundary on tied counts. | Low | The existing secondary sort (skill-name ascending, `:91-92`) already makes the order total; truncation is deterministic. Oracle check 1 asserts the exact surviving names. |

**Rollout:** purely additive, behind an opt-in flag; default behavior is byte-identical (Oracle check 3). **Rollback:** revert the diff — no migration, no persisted state, no schema change. The flag is independently revertible from the rest of the telemetry surface.
