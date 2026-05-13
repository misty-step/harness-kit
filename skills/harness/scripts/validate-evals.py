#!/usr/bin/env python3
"""Validate skill eval suite structure.

Checks every existing skills/<name>/evals/ tree. This is intentionally a
structure check, not a semantic judge.
"""

from pathlib import Path
import sys


def has_file(path: Path) -> bool:
    return path.exists() and any(child.is_file() for child in path.iterdir())


def main() -> int:
    root = Path("skills")
    if not root.exists():
        print("FAIL: skills/ not found", file=sys.stderr)
        return 1

    errors: list[str] = []
    checked = 0

    for evals_dir in sorted(root.glob("*/evals")):
        if not evals_dir.is_dir():
            continue
        checked += 1
        skill = evals_dir.parent.name

        readme = evals_dir / "README.md"
        if not readme.is_file():
            errors.append(f"skills/{skill}/evals: missing README.md")

        cases = evals_dir / "cases"
        if not cases.is_dir() or not has_file(cases):
            errors.append(f"skills/{skill}/evals: missing at least one case file")

        graders = evals_dir / "graders"
        if not graders.is_dir() or not has_file(graders):
            errors.append(f"skills/{skill}/evals: missing at least one grader")

    if errors:
        for error in errors:
            print(f"FAIL: {error}", file=sys.stderr)
        return 1

    print(f"OK: {checked} skill eval suite(s) valid")
    return 0


if __name__ == "__main__":
    sys.exit(main())
