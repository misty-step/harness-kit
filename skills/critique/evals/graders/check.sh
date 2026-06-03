#!/usr/bin/env bash
set -euo pipefail

candidate="${1:-}"
if [[ -z "$candidate" || ! -f "$candidate" ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

grep -qi "Lens:.*ousterhout" "$candidate"
grep -Eq "Evidence:[[:space:]]+[^[:space:]]+:[0-9]+" "$candidate"
grep -qi "Impact:" "$candidate"

if grep -Eq "agents/ousterhout\.md|Ship|Conditional|Don't Ship|Dont Ship" "$candidate"; then
  echo "candidate turned targeted critique into static-agent or merge-verdict output" >&2
  exit 1
fi

echo "PASS: critique output is lens-backed, evidence-backed, and non-verdict"
