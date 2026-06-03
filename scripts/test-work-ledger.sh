#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

store="$tmpdir/ledger.jsonl"

python3 scripts/work-ledger.py append \
  --store "$store" \
  --event-type phase_started \
  --work-id 058 \
  --parent-work-id flywheel-058 \
  --backlog 058 \
  --branch deliver/058-work-ledger-mission-control \
  --owning-skill deliver \
  --phase review \
  --evidence-ref ".harness-kit/traces/delegations.jsonl#abc" \
  --blocker "waiting for critic" \
  --spawned-agent "grok-build:critic" \
  --trace-ref ".harness-kit/traces/work-records.jsonl#trace-abc" \
  --next-action "address critic output" \
  --status active

python3 - "$store" <<'PY'
import json
import sys
from pathlib import Path

rows = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]
assert len(rows) == 1
row = rows[0]
assert row["schema_version"] == 1
assert row["record_type"] == "work-ledger-event"
assert row["event_type"] == "phase_started"
assert row["work_id"] == "058"
assert row["parent_work_id"] == "flywheel-058"
assert row["backlog_ref"] == "058"
assert row["branch"] == "deliver/058-work-ledger-mission-control"
assert row["owning_skill"] == "deliver"
assert row["phase"] == "review"
assert row["evidence_refs"] == [".harness-kit/traces/delegations.jsonl#abc"]
assert row["blockers"] == ["waiting for critic"]
assert row["spawned_agents"] == ["grok-build:critic"]
assert row["trace_refs"] == [".harness-kit/traces/work-records.jsonl#trace-abc"]
assert row["next_action"] == "address critic output"
assert row["status"] == "active"
assert row["created_at"].endswith("Z")
PY

summary="$(python3 scripts/work-ledger.py summary --store "$store")"
grep -q "branch: deliver/058-work-ledger-mission-control" <<<"$summary"
grep -q "backlog: 058" <<<"$summary"
grep -q "event_type: phase_started" <<<"$summary"
grep -q "phase: review" <<<"$summary"
grep -q "latest_evidence: .harness-kit/traces/delegations.jsonl#abc" <<<"$summary"
grep -q "blockers: waiting for critic" <<<"$summary"
grep -q "spawned_agents: grok-build:critic" <<<"$summary"
grep -q "trace_refs: .harness-kit/traces/work-records.jsonl#trace-abc" <<<"$summary"
grep -q "next_action: address critic output" <<<"$summary"

python3 scripts/work-ledger.py append \
  --store "$store" \
  --event-type phase_completed \
  --work-id 058 \
  --parent-work-id flywheel-058 \
  --backlog 058 \
  --branch deliver/058-work-ledger-mission-control \
  --owning-skill deliver \
  --phase done \
  --status completed \
  --next-action "none"

summary="$(python3 scripts/work-ledger.py summary --store "$store")"
grep -q "No active work ledger entries." <<<"$summary"

python3 scripts/work-ledger.py --self-test
