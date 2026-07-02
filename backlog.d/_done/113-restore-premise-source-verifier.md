# Context Packet: Restore premise-source verifier contract

Priority: P2 (shipped)
Status: done
Estimate: S
Shipped: 2026-07-02

## Goal

Make `/shape`'s premise-source verification command truthful again, either by
restoring the Rust `premise-source` subcommand or by updating the skill to the
actual supported verifier.

## Current Evidence

- `skills/shape/SKILL.md` names:
  `cargo run --locked -p harness-kit-checks -- premise-source validate <packet>`
  and `premise-source self-test`.
- During shaping for `backlog.d/112-harness-eval-bench.md`, that command failed
  because the current `harness-kit-checks` CLI does not expose a
  `premise-source` subcommand.
- The full repo gate still passes, so this is a skill/CLI contract drift, not a
  broad gate failure.

## Non-Goals

- Do not weaken premise-source discipline.
- Do not remove the shape requirement for source-backed premises.
- Do not add a prose-only workaround without a mechanical validation path.

## Repo Anchors

- `skills/shape/SKILL.md`
- `skills/shape/references/voice-transcript-metadata.md`
- `crates/harness-kit-checks/src/main.rs`
- `crates/harness-kit-checks/src/`
- `backlog.d/_done/095-shape-premise-source-artifact.md`

## Design Options

1. **Restore the subcommand.** Add `premise-source validate` and
   `premise-source self-test` to `harness-kit-checks`, backed by tests and the
   existing shape contract. This is preferred if the 095 design is still the
   intended primitive.
2. **Retarget `/shape` to an existing gate.** If premise-source validation was
   intentionally folded into another command, update the skill prose and
   examples to name the real command.
3. **Fold into the main gate only.** Acceptable only if the main gate emits a
   focused failure message for malformed or missing `Premise Source:` fields in
   shaped packets.

## Oracle

- [x] `cargo run --locked -p harness-kit-checks -- premise-source self-test` passes
  or the skill no longer names that command. (Passes: 6 built-in fixtures.)
- [x] A malformed shaped packet with no `Premise Source:` produces a focused,
  reviewable failure from the documented verifier. (Live-verified against a
  real archived packet with no field —
  `backlog.d/_done/000-require-projected-lane-harnesses.md` — produces
  exactly that error.)
- [x] `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Risk

If left unresolved, agents will continue reporting a stale command failure as a
waiver during shaping, which teaches the wrong lesson: the premise-source
contract is important, but its verifier is optional.

## Resolution

**2026-07-02 — Option 1 (restore the subcommand), shipped.**
`crates/harness-kit-checks/src/premise_source.rs`: `validate_packet` parses a
packet's `## Premise Source` section for either `Premise Source:
sha256:<64hex> <path-or-url>` or an explicit `Premise Source Waiver: <reason>`
line; a local path reference must exist and its recomputed sha256 must match
the declared hash (URLs are accepted structurally — unverifiable from here).
`self_test` runs 6 built-in fixtures covering 095's original oracle exactly:
missing section, missing local artifact, hash mismatch, valid hash+path,
explicit waiver, and (added after checking real committed packets) the
wrapped-line/backtick-quoted layout `backlog.d/112-harness-eval-bench.md`
actually uses — the documented single-line format
(`skills/shape/SKILL.md:73`) and this real variant both parse correctly;
substance (hash shape, non-empty reference) stays strict either way.

Live-verified against real repository data, not just synthetic fixtures:
`backlog.d/112-harness-eval-bench.md` (current, real premise artifact)
verifies correctly; `backlog.d/_done/000-require-projected-lane-harnesses.md`
(predates the 095 doctrine, no field) correctly fails with a focused error;
`backlog.d/_done/111-delete-first-doctrine-lens.md` correctly fails because
its referenced `HIT-LIST.md` artifact no longer exists in the tree — a real,
previously-invisible case of premise-source drift the checker now catches.

`skills/shape/SKILL.md` needed no edit — the commands it already documents
(`premise-source validate <packet>`, `premise-source self-test`) now exist
verbatim.

Dispatch logic (arg parsing, subcommand match) lives in
`premise_source::run`, not `main.rs`'s dispatch table like most other
commands — `main.rs` is an already-grandfathered god-file (1235 lines before
this change) and even a maximally-trimmed 2-line addition (one dispatch arm,
one usage-text line) pushed it over; keeping the CLI glue in the new module
avoided growing the largest file in the crate for a small, self-contained
command. The unavoidable +2 lines (the dispatch arm and usage-text entry
that must live in `main.rs` for the command to be discoverable) were
ratcheted via `check-godfiles --write-baseline`, matching how `main.rs`'s
recorded ceiling has grown before as commands were added — not a gate
loosened, the ratchet's documented intentional-growth path.
