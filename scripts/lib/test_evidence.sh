#!/usr/bin/env bash
# Tests for evidence.sh — git-native evidence directory helpers.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PASS=0
FAIL=0

setup() {
  ORIG_DIR="$(pwd)"
  TEST_DIR="$(mktemp -d)"
  cd "$TEST_DIR"
  git init -q
  mkdir -p .empty-hooks
  git config core.hooksPath .empty-hooks
  git config user.name "Test User"
  git config user.email "test@example.com"
  git commit --allow-empty -m "initial" -q
  git checkout -b feat/024-offline-evidence -q
  # shellcheck source=scripts/lib/evidence.sh
  source "$SCRIPT_DIR/evidence.sh"
}

teardown() {
  cd "$ORIG_DIR"
  rm -rf "$TEST_DIR"
}

assert_eq() {
  local desc="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    PASS=$((PASS + 1))
    echo "  PASS  $desc"
  else
    FAIL=$((FAIL + 1))
    echo "  FAIL  $desc (expected '$expected', got '$actual')"
  fi
}

test_branch_slug_normalizes_slashes() {
  assert_eq "branch slug normalizes slash" "feat-024-offline-evidence" "$(evidence_branch_slug)"
}

test_evidence_dir_uses_branch_and_date() {
  assert_eq "evidence dir is branch/date scoped" ".evidence/feature-x/2026-06-02/" "$(evidence_dir "feature/x" "2026-06-02")"
}

test_evidence_dir_create_makes_directory() {
  local dir
  dir="$(evidence_dir_create "feature/x" "2026-06-02")"
  assert_eq "evidence dir create returns path" ".evidence/feature-x/2026-06-02/" "$dir"
  if [ -d "$dir" ]; then
    assert_eq "evidence dir exists" "yes" "yes"
  else
    assert_eq "evidence dir exists" "yes" "no"
  fi
}

test_evidence_trailer_skips_empty_dir() {
  local dir trailer
  dir="$(evidence_dir_create "feature/x" "2026-06-02")"
  trailer="$(evidence_trailer "$dir")"
  assert_eq "empty evidence dir has no trailer" "" "$trailer"
}

test_evidence_trailer_prints_non_empty_dir() {
  local dir trailer
  dir="$(evidence_dir_create "feature/x" "2026-06-02")"
  printf 'proof\n' > "${dir}qa-report.md"
  trailer="$(evidence_trailer "$dir")"
  assert_eq "non-empty evidence dir has trailer" "QA-Evidence: .evidence/feature-x/2026-06-02/" "$trailer"
}

run_tests() {
  local funcs
  funcs="$(declare -F | awk '/test_evidence_/{print $3}')"
  for t in $funcs; do
    setup
    "$t"
    teardown
  done

  echo ""
  echo "Results: $PASS passed, $FAIL failed"
  [ "$FAIL" -eq 0 ]
}

run_tests
