#!/usr/bin/env python3
"""Freshness oracle for /document: flag committed doc pages whose covered source
changed since the page was generated.

Driver for the freshness falsifier in references/provenance-and-freshness.md.
Stdlib only — no PyYAML dependency. Parses the minimal front-matter this skill
writes (`generated-at-sha` + a `covers:` block of `- glob` lines).

Usage:
    python3 freshness.py [docs_dir]      # default docs_dir: docs

Exit codes:
    0  all pages fresh (or no stamped pages found)
    1  one or more pages stale
    2  not a git repo / git error
"""

from __future__ import annotations

import fnmatch
import subprocess
import sys
from pathlib import Path


def parse_frontmatter(text: str) -> dict | None:
    """Return {'generated-at-sha': str, 'covers': [glob, ...]} or None.

    Deliberately tiny: handles the front-matter shape templates/page.md emits,
    not arbitrary YAML.
    """
    if not text.startswith("---"):
        return None
    end = text.find("\n---", 3)
    if end == -1:
        return None
    block = text[3:end].splitlines()
    sha: str | None = None
    covers: list[str] = []
    in_covers = False
    for raw in block:
        line = raw.rstrip()
        if not line.strip():
            continue
        if line.startswith("covers:"):
            in_covers = True
            inline = line.split(":", 1)[1].strip()
            if inline and inline != "|":
                covers.append(inline.strip("'\""))
                in_covers = False
            continue
        if in_covers and (line.startswith("  -") or line.startswith("-")):
            covers.append(line.split("-", 1)[1].strip().strip("'\""))
            continue
        in_covers = False
        if line.startswith("generated-at-sha:"):
            sha = line.split(":", 1)[1].strip().strip("'\"")
    if not sha:
        return None
    return {"generated-at-sha": sha, "covers": covers}


def changed_since(sha: str) -> list[str] | None:
    """Files changed between sha and HEAD, or None on git error."""
    try:
        out = subprocess.run(
            ["git", "diff", "--name-only", f"{sha}..HEAD"],
            capture_output=True,
            text=True,
            check=True,
        )
    except (subprocess.CalledProcessError, FileNotFoundError):
        return None
    return [f for f in out.stdout.splitlines() if f.strip()]


def main(argv: list[str]) -> int:
    docs_dir = Path(argv[1]) if len(argv) > 1 else Path("docs")
    if not docs_dir.is_dir():
        print(f"freshness: no docs dir at {docs_dir}", file=sys.stderr)
        return 0

    pages = sorted(docs_dir.rglob("*.md"))
    stamped = 0
    stale: list[tuple[Path, list[str]]] = []

    for page in pages:
        meta = parse_frontmatter(page.read_text(encoding="utf-8", errors="replace"))
        if not meta:
            continue
        stamped += 1
        changed = changed_since(meta["generated-at-sha"])
        if changed is None:
            print(f"freshness: git diff failed for {page} (bad sha?)", file=sys.stderr)
            return 2
        hits = [
            f
            for f in changed
            for glob in meta["covers"]
            if fnmatch.fnmatch(f, glob) or f.startswith(glob.rstrip("*").rstrip("/"))
        ]
        if hits:
            stale.append((page, sorted(set(hits))))

    if stamped == 0:
        print("freshness: no provenance-stamped pages found")
        return 0

    if not stale:
        print(f"freshness: {stamped} page(s) checked, all fresh against HEAD")
        return 0

    print(f"freshness: {len(stale)}/{stamped} page(s) STALE against HEAD\n")
    for page, hits in stale:
        print(f"  {page}")
        for f in hits[:8]:
            print(f"      changed: {f}")
        if len(hits) > 8:
            print(f"      … +{len(hits) - 8} more")
    return 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
