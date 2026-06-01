#!/usr/bin/env bash
# Tests for verdicts.sh — git-native review verdict storage.
# Runs in a temporary git repo to avoid polluting the real one.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PASS=0
FAIL=0
TESTS=()

# Setup: create a temp git repo
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
  git checkout -b feat-foo -q
  git commit --allow-empty -m "feat commit" -q
  # shellcheck source=scripts/lib/verdicts.sh
  source "$SCRIPT_DIR/verdicts.sh"
}

# Teardown: clean up temp repo
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

assert_exit() {
  local desc="$1" expected="$2"
  shift 2
  local actual
  if "$@" >/dev/null 2>&1; then actual=0; else actual=$?; fi
  assert_eq "$desc" "$expected" "$actual"
}

# --- Tests ---

test_verdict_write_creates_ref() {
  local json='{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{"correctness":8},"sha":"'"$(git rev-parse HEAD)"'","date":"2026-04-06T15:00:00Z"}'
  assert_exit "verdict_write creates ref" 0 verdict_write feat-foo "$json"
}

test_verdict_read_returns_json() {
  local sha
  sha="$(git rev-parse HEAD)"
  local json='{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{"correctness":8},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  verdict_write feat-foo "$json"
  local result
  result="$(verdict_read feat-foo)"
  assert_eq "verdict_read returns written JSON" "$json" "$result"
}

test_verdict_read_nonexistent_fails() {
  assert_exit "verdict_read on nonexistent ref fails" 1 verdict_read no-such-branch
}

test_verdict_validate_passes_when_sha_matches() {
  local sha
  sha="$(git rev-parse HEAD)"
  local json='{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{"correctness":8},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  verdict_write feat-foo "$json"
  assert_exit "verdict_validate passes when SHA matches HEAD" 0 verdict_validate feat-foo
}

test_verdict_validate_fails_when_sha_stale() {
  local old_sha
  old_sha="$(git rev-parse HEAD)"
  local json='{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{"correctness":8},"sha":"'"$old_sha"'","date":"2026-04-06T15:00:00Z"}'
  verdict_write feat-foo "$json"
  # Make a new commit so HEAD moves
  git commit --allow-empty -m "post-review commit" -q
  assert_exit "verdict_validate fails when HEAD moved" 1 verdict_validate feat-foo
}

test_verdict_validate_fails_without_verdict() {
  git branch no-verdict-branch
  assert_exit "verdict_validate fails without verdict" 1 verdict_validate no-verdict-branch
}

test_verdict_validate_explains_unresolved_branch_without_verdict() {
  local output actual
  if output="$(verdict_validate no-verdict-branch 2>&1)"; then
    actual=0
  else
    actual=$?
  fi
  assert_eq "verdict_validate rejects unresolved branch without verdict" "1" "$actual"
  if printf '%s\n' "$output" | grep -q "could not resolve"; then
    assert_eq "verdict_validate explains unresolved branch without verdict" "0" "0"
  else
    assert_eq "verdict_validate explains unresolved branch without verdict" "0" "1"
    echo "  stderr was: $output"
  fi
}

test_verdict_validate_fails_for_unresolved_branch_even_when_sha_matches_head() {
  local sha output actual
  sha="$(git rev-parse HEAD)"
  local json='{"branch":"missing-branch","base":"master","verdict":"ship","reviewers":["critic"],"scores":{"correctness":8},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  verdict_write missing-branch "$json"
  if output="$(verdict_validate missing-branch 2>&1)"; then
    actual=0
  else
    actual=$?
  fi
  assert_eq "verdict_validate rejects unresolved branch instead of using HEAD" "1" "$actual"
  if printf '%s\n' "$output" | grep -q "could not resolve"; then
    assert_eq "verdict_validate explains unresolved target" "0" "0"
  else
    assert_eq "verdict_validate explains unresolved target" "0" "1"
    echo "  stderr was: $output"
  fi
}

test_verdict_delete_removes_ref() {
  local sha
  sha="$(git rev-parse HEAD)"
  local json='{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{"correctness":8},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  verdict_write feat-foo "$json"
  assert_exit "verdict_delete succeeds" 0 verdict_delete feat-foo
  assert_exit "verdict_read fails after delete" 1 verdict_read feat-foo
}

test_verdict_list_shows_verdicts() {
  local sha
  sha="$(git rev-parse HEAD)"
  local json='{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{"correctness":8},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  verdict_write feat-foo "$json"
  local list
  list="$(verdict_list)"
  assert_eq "verdict_list includes feat-foo" "verdicts/feat-foo" "$list"
}

test_verdict_write_rejects_invalid_json() {
  assert_exit "verdict_write rejects non-JSON" 1 verdict_write feat-foo "not json"
}

test_verdict_write_rejects_missing_fields() {
  assert_exit "verdict_write rejects missing sha" 1 verdict_write feat-foo '{"branch":"feat-foo","verdict":"ship"}'
}

test_verdict_write_preserves_json_bytes_with_edge_characters() {
  local sha json expected_size actual_size
  sha="$(git rev-parse HEAD)"
  json='{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{"correctness":8},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z","note":"edge chars: \"quote\" \\backslash $dollar [brackets]"}'
  verdict_write feat-foo "$json"
  expected_size="$(printf '%s' "$json" | wc -c | tr -d ' ')"
  actual_size="$(git cat-file -s "${VERDICTS_REF_PREFIX}/feat-foo")"
  assert_eq "verdict_write stores exact JSON bytes without echo newline" "$expected_size" "$actual_size"
}

test_check_landable_passes_ship() {
  local sha
  sha="$(git rev-parse HEAD)"
  verdict_write feat-foo '{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  assert_exit "verdict_check_landable passes ship" 0 verdict_check_landable feat-foo
}

test_check_landable_passes_conditional() {
  local sha
  sha="$(git rev-parse HEAD)"
  verdict_write feat-foo '{"branch":"feat-foo","base":"master","verdict":"conditional","reviewers":["critic"],"scores":{},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  assert_exit "verdict_check_landable passes conditional" 0 verdict_check_landable feat-foo
}

test_check_landable_rejects_dont_ship() {
  local sha
  sha="$(git rev-parse HEAD)"
  verdict_write feat-foo '{"branch":"feat-foo","base":"master","verdict":"dont-ship","reviewers":["critic"],"scores":{},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  assert_exit "verdict_check_landable rejects dont-ship" 2 verdict_check_landable feat-foo
}

test_check_landable_rejects_missing() {
  assert_exit "verdict_check_landable rejects missing verdict" 1 verdict_check_landable no-verdict-branch
}

test_check_landable_rejects_stale() {
  local sha
  sha="$(git rev-parse HEAD)"
  verdict_write feat-foo '{"branch":"feat-foo","base":"master","verdict":"ship","reviewers":["critic"],"scores":{},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  git commit --allow-empty -m "post-review" -q
  assert_exit "verdict_check_landable rejects stale" 1 verdict_check_landable feat-foo
}

test_check_landable_rejects_unknown_verdict() {
  local sha
  sha="$(git rev-parse HEAD)"
  verdict_write feat-foo '{"branch":"feat-foo","base":"master","verdict":"unknown","reviewers":["critic"],"scores":{},"sha":"'"$sha"'","date":"2026-04-06T15:00:00Z"}'
  assert_exit "verdict_check_landable rejects unknown verdict" 1 verdict_check_landable feat-foo
}

# --- Runner ---

run_tests() {
  local funcs
  funcs="$(declare -F | awk '/test_(verdict_|check_landable_)/{print $3}')"
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
