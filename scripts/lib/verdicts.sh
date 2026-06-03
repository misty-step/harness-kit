#!/usr/bin/env bash
# Git-native review verdict storage.
# Verdicts use git refs: refs/verdicts/<branch> pointing to JSON blobs.
# Source this file, then call verdict_write/verdict_read/verdict_validate/etc.
#
# Usage:
#   source scripts/lib/verdicts.sh
#   verdict_write "feat-foo" '{"branch":"feat-foo","verdict":"ship",...}'
#   verdict_read  "feat-foo"     # prints verdict JSON
#   verdict_validate "feat-foo"  # returns 0 if verdict exists and SHA matches HEAD
#   verdict_push origin          # pushes refs/verdicts/* to a remote
#   verdict_fetch origin         # fetches refs/verdicts/* from a remote
#   verdict_delete "feat-foo"    # removes verdict ref
#   verdict_list                 # prints all verdict refs

VERDICTS_REF_PREFIX="refs/verdicts"
VERDICT_REQUIRED_FIELDS='["branch","verdict","sha","date"]'

# Validate JSON and required fields.
# Args: <json>
# Returns: 0 if valid, 1 if invalid
_verdict_validate_json() {
  local json="$1"
  # Check it's valid JSON
  if ! printf '%s' "$json" | python3 -c "import sys,json; json.load(sys.stdin)" 2>/dev/null; then
    return 1
  fi
  # Check required fields
  local missing
  missing="$(printf '%s' "$json" | python3 -c "
import sys, json
data = json.load(sys.stdin)
required = $VERDICT_REQUIRED_FIELDS
missing = [f for f in required if f not in data]
if missing:
    print(','.join(missing))
    sys.exit(1)
" 2>&1)" || return 1
  return 0
}

# Write a verdict for a branch.
# Args: <branch> <json>
# Returns: 0 on success, 1 on validation failure
verdict_write() {
  local branch="$1" json="$2"
  if ! _verdict_validate_json "$json"; then
    echo "verdict_write: invalid JSON or missing required fields" >&2
    return 1
  fi
  local blob_sha
  blob_sha="$(printf '%s' "$json" | git hash-object -w --stdin)"
  git update-ref "${VERDICTS_REF_PREFIX}/${branch}" "$blob_sha"
}

# Read a verdict for a branch.
# Args: <branch>
# Returns: 0 and prints JSON, or 1 if no verdict exists
verdict_read() {
  local branch="$1"
  local ref="${VERDICTS_REF_PREFIX}/${branch}"
  if ! git rev-parse --verify "$ref" &>/dev/null; then
    return 1
  fi
  git cat-file -p "$ref"
}

# Validate that a verdict exists and its SHA matches the branch HEAD.
# Args: <branch>
# Returns: 0 if valid, 1 if missing or stale
verdict_validate() {
  local branch="$1"
  local head_sha
  if ! head_sha="$(git rev-parse --verify "${branch}^{commit}" 2>/dev/null)"; then
    echo "verdict_validate: could not resolve target '$branch'" >&2
    return 1
  fi
  local json
  json="$(verdict_read "$branch")" || return 1
  local verdict_sha
  verdict_sha="$(printf '%s' "$json" | python3 -c "import sys,json; print(json.load(sys.stdin)['sha'])")"
  [ "$verdict_sha" = "$head_sha" ]
}

# Check if a branch is landable: verdict exists, SHA matches HEAD, and is allowed.
# Args: <branch>
# Returns: 0 if landable, 1 if missing/stale/unknown, 2 if dont-ship
verdict_check_landable() {
  local branch="$1"
  verdict_validate "$branch" || return 1
  local json verdict_value
  json="$(verdict_read "$branch")" || return 1
  verdict_value="$(printf '%s' "$json" | python3 -c "import sys,json; print(json.load(sys.stdin)['verdict'])" 2>/dev/null)" || return 1
  case "$verdict_value" in
    ship | conditional) return 0 ;;
    dont-ship) return 2 ;;
    *) return 1 ;;
  esac
}

# Delete a verdict ref.
# Args: <branch>
# Returns: 0 on success
verdict_delete() {
  local branch="$1"
  git update-ref -d "${VERDICTS_REF_PREFIX}/${branch}" 2>/dev/null
}

# List all verdict refs.
# Prints: one ref per line (short name)
verdict_list() {
  git for-each-ref "${VERDICTS_REF_PREFIX}/" --format='%(refname:short)'
}

# Push all local verdict refs to a git remote.
# Args: [remote] (default: origin)
# Returns: 0 on success. No local verdicts is a successful no-op.
verdict_push() {
  local remote="${1:-origin}"
  if ! git for-each-ref --format='%(refname)' "${VERDICTS_REF_PREFIX}/" | grep -q .; then
    return 0
  fi
  git push "$remote" "${VERDICTS_REF_PREFIX}/*:${VERDICTS_REF_PREFIX}/*"
}

# Fetch all verdict refs from a git remote.
# Args: [remote] (default: origin)
# Returns: 0 on success. No remote verdicts is a successful no-op.
verdict_fetch() {
  local remote="${1:-origin}"
  local refs
  if ! refs="$(git ls-remote --refs "$remote" "${VERDICTS_REF_PREFIX}/*")"; then
    return 1
  fi
  if [ -z "$refs" ]; then
    return 0
  fi
  git fetch "$remote" "${VERDICTS_REF_PREFIX}/*:${VERDICTS_REF_PREFIX}/*"
}
