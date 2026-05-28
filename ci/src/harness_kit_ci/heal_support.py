"""Pure helpers for the self-healing CI flow."""

from dataclasses import dataclass
from datetime import UTC, datetime
import hashlib
from pathlib import Path
import re

HEALABLE_GATES = {
    "lint-yaml": "Fix YAML syntax or structural issues without changing meaning.",
    "lint-shell": "Fix shellcheck errors without changing script behavior.",
    "lint-python": "Fix Python syntax or import errors without changing behavior.",
    "check-frontmatter": "Fix invalid frontmatter metadata without broad rewrites.",
}


@dataclass(frozen=True, slots=True)
class GateFailure:
    """A failing CI gate extracted from the aggregated check summary."""

    name: str
    detail: str


def parse_check_failures(summary: str) -> list[GateFailure]:
    """Parse failing gates from the human-readable check() summary."""
    failures: list[GateFailure] = []
    current_name: str | None = None
    current_ok = True
    details: list[str] = []

    for line in summary.splitlines():
        match = re.match(r"^\s{2}(PASS|FAIL)\s{2}(.+)$", line)
        if match:
            if current_name and not current_ok:
                failures.append(
                    GateFailure(
                        name=current_name,
                        detail="\n".join(details).strip() or "No stderr captured.",
                    )
                )
            current_ok = match.group(1) == "PASS"
            current_name = match.group(2).strip()
            details = []
            continue

        if current_name and not current_ok and line.startswith("         "):
            details.append(line.strip())

    if current_name and not current_ok:
        failures.append(
            GateFailure(
                name=current_name,
                detail="\n".join(details).strip() or "No stderr captured.",
            )
        )

    return failures


def first_failed_gate(summary: str) -> str | None:
    """Return the first failing gate name from a check() summary, if any."""
    failures = parse_check_failures(summary)
    return failures[0].name if failures else None


def select_healable_failure(failures: list[GateFailure]) -> GateFailure:
    """Allow exactly one healable lint-style failure at a time."""
    if not failures:
        raise ValueError("heal requires at least one failing gate.")

    unsupported = [failure.name for failure in failures if failure.name not in HEALABLE_GATES]
    if unsupported:
        supported = ", ".join(sorted(HEALABLE_GATES))
        got = ", ".join(sorted(failure.name for failure in failures))
        raise ValueError(
            "heal currently supports one lint-style failure at a time. "
            f"Supported gates: {supported}. Got: {got}."
        )

    if len(failures) != 1:
        got = ", ".join(sorted(failure.name for failure in failures))
        raise ValueError(
            "heal currently supports one failing gate at a time. "
            f"Resolve the other failures first: {got}."
        )

    return failures[0]


def repair_branch_name(gate_name: str) -> str:
    """Create a unique branch name for the repair commit."""
    slug = re.sub(r"[^a-z0-9]+", "-", gate_name.lower()).strip("-")
    timestamp = datetime.now(UTC).strftime("%Y%m%d%H%M%S")
    return f"heal/{slug}-{timestamp}"


def repair_commit_message(gate_name: str) -> str:
    """Create the semantic commit message for a successful repair."""
    return f"ci: heal {gate_name}"


def snapshot_delta(
    before_root: Path,
    after_root: Path,
    *,
    excluded_names: frozenset[str] = frozenset({".git", ".env", "__pycache__"}),
) -> tuple[list[str], list[str]]:
    """Return paths to stage and remove when comparing two working-tree snapshots."""

    def file_digest(path: Path) -> bytes:
        digest = hashlib.sha256()
        with path.open("rb") as handle:
            while chunk := handle.read(64 * 1024):
                digest.update(chunk)
        return digest.digest()

    def collect(root: Path) -> dict[str, bytes]:
        files: dict[str, bytes] = {}
        for path in root.rglob("*"):
            rel = path.relative_to(root)
            if any(part in excluded_names for part in rel.parts):
                continue
            if path.is_file():
                files[rel.as_posix()] = file_digest(path)
        return files

    before_files = collect(before_root)
    after_files = collect(after_root)

    stage: list[str] = []
    remove: list[str] = []
    for rel in sorted(set(before_files) | set(after_files)):
        before = before_files.get(rel)
        after = after_files.get(rel)
        if before is None and after is not None:
            stage.append(rel)
        elif before is not None and after is None:
            remove.append(rel)
        elif before is not None and after is not None and before != after:
            stage.append(rel)

    return stage, remove
