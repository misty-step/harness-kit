#!/usr/bin/env bash
set -euo pipefail

if [[ $# -eq 1 ]]; then
  mode="rendered-critique"
  out=$1
elif [[ $# -eq 2 ]]; then
  mode=$1
  out=$2
else
  echo "usage: $0 [rendered-critique|scaffold-contract|token-only-critique] <candidate-output>" >&2
  exit 2
fi

case "$mode" in
  rendered-critique)
    grep -qi "screenshot\\|rendered\\|artifact" "$out"
    grep -qi "operational\\|on-call\\|operator\\|workbench" "$out"
    grep -qi "hierarchy" "$out"
    grep -qi "density\\|spacing" "$out"
    grep -qi "typograph\\|heading\\|font" "$out"
    grep -qi "focus state\\|focus ring\\|keyboard\\|icon-only\\|a11y" "$out"
    grep -qi "Design Gate" "$out"

    if grep -Eqi "install (a )?(framework|component library)|add (a )?(framework|component library)|new token engine|global token" "$out"; then
      echo "candidate over-scoped one-off design critique into framework/token work" >&2
      exit 1
    fi

    echo "PASS: design output critiques rendered dashboard without framework drift"
    ;;
  scaffold-contract)
    grep -qi "DESIGN.md" "$out"
    grep -qi "design-contract.md" "$out"
    for section in "Product Intent" "Audience and Context" "Brand Attributes" "Visual Language" "Layout and Density" "Components and Interaction" "Content Voice" "Accessibility and Responsiveness" "Evidence and Governance"; do
      grep -qi "$section" "$out"
    done
    grep -Eqi "Source.*Fact.*Provenance.*Confidence.*Use" "$out"
    grep -qi "observed" "$out"
    grep -qi "provided" "$out"
    grep -qi "inferred" "$out"
    grep -qi "keep" "$out"
    grep -qi "change" "$out"
    grep -Eqi "do-not-copy|do not copy" "$out"

    if grep -Eqi "brand is|brand should be|visual language is" "$out" \
      && ! grep -Eqi "observed|provided|inferred" "$out"; then
      echo "candidate invents design facts without provenance labels" >&2
      exit 1
    fi

    echo "PASS: design scaffold records repo-owned design facts with provenance"
    ;;
  token-only-critique)
    if ! grep -Eqi "screenshot|rendered|artifact|URL|route|unverified|cannot make a final design judgment|rendering is impossible" "$out"; then
      echo "candidate lacks rendered evidence or explicit unverified caveat" >&2
      exit 1
    fi

    if grep -Eqi "ready to ship|design succeeds|design passes|design is complete" "$out" \
      && ! grep -Eqi "screenshot|rendered|artifact|unverified|cannot make a final design judgment|rendering is impossible" "$out"; then
      echo "candidate claims design success from tokens/docs alone" >&2
      exit 1
    fi

    echo "PASS: token-only critique preserves rendered-evidence caveat"
    ;;
  *)
    echo "unknown design eval mode: $mode" >&2
    exit 2
    ;;
esac
