"""Shared search primitives for Harness Kit embedding search.

Delegates embedding to gemini_embeddings (the single Gemini client).
A copy of this file and gemini_embeddings.py ships at
skills/focus/scripts/ for standalone use when deployed via /focus.
"""

from __future__ import annotations

import json
import math
from pathlib import Path

MODEL = "gemini-embedding-2-preview"


def cosine_similarity(a: list[float], b: list[float]) -> float:
    dot = sum(x * y for x, y in zip(a, b))
    mag_a = math.sqrt(sum(x * x for x in a))
    mag_b = math.sqrt(sum(x * x for x in b))
    if mag_a == 0 or mag_b == 0:
        return 0.0
    return dot / (mag_a * mag_b)


def embed_texts(texts: list[str], dims: int, task_type: str) -> list[list[float]]:
    """Thin adapter around gemini_embeddings — the single embedding client."""
    from gemini_embeddings import embed_texts as _embed

    return _embed(
        model=MODEL,
        texts=texts,
        output_dimensionality=dims,
        task_type=task_type,
        user_agent="harness-kit-search",
    )


def embed_query(text: str, dims: int) -> list[float]:
    """Embed a single query for retrieval."""
    return embed_texts([text], dims, "RETRIEVAL_QUERY")[0]


def synthesize_project_context(project_dir: Path) -> str:
    """Read project signals and synthesize a description for embedding."""
    parts: list[str] = []

    for name in ["CLAUDE.md", "README.md"]:
        f = project_dir / name
        if f.exists():
            parts.append(f.read_text(encoding="utf-8")[:2000])
            break

    pkg = project_dir / "package.json"
    if pkg.exists():
        try:
            data = json.loads(pkg.read_text(encoding="utf-8"))
            deps = list(data.get("dependencies", {}).keys())
            dev = list(data.get("devDependencies", {}).keys())
            if deps:
                parts.append(f"Dependencies: {', '.join(deps[:30])}")
            if dev:
                parts.append(f"Dev dependencies: {', '.join(dev[:20])}")
        except json.JSONDecodeError:
            pass

    for manifest, label in [
        ("go.mod", "Go module"),
        ("mix.exs", "Elixir project"),
        ("Cargo.toml", "Rust project"),
        ("requirements.txt", "Python deps"),
        ("pyproject.toml", "Python project"),
    ]:
        f = project_dir / manifest
        if f.exists():
            parts.append(f"{label}: {f.read_text(encoding='utf-8')[:1000]}")

    dirs = [
        d.name
        for d in sorted(project_dir.iterdir())
        if d.is_dir() and not d.name.startswith(".")
    ][:20]
    if dirs:
        parts.append(f"Directories: {', '.join(dirs)}")

    return "\n".join(parts) if parts else "General software project"
