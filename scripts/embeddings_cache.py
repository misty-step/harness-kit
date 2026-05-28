#!/usr/bin/env python3
"""Shared cache helpers for Harness Kit embeddings."""

from __future__ import annotations

import hashlib
import os
import time
from pathlib import Path

FORMAT_VERSION = 1
DEFAULT_TTL_SECONDS = 86400


def harness_kit_cache_root() -> Path:
    override = os.environ.get("HARNESS_KIT_CACHE_DIR")
    if override:
        return Path(override).expanduser()

    codex_home = os.environ.get("CODEX_HOME")
    if codex_home:
        return Path(codex_home).expanduser() / "cache" / "harness-kit"

    xdg_cache = os.environ.get("XDG_CACHE_HOME")
    if xdg_cache:
        return Path(xdg_cache).expanduser() / "harness-kit"

    return Path.home() / ".cache" / "harness-kit"


def discovery_cache_paths() -> tuple[Path, Path]:
    cache_dir = harness_kit_cache_root() / "discovery"
    return cache_dir / "embeddings.json", cache_dir / "embeddings-meta.json"


def ttl_seconds() -> int:
    raw = os.environ.get("HARNESS_KIT_EMBEDDINGS_TTL_SECONDS")
    if not raw:
        return DEFAULT_TTL_SECONDS
    try:
        return int(raw)
    except ValueError:
        return DEFAULT_TTL_SECONDS


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_text(text: str) -> str:
    return sha256_bytes(text.encode("utf-8"))


def repo_hashes(repo_root: Path) -> dict[str, str]:
    return {
        "index_sha256": sha256_bytes((repo_root / "index.yaml").read_bytes()),
        "registry_sha256": sha256_bytes((repo_root / "registry.yaml").read_bytes()),
    }


def metadata_matches(
    metadata: dict,
    *,
    model: str,
    dimensions: int,
    index_sha256: str,
    registry_sha256: str,
) -> bool:
    return (
        metadata.get("format_version") == FORMAT_VERSION
        and metadata.get("model") == model
        and metadata.get("dimensions") == dimensions
        and metadata.get("index_sha256") == index_sha256
        and metadata.get("registry_sha256") == registry_sha256
    )


def is_stale(path: Path, *, now: float | None = None) -> bool:
    if not path.exists():
        return True
    if now is None:
        now = time.time()
    return now - path.stat().st_mtime > ttl_seconds()
