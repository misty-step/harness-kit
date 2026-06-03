#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
trace_script="$script_dir/trace_record.py"
store="$tmpdir/work-records.jsonl"

python3 "$trace_script" append \
  --store "$store" \
  --backlog 056 \
  --branch deliver/056-agent-session-trace-lifecycle \
  --commit abc1234 \
  --reviewer-verdict-ref ".harness-kit/traces/delegations.jsonl#abc" \
  --qa-ref ".evidence/qa/056.md" \
  --demo-ref ".evidence/demo/056.gif" \
  --transcript-ref ".harness-kit/traces/transcripts/056.md" \
  --shipped-ref "master@deadbeef" \
  --metadata source=self-test

python3 - "$store" <<'PY'
import json
import sys
from pathlib import Path

rows = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert len(rows) == 1
row = rows[0]
assert row["schema_version"] == 1
assert row["record_type"] == "agent-session-trace"
assert row["backlog_ref"] == "056"
assert row["branch"] == "deliver/056-agent-session-trace-lifecycle"
assert row["commits"] == ["abc1234"]
assert row["reviewer_verdict_refs"] == [".harness-kit/traces/delegations.jsonl#abc"]
assert row["qa_refs"] == [".evidence/qa/056.md"]
assert row["demo_refs"] == [".evidence/demo/056.gif"]
assert row["transcript_refs"] == [".harness-kit/traces/transcripts/056.md"]
assert row["shipped_ref"] == "master@deadbeef"
assert row["metadata"] == {"source": "self-test"}
assert row["trace_id"].startswith("trace-")
assert row["created_at"].endswith("Z")
PY

if python3 "$trace_script" append \
  --store "$store" \
  --backlog 056 \
  --branch deliver/056-agent-session-trace-lifecycle \
  --waiver-reason "No safe transcript export available." \
  --metadata "API_TOKEN=leak" 2>"$tmpdir/secret.err"; then
  echo "expected secret-like metadata to fail" >&2
  exit 1
fi

grep -q "secret-like" "$tmpdir/secret.err"

python3 "$trace_script" --self-test
