#!/usr/bin/env bash
set -euo pipefail

pythonpath="ci/src${PYTHONPATH:+:${PYTHONPATH}}"
touch .env
before_snapshot="$(mktemp -d)"
stage_plan="$(mktemp)"
cleanup() {
  rm -rf "$before_snapshot" "$stage_plan"
}
trap cleanup EXIT

if ! git diff --cached --quiet; then
  printf 'heal requires an empty index; unstage or commit existing staged changes first\n' >&2
  exit 1
fi

apply_snapshot_patch() {
  local path="$1"
  local before_path="$before_snapshot/$path"
  local patch_file
  local diff_status=0

  patch_file="$(mktemp)"

  if [[ -e "$before_path" && -e "$path" ]]; then
    diff -u \
      --label "a/$path" \
      --label "b/$path" \
      -- "$before_path" "$path" >"$patch_file" || diff_status=$?
  elif [[ -e "$before_path" ]]; then
    diff -u \
      --label "a/$path" \
      --label "b/$path" \
      -- "$before_path" /dev/null >"$patch_file" || diff_status=$?
  else
    diff -u \
      --label "a/$path" \
      --label "b/$path" \
      -- /dev/null "$path" >"$patch_file" || diff_status=$?
  fi

  if [[ "$diff_status" -gt 1 ]]; then
    rm -f "$patch_file"
    printf 'failed to compute a repair patch for %s\n' "$path" >&2
    exit 1
  fi

  if [[ ! -s "$patch_file" ]]; then
    rm -f "$patch_file"
    return 0
  fi

  if ! git apply --cached --whitespace=nowarn "$patch_file"; then
    rm -f "$patch_file"
    printf 'heal cannot stage %s without including pre-existing edits; commit the repair manually\n' "$path" >&2
    exit 1
  fi

  rm -f "$patch_file"
}

rsync -a --delete --exclude '.git' --exclude '.env' ./ "$before_snapshot"/

check_status=0
if ! check_output="$(DAGGER_NO_NAG="${DAGGER_NO_NAG:-1}" dagger call check 2>&1)"; then
  check_status=$?
fi

gate="$(PYTHONPATH="$pythonpath" python3 - <<'PY' "$check_output"
import sys

from harness_kit_ci.heal_support import first_failed_gate

print(first_failed_gate(sys.argv[1]) or "")
PY
)"

if [[ -z "$gate" ]]; then
  printf '%s\n' "$check_output"
  exit "$check_status"
fi

branch="$(PYTHONPATH="$pythonpath" python3 - <<'PY' "$gate"
import sys

from harness_kit_ci.heal_support import repair_branch_name

print(repair_branch_name(sys.argv[1]))
PY
)"

commit_message="$(PYTHONPATH="$pythonpath" python3 - <<'PY' "$gate"
import sys

from harness_kit_ci.heal_support import repair_commit_message

print(repair_commit_message(sys.argv[1]))
PY
)"

DAGGER_NO_NAG="${DAGGER_NO_NAG:-1}" dagger call --allow-llm all -o . heal "$@"
DAGGER_NO_NAG="${DAGGER_NO_NAG:-1}" dagger call check >/dev/null
git switch -c "$branch"

PYTHONPATH="$pythonpath" python3 - <<'PY' "$before_snapshot" > "$stage_plan"
from pathlib import Path
import sys

from harness_kit_ci.heal_support import snapshot_delta

before = Path(sys.argv[1])
after = Path(".")
stage, remove = snapshot_delta(before, after)

for path in remove:
    print(f"D\t{path}")
for path in stage:
    print(f"S\t{path}")
PY

while IFS=$'\t' read -r action path; do
  [[ -n "${action:-}" ]] || continue
  case "$action" in
    D | S) apply_snapshot_patch "$path" ;;
    *) printf 'unknown stage action: %s\n' "$action" >&2; exit 1 ;;
  esac
done < "$stage_plan"

if git diff --cached --quiet; then
  printf 'heal produced no commit-ready diff\n' >&2
  exit 1
fi

git commit -m "$commit_message"
printf 'Healed %s on %s\n' "$gate" "$branch"
