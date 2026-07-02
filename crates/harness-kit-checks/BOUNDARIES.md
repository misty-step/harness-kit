# harness-kit-checks module boundary inventory

Backlog `129-diet-checks-crate-boundaries.md` children 1 and 4. Classifies
every module in `crates/harness-kit-checks/src/*.rs` (12.3k LOC, 24 files as
of the roster split — was 18.1k LOC/26 files before it) by real dependency
evidence — `grep -n "^use crate::"` per file, not vibes — into
keep-in-checks, split-to-roster (done), split-to-hooks (done),
split-to-site-analytics, or park/delete. Regenerate this table by hand
whenever a module's `use crate::` imports change meaningfully; it is a plan,
not generated output, so no gate checks it for drift.

## Method

For each file: `grep -n "^use crate::" <file>` shows what it imports from
elsewhere in the crate. A file with **no internal `use crate::` line** has
zero code-level coupling to the rest of the crate (it may still share a
*data* contract, e.g. a JSON field name, which is called out explicitly below
since `grep` cannot see it).

## Keep in `harness-kit-checks` (core: gates, bootstrap, index, install)

The true "checks" identity — everything a fresh clone needs to gate itself
and stay installed correctly. ~12,300 LOC.

| Module | LOC | Why core |
|---|---:|---|
| `ci_check.rs` | 231 | The gate orchestrator itself; imports `docs_site, eval_coverage, frontmatter, generate_index, lint_gates, process, quality_gates` |
| `lint_gates.rs` | 759 | Gate implementations (`GateReport` is the shared return type `eval_coverage`/`quality_gates` reuse) |
| `quality_gates.rs` | 322 | Gate implementations; imports `lint_gates::GateReport` |
| `eval_coverage.rs` | 241 | Gate implementation; imports `lint_gates::GateReport` |
| `frontmatter.rs` | 405 | Catalog integrity (skill/agent frontmatter validation) |
| `generate_index.rs` | 376 | Catalog integrity (`index.yaml` generation + drift check) |
| `docs_site.rs` | 1883 | Site generator — kept here (not split-to-site-analytics) because `check-docs-site` is a core gate lane and `ci_check.rs` calls it directly; splitting it would make the core crate depend on a "site" crate for a gate it owns |
| `git_hooks.rs` | 682 | General repo pre-commit/post-commit hooks (NOT Claude-specific — imports `bootstrap, ci_check, docs_site, frontmatter, generate_index, lint_gates` directly, i.e. it orchestrates the gate/index/docs machinery on every commit) |
| `bootstrap.rs` | 677 | Install/sync logic |
| `cli_install.rs` | 188 | Split out of `bootstrap.rs` (backlog.d/133) purely to stay under the god-file ceiling after fixing the CLI self-update blind spot; imports `bootstrap::{blue, green}` for shared output formatting |
| `config_loader.rs` | 859 | Config loading for bootstrap/deploy targets |
| `backlog.rs` | 457 | Backlog trailer/ID parsing (`/ship` trailer canon); standalone, no internal deps |
| `external_skill_lint.rs` | 264 | Lints vendored external skills against `registry.yaml`; standalone |
| `external_sync.rs` + `external_sync_tests.rs` | 821 + 197 | Vendored-skill sync engine; standalone |
| `scout_skills.rs` | 439 | Skill scouting/duplicate detection; imports `external_sync` |
| `process.rs` | 65 | Tiny process-spawn utility; used transitively everywhere |
| `error_report.rs` | 37 | Tiny error-chain printer; used by `main.rs` |
| `pr_reviews.rs` | 325 | `gh`-backed PR review fetch/render; standalone, single CLI entrypoint (`fetch-pr-reviews`) — low-maintenance, no urgency to split |
| `skill_invocation_analytics.rs` | 1014 | Telemetry report (`harness-kit-checks telemetry`); standalone code-wise, but shares a **data contract** with `harness-kit-hooks` (reads the `invocation_kind`/`invocation_kinds` JSON fields the hook writes) — see note below |
| `lib.rs`, `main.rs` | ~19 + 1235 | Crate root / CLI dispatch. `main.rs` depends on both sibling crates (`harness_kit_hooks::claude_hooks`, `harness_kit_roster::{agent_roster, source_refs, summarize_delegations}`) for CLI dispatch, same shape as the hooks split — core is the binary that assembles the other crates, never the reverse |

**Data-contract note (`skill_invocation_analytics.rs`):** it has *zero*
`use crate::` lines — no code dependency on `harness-kit-hooks` — but its
`invocation_kind_counts` reads a JSON field (`"invocation_kind"`) that only
`harness-kit-hooks`'s `claude_hooks.rs` ever writes. Kept in core for now
because its audience (an operator running `telemetry` reports) matches
`docs_site`/`pr_reviews`'s audience, not the hook-runtime's. Revisit if a
future split wants "everything about skill invocation" co-located instead.

## Split to `harness-kit-roster` (agent orchestration — split this pass)

| Module | LOC | Coupling |
|---|---:|---|
| `agent_roster.rs` | 1975 | Imports `lane_harness, summarize_delegations` |
| `summarize_delegations.rs` | 1155 | Imports `lane_harness, source_refs` |
| `lane_harness.rs` | 811 | Imports `agent_roster` |
| `source_refs.rs` | 260 | Validates backlog/external ref shape; standalone, imported only by `summarize_delegations.rs` and (directly, for `record-delegation`'s `--work-source-ref` flag) `main.rs` |

**Tightly coupled cluster**, ~4200 LOC. `agent_roster` and `lane_harness`
import each other (mutual dependency) — the actual reason this was riskier
than the hooks split: a real cyclic pair, not a self-contained one. This
resolves cleanly because Rust crates can't be cyclic *between* crates, only
*within* one, so the mutual pair simply moves together as internal modules
of the same new crate. `source_refs.rs` was re-classified from "core" to
"roster" once the actual dependency graph was checked directly (not
assumed): despite BOUNDARIES.md originally guessing "core depends back on
roster for `source_refs`," `grep -rln "source_refs::"` showed the *only*
callers outside its own file are `summarize_delegations.rs` (moving with it
anyway) and `main.rs`'s CLI dispatch (which already depends on the roster
crate as a sibling library, same as it depends on `harness-kit-hooks`) — so
nothing in core actually needed `source_refs.rs` to stay in core. Moving it
fully into `harness-kit-roster` removed the one dependency edge that would
otherwise have made `harness-kit-checks` and `harness-kit-roster` depend on
each other in both directions (a real cycle Rust would reject outright).
Landed in `crates/harness-kit-roster/` — dispatch + receipt machinery, a
distinct audience (agents dispatching sub-agents) from gate-callers.
`harness-kit-checks` depends on it as a library the same way it already
depends on `harness-kit-hooks`; `harness-kit-roster` has zero dependency
back on `harness-kit-checks`. Verified live post-split:
`probe-agent-roster --validate-only`, and a full
`record-delegation` → `summarize-delegations` round-trip, both produce
correct output through the new crate boundary; `cargo test --workspace`
carries all 34 roster-cluster tests over unmodified.

## Split to `harness-kit-hooks` (Claude-specific hook runner — split this pass)

| Module | LOC | Coupling |
|---|---:|---|
| `claude_hooks.rs` | 2465 | Imports `invocation_kind` only |
| `invocation_kind.rs` | 179 | No internal imports |

~2644 LOC, **zero code-level coupling to anything else in the crate** beyond
the pair itself. `main.rs`'s only touchpoint is 18 call sites of the shape
`claude_hooks::run_<hook_name>_from_stdin().unwrap_or_else(exit_error)` — each
a parameterless `pub fn run_*() -> Result<()>` that reads its own input from
stdin. This is the cleanest possible extraction: no shared types cross the
boundary, no arguments to thread through, no other module reaches into it.
Executed in this pass (see `crates/harness-kit-hooks/`).

## Park / delete

None identified. Every module in the crate has at least one live CLI
entrypoint or gate consumer; nothing here is dead weight independent of the
skill-catalog cull already completed (PR #142).

## What this inventory is not

- Not a claim that `docs_site.rs` or `skill_invocation_analytics.rs` can
  never move — the "keep in core" calls above are the current best fit given
  today's actual dependency edges, not a permanent boundary.
- Not generated: re-derive by re-running the `grep -n "^use crate::"` sweep
  per file before trusting an old copy of this table.
