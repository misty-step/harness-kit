#!/usr/bin/env bash
# Tests for backlog.sh — backlog trailer parsing and ticket archival.
# Runs in a temporary git repo to avoid polluting the real one.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PASS=0
FAIL=0

# Setup: create a temp git repo with a backlog.d/ layout.
setup() {
  ORIG_DIR="$(pwd)"
  TEST_DIR="$(mktemp -d)"
  cd "$TEST_DIR"
  git init -q
  mkdir -p .empty-hooks
  git config core.hooksPath .empty-hooks
  git config user.name "Test User"
  git config user.email "test@example.com"
  mkdir -p backlog.d/_done
  cat >backlog.d/031-active-ticket.md <<'EOF'
# BACKLOG-031: Active ticket
Status: ready
EOF
  cat >backlog.d/042-another-active.md <<'EOF'
# BACKLOG-042: Another active
Status: ready
EOF
  cat >backlog.d/_done/007-archived-ticket.md <<'EOF'
# BACKLOG-007: Already archived
Status: done
EOF
  git add -A
  git commit -m "initial" -q
  # Force fresh source each setup so source guard in backlog.sh is reset.
  unset BACKLOG_SH_SOURCED
  # shellcheck source=scripts/lib/backlog.sh
  source "$SCRIPT_DIR/backlog.sh"
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

assert_exit() {
  local desc="$1" expected="$2"
  shift 2
  local actual
  if "$@" >/dev/null 2>&1; then actual=0; else actual=$?; fi
  assert_eq "$desc" "$expected" "$actual"
}

# Make a commit with a body that contains the given trailer lines.
# Args: <subject> <trailer-lines-newline-separated>
make_commit_with_trailers() {
  local subject="$1" trailers="$2"
  local msg
  msg="$(printf '%s\n\nbody line\n\n%s\n' "$subject" "$trailers")"
  git commit --allow-empty -q -m "$msg"
}

# --- Tests ---

test_trailer_keys_lists_all_three() {
  local out
  out="$(backlog_trailer_keys | paste -sd ',' -)"
  assert_eq "backlog_trailer_keys lists all three keys" "Closes-backlog,Ships-backlog,Refs-backlog" "$out"
}

test_closing_keys_excludes_refs() {
  local out
  out="$(backlog_closing_keys | paste -sd ',' -)"
  assert_eq "backlog_closing_keys lists only closure keys" "Closes-backlog,Ships-backlog" "$out"
}

test_ids_from_commit_extracts_multiple() {
  make_commit_with_trailers "feat: thing" "Closes-backlog: 031
Ships-backlog: 042
Refs-backlog: 099"
  local out
  out="$(backlog_ids_from_commit HEAD)"
  # Refs-backlog (099) should be excluded — it's non-closing.
  local expected
  expected="$(printf '031\n042\n')"
  assert_eq "backlog_ids_from_commit extracts closing trailers only" "$expected" "$out"
}

test_ids_from_commit_deduplicates() {
  make_commit_with_trailers "feat: dup" "Closes-backlog: 031
Ships-backlog: 031"
  local out
  out="$(backlog_ids_from_commit HEAD)"
  assert_eq "backlog_ids_from_commit deduplicates repeated IDs" "031" "$out"
}

test_ids_from_commit_no_trailers_fails() {
  git commit --allow-empty -q -m "chore: no trailers here"
  assert_exit "backlog_ids_from_commit exits 1 when no trailers present" 1 backlog_ids_from_commit HEAD
}

test_ids_from_commit_ignores_refs_only() {
  make_commit_with_trailers "chore: reference only" "Refs-backlog: 031"
  assert_exit "backlog_ids_from_commit ignores pure Refs-backlog trailers" 1 backlog_ids_from_commit HEAD
}

test_ids_from_range_deduplicates_across_commits() {
  local base
  base="$(git rev-parse HEAD)"
  make_commit_with_trailers "feat: one" "Closes-backlog: 031"
  make_commit_with_trailers "feat: two" "Ships-backlog: 042
Closes-backlog: 031"
  local out
  out="$(backlog_ids_from_range "${base}..HEAD")"
  local expected
  expected="$(printf '031\n042\n')"
  assert_eq "backlog_ids_from_range dedupes across commits" "$expected" "$out"
}

test_ids_from_range_empty_fails() {
  local head
  head="$(git rev-parse HEAD)"
  git commit --allow-empty -q -m "chore: no trailers"
  assert_exit "backlog_ids_from_range exits 1 when range has no trailers" 1 \
    backlog_ids_from_range "${head}..HEAD"
}

test_file_for_id_finds_active() {
  local out
  out="$(backlog_file_for_id 031)"
  assert_eq "backlog_file_for_id locates active ticket" "backlog.d/031-active-ticket.md" "$out"
}

test_file_for_id_finds_archived() {
  local out
  out="$(backlog_file_for_id 007)"
  assert_eq "backlog_file_for_id locates archived ticket" "backlog.d/_done/007-archived-ticket.md" "$out"
}

test_file_for_id_unknown_fails() {
  assert_exit "backlog_file_for_id exits 1 for unknown ID" 1 backlog_file_for_id 999
}

test_file_for_id_rejects_non_numeric() {
  assert_exit "backlog_file_for_id rejects non-numeric ID" 1 backlog_file_for_id "abc"
}

test_file_for_id_works_from_subdirectory() {
  mkdir -p nested/deeper
  (
    cd nested/deeper
    backlog_file_for_id 031
  ) >/tmp/backlog_test_out.$$ 2>&1
  local out
  out="$(cat /tmp/backlog_test_out.$$)"
  rm -f /tmp/backlog_test_out.$$
  assert_eq "backlog_file_for_id resolves from subdirectory" "backlog.d/031-active-ticket.md" "$out"
}

test_archive_moves_active_ticket() {
  backlog_archive 031
  [ -f backlog.d/_done/031-active-ticket.md ]
  local moved_exists=$?
  [ ! -f backlog.d/031-active-ticket.md ]
  local source_gone=$?
  assert_eq "backlog_archive moves file into _done/" "0" "$moved_exists"
  assert_eq "backlog_archive removes file from backlog.d/" "0" "$source_gone"
}

test_archive_moves_all_matching_ticket_files() {
  cat >backlog.d/031-active-ticket.ctx.md <<'EOF'
# Context packet
EOF
  git add backlog.d/031-active-ticket.ctx.md
  git commit -q -m "add context packet"

  backlog_archive 031
  [ -f backlog.d/_done/031-active-ticket.md ]
  local ticket_moved=$?
  [ -f backlog.d/_done/031-active-ticket.ctx.md ]
  local context_moved=$?
  [ ! -f backlog.d/031-active-ticket.md ]
  local ticket_source_gone=$?
  [ ! -f backlog.d/031-active-ticket.ctx.md ]
  local context_source_gone=$?

  assert_eq "backlog_archive moves ticket file" "0" "$ticket_moved"
  assert_eq "backlog_archive moves context packet" "0" "$context_moved"
  assert_eq "backlog_archive removes ticket source" "0" "$ticket_source_gone"
  assert_eq "backlog_archive removes context source" "0" "$context_source_gone"
}

test_archive_uses_git_mv() {
  backlog_archive 031
  # `git mv` stages a rename; expect both old and new paths in the index diff.
  local staged
  staged="$(git diff --cached --name-status)"
  if echo "$staged" | grep -qE '^R' && \
     echo "$staged" | grep -q '031-active-ticket.md'; then
    assert_eq "backlog_archive stages a rename" "0" "0"
  else
    assert_eq "backlog_archive stages a rename" "0" "1"
    echo "  staged diff was:"
    echo "$staged" | sed 's/^/    /'
  fi
}

test_archive_aborts_when_root_cd_fails_before_git_mv() {
  local marker="$TEST_DIR/git-mv-attempted"
  _backlog_cd_repo_root() {
    return 1
  }
  git() {
    if [ "${1:-}" = "mv" ]; then
      : >"$marker"
      return 0
    fi
    command git "$@"
  }

  local actual
  if backlog_archive 031 >/tmp/backlog_cd_fail.$$ 2>&1; then
    actual=0
  else
    actual=$?
  fi
  local output
  output="$(cat /tmp/backlog_cd_fail.$$)"
  rm -f /tmp/backlog_cd_fail.$$
  unset -f _backlog_cd_repo_root
  unset -f git

  assert_eq "backlog_archive fails when root cd fails" "1" "$actual"
  if printf '%s\n' "$output" | grep -q "could not cd"; then
    assert_eq "backlog_archive explains root cd failure" "0" "0"
  else
    assert_eq "backlog_archive explains root cd failure" "0" "1"
    echo "  stderr was: $output"
  fi
  if [ -e "$marker" ]; then
    assert_eq "backlog_archive does not attempt git mv after cd failure" "0" "1"
  else
    assert_eq "backlog_archive does not attempt git mv after cd failure" "0" "0"
  fi
}

test_archive_aborts_when_git_mv_fails() {
  local marker="$TEST_DIR/git-mv-attempted"
  git() {
    if [ "${1:-}" = "mv" ]; then
      printf '%s\n' "$*" >>"$marker"
      return 1
    fi
    command git "$@"
  }

  local actual
  if backlog_archive 031 >/tmp/backlog_git_mv_fail.$$ 2>&1; then
    actual=0
  else
    actual=$?
  fi
  local output
  output="$(cat /tmp/backlog_git_mv_fail.$$)"
  rm -f /tmp/backlog_git_mv_fail.$$
  unset -f git

  assert_eq "backlog_archive fails when git mv fails" "1" "$actual"
  if printf '%s\n' "$output" | grep -q "could not move"; then
    assert_eq "backlog_archive explains git mv failure" "0" "0"
  else
    assert_eq "backlog_archive explains git mv failure" "0" "1"
    echo "  stderr was: $output"
  fi
  if [ -e "$marker" ]; then
    assert_eq "backlog_archive attempted git mv before failing" "0" "0"
  else
    assert_eq "backlog_archive attempted git mv before failing" "0" "1"
  fi
}

test_archive_is_idempotent() {
  backlog_archive 031
  git commit -q -m "archive 031"
  assert_exit "backlog_archive is idempotent when already archived" 0 backlog_archive 031
}

test_archive_idempotent_for_preexisting_done() {
  # 007 is seeded in _done/ and not in backlog.d/.
  assert_exit "backlog_archive exits 0 for already-archived ID" 0 backlog_archive 007
}

test_archive_unknown_id_fails() {
  assert_exit "backlog_archive exits 1 for unknown ID" 1 backlog_archive 999
}

test_archive_rejects_non_numeric() {
  assert_exit "backlog_archive rejects non-numeric ID" 1 backlog_archive "abc"
}

test_source_guard_is_safe() {
  # Re-sourcing should not clobber or error. BACKLOG_SH_SOURCED is already set
  # from setup(); sourcing again must be a silent no-op.
  # shellcheck source=scripts/lib/backlog.sh
  source "$SCRIPT_DIR/backlog.sh"
  assert_eq "re-source is a no-op" "1" "${BACKLOG_SH_SOURCED:-}"
}

# --- Runner ---

run_tests() {
  local funcs
  funcs="$(declare -F | awk '/test_/{print $3}')"
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
