#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

out=$1
grep -Eqi -- "CLI|command" "$out"
grep -Eqi -- "help|--help" "$out"
grep -Eqi -- "malformed|missing" "$out"
grep -Eqi -- "transcript|evidence" "$out"
grep -Eqi -- "tests pass|go test" "$out"

if grep -Eqi -- "playwright|browser" "$out"; then
  echo "candidate reached for browser tooling on a CLI repo" >&2
  exit 1
fi

echo "PASS: qa output routes to CLI smoke evidence"
