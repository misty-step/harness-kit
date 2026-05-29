#!/usr/bin/env bash
set -euo pipefail

if ! command -v gh >/dev/null 2>&1; then
  echo "gh CLI is required" >&2
  exit 1
fi

if ! command -v jq >/dev/null 2>&1; then
  echo "jq is required" >&2
  exit 1
fi

selector="${1:-}"

if [[ -n "$selector" ]]; then
  pr_json="$(gh pr view "$selector" --json number,title,url,baseRefName,headRefName)"
else
  pr_json="$(gh pr view --json number,title,url,baseRefName,headRefName)"
fi

pr_number="$(jq -r '.number' <<<"$pr_json")"
pr_title="$(jq -r '.title' <<<"$pr_json")"
pr_url="$(jq -r '.url' <<<"$pr_json")"
base_ref="$(jq -r '.baseRefName' <<<"$pr_json")"
head_ref="$(jq -r '.headRefName' <<<"$pr_json")"
repo="$(jq -r '.url | capture("https?://[^/]*/(?<repo>[^/]+/[^/]+)/pull/") | .repo' <<<"$pr_json")"

fetch_array() {
  local endpoint="$1"
  gh api --paginate "$endpoint" | jq -s 'add // []'
}

reviews_json="$(fetch_array "/repos/$repo/pulls/$pr_number/reviews?per_page=100")"
inline_comments_json="$(fetch_array "/repos/$repo/pulls/$pr_number/comments?per_page=100")"
issue_comments_json="$(fetch_array "/repos/$repo/issues/$pr_number/comments?per_page=100")"

printf '# PR %s: %s\n' "$pr_number" "$pr_title"
printf 'URL: %s\n' "$pr_url"
printf 'Base: %s\n' "$base_ref"
printf 'Head: %s\n\n' "$head_ref"

printf '## Review Summaries\n\n'
jq -r '
  if length == 0 then
    "_none_\n"
  else
    sort_by(.submitted_at // .created_at)[] |
    "### Review #\(.id) — \(.user.login) — \(.state) — \(.submitted_at // .submittedAt // .created_at)\n" +
    "Commit: \(.commit_id // "n/a")\n" +
    "URL: \(.html_url // "n/a")\n\n" +
    ((.body // "") | if . == "" then "_no body_" else . end) +
    "\n"
  end
' <<<"$reviews_json"

printf '\n## Inline Review Comments\n\n'
jq -r '
  if length == 0 then
    "_none_\n"
  else
    sort_by(.created_at)[] |
    "### Comment #\(.id) — \(.user.login) — \(.path):\(.line // .original_line // "n/a")\n" +
    "Created: \(.created_at)\n" +
    "Review ID: \(.pull_request_review_id // "n/a")\n" +
    "In reply to: \(.in_reply_to_id // "n/a")\n" +
    "URL: \(.html_url // "n/a")\n\n" +
    (if (.diff_hunk // "") == "" then "" else "```diff\n\(.diff_hunk)\n```\n\n" end) +
    ((.body // "") | if . == "" then "_no body_" else . end) +
    "\n"
  end
' <<<"$inline_comments_json"

printf '\n## PR Conversation Comments\n\n'
jq -r '
  if length == 0 then
    "_none_\n"
  else
    sort_by(.created_at)[] |
    "### Comment #\(.id) — \(.user.login) — \(.created_at)\n" +
    "URL: \(.html_url // "n/a")\n\n" +
    ((.body // "") | if . == "" then "_no body_" else . end) +
    "\n"
  end
' <<<"$issue_comments_json"
