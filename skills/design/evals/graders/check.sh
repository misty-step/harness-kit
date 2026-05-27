#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

out=$1
grep -qi "screenshot\\|rendered\\|artifact" "$out"
grep -qi "operational\\|on-call\\|operator\\|workbench" "$out"
grep -qi "hierarchy" "$out"
grep -qi "density\\|spacing" "$out"
grep -qi "typograph\\|heading\\|font" "$out"
grep -qi "focus state\\|focus ring\\|keyboard\\|icon-only\\|a11y" "$out"
grep -qi "Design Gate" "$out"

if grep -Eqi "install (a )?(framework|component library)|add (a )?(framework|component library)|new token engine|global token" "$out"; then
  echo "candidate over-scoped one-off design critique into framework/token work" >&2
  exit 1
fi

echo "PASS: design output critiques rendered dashboard without framework drift"
