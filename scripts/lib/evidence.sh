#!/usr/bin/env bash
# Git-native evidence directory helpers.
# Source this file, then call evidence_dir/evidence_dir_create/evidence_trailer.
#
# Usage:
#   source scripts/lib/evidence.sh
#   EVIDENCE_DIR="$(evidence_dir_create)"   # .evidence/<branch-slug>/<date>/
#   evidence_trailer "$EVIDENCE_DIR"        # QA-Evidence: ... when non-empty

if [ -n "${EVIDENCE_SH_SOURCED:-}" ]; then
  return 0 2>/dev/null || exit 0
fi
EVIDENCE_SH_SOURCED=1

evidence_branch_slug() {
  local branch="${1:-}"
  if [ -z "$branch" ]; then
    branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null)" || return 1
  fi
  printf '%s' "$branch" |
    LC_ALL=C tr -c 'A-Za-z0-9._-' '-' |
    sed -E 's/-+/-/g; s/^-//; s/-$//'
}

evidence_date() {
  date -u +%Y-%m-%d
}

evidence_dir() {
  local branch="${1:-}" day="${2:-}"
  local slug
  slug="$(evidence_branch_slug "$branch")" || return 1
  if [ -z "$slug" ]; then
    echo "evidence_dir: empty branch slug" >&2
    return 1
  fi
  if [ -z "$day" ]; then
    day="$(evidence_date)"
  fi
  printf '.evidence/%s/%s/\n' "$slug" "$day"
}

evidence_dir_create() {
  local dir
  dir="$(evidence_dir "${1:-}" "${2:-}")" || return 1
  mkdir -p "$dir"
  printf '%s\n' "$dir"
}

evidence_trailer() {
  local dir="${1:-}"
  if [ -z "$dir" ]; then
    dir="$(evidence_dir "${2:-}" "${3:-}")" || return 1
  fi
  if [ -d "$dir" ] && find "$dir" -mindepth 1 -print -quit | grep -q .; then
    printf 'QA-Evidence: %s\n' "$dir"
  fi
}
