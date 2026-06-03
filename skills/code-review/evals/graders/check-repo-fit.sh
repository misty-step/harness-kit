#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

out=$1
grep -qi "structural\\|frontmatter\\|scaffold" "$out"
grep -qi "repo-fit\\|repo fit\\|live repo" "$out"
grep -q "python3 -m example_tool --help" "$out"
grep -qi "block\\|blocking\\|not ship\\|don't ship" "$out"

echo "PASS: code-review output blocks structurally valid but non-repo-fit work"
