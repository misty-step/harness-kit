#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
AUDIT="$SCRIPT_DIR/audit-skills.py"
PASS=0
FAIL=0

assert_contains() {
  local desc="$1" haystack="$2" needle="$3"
  case "$haystack" in
    *"$needle"*)
      PASS=$((PASS + 1))
      echo "  PASS  $desc"
      ;;
    *)
      FAIL=$((FAIL + 1))
      echo "  FAIL  $desc (missing '$needle')"
      ;;
  esac
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

setup_repo() {
  TEST_DIR="$(mktemp -d)"
  git -C "$TEST_DIR" init -q
  mkdir -p "$TEST_DIR/.empty-hooks"
  git -C "$TEST_DIR" config core.hooksPath .empty-hooks
  git -C "$TEST_DIR" config user.email test@example.com
  git -C "$TEST_DIR" config user.name "Groom Audit Test"
  mkdir -p \
    "$TEST_DIR/skills/good/scripts" \
    "$TEST_DIR/skills/weak" \
    "$TEST_DIR/skills/empty-trigger" \
    "$TEST_DIR/skills/empty-use" \
    "$TEST_DIR/skills/uncataloged" \
    "$TEST_DIR/harnesses/shared"
  cat >"$TEST_DIR/AGENTS.md" <<'DOC'
# root project guidance should not satisfy shared routing
DOC
  cat >"$TEST_DIR/harnesses/shared/AGENTS.md" <<'DOC'
Use the generated skill catalog for skill discovery; do not mirror skills here.
DOC
  cat >"$TEST_DIR/index.yaml" <<'DOC'
skills:
  - name: good
  - name: weak
  - name: empty-trigger
  - name: empty-use
DOC
  cat >"$TEST_DIR/skills/good/SKILL.md" <<'DOC'
---
name: good
description: |
  Use when: "testing the audit helper on a well-shaped skill".
  Trigger: /good.
---

# good
DOC
  cat >"$TEST_DIR/skills/good/scripts/test_good.sh" <<'DOC'
#!/usr/bin/env bash
exit 0
DOC
  cat >"$TEST_DIR/skills/weak/SKILL.md" <<'DOC'
---
name: weak
description: This skill helps.
---

# weak
DOC
  cat >"$TEST_DIR/skills/empty-trigger/SKILL.md" <<'DOC'
---
name: empty-trigger
description: |
  Use when: "testing an empty trigger label".
  Trigger:
---

# empty-trigger
DOC
  cat >"$TEST_DIR/skills/empty-use/SKILL.md" <<'DOC'
---
name: empty-use
description: |
  Use when:
  Trigger: /empty-use.
---

# empty-use
DOC
  cat >"$TEST_DIR/skills/uncataloged/SKILL.md" <<'DOC'
---
name: uncataloged
description: |
  Use when: "testing catalog absence".
  Trigger: /uncataloged.
---

# uncataloged
DOC
  git -C "$TEST_DIR" add .
  git -C "$TEST_DIR" commit -q -m init
}

teardown_repo() {
  rm -rf "$TEST_DIR"
}

test_report_orders_by_severity() {
  setup_repo
  local report weak_line good_line
  report="$(python3 "$AUDIT" --repo "$TEST_DIR")"
  weak_line="$(printf '%s\n' "$report" | grep -n '### weak' | cut -d: -f1)"
  good_line="$(printf '%s\n' "$report" | grep -n '### good' | cut -d: -f1)"
  if [ "$weak_line" -lt "$good_line" ]; then
    assert_eq "report orders failing skill first" "ok" "ok"
  else
    assert_eq "report orders failing skill first" "ok" "bad"
  fi
  assert_contains "report names trigger failure" "$report" "description is generic"
  assert_contains "report rejects empty trigger label" "$report" "description lacks explicit Trigger alias"
  assert_contains "report rejects empty use label" "$report" "description lacks concrete use-case phrase"
  assert_contains "report names testing failure" "$report" "no tests/, __tests__/, evals/"
  assert_contains "report uses generated catalog" "$report" "Catalog source: index.yaml"
  assert_contains "report rejects uncataloged skill" "$report" "not present in generated skill catalog"
  teardown_repo
}

test_report_is_reproducible() {
  setup_repo
  local first second
  first="$(python3 "$AUDIT" --repo "$TEST_DIR")"
  second="$(python3 "$AUDIT" --repo "$TEST_DIR")"
  assert_eq "unchanged repo gives identical report" "$first" "$second"
  teardown_repo
}

test_output_flag_writes_report() {
  setup_repo
  python3 "$AUDIT" --repo "$TEST_DIR" --output .groom/audit-baseline-test.txt >/tmp/audit-output.txt
  if [ -f "$TEST_DIR/.groom/audit-baseline-test.txt" ]; then
    assert_eq "output flag writes report" "exists" "exists"
  else
    assert_eq "output flag writes report" "exists" "missing"
  fi
  teardown_repo
}

test_report_orders_by_severity
test_report_is_reproducible
test_output_flag_writes_report

if [ "$FAIL" -gt 0 ]; then
  echo "$FAIL failed, $PASS passed"
  exit 1
fi

echo "$PASS passed"
