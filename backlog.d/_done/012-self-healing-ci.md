# Self-healing CI — auto-repair Dagger gate failures

Priority: low
Status: done
Estimate: L

## Goal

When `dagger call check` fails, auto-spawn a builder sub-agent to diagnose and fix the failure. Dagger 0.18+ has native LLM integration.

## How It Works

1. `check()` captures failure output: gate name, error message, affected files
2. `heal()` receives failure context, spawns builder with focused prompt
3. Builder operates in a branch, commits fix, re-runs failing gate
4. Pass → squash-commit the fix. Fail after 2 attempts → escalate to human.
5. Optional: wire into pre-push hook (push fails → offer to auto-heal → re-push)

## Research Needed
- Explore Dagger 0.18+ LLM integration API
- Study Nx's self-healing CI patterns
- Understand how Dagger Functions invoke external agents
- Prototype: can a Dagger Function call `claude` CLI as a subprocess?

## Oracle
- [x] `dagger call -o . heal` after a lint failure writes a repaired repo directory to disk
- [x] `scripts/heal-commit.sh` fixes the lint issue, creates a branch, and commits the repair
- [x] Re-running `dagger call check` after heal succeeds
- [x] After 2 failed heal attempts, escalation message is clear

## Non-Goals
- Don't try to heal test failures (too complex for v1 — start with lint/format only)
- Don't auto-merge heal commits — human reviews

## What Was Built

- `ci/src/harness_kit_ci/main.py` — Added `heal()` to the Dagger module. It:
  reads the aggregated `check()` summary, selects exactly one healable lint-style
  failure, creates a writable repair container, prompts Dagger LLM with the actual
  failing gate output, verifies the targeted gate plus full `check()`, and returns
  the repaired repo directory for host-side export.
- `ci/src/harness_kit_ci/heal_support.py` — Pure helpers for parsing failed gates,
  enforcing the one-gate-at-a-time contract, generating repair metadata, and
  computing the delta between pre/post-heal worktree snapshots.
- `ci/tests/test_heal_support.py` and `ci/tests/test_self_healing.py` — Unit tests
  covering summary parsing, gate selection, repair metadata, and snapshot-delta
  staging logic for the host wrapper.
- `scripts/heal-commit.sh` — Host-side wrapper that:
  1. ensures a repo-local `.env` exists for Dagger module loading,
  2. inspects `dagger call check` to find the failing gate,
  3. runs `dagger call --allow-llm all -o . heal`,
  4. re-runs `dagger call check`,
  5. creates `heal/<gate>-<timestamp>` branch,
  6. stages only a repair-only patch for the post-heal delta and refuses
     ambiguous overlap with pre-existing staged or local edits,
  7. commits `ci: heal <gate>`.

## Workarounds

- Dagger module functions cannot directly mutate the host worktree from inside the
  Python runtime. The repair function therefore returns a `Directory`, and the host
  wrapper applies it with `dagger call -o . heal`.
- Local module loading still requires a repo-local `.env` file in this repo due the
  existing Dagger user-defaults lookup bug. `scripts/heal-commit.sh` creates it
  automatically (`touch .env`) before running Dagger commands.
