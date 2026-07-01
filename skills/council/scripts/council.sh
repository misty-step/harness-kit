#!/usr/bin/env bash
# council.sh — fan one task out to a bench of distinct models/personas in
# parallel, collect every lane's output, report which lanes failed.
#
# The CALLER owns composition (which families, which personas) — this script
# only executes the bench and collects results. See ../SKILL.md.
#
# Usage:
#   council.sh --task <task-file> --members <members.tsv> [--outdir DIR] [--timeout SEC]
#
# Default timeout is intentionally generous for strong reasoning lanes. Use a
# short cap only for smoke/slug checks; a timed-out lane is caller-capped
# evidence, not a model-quality verdict.
#
# members.tsv — tab-separated, one council member per line, '#' comments ok:
#   label <TAB> cli <TAB> model <TAB> persona
#     label   short slug for the output file (e.g. contrarian)
#     cli     opencode | pi   (default opencode)
#     model   OpenRouter slug WITHOUT the openrouter/ prefix
#             (e.g. moonshotai/kimi-k2.7-code)
#     persona one-line role/lens steer prepended to the shared task
#
# Each member is run cold and headless; nothing is shared between lanes but the
# task. Outputs land in <outdir>/<label>.out; a summary prints at the end.
# Exit 0 if >=1 lane succeeded, 1 if all failed, 2 on bad arguments.
set -uo pipefail

DEFAULT_TIMEOUT="${COUNCIL_TIMEOUT_SEC:-1200}"
TASK_FILE="" MEMBERS_FILE="" OUTDIR="" TIMEOUT="$DEFAULT_TIMEOUT"

die() { printf 'council.sh: %s\n' "$1" >&2; exit 2; }

while [ $# -gt 0 ]; do
  case "$1" in
    --task) TASK_FILE="${2:-}"; shift 2 ;;
    --members) MEMBERS_FILE="${2:-}"; shift 2 ;;
    --outdir) OUTDIR="${2:-}"; shift 2 ;;
    --timeout) TIMEOUT="${2:-}"; shift 2 ;;
    -h|--help) sed -n '2,28p' "$0"; exit 0 ;;
    *) die "unknown arg: $1" ;;
  esac
done

if [ -z "$TASK_FILE" ] || [ ! -f "$TASK_FILE" ]; then
  die "missing/unreadable --task file"
fi
if [ -z "$MEMBERS_FILE" ] || [ ! -f "$MEMBERS_FILE" ]; then
  die "missing/unreadable --members file"
fi
if ! [ "$TIMEOUT" -eq "$TIMEOUT" ] 2>/dev/null || [ "$TIMEOUT" -le 0 ]; then
  die "--timeout must be a positive integer"
fi
[ -n "$OUTDIR" ] || OUTDIR="$(dirname "$MEMBERS_FILE")/council-out"
mkdir -p "$OUTDIR" || die "cannot create outdir: $OUTDIR"

TASK="$(cat "$TASK_FILE")"

if [ "$TIMEOUT" -lt 600 ]; then
  printf 'council.sh: warning: %ss cap is for smoke/cheap lanes; timed-out reasoning lanes are caller-capped.\n' "$TIMEOUT" >&2
fi

# Portable per-lane timeout: run cmd in bg, kill it if it overruns.
run_capped() {
  local secs="$1"; shift
  local timeout_marker
  timeout_marker="$(mktemp "${TMPDIR:-/tmp}/council-timeout.XXXXXX")" || return 2
  rm -f "$timeout_marker"
  "$@" & local pid=$!
  (
    sleep "$secs"
    if kill -0 "$pid" 2>/dev/null; then
      printf 'timeout\n' >"$timeout_marker"
      kill -TERM "$pid" 2>/dev/null
      sleep 5
      kill -KILL "$pid" 2>/dev/null
    fi
  ) & local guard=$!
  wait "$pid" 2>/dev/null; local rc=$?
  kill -TERM "$guard" 2>/dev/null; wait "$guard" 2>/dev/null
  if [ -s "$timeout_marker" ]; then
    rm -f "$timeout_marker"
    return 124
  fi
  rm -f "$timeout_marker"
  return "$rc"
}

run_member() {
  local label="$1" cli="$2" model="$3" persona="$4"
  local prompt out status
  prompt=$(printf '%s\n\nYour task (deliberate from your perspective; be specific, surface the non-obvious, state where you disagree with the obvious answer):\n\n%s' "$persona" "$TASK")
  out="$OUTDIR/$label.out"

  case "$cli" in
    opencode) run_capped "$TIMEOUT" opencode run --model "openrouter/$model" "$prompt" >"$out" 2>&1 ;;
    pi)       run_capped "$TIMEOUT" pi -p --no-extensions --no-tools --provider openrouter --model "$model" "$prompt" >"$out" 2>&1 ;;
    *)        printf 'unknown cli: %s\n' "$cli" >"$out"; status=2 ;;
  esac
  status=${status:-$?}
  # A lane that produced no output is a failure even on rc 0.
  if [ "$status" -eq 0 ] && [ ! -s "$out" ]; then status=3; fi
  printf '%s\t%s\t%s\n' "$label" "$status" "$out" >"$OUTDIR/$label.status"
}

printf 'Convening %s lane(s), %ss cap each → %s\n' \
  "$(grep -cvE '^\s*(#|$)' "$MEMBERS_FILE")" "$TIMEOUT" "$OUTDIR" >&2

pids=()
# Manual tab split: `read` with IFS=$'\t' collapses runs of tabs (tab is
# whitespace), silently dropping empty interior fields. Split by hand instead.
while IFS= read -r raw || [ -n "$raw" ]; do
  case "$raw" in ''|\#*) continue ;; esac
  tabs="${raw//[!$'\t']/}"
  label="${raw%%$'\t'*}"
  if [ "${#tabs}" -lt 3 ]; then
    printf '%s\tmalformed-line\t-\n' "${label:-line}" >"$OUTDIR/${label:-line}.status"
    continue
  fi
  rest="${raw#*$'\t'}"
  cli="${rest%%$'\t'*}"; rest="${rest#*$'\t'}"
  model="${rest%%$'\t'*}"; persona="${rest#*$'\t'}"   # persona = everything after the 3rd tab
  cli="${cli:-opencode}"
  [ -n "$model" ] || { printf '%s\tskip-no-model\t-\n' "$label" >"$OUTDIR/$label.status"; continue; }
  run_member "$label" "$cli" "$model" "${persona:-Offer your most useful perspective.}" &
  pids+=("$!")
done <"$MEMBERS_FILE"

for p in "${pids[@]:-}"; do [ -n "$p" ] && wait "$p" 2>/dev/null; done

# Summary
ok=0 fail=0
printf '\n=== council summary ===\n'
for s in "$OUTDIR"/*.status; do
  [ -f "$s" ] || continue
  IFS=$'\t' read -r label status out <"$s"
  if [ "$status" = "0" ]; then ok=$((ok+1)); printf '  ✓ %-18s %s\n' "$label" "$out";
  else
    fail=$((fail+1))
    if [ "$status" = "124" ]; then
      printf '  ✗ %-18s (timeout after %ss; caller cap) %s\n' "$label" "$TIMEOUT" "$out"
    else
      printf '  ✗ %-18s (status %s) %s\n' "$label" "$status" "$out"
    fi
  fi
done
printf '%s ok, %s failed. Read the .out files; synthesize as chair (see SKILL.md).\n' "$ok" "$fail"
[ "$ok" -gt 0 ]
