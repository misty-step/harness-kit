#!/usr/bin/env bash
set -euo pipefail

# Harness Kit Bootstrap
#
# Two modes:
#   LOCAL:  Symlinks harness dirs to a local Harness Kit checkout (fast, editable)
#   REMOTE: Downloads from GitHub (works on any machine without a checkout)
#
# Local mode is preferred. Remote is the fallback for fresh machines.
#
# Run: curl -sL https://raw.githubusercontent.com/misty-step/harness-kit/master/bootstrap.sh | bash

REPO="${HARNESS_KIT_REPO:-misty-step/harness-kit}"
RAW="https://raw.githubusercontent.com/$REPO/master"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REMOTE_TMP=""
REMOTE_HARNESS_KIT=""
ALLOWED_GLOBAL_AGENTS=(a11y-auditor a11y-fixer a11y-critic)
RETIRED_GLOBAL_AGENTS=(beck builder carmack cooper critic grug ousterhout planner)

cleanup_remote_tmp() {
  [ -z "$REMOTE_TMP" ] && return 0
  rm -rf "$REMOTE_TMP"
}
trap cleanup_remote_tmp EXIT

info()  { printf '\033[0;34m%s\033[0m\n' "$*"; }
ok()    { printf '\033[0;32m%s\033[0m\n' "$*"; }
warn()  { printf '\033[0;33m%s\033[0m\n' "$*"; }
err()   { printf '\033[0;31m%s\033[0m\n' "$*" >&2; }

is_harness_kit_checkout() {
  local dir="$1"
  [ -d "$dir/skills" ] && [ -d "$dir/agents" ] && [ -d "$dir/harnesses" ]
}

is_worktree_checkout() {
  local dir="$1"
  [ -f "$dir/.git" ]
}

resolve_harness_kit_dir() {
  if [ -n "${HARNESS_KIT_DIR:-}" ] && is_harness_kit_checkout "$HARNESS_KIT_DIR"; then
    printf '%s\n' "$HARNESS_KIT_DIR"
    return 0
  fi

  local candidate
  if is_worktree_checkout "$SCRIPT_DIR"; then
    for candidate in \
      "$HOME/Development/harness-kit" \
      "$HOME/dev/harness-kit" \
      "$HOME/src/harness-kit" \
      "$HOME/code/harness-kit"
    do
      if is_harness_kit_checkout "$candidate"; then
        printf '%s\n' "$candidate"
        return 0
      fi
    done
  fi

  if is_harness_kit_checkout "$SCRIPT_DIR"; then
    printf '%s\n' "$SCRIPT_DIR"
    return 0
  fi

  for candidate in \
    "$HOME/Development/harness-kit" \
    "$HOME/dev/harness-kit" \
    "$HOME/src/harness-kit" \
    "$HOME/code/harness-kit"
  do
    if is_harness_kit_checkout "$candidate"; then
      printf '%s\n' "$candidate"
      return 0
    fi
  done

  return 1
}

contains() {
  local needle="$1"
  shift
  local item
  for item in "$@"; do
    [ "$item" = "$needle" ] && return 0
  done
  return 1
}

is_global_agent_allowed() {
  local agent="$1"
  contains "$agent" "${ALLOWED_GLOBAL_AGENTS[@]}"
}

is_harness_kit_agents_target() {
  local target="$1"
  case "$target" in
    */harness-kit/agents|*/harness-kit/agents/*) return 0 ;;
    *) return 1 ;;
  esac
}

retired_agent_sha256() {
  case "$1" in
    beck) printf '%s\n' "e62243327817aa919d1fc6e1f9b50bacf915a977b1cb62b194a7d9933047d883" ;;
    builder) printf '%s\n' "44866cd33025a82492c0265b054e7d9dcb34791ccc402778acdaf154d869176c" ;;
    carmack) printf '%s\n' "a3779a7e2eb3551d1f82d378d88d739ca2c5a2fbffd9ec3b4f235cf8658279cb" ;;
    cooper) printf '%s\n' "01b056a4ac5c2702e249063da0d2bd38e6fdf2f02dd7c129d26b8f6b046ecc34" ;;
    critic) printf '%s\n' "da4f51f631355b445586193edff56ac7435b960d48629bdcb754aed2c9777566" ;;
    grug) printf '%s\n' "0e4166caa7b103db02e983d8fb9c784b5d617dabe0b8f6d6e3d62787093ddbd7" ;;
    ousterhout) printf '%s\n' "39a5f7bb89296cf0fb6fc653b8d40c2c386b33a446659a0b8419f3cf784aa140" ;;
    planner) printf '%s\n' "8c65b4132a0b31de176f22610d800bcbb19af60e2ceb73f124da2efc173e697e" ;;
    *) return 1 ;;
  esac
}

file_sha256() {
  local path="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$path" | awk '{print $1}'
  else
    shasum -a 256 "$path" | awk '{print $1}'
  fi
}

cleanup_symlinks_under_prefix() {
  local dir="$1"
  local prefix="$2"
  shift 2
  local expected=("$@")

  mkdir -p "$dir"

  local entry base target
  for entry in "$dir"/*; do
    [ -e "$entry" ] || [ -L "$entry" ] || continue
    [ -L "$entry" ] || continue
    target="$(readlink "$entry" || true)"
    case "$target" in
      "$prefix"/*)
        base="$(basename "$entry")"
        if ! contains "$base" "${expected[@]}"; then
          rm -rf "$entry"
          ok "    removed stale $(basename "$dir")/$base"
        fi
        ;;
    esac
  done
}

prepare_agents_dir() {
  local dir="$1"
  local source_agents_dir="$2"
  local label="$3"

  if [ -L "$dir" ]; then
    local target
    target="$(readlink "$dir" || true)"
    if [ "$target" = "$source_agents_dir" ] || is_harness_kit_agents_target "$target"; then
      rm -f "$dir"
      ok "    removed stale $label parent symlink"
    else
      warn "    $label is a user-owned symlink; leaving agents unchanged"
      return 1
    fi
  elif [ -e "$dir" ] && [ ! -d "$dir" ]; then
    warn "    $label is not a directory; leaving agents unchanged"
    return 1
  fi

  mkdir -p "$dir"
}

cleanup_retired_agents() {
  local dir="$1"
  local source_root="$2"
  local agent dest src target

  [ -d "$dir" ] || return 0

  for agent in "${RETIRED_GLOBAL_AGENTS[@]}"; do
    dest="$dir/$agent.md"
    src="$source_root/agents/$agent.md"
    [ -e "$dest" ] || [ -L "$dest" ] || continue

    if [ -L "$dest" ]; then
      target="$(readlink "$dest" || true)"
      if is_harness_kit_agents_target "$target"; then
        rm -f "$dest"
        ok "    removed retired agent $agent"
      else
        warn "    preserving user-owned agent $agent"
      fi
    elif [ -f "$dest" ]; then
      local expected actual
      expected="$(retired_agent_sha256 "$agent" || true)"
      actual="$(file_sha256 "$dest")"
      if { [ -f "$src" ] && cmp -s "$src" "$dest"; } || { [ -n "$expected" ] && [ "$actual" = "$expected" ]; }; then
        rm -f "$dest"
        ok "    removed retired copied agent $agent"
      else
        warn "    preserving user-owned agent $agent"
      fi
    else
      warn "    preserving user-owned agent $agent"
    fi
  done
}

remove_path_if_symlink_to_prefix() {
  local path="$1"
  local prefix="$2"
  local label="$3"

  [ -L "$path" ] || return 0

  local target
  target="$(readlink "$path" || true)"
  case "$target" in
    "$prefix"/*)
      rm -f "$path"
      ok "    removed stale $label"
      ;;
  esac
}

link_file_if_present() {
  local src="$1"
  local dest="$2"
  local label="$3"

  [ -e "$src" ] || return 0

  mkdir -p "$(dirname "$dest")"
  ln -sfn "$src" "$dest"
  ok "    $label"
}

link_dir_entries_if_present() {
  local src_dir="$1"
  local dest_dir="$2"
  local label="$3"

  [ -d "$src_dir" ] || return 0

  local expected=()
  local src
  for src in "$src_dir"/*; do
    [ -e "$src" ] || continue
    case "$(basename "$src")" in
      __pycache__|*.pyc) continue ;;
    esac
    expected+=("$(basename "$src")")
  done

  cleanup_symlinks_under_prefix "$dest_dir" "$src_dir" "${expected[@]}"
  rm -rf "$dest_dir/__pycache__"

  mkdir -p "$dest_dir"
  for src in "$src_dir"/*; do
    [ -e "$src" ] || continue
    case "$(basename "$src")" in
      __pycache__|*.pyc) continue ;;
    esac
    ln -sfn "$src" "$dest_dir/$(basename "$src")"
  done

  ok "    $label"
}

sanitize_claude_settings_json() {
  local settings_file="$1"
  [ -f "$settings_file" ] || return 0

  python3 - "$settings_file" <<'PY'
import json
import os
import re
import sys
from pathlib import Path

settings_path = Path(sys.argv[1]).expanduser()
data = json.loads(settings_path.read_text())

hook_path_re = re.compile(r'~/.claude/hooks/[^ "\']+')
changed = False

hooks = data.get("hooks")
if isinstance(hooks, dict):
    cleaned = {}
    for event, groups in hooks.items():
        if not isinstance(groups, list):
            cleaned[event] = groups
            continue

        kept_groups = []
        for group in groups:
            if not isinstance(group, dict):
                kept_groups.append(group)
                continue

            entries = group.get("hooks")
            if not isinstance(entries, list):
                kept_groups.append(group)
                continue

            kept_entries = []
            for entry in entries:
                if not isinstance(entry, dict):
                    kept_entries.append(entry)
                    continue

                command = entry.get("command", "")
                match = hook_path_re.search(command)
                if match:
                    hook_file = Path(os.path.expanduser(match.group(0)))
                    if not hook_file.exists():
                        changed = True
                        continue
                kept_entries.append(entry)

            if kept_entries:
                if len(kept_entries) != len(entries):
                    changed = True
                group = dict(group)
                group["hooks"] = kept_entries
                kept_groups.append(group)
            else:
                changed = True

        cleaned[event] = kept_groups

    data["hooks"] = cleaned

if changed:
    settings_path.write_text(json.dumps(data, indent=2) + "\n")
PY
}

copy_claude_settings_if_present() {
  local src="$1"
  local dest="$2"

  [ -f "$src" ] || return 0

  mkdir -p "$(dirname "$dest")"
  cp "$src" "$dest"
  sanitize_claude_settings_json "$dest"
  ok "    settings.json (copied)"
}

verify_no_broken_harness_kit_symlinks() {
  local dir="$1"
  local maxdepth="$2"
  local broken=0
  local link target

  while IFS= read -r link; do
    target="$(readlink "$link" || true)"
    case "$target" in
      "$HARNESS_KIT"/*)
        if [ ! -e "$link" ]; then
          err "Broken symlink: $link -> $target"
          broken=1
        fi
        ;;
    esac
  done < <(find "$dir" -maxdepth "$maxdepth" -type l 2>/dev/null)

  return "$broken"
}

install_system_file() {
  local src="$1"
  local dest="$2"
  local label="$3"

  [ -e "$src" ] || { warn "    missing $label"; return 0; }
  mkdir -p "$(dirname "$dest")"
  rm -rf "$dest"
  if [ -n "$HARNESS_KIT" ]; then
    ln -sfn "$src" "$dest"
  else
    cp -R "$src" "$dest"
  fi
  ok "    $label"
}

install_system_roster() {
  local source_root="${HARNESS_KIT:-$REMOTE_HARNESS_KIT}"
  local system_dir="$HOME/.harness-kit"
  local legacy_system_dir="$HOME/.spellbook"

  info "Installing system roster..."
  install_system_file "$source_root/.harness-kit/agents.yaml" \
    "$system_dir/agents.yaml" "agents.yaml"
  install_system_file "$source_root/.harness-kit/agents.yaml" \
    "$legacy_system_dir/agents.yaml" "legacy agents.yaml"
  install_system_file "$source_root/.harness-kit/examples" \
    "$system_dir/examples" "examples/"
  install_system_file "$source_root/scripts/probe-agent-roster.py" \
    "$system_dir/scripts/probe-agent-roster.py" "scripts/probe-agent-roster.py"
  install_system_file "$source_root/scripts/record-delegation.py" \
    "$system_dir/scripts/record-delegation.py" "scripts/record-delegation.py"
  install_system_file "$source_root/scripts/summarize-delegations.py" \
    "$system_dir/scripts/summarize-delegations.py" "scripts/summarize-delegations.py"
  install_system_file "$source_root/scripts/lib/agent_roster.py" \
    "$system_dir/scripts/lib/agent_roster.py" "scripts/lib/agent_roster.py"
  echo
}

remove_source_skill_bridge_dir() {
  local dir="$1"
  local label="$2"

  [ -d "$dir" ] || return 0

  local entry real_target removed=0 has_foreign=0
  for entry in "$dir"/*; do
    [ -e "$entry" ] || [ -L "$entry" ] || continue
    if [ -L "$entry" ]; then
      real_target="$(python3 -c 'import os,sys; print(os.path.realpath(sys.argv[1]))' "$entry")"
      case "$real_target" in
        "$HARNESS_KIT/skills"/*)
          rm -f "$entry"
          removed=1
          ;;
        *)
          has_foreign=1
          ;;
      esac
    else
      has_foreign=1
    fi
  done

  if rmdir "$dir" 2>/dev/null; then
    [ "$removed" -eq 1 ] && ok "    removed stale source $label"
  elif [ "$removed" -eq 1 ]; then
    warn "    $label contains non-Harness Kit entries; removed only stale source symlinks"
  elif [ "$has_foreign" -eq 1 ]; then
    warn "    $label contains non-Harness Kit entries; leaving it alone"
  fi
}

cleanup_source_skill_bridges() {
  [ -n "$HARNESS_KIT" ] || return 0

  info "Cleaning source-repo skill bridges..."
  remove_source_skill_bridge_dir "$HARNESS_KIT/.codex/skills" ".codex/skills/"
  remove_source_skill_bridge_dir "$HARNESS_KIT/.claude/skills" ".claude/skills/"
  remove_source_skill_bridge_dir "$HARNESS_KIT/.pi/skills" ".pi/skills/"
  remove_source_skill_bridge_dir "$HARNESS_KIT/.agents/skills" ".agents/skills/"
  echo
}

discover_local() {
  local agent
  local skill
  GLOBAL_SKILLS=()
  GLOBAL_AGENTS=()

  for skill in "$HARNESS_KIT"/skills/*; do
    [ -d "$skill" ] || continue
    [ -f "$skill/SKILL.md" ] || continue
    GLOBAL_SKILLS+=("$(basename "$skill")")
  done

  for agent in "$HARNESS_KIT"/agents/*.md; do
    [ -f "$agent" ] || continue
    agent="$(basename "$agent" .md)"
    is_global_agent_allowed "$agent" || continue
    GLOBAL_AGENTS+=("$agent")
  done
}

download_archive() {
  local repo="$1"
  local archive="$2"
  local target_dir="$3"
  curl -sfL "https://github.com/$repo/archive/refs/heads/master.tar.gz" -o "$archive" \
    || return 1
  tar -xzf "$archive" -C "$target_dir" || return 1
}

discover_remote() {
  REMOTE_TMP="$(mktemp -d)"
  local archive="$REMOTE_TMP/harness-kit.tar.gz"
  download_archive "$REPO" "$archive" "$REMOTE_TMP" \
    || { err "Failed to download Harness Kit archive from $REPO"; exit 1; }

  local extracted
  extracted="$(find "$REMOTE_TMP" -maxdepth 1 -type d -name '*-master' | head -n 1)"
  REMOTE_HARNESS_KIT="$extracted"
  is_harness_kit_checkout "$REMOTE_HARNESS_KIT" \
    || { err "Downloaded archive is not a Harness Kit checkout"; exit 1; }

  local skill agent
  GLOBAL_SKILLS=()
  GLOBAL_AGENTS=()

  for skill in "$REMOTE_HARNESS_KIT"/skills/*; do
    [ -d "$skill" ] || continue
    [ -f "$skill/SKILL.md" ] || continue
    GLOBAL_SKILLS+=("$(basename "$skill")")
  done

  for agent in "$REMOTE_HARNESS_KIT"/agents/*.md; do
    [ -f "$agent" ] || continue
    agent="$(basename "$agent" .md)"
    is_global_agent_allowed "$agent" || continue
    GLOBAL_AGENTS+=("$agent")
  done
}

HARNESS_KIT="$(resolve_harness_kit_dir || true)"

if [ -n "$HARNESS_KIT" ]; then
  discover_local
else
  discover_remote
fi

if [ ${#GLOBAL_SKILLS[@]} -eq 0 ]; then
  err "No skills found"
  exit 1
fi

if [ ${#GLOBAL_AGENTS[@]} -eq 0 ]; then
  err "No agents found"
  exit 1
fi

link_local() {
  local harness="$1"        # e.g. "claude"
  local harness_dir="$2"    # e.g. "$HOME/.claude"
  local skills_dir="$harness_dir/skills"
  local agents_dir="$harness_dir/agents"

  info "  Linking skills..."
  # Per-entry symlinks make all first-party skills globally available while
  # preserving user-owned files in the harness skill dir.
  if [ -L "$skills_dir" ]; then
    local link_target
    link_target="$(readlink "$skills_dir" || true)"
    if [ -n "${HARNESS_KIT:-}" ] && [ "$link_target" = "$HARNESS_KIT/skills" ] || [ ! -e "$skills_dir" ]; then
      rm -f "$skills_dir"
    fi
  fi

  local skill src
  cleanup_symlinks_under_prefix "$skills_dir" "$HARNESS_KIT/skills" "${GLOBAL_SKILLS[@]}"
  mkdir -p "$skills_dir"
  for skill in "${GLOBAL_SKILLS[@]}"; do
    src="$HARNESS_KIT/skills/$skill"
    [ -d "$src" ] || { warn "    missing local skill: $skill"; continue; }
    ln -sfn "$src" "$skills_dir/$skill"
    ok "    $skill"
  done

  info "  Linking agents..."
  if prepare_agents_dir "$agents_dir" "$HARNESS_KIT/agents" "agents/"; then
    local agent src
    local agent_files=()
    for agent in "${GLOBAL_AGENTS[@]}"; do agent_files+=("$agent.md"); done
    cleanup_retired_agents "$agents_dir" "$HARNESS_KIT"
    cleanup_symlinks_under_prefix "$agents_dir" "$HARNESS_KIT/agents" "${agent_files[@]}"
    for agent in "${GLOBAL_AGENTS[@]}"; do
      src="$HARNESS_KIT/agents/$agent.md"
      [ -f "$src" ] || { warn "    missing local agent: $agent"; continue; }
      ln -sfn "$src" "$agents_dir/$agent.md"
      ok "    $agent"
    done
  fi

  # Link harness-specific configs if they exist
  local harness_config="$HARNESS_KIT/harnesses/$harness"
  if [ -d "$harness_config" ]; then
    info "  Linking harness config..."
    case "$harness" in
      claude)
        link_file_if_present "$HARNESS_KIT/harnesses/shared/AGENTS.md" "$harness_dir/CLAUDE.md" "CLAUDE.md (← shared AGENTS.md)"
        link_dir_entries_if_present "$harness_config/hooks" "$harness_dir/hooks" "hooks/"
        copy_claude_settings_if_present "$harness_config/settings.json" "$harness_dir/settings.json"
        remove_path_if_symlink_to_prefix "$harness_dir/.claude/settings.local.json" "$HARNESS_KIT" ".claude/settings.local.json"
        ;;
      codex)
        cleanup_symlinks_under_prefix "$harness_dir/config" "$harness_config" "config.toml"
        link_file_if_present "$harness_config/config.toml" "$harness_dir/config/config.toml" "config.toml"
        link_file_if_present "$HARNESS_KIT/harnesses/shared/AGENTS.md" "$harness_dir/AGENTS.md" "AGENTS.md (← shared)"
        ;;
      pi)
        link_file_if_present "$HARNESS_KIT/harnesses/shared/AGENTS.md" "$harness_dir/agent/AGENTS.md" "AGENTS.md (← shared)"
        remove_path_if_symlink_to_prefix "$harness_dir/agent/APPEND_SYSTEM.md" "$HARNESS_KIT" "agent/APPEND_SYSTEM.md"
        link_file_if_present "$harness_config/settings.json" "$harness_dir/settings.json" "settings.json"
        cleanup_symlinks_under_prefix "$harness_dir/prompts" "$HARNESS_KIT"
        remove_path_if_symlink_to_prefix "$harness_dir/persona.md" "$HARNESS_KIT" "persona.md"
        ;;
      antigravity-cli|antigravity-ide|antigravity)
        link_file_if_present "$HARNESS_KIT/harnesses/shared/AGENTS.md" "$harness_dir/AGENTS.md" "AGENTS.md (← shared)"
        ;;
    esac
  fi

  verify_no_broken_harness_kit_symlinks "$harness_dir" 4
}

# --- Remote mode: download from GitHub ---

download_skill() {
  local skills_dir="$1"
  local name="$2"
  local target="$skills_dir/$name"

  [ -n "$REMOTE_HARNESS_KIT" ] || { err "Remote checkout not available"; return 1; }
  [ -d "$REMOTE_HARNESS_KIT/skills/$name" ] || { err "Failed: missing skill $name"; return 1; }

  rm -rf "$target"
  mkdir -p "$(dirname "$target")"
  cp -R "$REMOTE_HARNESS_KIT/skills/$name" "$target"
  ok "  $name → $target"
}

download_agent() {
  local agents_dir="$1"
  local name="$2"

  [ -n "$REMOTE_HARNESS_KIT" ] || { err "Remote checkout not available"; return 1; }
  [ -f "$REMOTE_HARNESS_KIT/agents/$name.md" ] || { err "Failed: agent $name"; return 1; }

  mkdir -p "$agents_dir"
  cp "$REMOTE_HARNESS_KIT/agents/$name.md" "$agents_dir/$name.md"
  ok "  $name → $agents_dir/$name.md"
}

install_remote() {
  local skills_dir="$1"
  local agents_dir="$2"

  for skill in "${GLOBAL_SKILLS[@]}"; do
    download_skill "$skills_dir" "$skill"
  done

  info "  Installing agents..."
  prepare_agents_dir "$agents_dir" "$REMOTE_HARNESS_KIT/agents" "agents/" || return 0
  cleanup_retired_agents "$agents_dir" "$REMOTE_HARNESS_KIT"
  for agent in "${GLOBAL_AGENTS[@]}"; do
    download_agent "$agents_dir" "$agent"
  done
}

# --- Orchestration ---

info "Harness Kit Bootstrap"
if [ -n "$HARNESS_KIT" ]; then
  info "Local checkout detected: $HARNESS_KIT"
  info "Mode: symlink"
else
  info "No local checkout found."
  info "Mode: download from GitHub"
fi
echo

install_system_roster
cleanup_source_skill_bridges

installed=0

for harness in claude codex pi antigravity-cli antigravity-ide antigravity; do
  case "$harness" in
    antigravity-cli) harness_dir="$HOME/.gemini/antigravity-cli" ;;
    antigravity-ide) harness_dir="$HOME/.gemini/antigravity-ide" ;;
    antigravity)     harness_dir="$HOME/.gemini/antigravity" ;;
    *)               harness_dir="$HOME/.$harness" ;;
  esac

  # Detect harness
  detected=0
  if [ -d "$harness_dir" ]; then
    detected=1
  elif [ "$harness" = "antigravity-cli" ] && command -v agy &>/dev/null; then
    detected=1
  elif [ "$harness" != "antigravity-cli" ] && [ "$harness" != "antigravity-ide" ] && [ "$harness" != "antigravity" ] && command -v "$harness" &>/dev/null; then
    detected=1
  fi

  if [ "$detected" -eq 0 ]; then
    continue
  fi

  info "Detected: $harness"
  mkdir -p "$harness_dir"

  if [ -n "$HARNESS_KIT" ]; then
    link_local "$harness" "$harness_dir"
  else
    agents_dir="$harness_dir/agents"
    install_remote "$harness_dir/skills" "$agents_dir"
  fi

  installed=$((installed + 1))
  echo
done

if [ "$installed" -eq 0 ]; then
  warn "No agent harnesses detected."
  warn "Installing to ~/.claude/ as default."
  mkdir -p "$HOME/.claude"
  if [ -n "$HARNESS_KIT" ]; then
    link_local "claude" "$HOME/.claude"
  else
    install_remote "$HOME/.claude/skills" "$HOME/.claude/agents"
  fi
  installed=1
fi

# --- Git hooks: ensure core.hooksPath is set ---
if [ -n "$HARNESS_KIT" ] \
  && [ -d "$HARNESS_KIT/.githooks" ] \
  && command -v git >/dev/null 2>&1 \
  && git -C "$HARNESS_KIT" rev-parse --git-dir >/dev/null 2>&1; then
  current_hooks_path="$(git -C "$HARNESS_KIT" config core.hooksPath 2>/dev/null || true)"
  if [ "$current_hooks_path" != ".githooks" ]; then
    git -C "$HARNESS_KIT" config core.hooksPath .githooks
    info "Set core.hooksPath → .githooks"
  fi
fi

ok "Done. Installed to $installed harness(es)."
echo
info "Skills (${#GLOBAL_SKILLS[@]}): ${GLOBAL_SKILLS[*]}"
info "Agents (${#GLOBAL_AGENTS[@]}): ${GLOBAL_AGENTS[*]}"
info "All first-party skills are installed system-wide for each detected harness."
echo
if [ -n "$HARNESS_KIT" ]; then
  info "Mode: symlink (edits in $HARNESS_KIT propagate instantly)"
else
  info "Mode: downloaded from GitHub"
  info "For symlink mode, clone harness-kit and re-run."
fi
