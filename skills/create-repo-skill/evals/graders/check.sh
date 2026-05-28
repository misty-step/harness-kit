#!/usr/bin/env bash
set -euo pipefail

target="${1:-}"
if [ -z "$target" ] || [ ! -f "$target" ]; then
  echo "usage: $0 path/to/generated/SKILL.md" >&2
  exit 2
fi

required=(
  "persona"
  "value"
  "evidence"
  "report"
  "eval"
  ".agents/skills"
  "Completion Gate"
  "Residual"
)

for phrase in "${required[@]}"; do
  if ! grep -qi -- "$phrase" "$target"; then
    echo "missing required phrase: $phrase" >&2
    exit 1
  fi
done

if grep -Eq "TODO|\\[fill in\\]|your-app" "$target"; then
  echo "placeholder text remains" >&2
  exit 1
fi

echo "generated repo skill shape looks concrete"
