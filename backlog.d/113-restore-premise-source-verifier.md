# Context Packet: Restore premise-source verifier contract

Priority: P2
Status: ready
Estimate: S

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

- `cargo run --locked -p harness-kit-checks -- premise-source self-test` passes
  or the skill no longer names that command.
- A malformed shaped packet with no `Premise Source:` produces a focused,
  reviewable failure from the documented verifier.
- `cargo run --locked -p harness-kit-checks -- check --repo .` passes.

## Risk

If left unresolved, agents will continue reporting a stale command failure as a
waiver during shaping, which teaches the wrong lesson: the premise-source
contract is important, but its verifier is optional.
