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

if matches 'into \.claude/ with no filtering' \
  skills/seed/SKILL.md index.yaml; then
  fail "seed must not describe repo installs as Claude-only"
fi

if matches 'Copy every skill in .+ into `?\.claude/skills/' \
  skills/seed/SKILL.md; then
  fail "seed must install into a shared skill root, not copy skills directly into .claude/skills/"
fi

if ! matches 'shared skill root|shared repo-local skill layer|shared skill layer' \
  skills/seed/SKILL.md; then
  fail "seed must name the shared skill root as the canonical install target"
fi

if ! matches '\.claude/skills/.+symlink|\.claude/skills/.+bridge|bridge layer' \
  skills/seed/SKILL.md; then
  fail "seed must describe .claude/skills as a bridge, not the source of truth"
fi

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
