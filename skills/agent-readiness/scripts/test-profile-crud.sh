#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPT="$ROOT/skills/agent-readiness/scripts/profile-crud.py"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

future_date="$(python3 - <<'PY'
from datetime import UTC, datetime, timedelta
print((datetime.now(UTC) + timedelta(days=30)).date().isoformat())
PY
)"

profile="$TMP/agent-readiness.yaml"
python3 "$SCRIPT" --profile "$profile" create --repo-root "$ROOT"
python3 "$SCRIPT" --profile "$profile" validate
python3 "$SCRIPT" --profile "$profile" read | grep -q "Agent readiness profile"

python3 "$SCRIPT" --profile "$profile" update \
  --waiver-id "waiver-1" \
  --scope "coverage" \
  --reason "coverage command is tracked in a follow-up ticket" \
  --expires-on "$future_date" \
  --adr "not-required:temporary remediation waiver" \
  --readiness-state preserved
python3 "$SCRIPT" --profile "$profile" validate
python3 "$SCRIPT" --profile "$profile" delete --waiver-id "waiver-1"
python3 "$SCRIPT" --profile "$profile" validate

cat > "$TMP/expired.yaml" <<PY
version: 1
generated_at: "2026-06-02T00:00:00Z"
profile:
  repo_root: "$ROOT"
  detected_stack: ["python"]
  stack_feedback_strength: "strict"
gates:
  local: ["dagger call check --source=."]
  ci: ["dagger call check --source=."]
  coverage:
    command: ""
    threshold: ""
adr_policy:
  required_when: "hard to reverse"
  paths: ["docs/adr/"]
infrastructure:
  manageability: "unknown"
  surfaces: []
module_boundaries: []
mock_policy: "Mock only external boundaries."
observability:
  access: "unknown"
  signals: []
readiness_state: "unknown"
waivers:
  - id: "waiver-expired"
    scope: "tests"
    reason: "TBD"
    expires_on: "2020-01-01"
    adr: "n/a"
PY

if python3 "$SCRIPT" --profile "$TMP/expired.yaml" validate >/tmp/profile-crud-invalid.out 2>&1; then
  echo "expired placeholder waiver unexpectedly passed" >&2
  exit 1
fi
grep -Eq "non-placeholder|future" /tmp/profile-crud-invalid.out

echo "agent-readiness profile CRUD ok"
