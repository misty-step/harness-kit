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

HOME="$TMP/home"
export HOME
mkdir -p "$HOME/.claude/agents" "$HOME/.codex"

printf 'user owned\n' > "$HOME/.claude/agents/ousterhout.md"
ln -s "$ROOT/agents/critic.md" "$HOME/.claude/agents/critic.md"
ln -s "$ROOT/agents" "$HOME/.codex/agents"

HARNESS_KIT_DIR="$ROOT" bash "$ROOT/bootstrap.sh" >"$TMP/bootstrap-1.out"
HARNESS_KIT_DIR="$ROOT" bash "$ROOT/bootstrap.sh" >"$TMP/bootstrap-2.out"

for harness in "$HOME/.claude" "$HOME/.codex"; do
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

  for retired in beck builder carmack cooper critic grug planner; do
    assert_not_exists "$harness/agents/$retired.md"
  done
done

assert_exists "$HOME/.claude/agents/ousterhout.md"
if [ "$(cat "$HOME/.claude/agents/ousterhout.md")" != "user owned" ]; then
  echo "user-owned retired agent was modified" >&2
  exit 1
fi
assert_not_exists "$HOME/.codex/agents/ousterhout.md"

if ! grep -q "Agents (3):" "$TMP/bootstrap-2.out"; then
  echo "bootstrap summary should report only three global agents" >&2
  exit 1
fi

echo "bootstrap agent allowlist ok"
