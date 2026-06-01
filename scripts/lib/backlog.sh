#!/usr/bin/env bash
# Backlog trailer parsing and ticket archival helpers.
# Parses `Closes-backlog:` / `Ships-backlog:` / `Refs-backlog:` trailers from
# git commits and moves ticket files between `backlog.d/` and `backlog.d/_done/`.
#
# Usage:
#   source scripts/lib/backlog.sh
#   backlog_ids_from_commit HEAD             # prints unique numeric IDs
#   backlog_ids_from_range origin/main..HEAD # same, for a rev-range
#   backlog_file_for_id 031                  # prints path to ticket file
#   backlog_archive 031                      # git mv backlog.d/<id>-*.md into _done/
#
# Conventions:
#   - Closes-backlog and Ships-backlog are closure-intent trailers.
#   - Refs-backlog is a non-closing reference trailer.
#   - IDs are bare numeric strings (e.g. "031"), not "BACKLOG-031".
#   - Archival is idempotent: re-archiving an already-archived ID exits 0.
#   - Anchors to repo root via `git rev-parse --show-toplevel`, so helpers
#     are safe to call from subdirectories and worktrees.

# Source guard: sourcing twice is a no-op.
if [ -n "${BACKLOG_SH_SOURCED:-}" ]; then
  return 0 2>/dev/null || exit 0
fi
BACKLOG_SH_SOURCED=1

# Print all trailer keys this lib recognizes, one per line.
# Closes-backlog and Ships-backlog are closure-intent; Refs-backlog is reference-only.
backlog_trailer_keys() {
  printf '%s\n' \
    'Closes-backlog' \
    'Ships-backlog' \
    'Refs-backlog'
}

# Print only the closure-intent trailer keys, one per line.
backlog_closing_keys() {
  printf '%s\n' \
    'Closes-backlog' \
    'Ships-backlog'
}

# Extract unique sorted backlog IDs from a single <commit-ish>.
# Parses trailers via `git interpret-trailers --parse --no-divider`, keeping
# only values under keys from `backlog_closing_keys`.
# Args: <commit-ish>
# Returns: 0 and prints one ID per line if any found; 1 with no output otherwise.
backlog_ids_from_commit() {
  local commit="$1"
  local message
  if ! message="$(git log -1 --format=%B "$commit" 2>/dev/null)"; then
    return 1
  fi
  _backlog_ids_from_message "$message"
}

# Extract unique sorted backlog IDs from a <rev-range>.
# Deduplicates across every commit in the range.
# Args: <rev-range>   e.g. origin/main..HEAD, or <sha>^..<sha>
# Returns: 0 and prints one ID per line if any found; 1 with no output otherwise.
#
# Note: `git interpret-trailers --parse` only recognizes trailers in the
# final paragraph of its input, so concatenating messages would hide all but
# the last commit's trailers. We parse each commit in isolation and merge.
backlog_ids_from_range() {
  local range="$1"
  local shas
  if ! shas="$(git rev-list "$range" 2>/dev/null)"; then
    return 1
  fi
  local aggregate="" sha commit_ids
  for sha in $shas; do
    if commit_ids="$(backlog_ids_from_commit "$sha" 2>/dev/null)"; then
      aggregate="${aggregate}${commit_ids}"$'\n'
    fi
  done
  if [ -z "${aggregate//[[:space:]]/}" ]; then
    return 1
  fi
  printf '%s' "$aggregate" | awk 'NF' | sort -u
}

# Locate the ticket file matching <id>.
# Searches `backlog.d/<id>-*.md` first, then `backlog.d/_done/<id>-*.md`.
# Args: <id>
# Returns: 0 and prints repo-relative path on match; 1 on miss (no output).
backlog_file_for_id() {
  local id="$1"
  if ! _backlog_validate_id "$id"; then
    echo "backlog_file_for_id: invalid ID '$id'" >&2
    return 1
  fi
  local root
  if ! root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
    echo "backlog_file_for_id: not in a git repo" >&2
    return 1
  fi
  local match
  # Active tickets first.
  match="$(_backlog_first_match "$root/backlog.d" "$id")"
  if [ -n "$match" ]; then
    printf '%s\n' "backlog.d/$match"
    return 0
  fi
  # Then archived tickets.
  match="$(_backlog_first_match "$root/backlog.d/_done" "$id")"
  if [ -n "$match" ]; then
    printf '%s\n' "backlog.d/_done/$match"
    return 0
  fi
  return 1
}

# Archive the ticket file for <id> into backlog.d/_done/ via `git mv`.
# Idempotent: if the file is already in _done/, exits 0 silently.
# Fails with exit 1 and a stderr message if no file matches the ID.
# Does NOT commit; caller is responsible. Does NOT rewrite file contents.
# Args: <id>
backlog_archive() {
  local id="$1"
  if ! _backlog_validate_id "$id"; then
    echo "backlog_archive: invalid ID '$id'" >&2
    return 1
  fi
  local root
  if ! root="$(git rev-parse --show-toplevel 2>/dev/null)"; then
    echo "backlog_archive: not in a git repo" >&2
    return 1
  fi
  local active_matches done_match
  active_matches="$(_backlog_all_matches "$root/backlog.d" "$id")"
  done_match="$(_backlog_first_match "$root/backlog.d/_done" "$id")"

  if [ -z "$active_matches" ] && [ -n "$done_match" ]; then
    # Already archived — idempotent success.
    return 0
  fi
  if [ -z "$active_matches" ]; then
    echo "backlog_archive: no ticket file found for ID '$id'" >&2
    return 1
  fi

  if ! mkdir -p "$root/backlog.d/_done"; then
    echo "backlog_archive: could not create archive directory under '$root'" >&2
    return 1
  fi
  (
    _backlog_cd_repo_root "$root" || {
      echo "backlog_archive: could not cd to repo root '$root'" >&2
      exit 1
    }
    while IFS= read -r active_match; do
      [ -n "$active_match" ] || continue
      if ! git mv "backlog.d/$active_match" "backlog.d/_done/$active_match"; then
        echo "backlog_archive: could not move backlog.d/$active_match into backlog.d/_done/" >&2
        exit 1
      fi
    done <<EOF
$active_matches
EOF
  )
}

# --- Internal helpers ---

# Validate that an ID is a bare numeric string.
# Args: <id>
# Returns: 0 if valid, 1 otherwise.
_backlog_validate_id() {
  local id="$1"
  [[ "$id" =~ ^[0-9]+$ ]]
}

# Change to the repository root. Separated so tests can force the failure path
# without damaging the temporary repository.
_backlog_cd_repo_root() {
  cd "$1"
}

# Return the first `<id>-*.md` basename in <dir>, or empty string if none.
# Args: <dir> <id>
_backlog_first_match() {
  local dir="$1" id="$2"
  [ -d "$dir" ] || return 0
  local f
  for f in "$dir/$id"-*.md; do
    [ -e "$f" ] || continue
    basename "$f"
    return 0
  done
}

# Return every `<id>-*.md` basename in <dir>, one per line.
# Args: <dir> <id>
_backlog_all_matches() {
  local dir="$1" id="$2"
  [ -d "$dir" ] || return 0
  local f
  for f in "$dir/$id"-*.md; do
    [ -e "$f" ] || continue
    basename "$f"
  done
}

# Parse a commit-message blob (possibly multiple messages concatenated) and
# print unique sorted numeric IDs under closing keys. Exit 1 if none.
# Args: <message-text>   (passed on stdin-equivalent via here-string)
_backlog_ids_from_message() {
  local message="$1"
  local parsed keys_pattern
  parsed="$(printf '%s\n' "$message" | git interpret-trailers --parse --no-divider 2>/dev/null)" || return 1
  # Build anchored alternation: ^(Closes-backlog|Ships-backlog):
  keys_pattern="^($(backlog_closing_keys | paste -sd '|' -)):"
  local ids
  ids="$(
    printf '%s\n' "$parsed" |
      grep -E "$keys_pattern" |
      awk -F': ' '{print $2}' |
      awk '{gsub(/[[:space:]]/,""); if ($0 ~ /^[0-9]+$/) print}' |
      sort -u
  )"
  if [ -z "$ids" ]; then
    return 1
  fi
  printf '%s\n' "$ids"
}
