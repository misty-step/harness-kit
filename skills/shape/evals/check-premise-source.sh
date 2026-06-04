#!/usr/bin/env bash
set -euo pipefail

python3 "$(dirname "$0")/graders/check-premise-source.py" self-test
