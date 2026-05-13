#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

out=$1
grep -qi "unverified" "$out"
grep -qi "entrypoint\\|runtime path\\|executable path" "$out"
grep -qi "block\\|blocking\\|not ship\\|don't ship" "$out"
grep -qi "scripts/import-users.py" "$out"

echo "PASS: code-review output blocks unverified entrypoint"
