#!/usr/bin/env bash
# Focused tests for scripts/load-harness-kit-config.py.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LOADER="$SCRIPT_DIR/load-harness-kit-config.py"
PASS=0
FAIL=0

setup() {
  ORIG_DIR="$(pwd)"
  TEST_DIR="$(mktemp -d)"
  cd "$TEST_DIR"
  git init -q
  mkdir -p .empty-hooks .harness-kit
  git config core.hooksPath .empty-hooks
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

assert_contains() {
  local desc="$1" haystack="$2" needle="$3"
  if [[ "$haystack" == *"$needle"* ]]; then
    PASS=$((PASS + 1))
    echo "  PASS  $desc"
  else
    FAIL=$((FAIL + 1))
    echo "  FAIL  $desc (missing '$needle')"
  fi
}

run_loader() {
  local out="$1" err="$2"
  shift 2
  python3 "$LOADER" "$@" >"$out" 2>"$err"
}

json_get() {
  python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))[sys.argv[2]])' "$1" "$2"
}

test_valid_deploy_with_envs() {
  cat >"$TEST_DIR/.harness-kit/deploy.yaml" <<'EOF'
schema_version: 1
target: custom
app: api
healthcheck: https://example.com/health
rollback_grace_seconds: 300
deploy_cmd: ./scripts/deploy.sh
current_sha_cmd: ./scripts/current-sha.sh
rollback_handle_cmd: ./scripts/current-release.sh
rollback_cmd: "./scripts/rollback.sh {{handle}}"
envs:
  prod:
    app: api-prod
    healthcheck: https://example.com/health
    rollback_grace_seconds: 300
    require_ci_green: true
EOF
  local out="$TEST_DIR/out.json" err="$TEST_DIR/err.txt"
  if run_loader "$out" "$err" deploy --repo "$TEST_DIR"; then
    assert_eq "deploy config loads" "custom" "$(json_get "$out" target)"
  else
    assert_eq "deploy config loads" "0" "$?"
  fi
}

test_valid_monitor_normalizes_durations() {
  cat >"$TEST_DIR/.harness-kit/monitor.yaml" <<'EOF'
schema_version: 1
grace_window: 5m
poll_interval: 30s
observability:
  delegation_receipts: .harness-kit/traces/delegations.jsonl
  workflow_events: .harness-kit/work/*.jsonl
  evidence_dirs: [".evidence", ".harness-kit/monitor"]
healthcheck:
  url: https://example.com/health
  expected_status: 200
signals:
  - name: error_rate
    source: datadog
    query: "sum:errors{service:api}.as_rate()"
    threshold: "> 0.01"
EOF
  local out="$TEST_DIR/out.json" err="$TEST_DIR/err.txt"
  if run_loader "$out" "$err" monitor --repo "$TEST_DIR"; then
    assert_eq "monitor grace_window normalized" "300" "$(json_get "$out" grace_window_seconds)"
    assert_eq "monitor poll_interval normalized" "30" "$(json_get "$out" poll_interval_seconds)"
  else
    assert_eq "monitor config loads" "0" "$?"
  fi
}

test_valid_flywheel_normalizes_cadence() {
  cat >"$TEST_DIR/.harness-kit/flywheel.yaml" <<'EOF'
schema_version: 1
cadence: 1h
max_cycles: 3
budget_tokens: 50000
backlog_includes:
  - "052"
stop_on_monitor_alert: true
stop_on_phase_failed: true
EOF
  local out="$TEST_DIR/out.json" err="$TEST_DIR/err.txt"
  if run_loader "$out" "$err" flywheel --repo "$TEST_DIR"; then
    assert_eq "flywheel cadence normalized" "3600" "$(json_get "$out" cadence_seconds)"
  else
    assert_eq "flywheel config loads" "0" "$?"
  fi
}

test_optional_missing_returns_empty_object() {
  local out="$TEST_DIR/out.json" err="$TEST_DIR/err.txt" code=0
  if run_loader "$out" "$err" deploy --repo "$TEST_DIR" --optional; then
    code=0
  else
    code=$?
  fi
  assert_eq "optional missing exits zero" "0" "$code"
  assert_eq "optional missing prints object" "{}" "$(cat "$out")"
}

test_required_missing_exits_two() {
  local out="$TEST_DIR/out.json" err="$TEST_DIR/err.txt" code=0
  if run_loader "$out" "$err" deploy --repo "$TEST_DIR"; then
    code=0
  else
    code=$?
  fi
  assert_eq "required missing exits two" "2" "$code"
  assert_contains "missing error names path" "$(cat "$err")" ".harness-kit/deploy.yaml"
}

test_unknown_key_is_rejected() {
  cat >"$TEST_DIR/.harness-kit/deploy.yaml" <<'EOF'
schema_version: 1
target: fly
app: api
unknown_field: true
EOF
  local out="$TEST_DIR/out.json" err="$TEST_DIR/err.txt" code=0
  if run_loader "$out" "$err" deploy --repo "$TEST_DIR"; then
    code=0
  else
    code=$?
  fi
  assert_eq "unknown key exits one" "1" "$code"
  assert_contains "unknown key error is actionable" "$(cat "$err")" "unknown key"
  assert_contains "unknown key error names field" "$(cat "$err")" "unknown_field"
}

test_bad_duration_reports_field() {
  cat >"$TEST_DIR/.harness-kit/monitor.yaml" <<'EOF'
schema_version: 1
grace_window: soon
healthcheck:
  url: https://example.com/health
EOF
  local out="$TEST_DIR/out.json" err="$TEST_DIR/err.txt" code=0
  if run_loader "$out" "$err" monitor --repo "$TEST_DIR"; then
    code=0
  else
    code=$?
  fi
  assert_eq "bad duration exits one" "1" "$code"
  assert_contains "duration error names field" "$(cat "$err")" "grace_window"
  assert_contains "duration error names units" "$(cat "$err")" "s, m, h, d"
}

test_custom_deploy_requires_commands() {
  cat >"$TEST_DIR/.harness-kit/deploy.yaml" <<'EOF'
schema_version: 1
target: custom
app: api
EOF
  local out="$TEST_DIR/out.json" err="$TEST_DIR/err.txt" code=0
  if run_loader "$out" "$err" deploy --repo "$TEST_DIR"; then
    code=0
  else
    code=$?
  fi
  assert_eq "custom deploy missing commands exits one" "1" "$code"
  assert_contains "custom deploy error names command" "$(cat "$err")" "deploy_cmd"
}

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
