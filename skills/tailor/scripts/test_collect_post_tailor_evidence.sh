#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COLLECTOR="$SCRIPT_DIR/collect-post-tailor-evidence.py"
PASS=0
FAIL=0

assert_json() {
  local desc="$1" file="$2" expr="$3"
  if python3 - "$file" "$expr" <<'PY'
import json
import sys

path, expr = sys.argv[1], sys.argv[2]
with open(path, encoding="utf-8") as handle:
    data = json.load(handle)
if not eval(expr, {"data": data}):
    raise SystemExit(1)
PY
  then
    PASS=$((PASS + 1))
    echo "  PASS  $desc"
  else
    FAIL=$((FAIL + 1))
    echo "  FAIL  $desc"
  fi
}

setup_repo() {
  TEST_DIR="$(mktemp -d)"
  SPELLBOOK_DIR="$TEST_DIR/spellbook"
  REPO_DIR="$TEST_DIR/repo"
  mkdir -p "$SPELLBOOK_DIR/skills/"{trace,monitor,deliver,ship,groom,settle,implement,flywheel,qa,demo,code-review,refactor,ci,diagnose,research,shape,yeet} \
    "$SPELLBOOK_DIR/scripts/lib" \
    "$REPO_DIR/.agents/skills" \
    "$REPO_DIR/.claude/skills" \
    "$REPO_DIR/.codex/skills" \
    "$REPO_DIR/.pi/skills" \
    "$REPO_DIR/.claude/agents" \
    "$REPO_DIR/scripts/lib" \
    "$REPO_DIR/.spellbook"

  git -C "$REPO_DIR" init -q
  mkdir -p "$REPO_DIR/.empty-hooks"
  git -C "$REPO_DIR" config core.hooksPath .empty-hooks
  git -C "$REPO_DIR" config user.email test@example.com
  git -C "$REPO_DIR" config user.name "Tailor Evidence Test"

  for skill in trace monitor deliver ship groom settle implement flywheel qa demo code-review refactor ci diagnose research shape yeet; do
    cat > "$SPELLBOOK_DIR/skills/$skill/SKILL.md" <<DOC
---
name: $skill
description: Source $skill.
---

# $skill
DOC
  done
  printf 'source backlog\n' > "$SPELLBOOK_DIR/scripts/lib/backlog.sh"
  printf 'source verdicts\n' > "$SPELLBOOK_DIR/scripts/lib/verdicts.sh"
  cp "$SPELLBOOK_DIR/scripts/lib/backlog.sh" "$REPO_DIR/scripts/lib/backlog.sh"
  cp "$SPELLBOOK_DIR/scripts/lib/verdicts.sh" "$REPO_DIR/scripts/lib/verdicts.sh"

  for skill in monitor deliver ship groom settle implement flywheel qa demo code-review refactor ci diagnose research shape yeet; do
    mkdir -p "$REPO_DIR/.agents/skills/$skill"
    cp "$SPELLBOOK_DIR/skills/$skill/SKILL.md" "$REPO_DIR/.agents/skills/$skill/SKILL.md"
    cat > "$REPO_DIR/.agents/skills/$skill/.spellbook" <<DOC
source: $skill
installed-by: tailor
category: workflow
DOC
    ln -s "../../.agents/skills/$skill" "$REPO_DIR/.claude/skills/$skill"
    ln -s "../../.agents/skills/$skill" "$REPO_DIR/.codex/skills/$skill"
    ln -s "../../.agents/skills/$skill" "$REPO_DIR/.pi/skills/$skill"
  done

  local user_path
  user_path="/""Users/agent/repo"
  cat > "$REPO_DIR/.agents/skills/monitor/SKILL.md" <<DOC
---
name: monitor
description: Monitor.
---

# monitor

Use the deploy receipt after /deploy. See $user_path.
DOC

  cat > "$REPO_DIR/.agents/skills/deliver/SKILL.md" <<'DOC'
---
name: deliver
description: Deliver.
---

# deliver

Active work lives in backlog.d; closed work moves to _done with Closes-backlog.
Detector command: scripts/lib/backlog.sh detect.
DOC

  cat > "$REPO_DIR/.agents/skills/ship/SKILL.md" <<'DOC'
---
name: ship
description: Ship.
---

# ship

Archive backlog.d tickets into _done and write trace.final.
DOC

  cat > "$REPO_DIR/.spellbook/repo-brief.md" <<'DOC'
---
generated: 2026-05-21
---

# Brief

This repo has no deploy target.
The load-bearing gate is `dagger call check --source=.`
Open in `backlog.d/`: 060-tailor-post-install-acceptance-audit.md.
DOC

  cat > "$REPO_DIR/AGENTS.md" <<'DOC'
# AGENTS

## Stack & boundaries
CLI.

## Gate contract
The gate is `dagger call check --source=.`

## Lifecycle
backlog.d to _done.

## Known debt
069-runtime.md

## Harness index
ci, deliver, ship.

## Invariants
No deploy target.
DOC

  git -C "$REPO_DIR" add .
  git -C "$REPO_DIR" commit -q -m init
}

teardown_repo() {
  rm -rf "$TEST_DIR"
}

test_collects_stable_json_shape() {
  setup_repo
  local output evidence
  output="$(python3 "$COLLECTOR" "$REPO_DIR" --spellbook-root "$SPELLBOOK_DIR" --run-id test-run)"
  evidence="$REPO_DIR/$output"
  assert_json "writes evidence schema" "$evidence" "data['schema_version'] == 1 and data['run_id'] == 'test-run'"
  assert_json "records missing trace" "$evidence" "'trace' in data['always_install']['missing'] and data['always_install']['trace_present'] is False"
  assert_json "writes supporting artifacts" "$evidence" "all(key in data['artifacts'] for key in ['portable_paths', 'lifecycle', 'readlinks', 'byte_identity'])"
  teardown_repo
}

test_captures_broken_symlink_and_byte_identity() {
  setup_repo
  rm "$REPO_DIR/.codex/skills/monitor"
  ln -s "../../.agents/skills/missing-monitor" "$REPO_DIR/.codex/skills/monitor"
  local evidence
  python3 "$COLLECTOR" --repo "$REPO_DIR" --spellbook-root "$SPELLBOOK_DIR" --run-id broken >/tmp/tailor-evidence-path.txt
  evidence="$REPO_DIR/$(cat /tmp/tailor-evidence-path.txt)"
  assert_json "captures broken bridge" "$evidence" "any(e['name'] == 'monitor' and e['broken'] for b in data['bridges'] for e in b['entries'])"
  assert_json "captures byte-identical workflow" "$evidence" "'qa' in data['workflow_byte_identical']"
  teardown_repo
}

test_captures_lifecycle_and_portable_path_facts() {
  setup_repo
  printf '\nKnown debt: (unfiled)\n' >> "$REPO_DIR/AGENTS.md"
  local evidence
  python3 "$COLLECTOR" "$REPO_DIR" --spellbook-root "$SPELLBOOK_DIR" --run-id facts >/tmp/tailor-evidence-path.txt
  evidence="$REPO_DIR/$(cat /tmp/tailor-evidence-path.txt)"
  assert_json "captures portable path hits" "$evidence" "any('/Users/' in hit['text'] for hit in data['portable_path_hits'])"
  assert_json "captures deploy drift when brief says no deploy" "$evidence" "any('/deploy' in hit['text'] or 'deploy receipt' in hit['text'] for hit in data['stale_lifecycle_hits'])"
  assert_json "captures lifecycle fact table" "$evidence" "any('backlog.d' in hit['text'] for hit in data['lifecycle_facts']['deliver'])"
  assert_json "captures AGENTS debt hygiene" "$evidence" "len(data['agents_debt']['unfiled_hits']) == 1"
  teardown_repo
}

test_captures_concise_agents_facts() {
  setup_repo
  local evidence
  python3 "$COLLECTOR" "$REPO_DIR" --spellbook-root "$SPELLBOOK_DIR" --run-id agents >/tmp/tailor-evidence-path.txt
  evidence="$REPO_DIR/$(cat /tmp/tailor-evidence-path.txt)"
  assert_json "captures concise AGENTS sections" "$evidence" "data['agents_md']['missing_required_sections'] == [] and data['agents_md']['too_many_headings'] is False"
  assert_json "captures AGENTS size budget" "$evidence" "data['agents_md']['too_many_words'] is False"
  teardown_repo
}

test_collects_stable_json_shape
test_captures_broken_symlink_and_byte_identity
test_captures_lifecycle_and_portable_path_facts
test_captures_concise_agents_facts

if [ "$FAIL" -gt 0 ]; then
  echo "$FAIL failed, $PASS passed"
  exit 1
fi

echo "$PASS passed"
