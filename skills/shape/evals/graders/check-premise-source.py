#!/usr/bin/env python3
"""Validate /shape premise-source blocks for context packet fixtures."""

from __future__ import annotations

import argparse
from datetime import UTC, datetime
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
METADATA_START_RE = re.compile(r"^Voice Transcript Metadata:\s*$", re.IGNORECASE | re.MULTILINE)
METADATA_LINE_RE = re.compile(r"^-\s+([a-z_]+):\s*(.*)\s*$")
AUDIO_PATH_RE = re.compile(
    r"(?i)(?:raw_audio_path\s*:|(?:^|\s)\S+\.(?:wav|mp3|m4a|flac|aac|aiff|ogg)\b)"
)
VOICE_SOURCE_KINDS = {"voice", "raw_transcript"}
REQUIRED_VOICE_METADATA = [
    "source_kind",
    "source_hash",
    "transcript_model",
    "transcript_confidence",
    "audio_duration_seconds",
    "redaction_status",
    "redaction_tool",
    "created_at",
    "residual_risk",
]
REDACTION_STATUSES = {"redacted", "sanitized"}


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


def parse_metadata(block: str) -> dict[str, str] | None:
    marker = METADATA_START_RE.search(block)
    if not marker and not re.search(r"^-\s+source_kind:\s*(voice|raw_transcript)\s*$", block, re.I | re.M):
        return None

    lines = block[marker.end() :].splitlines() if marker else block.splitlines()
    metadata: dict[str, str] = {}
    for line in lines:
        if not line.strip():
            continue
        match = METADATA_LINE_RE.match(line.strip())
        if not match:
            if metadata:
                break
            continue
        key, value = match.group(1), match.group(2).strip()
        metadata[key] = value
    return metadata


def parse_created_at(value: str) -> datetime:
    try:
        parsed = datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise SystemExit("voice transcript metadata created_at must be ISO-8601") from exc
    if parsed.tzinfo is None:
        parsed = parsed.replace(tzinfo=UTC)
    return parsed.astimezone(UTC)


def validate_voice_metadata(block: str, expected_hash: str) -> None:
    if AUDIO_PATH_RE.search(block):
        raise SystemExit("voice transcript metadata must not retain raw audio paths")

    metadata = parse_metadata(block)
    if metadata is None:
        return

    missing = [key for key in REQUIRED_VOICE_METADATA if key not in metadata]
    if missing:
        raise SystemExit(f"voice transcript metadata missing field(s): {', '.join(missing)}")

    source_kind = metadata["source_kind"]
    if source_kind not in VOICE_SOURCE_KINDS:
        raise SystemExit("voice transcript metadata source_kind is invalid")
    source_hash = metadata["source_hash"]
    if not re.fullmatch(r"sha256:[0-9a-fA-F]{64}", source_hash):
        raise SystemExit("voice transcript metadata source_hash must be sha256:<64 hex>")
    if source_hash.split(":", 1)[1].lower() != expected_hash:
        raise SystemExit("voice transcript metadata source_hash must match Premise Source digest")

    if metadata["transcript_model"] == "" or metadata["transcript_confidence"] == "":
        raise SystemExit("voice transcript metadata unknowns must be explicit")
    confidence = metadata["transcript_confidence"]
    if confidence != "unknown":
        try:
            confidence_value = float(confidence)
        except ValueError as exc:
            raise SystemExit("voice transcript metadata transcript_confidence must be unknown or numeric") from exc
        if not 0 <= confidence_value <= 1:
            raise SystemExit("voice transcript metadata transcript_confidence must be between 0 and 1")

    duration = metadata["audio_duration_seconds"]
    if duration != "unknown":
        try:
            duration_value = float(duration)
        except ValueError as exc:
            raise SystemExit("voice transcript metadata audio_duration_seconds must be unknown or numeric") from exc
        if duration_value < 0:
            raise SystemExit("voice transcript metadata audio_duration_seconds must be non-negative")

    if metadata["redaction_status"] not in REDACTION_STATUSES:
        raise SystemExit("voice transcript metadata redaction_status is invalid")
    if len(metadata["redaction_tool"]) < 3:
        raise SystemExit("voice transcript metadata redaction_tool must be set")
    if len(metadata["residual_risk"]) < 12:
        raise SystemExit("voice transcript metadata residual_risk must be substantive")
    if parse_created_at(metadata["created_at"]) > datetime.now(UTC):
        raise SystemExit("voice transcript metadata created_at must not be in the future")


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
        validate_voice_metadata(block, expected_hash)
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
    validate_voice_metadata(block, expected_hash)


def self_test() -> None:
    cases = Path(__file__).resolve().parents[1] / "cases"
    valid = cases / "premise-source-valid.md"
    voice_valid = cases / "premise-source-voice-valid.md"
    voice_unknowns = cases / "premise-source-voice-unknowns.md"
    waiver = cases / "premise-source-waiver.md"
    small = cases / "premise-source-small-skip.md"
    invalid_cases = [
        cases / "premise-source-missing.md",
        cases / "premise-source-missing-path.md",
        cases / "premise-source-bad-hash.md",
        cases / "premise-source-raw-transcript.md",
        cases / "premise-source-voice-missing-hash.md",
        cases / "premise-source-voice-missing-unknowns.md",
        cases / "premise-source-voice-raw-audio-path.md",
    ]

    validate_packet(valid)
    validate_packet(voice_valid)
    validate_packet(voice_unknowns)
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
