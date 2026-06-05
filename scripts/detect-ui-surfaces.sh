#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat >&2 <<'EOF'
usage: scripts/detect-ui-surfaces.sh [--staged|--unstaged|--base <ref>|--paths <path>...]

Prints JSON describing whether any provided or changed path is likely a UI or
visual surface. The result is a routing signal, not a quality verdict.
EOF
}

mode="unstaged"
base_ref=""
declare -a explicit_paths=()

while [[ $# -gt 0 ]]; do
  case "$1" in
    --staged)
      mode="staged"
      shift
      ;;
    --unstaged)
      mode="unstaged"
      shift
      ;;
    --base)
      if [[ $# -lt 2 ]]; then
        usage
        exit 2
      fi
      mode="base"
      base_ref="$2"
      shift 2
      ;;
    --paths)
      mode="paths"
      shift
      if [[ $# -eq 0 ]]; then
        usage
        exit 2
      fi
      explicit_paths=("$@")
      break
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      usage
      exit 2
      ;;
  esac
done

json_escape() {
  local value="$1"
  value=${value//\\/\\\\}
  value=${value//\"/\\\"}
  value=${value//$'\n'/\\n}
  printf '%s' "$value"
}

path_matches_visual_surface() {
  local path="$1"

  case "$path" in
    *.tsx|*.jsx|*.vue|*.svelte|*.css|*.scss|*.sass|*.less|*.html|*.mdx|*.svg)
      return 0
      ;;
    *.png|*.jpg|*.jpeg|*.webp|*.gif|*.ppt|*.pptx|*.key|*.pdf)
      return 0
      ;;
    tailwind.config.*|components.json|tokens.*|theme.*|*/tokens.*|*/theme.*)
      return 0
      ;;
  esac

  case "$path" in
    app/*|pages/*|components/*|src/components/*|stories/*|*.stories.*|*.story.*)
      return 0
      ;;
    docs/site/*|docs/copy/images/*|docs/copy/site.json|reports/*|presentations/*|slides/*)
      return 0
      ;;
  esac

  return 1
}

collect_paths() {
  case "$mode" in
    staged)
      git diff --cached --name-only --diff-filter=ACMR
      ;;
    unstaged)
      git diff --name-only --diff-filter=ACMR
      ;;
    base)
      git diff --name-only --diff-filter=ACMR "$base_ref"...HEAD
      ;;
    paths)
      printf '%s\n' "${explicit_paths[@]}"
      ;;
    *)
      return 2
      ;;
  esac
}

paths_file="$(mktemp "${TMPDIR:-/tmp}/detect-ui-surfaces.XXXXXX")"
trap 'rm -f "$paths_file"' EXIT
if ! collect_paths >"$paths_file"; then
  echo "failed to collect paths for mode: $mode" >&2
  exit 2
fi

declare -a matches=()
while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  if path_matches_visual_surface "$path"; then
    matches+=("$path")
  fi
done <"$paths_file"

printf '{"ui_surface":'
if [[ "${#matches[@]}" -gt 0 ]]; then
  printf 'true'
else
  printf 'false'
fi
printf ',"visual_surface":'
if [[ "${#matches[@]}" -gt 0 ]]; then
  printf 'true'
else
  printf 'false'
fi
printf ',"mode":"%s","matches":[' "$(json_escape "$mode")"
if [[ "${#matches[@]}" -gt 0 ]]; then
  for i in "${!matches[@]}"; do
    if [[ "$i" -gt 0 ]]; then
      printf ','
    fi
    printf '"%s"' "$(json_escape "${matches[$i]}")"
  done
fi
printf ']}\n'
