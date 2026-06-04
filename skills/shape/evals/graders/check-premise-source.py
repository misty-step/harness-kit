#!/usr/bin/env python3
"""Validate /shape premise-source blocks for context packet fixtures."""

from __future__ import annotations

import argparse
import hashlib
import re
import sys
from pathlib import Path
from urllib.parse import urlparse


REPO_ROOT = Path(__file__).resolve().parents[4]
SHA_RE = re.compile(r"\bsha256:([0-9a-fA-F]{64})\b")
SOURCE_RE = re.compile(
    r"^Premise Source:\s+sha256:([0-9a-fA-F]{64})\s+(\S+)\s*$",
    re.IGNORECASE | re.MULTILINE,
)
WAIVER_RE = re.compile(r"^Premise Source Waiver:\s*(.+)$", re.IGNORECASE | re.MULTILINE)
RESIDUAL_RE = re.compile(r"^Residual risk:\s*(.+)$", re.IGNORECASE | re.MULTILINE)
RAW_TRANSCRIPT_RE = re.compile(
    r"(raw transcript|system prompt|developer prompt|tool output|bearer\s+[a-z0-9._-]+|api[_-]?key\s*[:=])",
    re.IGNORECASE,
)
ESTIMATE_RE = re.compile(r"^Estimate:\s*(\S+)\s*$", re.IGNORECASE | re.MULTILINE)


def read(path: Path) -> str:
    try:
        return path.read_text()
    except OSError as exc:
        raise SystemExit(f"cannot read {path}: {exc}") from exc


def section(text: str, heading: str) -> str | None:
    pattern = re.compile(
        rf"^##\s+{re.escape(heading)}\s*$([\s\S]*?)(?=^##\s+|\Z)",
        re.MULTILINE,
    )
    match = pattern.search(text)
    if not match:
        return None
    return match.group(1).strip()


def is_url(value: str) -> bool:
    parsed = urlparse(value)
    return parsed.scheme in {"http", "https"} and bool(parsed.netloc)


def resolve_source(value: str, packet_path: Path) -> Path:
    source = Path(value).expanduser()
    if source.is_absolute():
        return source
    candidates = [
        (REPO_ROOT / source).resolve(),
        (packet_path.parent / source).resolve(),
    ]
    for candidate in candidates:
        if candidate.exists():
            return candidate
    return candidates[0]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def requires_premise(text: str) -> bool:
    match = ESTIMATE_RE.search(text)
    if not match:
        return True
    estimate = match.group(1).strip().lower()
    return estimate not in {"xs", "s", "small", "trivial"}


def validate_packet(path: Path) -> None:
    text = read(path)
    block = section(text, "Premise Source")
    if block is None:
        if not requires_premise(text):
            return
        raise SystemExit("missing ## Premise Source section")

    waiver = WAIVER_RE.search(block)
    source = SOURCE_RE.search(block)
    if waiver:
        reason = waiver.group(1).strip()
        residual = RESIDUAL_RE.search(block)
        if len(reason) < 12:
            raise SystemExit("premise source waiver has no reason")
        if not residual or len(residual.group(1).strip()) < 12:
            raise SystemExit("premise source waiver missing residual risk")
        return

    if RAW_TRANSCRIPT_RE.search(block):
        raise SystemExit("premise source block appears to include raw transcript or secret-like text")

    if not source:
        raise SystemExit("missing Premise Source: sha256:<digest> <path-or-url>")

    expected_hash, source_ref = source.group(1).lower(), source.group(2)
    if not SHA_RE.search(source.group(0)):
        raise SystemExit("premise source digest must be sha256:<64 hex>")
    if is_url(source_ref):
        return

    source_path = resolve_source(source_ref, path)
    if not source_path.exists():
        raise SystemExit("premise source path does not exist")
    if not source_path.is_file():
        raise SystemExit("premise source path is not a file")
    actual_hash = sha256(source_path)
    if actual_hash != expected_hash:
        raise SystemExit(
            "premise source hash mismatch: "
            f"expected {expected_hash}, got {actual_hash}"
        )


def self_test() -> None:
    cases = Path(__file__).resolve().parents[1] / "cases"
    valid = cases / "premise-source-valid.md"
    waiver = cases / "premise-source-waiver.md"
    small = cases / "premise-source-small-skip.md"
    invalid_cases = [
        cases / "premise-source-missing.md",
        cases / "premise-source-missing-path.md",
        cases / "premise-source-bad-hash.md",
        cases / "premise-source-raw-transcript.md",
    ]

    validate_packet(valid)
    validate_packet(waiver)
    validate_packet(small)

    rejected = 0
    for invalid in invalid_cases:
        try:
            validate_packet(invalid)
        except SystemExit:
            rejected += 1
    if rejected != len(invalid_cases):
        raise SystemExit("self-test failed to reject all invalid premise-source fixtures")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=["validate", "self-test"])
    parser.add_argument("packet", nargs="?")
    args = parser.parse_args()

    if args.mode == "self-test":
        self_test()
        print("PASS: premise-source checker self-test")
        return 0

    if not args.packet:
        parser.error("packet is required for validate mode")
    validate_packet(Path(args.packet))
    print(f"PASS: premise source valid in {args.packet}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
