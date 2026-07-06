#!/usr/bin/env bash
set -euo pipefail

# Harness Kit Bootstrap
#
# Durable bootstrap behavior lives in the Rust CLI. This file is the
# curl-compatible launcher boundary for fresh machines.
#
# Run: curl -sL https://raw.githubusercontent.com/misty-step/harness-kit/master/bootstrap.sh | bash

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# STAND-DOWN GUARD (roster-926 cutover, 2026-07-07): once roster owns this
# machine's harness config, harness-kit bootstrap must not re-link catalogs
# back to harness-kit — the two sync systems fight and sessions get a mixed
# catalog. Roster ownership is detected by its sync manifest. Override only
# for an explicit rollback: ROSTER_ROLLBACK=1 ./bootstrap.sh
if [ -f "$HOME/.roster/orchestrator/manifest.json" ] && [ "${ROSTER_ROLLBACK:-0}" != "1" ]; then
  printf '%s\n' "harness-kit bootstrap: standing down — this machine is roster-managed." \
    "Use 'roster sync' (misty-step/roster). To force harness-kit anyway: ROSTER_ROLLBACK=1 ./bootstrap.sh" >&2
  exit 0
fi

require_cargo() {
  if ! command -v cargo >/dev/null 2>&1; then
    printf '%s\n' "cargo is required for the Rust Harness Kit bootstrap." >&2
    exit 1
  fi
}

if command -v harness-kit-checks >/dev/null 2>&1; then
  exec harness-kit-checks bootstrap "$@"
fi

if [ -d "$SCRIPT_DIR/crates/harness-kit-checks" ]; then
  require_cargo
  exec cargo run --quiet --locked -p harness-kit-checks -- bootstrap "$@"
fi

if [ -n "${HARNESS_KIT_DIR:-}" ] && [ -d "$HARNESS_KIT_DIR/crates/harness-kit-checks" ]; then
  require_cargo
  cd "$HARNESS_KIT_DIR"
  exec cargo run --quiet --locked -p harness-kit-checks -- bootstrap "$@"
fi

require_cargo

REPO="${HARNESS_KIT_REPO:-misty-step/harness-kit}"
TMP="$(mktemp -d)"
cleanup() { rm -rf "$TMP"; }
trap cleanup EXIT

curl -sfL "https://github.com/$REPO/archive/refs/heads/master.tar.gz" -o "$TMP/harness-kit.tar.gz"
tar -xzf "$TMP/harness-kit.tar.gz" -C "$TMP"
CHECKOUT="$(find "$TMP" -maxdepth 1 -type d -name '*-master' | head -n 1)"
if [ -z "$CHECKOUT" ]; then
  printf '%s\n' "downloaded archive did not contain a *-master directory" >&2
  exit 1
fi

cd "$CHECKOUT"
exec cargo run --quiet --locked -p harness-kit-checks -- bootstrap "$@"
