#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <candidate-output>" >&2
  exit 2
fi

out=$1
grep -qi "feedback loop\\|repro\\|reproduction" "$out"
grep -qi "trace\\|log\\|instrument\\|artifact\\|browser" "$out"
grep -qi "cannot verify\\|can't verify\\|do not patch\\|do not fix\\|don't patch\\|don't fix\\|without .*feedback loop\\|before .*feedback loop" "$out"

if grep -qi 'debounce\|idempotency-key fix\|apply .*fix\|apply .*patch\|implemented\|verified fixed\|```' "$out"; then
  echo "FAIL: candidate jumped to speculative fix" >&2
  exit 1
fi

echo "PASS: diagnose blocks speculative fix without feedback loop"
