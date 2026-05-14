#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

out=$1
require() {
  local pattern="$1"
  local message="$2"
  if ! grep -Eqi "$pattern" "$out"; then
    echo "$message" >&2
    exit 1
  fi
}

require "qa" "missing QA surface"
require "demo" "missing demo surface"
require "monitor" "missing monitor surface"
require "CLI" "missing CLI-specific tailoring"
require "workflow" "missing workflow install bucket"
require "universal" "missing universal install bucket"
require "external" "missing external install bucket"
require "agent" "missing agent install bucket"
require "skills/\\.external/" "missing external cache path"
require "symlink|link" "missing symlink install"
require "sibling.{0,80}\\.spellbook|\\.spellbook.{0,80}sibling" \
  "missing sibling marker rule"
require "not inside|never inside|outside" "missing no-marker-inside-target rule"
require "category.{0,20}external" "missing external marker category"
require "source" "missing external marker source metadata"
require "alias" "missing external marker alias metadata"
require "target" "missing external marker target metadata"
require "(zero|no).{0,40}frontend externals" \
  "non-frontend repo must not pick frontend externals"
require "(re-resolve|refresh|repair).{0,80}symlink" \
  "missing kept-external symlink reconciliation"
require "remove.{0,80}symlink.{0,80}marker|remove.{0,80}marker.{0,80}symlink" \
  "missing dropped-external symlink + marker cleanup"
require "(never|not).{0,80}(modify|write|edit|touch).{0,80}(target|cache|upstream)|target content is never modified" \
  "missing upstream-target immutability"
require "\\.claude/skills/" "missing Claude bridge path"
require "\\.codex/skills/" "missing Codex bridge path"
require "\\.pi/skills/" "missing Pi bridge path"
require "relative.{0,80}symlink.{0,80}shared root|symlink.{0,80}back to the shared root" \
  "missing relative bridge-to-shared-root semantics"

if grep -Eqi "(^|[^[:alpha:]])(skip|no) (qa|demo|monitor)" "$out" \
  && ! grep -Eqi "does not skip (qa|demo|monitor)" "$out"; then
  echo "candidate skipped an always-core lifecycle skill" >&2
  exit 1
fi

if grep -Eqi "(installs|picks|adds).{0,20}frontend externals.*(for|in).*(CLI|library|non-frontend)" "$out" \
  && ! grep -Eqi "(zero|no) frontend externals.*(CLI|library|non-frontend)" "$out"; then
  echo "candidate installs frontend externals for non-frontend repos" >&2
  exit 1
fi

if grep -Eqi "vercel-|jakub-|emil-|leon-" "$out" \
  && ! grep -Eqi "(zero|no) frontend externals" "$out"; then
  echo "candidate names frontend external aliases for non-frontend repo" >&2
  exit 1
fi

echo "PASS: tailor lifecycle output names required external-install surfaces"
