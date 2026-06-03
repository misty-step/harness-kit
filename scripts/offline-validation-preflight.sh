#!/usr/bin/env bash
# Manual preflight for Spellbook's git-native offline validation run.
set -euo pipefail

warn_count=0
fail_count=0

info() { printf 'INFO: %s\n' "$*"; }
ok() { printf 'OK: %s\n' "$*"; }
warn() { warn_count=$((warn_count + 1)); printf 'WARN: %s\n' "$*" >&2; }
fail() { fail_count=$((fail_count + 1)); printf 'FAIL: %s\n' "$*" >&2; }

has_cmd() {
  command -v "$1" >/dev/null 2>&1
}

repo_root() {
  git rev-parse --show-toplevel 2>/dev/null
}

check_file() {
  local path="$1" label="$2"
  if [ -f "$path" ]; then
    ok "$label: $path"
  else
    fail "$label missing: $path"
  fi
}

check_executable() {
  local path="$1" label="$2"
  if [ -x "$path" ]; then
    ok "$label executable: $path"
  else
    fail "$label missing or not executable: $path"
  fi
}

main() {
  local root branch evidence_dir
  if ! root="$(repo_root)"; then
    fail "not inside a git repository"
    printf '\nSummary: %s failure(s), %s warning(s)\n' "$fail_count" "$warn_count"
    exit 1
  fi
  cd "$root"

  branch="$(git rev-parse --abbrev-ref HEAD)"
  info "repo: $root"
  info "branch: $branch"

  check_file "scripts/lib/evidence.sh" "evidence helper"
  check_file "scripts/lib/verdicts.sh" "verdict helper"
  check_executable ".githooks/pre-merge-commit" "pre-merge hook"

  if [ -f ".gitattributes" ] \
    && grep -Fq '.evidence/**/*.png filter=lfs' .gitattributes \
    && grep -Fq '.evidence/**/*.webm filter=lfs' .gitattributes; then
    ok ".evidence binary LFS attributes present"
  else
    fail ".gitattributes does not contain required .evidence LFS rules"
  fi

  if [ -f "scripts/lib/evidence.sh" ]; then
    # shellcheck source=scripts/lib/evidence.sh
    source scripts/lib/evidence.sh
    evidence_dir="$(evidence_dir)"
    ok "evidence directory would be: $evidence_dir"
  fi

  if has_cmd dagger; then
    ok "dagger CLI found: $(command -v dagger)"
  else
    fail "dagger CLI not found; pre-merge gate cannot run offline"
  fi

  if has_cmd docker && docker info >/dev/null 2>&1; then
    ok "docker runtime is reachable"
    if docker image ls --format '{{.Repository}}:{{.Tag}}' \
      | grep -Eq '^registry\.dagger\.io/engine:'; then
      ok "Dagger engine image appears cached locally"
    else
      warn "no registry.dagger.io/engine:<version> image visible in docker cache"
    fi
  else
    warn "docker runtime not reachable; verify your Dagger local runtime before airplane mode"
  fi

  if has_cmd git-lfs; then
    ok "git-lfs command found"
  elif git lfs version >/dev/null 2>&1; then
    ok "git lfs is available"
  else
    warn "git lfs not available; binary evidence may commit as full files or pointers may not hydrate"
  fi

  printf '\nSummary: %s failure(s), %s warning(s)\n' "$fail_count" "$warn_count"
  [ "$fail_count" -eq 0 ]
}

main "$@"
