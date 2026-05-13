#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

out=$1
grep -qi "qa" "$out"
grep -qi "demo" "$out"
grep -qi "monitor" "$out"
grep -qi "CLI" "$out"
grep -qi "eval seed" "$out"

if grep -Eqi "skip (qa|demo|monitor)|no (qa|demo|monitor)" "$out"; then
  echo "candidate skipped an always-core lifecycle skill" >&2
  exit 1
fi

echo "PASS: tailor lifecycle output names required surfaces"
