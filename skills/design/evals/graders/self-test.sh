#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

grader="$(dirname "$0")/check.sh"

cat >"$tmpdir/scaffold-pass.md" <<'EOF'
Generated files:

- DESIGN.md
- design-contract.md

DESIGN.md sections:

1. Product Intent
2. Audience and Context
3. Brand Attributes
4. Visual Language
5. Layout and Density
6. Components and Interaction
7. Content Voice
8. Accessibility and Responsiveness
9. Evidence and Governance

design-contract.md:

| Source | Fact | Provenance | Confidence | Use | Evidence / Notes |
|---|---|---|---|---|---|
| app screenshot | Dense operator dashboard | observed | high | keep | Rendered dashboard artifact |
| user note | Avoid playful illustration | provided | medium | change | Stakeholder brief |
| competitor site | Hero motion direction | inferred | low | do-not-copy | Reference brand only |
EOF

cat >"$tmpdir/scaffold-fail.md" <<'EOF'
The app brand is premium fintech with cinematic depth.

Create DESIGN.md with these sections:

1. Product Intent
2. Audience and Context
3. Brand Attributes
4. Visual Language
5. Layout and Density
6. Components and Interaction
7. Content Voice
8. Accessibility and Responsiveness
9. Evidence and Governance

Use the competitor reference directly.
EOF

cat >"$tmpdir/token-pass.md" <<'EOF'
Token file inspected: src/theme.ts.

Unverified caveat: I cannot make a final design judgment because no screenshot,
rendered route, URL, or artifact was available. The token layer suggests a
coherent spacing scale, but rendered evidence is still required.
EOF

cat >"$tmpdir/token-fail.md" <<'EOF'
The token document has colors, spacing, and component names. The design is
complete and ready to ship.
EOF

bash "$grader" scaffold-contract "$tmpdir/scaffold-pass.md"
if bash "$grader" scaffold-contract "$tmpdir/scaffold-fail.md" >/tmp/design-eval-fail.log 2>&1; then
  echo "expected scaffold output without provenance/do-not-copy to fail" >&2
  exit 1
fi

bash "$grader" token-only-critique "$tmpdir/token-pass.md"
if bash "$grader" token-only-critique "$tmpdir/token-fail.md" >/tmp/design-eval-fail.log 2>&1; then
  echo "expected token-only success claim to fail" >&2
  exit 1
fi

echo "PASS: design eval self-test"
