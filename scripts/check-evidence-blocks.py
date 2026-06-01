#!/usr/bin/env python3
"""Validate committed Completion Gate and Acceptance Evidence templates."""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass
from pathlib import Path

BLOCK_HEADINGS = {"Completion Gate", "Acceptance Evidence"}

REQUIRED_FIELDS = {
    "Completion Gate": (
        "Evidence that proves it",
        "Exact command/path/route exercised",
        "Residual risk",
    ),
    "Acceptance Evidence": (
        "Acceptance source",
        "Evidence that proves it",
        "Exact command/path/route exercised",
        "Oracle / acceptance artifact hash",
        "Contract-change acknowledgment",
        "Residual risk",
    ),
}

PLACEHOLDER_VALUES = {
    "",
    "???",
    "fixme",
    "n/a",
    "na",
    "none",
    "placeholder",
    "tbd",
    "todo",
    "unknown",
}

FIELD_RE = re.compile(r"^\s*-\s+([^:\n]+):\s*(.*)$")
FENCE_RE = re.compile(r"^\s*```")
HEADING_RE = re.compile(r"^(#{2,3})\s+(.+?)\s*$")


@dataclass(frozen=True)
class EvidenceBlock:
    path: Path
    heading: str
    line: int
    fields: dict[str, tuple[str, int]]


def is_placeholder(value: str) -> bool:
    normalized = value.strip().strip(".").lower()
    if normalized in PLACEHOLDER_VALUES:
        return True
    if re.fullmatch(r"<[^>]*>", value.strip()):
        return True
    if re.fullmatch(r"\[[^\]]*\]", value.strip()):
        return True
    return False


def parse_evidence_blocks(path: Path, text: str) -> list[EvidenceBlock]:
    lines = text.splitlines()
    blocks: list[EvidenceBlock] = []
    index = 0
    in_fence = False

    while index < len(lines):
        stripped = lines[index].strip()
        if FENCE_RE.match(stripped):
            in_fence = not in_fence
            index += 1
            continue

        heading_match = HEADING_RE.match(stripped)
        heading = heading_match.group(2) if heading_match else ""
        if heading_match and heading in BLOCK_HEADINGS:
            start_line = index + 1
            start_level = len(heading_match.group(1))
            fields: dict[str, tuple[str, int]] = {}
            cursor = index + 1
            section_in_fence = in_fence

            while cursor < len(lines):
                current = lines[cursor].strip()
                if FENCE_RE.match(current):
                    section_in_fence = not section_in_fence
                    cursor += 1
                    continue
                current_heading = HEADING_RE.match(current)
                if (
                    not section_in_fence
                    and cursor != index
                    and current_heading
                    and len(current_heading.group(1)) <= start_level
                ):
                    break
                match = FIELD_RE.match(lines[cursor])
                if match:
                    field = " ".join(match.group(1).split())
                    fields[field] = (match.group(2).strip(), cursor + 1)
                cursor += 1

            blocks.append(EvidenceBlock(path, heading, start_line, fields))
            index = cursor
            in_fence = section_in_fence
            continue

        index += 1

    return blocks


def check_block(block: EvidenceBlock) -> list[str]:
    errors: list[str] = []
    required = REQUIRED_FIELDS[block.heading]
    for field in required:
        if field not in block.fields:
            errors.append(
                f"{block.path}:{block.line}: {block.heading} missing field "
                f"{field!r}"
            )

    for field, (value, line) in sorted(block.fields.items(), key=lambda item: item[1][1]):
        if is_placeholder(value):
            errors.append(
                f"{block.path}:{line}: {block.heading} field {field!r} has "
                "blank or placeholder-only evidence"
            )

    return errors


def iter_markdown_files(root: Path) -> list[Path]:
    if root.is_file():
        return [root]
    paths: list[Path] = []
    for path in sorted(root.rglob("*.md")):
        parts = set(path.parts)
        if ".external" in parts:
            continue
        paths.append(path)
    return paths


def check_paths(paths: list[Path]) -> list[str]:
    errors: list[str] = []
    for root in paths:
        for path in iter_markdown_files(root):
            try:
                text = path.read_text()
            except UnicodeDecodeError as error:
                errors.append(f"{path}: cannot read markdown as text: {error}")
                continue
            for block in parse_evidence_blocks(path, text):
                errors.extend(check_block(block))
    return errors


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate Completion Gate and Acceptance Evidence blocks."
    )
    parser.add_argument(
        "paths",
        nargs="*",
        type=Path,
        default=[Path("skills")],
        help="Markdown files or directories to scan.",
    )
    args = parser.parse_args()

    errors = check_paths(args.paths)
    if errors:
        print("Evidence block check failed:", file=sys.stderr)
        for error in errors:
            print(f"  {error}", file=sys.stderr)
        return 1

    print("Evidence blocks valid.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
