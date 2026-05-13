#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

out=$1
grep -qi "CLI\\|command" "$out"
grep -qi "help\\|--help" "$out"
grep -qi "malformed\\|missing" "$out"
grep -qi "transcript\\|evidence" "$out"
grep -qi "tests pass\\|go test" "$out"

if grep -Eqi "playwright|browser" "$out"; then
  echo "candidate reached for browser tooling on a CLI repo" >&2
  exit 1
fi

echo "PASS: qa output routes to CLI smoke evidence"
