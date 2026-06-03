#!/usr/bin/env bash
# Focused tests for scripts/offline-validation-preflight.sh.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
PASS=0
FAIL=0

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

setup_repo() {
  ORIG_DIR="$(pwd)"
  ORIG_PATH="$PATH"
  TEST_DIR="$(mktemp -d)"
  cd "$TEST_DIR"
  git init -q
  mkdir -p .empty-hooks
  git config core.hooksPath .empty-hooks
  git config user.name "Test User"
  git config user.email "test@example.com"
  git commit --allow-empty -m "initial" -q
  mkdir -p scripts/lib .githooks bin
  cp "$REPO_ROOT/scripts/offline-validation-preflight.sh" scripts/offline-validation-preflight.sh
  cp "$REPO_ROOT/scripts/lib/evidence.sh" scripts/lib/evidence.sh
  cp "$REPO_ROOT/scripts/lib/verdicts.sh" scripts/lib/verdicts.sh
  cp "$REPO_ROOT/.githooks/pre-merge-commit" .githooks/pre-merge-commit
  chmod +x scripts/offline-validation-preflight.sh .githooks/pre-merge-commit
  cp "$REPO_ROOT/.gitattributes" .gitattributes
}

teardown_repo() {
  cd "$ORIG_DIR"
  PATH="$ORIG_PATH"
  export PATH
  rm -rf "$TEST_DIR"
}

stub_command() {
  local name="$1" body="$2"
  printf '#!/usr/bin/env bash\n%s\n' "$body" > "$TEST_DIR/bin/$name"
  chmod +x "$TEST_DIR/bin/$name"
}

test_preflight_passes_with_required_files() {
  setup_repo
  stub_command dagger 'echo dagger-stub'
  PATH="$TEST_DIR/bin:$PATH"
  local rc=0 output
  output="$(scripts/offline-validation-preflight.sh 2>&1)" || rc=$?
  assert_eq "preflight passes with required files and dagger" "0" "$rc"
  case "$output" in
    *".evidence/master/"*) assert_eq "preflight prints evidence path" "ok" "ok" ;;
    *) assert_eq "preflight prints evidence path" "ok" "missing" ;;
  esac
  teardown_repo
}

test_preflight_fails_without_dagger() {
  setup_repo
  PATH="$TEST_DIR/bin:/usr/bin:/bin:/usr/sbin:/sbin"
  local rc=0
  scripts/offline-validation-preflight.sh >/tmp/offline-preflight.out 2>&1 || rc=$?
  assert_eq "preflight fails without dagger" "1" "$rc"
  teardown_repo
}

test_preflight_fails_without_lfs_attributes() {
  setup_repo
  stub_command dagger 'echo dagger-stub'
  PATH="$TEST_DIR/bin:$PATH"
  : > .gitattributes
  local rc=0
  scripts/offline-validation-preflight.sh >/tmp/offline-preflight.out 2>&1 || rc=$?
  assert_eq "preflight fails without evidence LFS attributes" "1" "$rc"
  teardown_repo
}

test_preflight_passes_with_required_files
test_preflight_fails_without_dagger
test_preflight_fails_without_lfs_attributes

if [ "$FAIL" -gt 0 ]; then
  echo "$FAIL failed, $PASS passed"
  exit 1
fi

echo "$PASS passed"
