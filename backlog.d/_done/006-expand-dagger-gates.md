# Expand Dagger CI gates — replace hook enforcement

Priority: medium
Status: done
Estimate: M

## Goal

Add new Dagger gates to `ci/src/harness_kit_ci/main.py` that replace Claude hook enforcement with harness-agnostic checks. Each gate fires once per `dagger call check`, works across all harnesses and human workflows.

## Non-Goals
- Don't make gates fail by default — warn first, repos escalate to hard failure
- Don't add external tool dependencies — use grep/AST heuristics

## New Gates

### check-no-hardcoded-paths
Replaces `portable-code-guard.py`. Scans shell scripts and config files for `/Users/<username>/` or `/home/<username>/` patterns. Excludes build artifacts.

### check-no-exclusion-shortcuts
Replaces `exclusion-guard.py`. Scans for `@ts-ignore`, `eslint-disable`, `.skip`, `.xit`, `istanbul ignore`, `coverage exclude` patterns. Reports count and locations.

### check-no-echo-pipe-env
Replaces `env-var-newline-guard.py`. Scans for `echo ... | ... env add/set` patterns that corrupt secrets with trailing newlines.

### check-complexity-budget (optional, repos opt in)
Reads `.harness-kit.yaml` complexity thresholds. Measures LOC per file, nesting depth. Warns on overages.

## Oracle
- [x] `dagger call check` runs all new gates alongside existing 7 — 9 gates total, 0 failures
- [x] New gates discover files from filesystem (not hardcoded lists) — glob-based discovery
- [x] Each gate fails on findings (upgraded from warn — enforcement is the point)
- [x] Running `dagger call check` on harness-kit repo catches exclusion patterns if present

## What Was Built

Completed as part of 004-hook-migration:
- `check_exclusions`: scans TS/JS/Python for @ts-ignore, eslint-disable, .skip, as any
- `check_portable_paths`: scans shell/config for hardcoded /Users/ paths
- `check-no-echo-pipe-env`: dropped (runtime command guard, not static-analysable)
- `check-complexity-budget`: deferred (optional, repos opt in when needed)
