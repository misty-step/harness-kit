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
# Run: curl -sL https://raw.githubusercontent.com/misty-step/spellbook/master/bootstrap.sh | bash

REPO="${HARNESS_KIT_REPO:-misty-step/harness-kit}"
LEGACY_REPO="misty-step/spellbook"
RAW="https://raw.githubusercontent.com/$REPO/master"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REMOTE_TMP=""
REMOTE_HARNESS_KIT=""

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

  if [ -n "${SPELLBOOK_DIR:-}" ] && is_harness_kit_checkout "$SPELLBOOK_DIR"; then
    printf '%s\n' "$SPELLBOOK_DIR"
    return 0
  fi

  local candidate
  if is_worktree_checkout "$SCRIPT_DIR"; then
    for candidate in \
      "$HOME/Development/harness-kit" \
      "$HOME/dev/harness-kit" \
      "$HOME/src/harness-kit" \
      "$HOME/code/harness-kit" \
      "$HOME/Development/spellbook" \
      "$HOME/dev/spellbook" \
      "$HOME/src/spellbook" \
      "$HOME/code/spellbook"
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
    "$HOME/code/harness-kit" \
    "$HOME/Development/spellbook" \
    "$HOME/dev/spellbook" \
    "$HOME/src/spellbook" \
    "$HOME/code/spellbook"
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
    expected+=("$(basename "$src")")
  done

  cleanup_symlinks_under_prefix "$dest_dir" "$src_dir" "${expected[@]}"

  mkdir -p "$dest_dir"
  for src in "$src_dir"/*; do
    [ -e "$src" ] || continue
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
  local system_dir="$HOME/.spellbook"

  info "Installing system roster..."
  install_system_file "$source_root/.spellbook/agents.yaml" \
    "$system_dir/agents.yaml" "agents.yaml"
  install_system_file "$source_root/.spellbook/examples" \
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
    GLOBAL_AGENTS+=("$(basename "$agent" .md)")
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
  local repo="$REPO"
  if ! download_archive "$repo" "$archive" "$REMOTE_TMP"; then
    warn "Failed to download $repo archive; trying legacy $LEGACY_REPO"
    repo="$LEGACY_REPO"
    download_archive "$repo" "$archive" "$REMOTE_TMP" \
      || { err "Failed to download Harness Kit archive"; exit 1; }
  fi

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
    GLOBAL_AGENTS+=("$(basename "$agent" .md)")
  done
}

HARNESS_KIT="$(resolve_harness_kit_dir || true)"
# Legacy alias used by existing downstream hook snippets and old user shells.
SPELLBOOK="$HARNESS_KIT"

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

link_parent_dir() {
  local src="$1"
  local dest="$2"
  local label="$3"

  if [ -L "$dest" ]; then
    local current
    current="$(readlink "$dest")"
    if [ "$current" = "$src" ]; then
      ok "    $label (already linked)"
      return 0
    fi
    # Stale symlink to different location — replace
    rm -f "$dest"
  elif [ -d "$dest" ]; then
    # Migrate from per-entry symlinks to parent symlink.
    # Remove Harness Kit-managed symlinks; warn about non-symlink entries.
    local has_non_symlink=0
    local entry target
    for entry in "$dest"/*; do
      [ -e "$entry" ] || [ -L "$entry" ] || continue
      if [ -L "$entry" ]; then
        target="$(readlink "$entry" || true)"
        case "$target" in
          "$src"/*|"$HARNESS_KIT"/*) rm -f "$entry" ;;
          *) has_non_symlink=1 ;;
        esac
      else
        has_non_symlink=1
      fi
    done
    if [ "$has_non_symlink" -eq 1 ]; then
      warn "    $label: non-Harness Kit entries exist, keeping per-entry links"
      return 1
    fi
    rmdir "$dest" 2>/dev/null || { warn "    $label: dir not empty after cleanup"; return 1; }
  fi

  ln -sfn "$src" "$dest"
  ok "    $label → $src"
}

link_local() {
  local harness="$1"        # e.g. "claude"
  local harness_dir="$2"    # e.g. "$HOME/.claude"
  local skills_dir="$harness_dir/skills"
  local agents_dir="$harness_dir/agents"

  info "  Linking skills..."
  # Per-entry symlinks make all first-party skills globally available while
  # preserving user-owned files in the harness skill dir.
  if [ -L "$skills_dir" ]; then
    rm -f "$skills_dir"
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
  if ! link_parent_dir "$HARNESS_KIT/agents" "$agents_dir" "agents/"; then
    # Fallback: per-agent symlinks
    local agent src
    local agent_files=()
    for agent in "${GLOBAL_AGENTS[@]}"; do agent_files+=("$agent.md"); done
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

for harness in claude codex pi; do
  harness_dir="$HOME/.$harness"

  # Detect harness
  if [ ! -d "$harness_dir" ] && ! command -v "$harness" &>/dev/null; then
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
if [ -n "$HARNESS_KIT" ] && [ -d "$HARNESS_KIT/.githooks" ]; then
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
