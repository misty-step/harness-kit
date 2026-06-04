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
python3 "$SCRIPT" --profile "$profile" read | grep -q "state_surfaces: 0"

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
state_surfaces: []
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

cat > "$TMP/agent-accessible-state.yaml" <<PY
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
  manageability: "mixed"
  surfaces: ["cms export"]
module_boundaries: []
mock_policy: "Mock only external boundaries."
observability:
  access: "logs"
  signals: ["profile validation"]
state_surfaces:
  - name: "Readiness profile contract"
    system_of_record: "Agent readiness skill"
    agent_access: "skill"
    source_path: "skills/agent-readiness/SKILL.md"
    verification_command: "test -f skills/agent-readiness/SKILL.md"
    waiver: ""
    waiver_expires: ""
readiness_state: "preserved"
waivers: []
PY
python3 "$SCRIPT" --profile "$TMP/agent-accessible-state.yaml" validate

cat > "$TMP/ui-only-state.yaml" <<PY
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
  manageability: "human_only"
  surfaces: ["admin ui"]
module_boundaries: []
mock_policy: "Mock only external boundaries."
observability:
  access: "unknown"
  signals: []
state_surfaces:
  - name: "Launch flags"
    system_of_record: "Admin console"
    agent_access: "admin-ui-only"
    source_path: ""
    verification_command: ""
    waiver: ""
    waiver_expires: ""
readiness_state: "unknown"
waivers: []
PY
if python3 "$SCRIPT" --profile "$TMP/ui-only-state.yaml" validate >/tmp/profile-crud-ui-only.out 2>&1; then
  echo "admin-ui-only state surface unexpectedly passed without waiver" >&2
  exit 1
fi
grep -Eq "requires a non-placeholder waiver" /tmp/profile-crud-ui-only.out

for hidden_access in cms-only unknown; do
  cat > "$TMP/${hidden_access}-state.yaml" <<PY
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
  manageability: "human_only"
  surfaces: ["external state"]
module_boundaries: []
mock_policy: "Mock only external boundaries."
observability:
  access: "unknown"
  signals: []
state_surfaces:
  - name: "External truth"
    system_of_record: "External system"
    agent_access: "$hidden_access"
    source_path: ""
    verification_command: ""
    waiver: ""
    waiver_expires: ""
readiness_state: "unknown"
waivers: []
PY
  if python3 "$SCRIPT" --profile "$TMP/${hidden_access}-state.yaml" validate >/tmp/profile-crud-${hidden_access}.out 2>&1; then
    echo "$hidden_access state surface unexpectedly passed without waiver" >&2
    exit 1
  fi
  grep -Eq "requires a non-placeholder waiver" /tmp/profile-crud-${hidden_access}.out
done

cat > "$TMP/missing-verification-command-state.yaml" <<PY
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
  manageability: "mixed"
  surfaces: ["api export"]
module_boundaries: []
mock_policy: "Mock only external boundaries."
observability:
  access: "logs"
  signals: ["profile validation"]
state_surfaces:
  - name: "Billing catalog"
    system_of_record: "Billing API"
    agent_access: "api"
    source_path: "https://billing.example.test/catalog"
    verification_command: ""
    waiver: ""
    waiver_expires: ""
readiness_state: "preserved"
waivers: []
PY
if python3 "$SCRIPT" --profile "$TMP/missing-verification-command-state.yaml" validate >/tmp/profile-crud-missing-command.out 2>&1; then
  echo "agent-accessible state surface unexpectedly passed without verification command" >&2
  exit 1
fi
grep -Eq "verification_command must be set" /tmp/profile-crud-missing-command.out

cat > "$TMP/prose-verification-command-state.yaml" <<PY
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
  manageability: "mixed"
  surfaces: ["skill export"]
module_boundaries: []
mock_policy: "Mock only external boundaries."
observability:
  access: "logs"
  signals: ["profile validation"]
state_surfaces:
  - name: "Content export"
    system_of_record: "Content API"
    agent_access: "skill"
    source_path: "skills/content-export/SKILL.md"
    verification_command: "see README"
    waiver: ""
    waiver_expires: ""
readiness_state: "preserved"
waivers: []
PY
if python3 "$SCRIPT" --profile "$TMP/prose-verification-command-state.yaml" validate >/tmp/profile-crud-prose-command.out 2>&1; then
  echo "agent-accessible state surface unexpectedly passed with prose verification command" >&2
  exit 1
fi
grep -Eq "verification_command must be command-shaped" /tmp/profile-crud-prose-command.out

cat > "$TMP/invalid-agent-access-state.yaml" <<PY
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
  manageability: "mixed"
  surfaces: ["documented export"]
module_boundaries: []
mock_policy: "Mock only external boundaries."
observability:
  access: "logs"
  signals: ["profile validation"]
state_surfaces:
  - name: "Documented truth"
    system_of_record: "Wiki"
    agent_access: "documented"
    source_path: "docs/export.md"
    verification_command: "python3 --version"
    waiver: ""
    waiver_expires: ""
readiness_state: "preserved"
waivers: []
PY
if python3 "$SCRIPT" --profile "$TMP/invalid-agent-access-state.yaml" validate >/tmp/profile-crud-invalid-access.out 2>&1; then
  echo "state surface unexpectedly passed with invalid agent_access" >&2
  exit 1
fi
grep -Eq "agent_access is invalid" /tmp/profile-crud-invalid-access.out

cat > "$TMP/ui-only-waived-state.yaml" <<PY
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
  manageability: "human_only"
  surfaces: ["admin ui"]
module_boundaries: []
mock_policy: "Mock only external boundaries."
observability:
  access: "unknown"
  signals: []
state_surfaces:
  - name: "Launch flags"
    system_of_record: "Admin console"
    agent_access: "admin-ui-only"
    source_path: ""
    verification_command: ""
    waiver: "Temporary UI-only launch flag ownership; owner will expose CLI export."
    waiver_expires: "$future_date"
readiness_state: "preserved"
waivers: []
PY
python3 "$SCRIPT" --profile "$TMP/ui-only-waived-state.yaml" validate

echo "agent-readiness profile CRUD ok"
