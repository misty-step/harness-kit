#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

declare -a failures=()

fail() {
  failures+=("$1")
}

matches() {
  local pattern="$1"
  shift
  grep -nE "$pattern" "$@" >/dev/null
}


if matches 'GLOBAL_SKILLS=\(tailor seed\)|minimal global|/tailor or /seed|per-repo via /tailor' \
  bootstrap.sh README.md AGENTS.md CODEBASE.md; then
  fail "global install docs/scripts must not describe the retired minimal tailor/seed model"
fi

if ! matches 'All first-party skills are installed system-wide' bootstrap.sh; then
  fail "bootstrap must report the all-first-party-skills system-wide install contract"
fi

if ! matches 'install_system_roster|~/.harness-kit/agents.yaml|\\$HOME/.harness-kit' bootstrap.sh; then
  fail "bootstrap must install the provider roster into a system-wide Harness Kit location"
fi

if ! matches 'legacy_system_dir=.*\\.spellbook|legacy agents.yaml' bootstrap.sh; then
  fail "bootstrap must keep a legacy Spellbook roster alias for long-running stale instruction contexts"
fi

if ! matches 'HARNESS_KIT_ROSTER|system_harness_kit_dir|\\.harness-kit.*agents.yaml' \
  scripts/lib/agent_roster.py; then
  fail "roster helpers must fall back to a system-wide roster when repo-local roster is absent"
fi

if ! matches '\+skills/\*\*' harnesses/pi/settings.json; then
  fail "Pi settings must allow all globally installed first-party skills"
fi

if [ "${#failures[@]}" -gt 0 ]; then
  echo "harness install path check failed:" >&2
  for failure in "${failures[@]}"; do
    echo "  - $failure" >&2
  done
  exit 1
fi

echo "harness install paths are cross-harness and global-skill first."
