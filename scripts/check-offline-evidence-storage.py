#!/usr/bin/env python3
"""Validate the git-native offline evidence storage contract."""

from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]

REQUIRED_ATTRS = {
    ".evidence/**/*.png filter=lfs diff=lfs merge=lfs -text",
    ".evidence/**/*.gif filter=lfs diff=lfs merge=lfs -text",
    ".evidence/**/*.webm filter=lfs diff=lfs merge=lfs -text",
}

NO_TMP_FILES = [
    "skills/qa/SKILL.md",
    "skills/qa/references/evidence-capture.md",
    "skills/qa/references/scaffold.md",
    "skills/demo/SKILL.md",
    "skills/demo/references/pr-evidence-upload.md",
    "skills/demo/references/remotion.md",
    "skills/demo/references/tts-narration.md",
    "skills/demo/references/scaffold.md",
    "skills/deliver/references/evidence.md",
    "skills/deliver/references/receipt.md",
]


def read(path: str) -> str:
    return (ROOT / path).read_text()


def require(condition: bool, message: str, errors: list[str]) -> None:
    if not condition:
        errors.append(message)


def main() -> int:
    errors: list[str] = []

    attrs_path = ROOT / ".gitattributes"
    require(attrs_path.exists(), ".gitattributes is missing", errors)
    if attrs_path.exists():
        attrs = {
            line.strip()
            for line in attrs_path.read_text().splitlines()
            if line.strip() and not line.startswith("#")
        }
        missing = sorted(REQUIRED_ATTRS - attrs)
        require(not missing, f".gitattributes missing LFS rules: {missing}", errors)

    ignored = None
    gitignore = ROOT / ".gitignore"
    if gitignore.exists():
        ignored = re.search(r"^\s*\.evidence/?\s*$", gitignore.read_text(), re.M)
    require(not ignored, ".gitignore must not ignore .evidence/", errors)

    for rel in NO_TMP_FILES:
        text = read(rel)
        for bad in ("/tmp/qa", "/tmp/demo-slug", "/tmp/demo-evidence"):
            require(bad not in text, f"{rel} still references {bad}", errors)

    deliver_evidence = read("skills/deliver/references/evidence.md")
    require(
        "NOT\nversion-controlled" not in deliver_evidence,
        "deliver evidence doctrine still says evidence is not version-controlled",
        errors,
    )
    require(
        ".evidence/<branch>/<date>/" in deliver_evidence,
        "deliver evidence doctrine must name .evidence/<branch>/<date>/",
        errors,
    )

    demo = read("skills/demo/SKILL.md")
    require(
        "Draft GitHub release (`gh release create --draft`) | Optional mirror" in demo,
        "demo surfaces table must make draft releases optional mirrors",
        errors,
    )

    pr_upload = read("skills/demo/references/pr-evidence-upload.md")
    require(
        "HARNESS_EVIDENCE_GITHUB=1" in pr_upload,
        "PR evidence upload reference must gate GitHub uploads behind HARNESS_EVIDENCE_GITHUB=1",
        errors,
    )

    code_review = read("skills/code-review/SKILL.md")
    require(
        ".evidence/<branch>/<date>/review-synthesis.md" in code_review
        and ".evidence/<branch>/<date>/verdict.json" in code_review,
        "code-review must write synthesis and verdict into .evidence",
        errors,
    )

    ship = read("skills/ship/SKILL.md")
    require(
        "QA-Evidence:" in ship and "evidence_trailer" in ship,
        "ship must preserve QA-Evidence trailers from non-empty .evidence dirs",
        errors,
    )

    helper = read("scripts/lib/evidence.sh")
    require("evidence_dir_create" in helper, "evidence helper missing evidence_dir_create", errors)
    require("evidence_trailer" in helper, "evidence helper missing evidence_trailer", errors)

    if errors:
        print("Offline evidence storage contract failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1

    print("offline evidence storage contract valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
