#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

PYTHON_BIN="$(python3 -c 'import sys; print(sys.executable)')"
mkdir -p "$TMP/bin"
ln -s "$PYTHON_BIN" "$TMP/bin/python3"
export PATH="$TMP/bin:/usr/bin:/bin:/usr/sbin:/sbin:$PATH"

assert_exists() {
  local path="$1"
  [ -e "$path" ] || [ -L "$path" ] || {
    echo "missing expected path: $path" >&2
    exit 1
  }
}

assert_not_exists() {
  local path="$1"
  [ ! -e "$path" ] && [ ! -L "$path" ] || {
    echo "unexpected path exists: $path" >&2
    exit 1
  }
}

assert_symlink_to() {
  local path="$1"
  local target="$2"
  assert_exists "$path"
  [ -L "$path" ] || {
    echo "expected symlink: $path" >&2
    exit 1
  }
  [ "$(readlink "$path")" = "$target" ] || {
    echo "unexpected symlink target for $path: $(readlink "$path")" >&2
    exit 1
  }
}

assert_file_equals() {
  local actual="$1"
  local expected="$2"
  assert_exists "$actual"
  cmp -s "$actual" "$expected" || {
    echo "file does not match source: $actual" >&2
    exit 1
  }
}

assert_harness_skill_projection() {
  local harness="$1"
  local skill
  for skill in "$ROOT"/skills/*; do
    [ -d "$skill" ] || continue
    [ -f "$skill/SKILL.md" ] || continue
    assert_symlink_to "$harness/skills/$(basename "$skill")" "$skill"
  done
}

assert_harness_agent_projection() {
  local harness="$1"
  [ -d "$harness/agents" ] || {
    echo "expected agents directory for $harness" >&2
    exit 1
  }
  [ ! -L "$harness/agents" ] || {
    echo "agents directory should not be a parent symlink: $harness/agents" >&2
    exit 1
  }

  assert_symlink_to "$harness/agents/a11y-auditor.md" "$ROOT/agents/a11y-auditor.md"
  assert_symlink_to "$harness/agents/a11y-critic.md" "$ROOT/agents/a11y-critic.md"
  assert_symlink_to "$harness/agents/a11y-fixer.md" "$ROOT/agents/a11y-fixer.md"
}

assert_no_retired_agents() {
  local harness="$1"
  local retired
  for retired in beck builder carmack cooper critic grug ousterhout planner; do
    if [ "$harness" = "$HOME/.claude" ] && [ "$retired" = "ousterhout" ]; then
      continue
    fi
    assert_not_exists "$harness/agents/$retired.md"
  done
}

HOME="$TMP/home"
export HOME
mkdir -p \
  "$HOME/.claude/agents" \
  "$HOME/.codex" \
  "$HOME/.pi" \
  "$HOME/.gemini/antigravity-cli" \
  "$HOME/.gemini/antigravity-ide" \
  "$HOME/.gemini/antigravity"

printf 'user owned\n' > "$HOME/.claude/agents/ousterhout.md"
ln -s "$ROOT/agents/critic.md" "$HOME/.claude/agents/critic.md"
ln -s "$ROOT/agents" "$HOME/.codex/agents"
mkdir -p "$HOME/.claude/hooks/__pycache__"
ln -s "$ROOT/harnesses/claude/hooks" "$HOME/.claude/hooks/__pycache__/hooks"

HARNESS_KIT_DIR="$ROOT" bash "$ROOT/bootstrap.sh" >"$TMP/bootstrap-1.out"
HARNESS_KIT_DIR="$ROOT" bash "$ROOT/bootstrap.sh" >"$TMP/bootstrap-2.out"

for harness in \
  "$HOME/.claude" \
  "$HOME/.codex" \
  "$HOME/.pi" \
  "$HOME/.gemini/antigravity-cli" \
  "$HOME/.gemini/antigravity-ide" \
  "$HOME/.gemini/antigravity"
do
  assert_harness_skill_projection "$harness"
  assert_harness_agent_projection "$harness"
  assert_no_retired_agents "$harness"
done

assert_exists "$HOME/.claude/agents/ousterhout.md"
if [ "$(cat "$HOME/.claude/agents/ousterhout.md")" != "user owned" ]; then
  echo "user-owned retired agent was modified" >&2
  exit 1
fi
assert_not_exists "$HOME/.codex/agents/ousterhout.md"

assert_symlink_to "$HOME/.claude/CLAUDE.md" "$ROOT/harnesses/shared/AGENTS.md"
assert_file_equals "$HOME/.claude/settings.json" "$ROOT/harnesses/claude/settings.json"
for hook in "$ROOT"/harnesses/claude/hooks/*; do
  [ -f "$hook" ] || continue
  assert_symlink_to "$HOME/.claude/hooks/$(basename "$hook")" "$hook"
done
assert_not_exists "$HOME/.claude/hooks/__pycache__"

assert_symlink_to "$HOME/.codex/AGENTS.md" "$ROOT/harnesses/shared/AGENTS.md"
assert_symlink_to "$HOME/.codex/config/config.toml" "$ROOT/harnesses/codex/config.toml"

assert_symlink_to "$HOME/.pi/agent/AGENTS.md" "$ROOT/harnesses/shared/AGENTS.md"
assert_symlink_to "$HOME/.pi/settings.json" "$ROOT/harnesses/pi/settings.json"

assert_symlink_to "$HOME/.gemini/antigravity-cli/AGENTS.md" "$ROOT/harnesses/shared/AGENTS.md"
assert_symlink_to "$HOME/.gemini/antigravity-ide/AGENTS.md" "$ROOT/harnesses/shared/AGENTS.md"

assert_symlink_to "$HOME/.harness-kit/agents.yaml" "$ROOT/.harness-kit/agents.yaml"
assert_symlink_to "$HOME/.harness-kit/scripts/probe-agent-roster.py" "$ROOT/scripts/probe-agent-roster.py"
assert_symlink_to "$HOME/.spellbook/agents.yaml" "$ROOT/.harness-kit/agents.yaml"

if ! grep -q "Agents (3):" "$TMP/bootstrap-2.out"; then
  echo "bootstrap summary should report only three global agents" >&2
  exit 1
fi

echo "bootstrap projection ok"
