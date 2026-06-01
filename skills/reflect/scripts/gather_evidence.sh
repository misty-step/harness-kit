#!/usr/bin/env bash
set -euo pipefail

# Gather evidence for /reflect session retrospective.
# Usage: gather_evidence.sh [N]  (default: 10 commits)

N=${1:-10}

echo "## Recent Commits"
git log --oneline -"$N" 2>/dev/null || echo "(not in a git repo)"

echo ""
echo "## Changed Files"
git diff --stat "HEAD~$N" 2>/dev/null || echo "(no git history)"

echo ""
echo "## Uncommitted"
git status --short 2>/dev/null || echo "(not in a git repo)"

echo ""
echo "## Environment Hints"
echo "### .env files (existence only, no values)"
find . -maxdepth 2 -name '.env*' -not -path '*/node_modules/*' 2>/dev/null | head -10

echo ""
echo "### Dev server config"
grep -l -- 'dev.*server\|"dev"' package.json */package.json 2>/dev/null | head -5
