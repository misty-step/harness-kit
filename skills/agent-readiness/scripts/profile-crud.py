#!/usr/bin/env python3
"""Create, read, update, delete, and validate agent-readiness profiles."""

from __future__ import annotations

import argparse
import shlex
import sys
from datetime import UTC, date, datetime
from pathlib import Path
from typing import Any

import yaml

DEFAULT_PROFILE = Path(".harness-kit/agent-readiness.yaml")
PLACEHOLDERS = {"", "tbd", "todo", "n/a", "na", "none", "placeholder", "unknown"}
REQUIRED_TOP_LEVEL = {
    "version",
    "generated_at",
    "profile",
    "gates",
    "adr_policy",
    "infrastructure",
    "module_boundaries",
    "mock_policy",
    "observability",
    "state_surfaces",
    "readiness_state",
    "waivers",
}
READINESS_STATES = {"unknown", "improved", "preserved", "regressed"}
FEEDBACK_STRENGTH = {"unknown", "weak", "moderate", "strong", "strict"}
INFRA_MANAGEABILITY = {"unknown", "human_only", "cli_api_sdk", "mixed"}
OBSERVABILITY_ACCESS = {"unknown", "none", "logs", "metrics", "traces", "full"}
AGENT_ACCESS = {"code", "local-file", "mcp", "cli", "api", "skill", "admin-ui-only", "cms-only", "unknown"}
HIDDEN_AGENT_ACCESS = {"admin-ui-only", "cms-only", "unknown"}
PROSE_COMMAND_PREFIXES = {"ask", "read", "see", "visit", "open", "check"}


class ProfileError(ValueError):
    pass


def now_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def is_placeholder(value: object) -> bool:
    if not isinstance(value, str):
        return True
    normalized = value.strip().lower()
    return normalized in PLACEHOLDERS


def detect_stack(repo_root: Path) -> list[str]:
    checks = [
        ("node", "package.json"),
        ("python", "pyproject.toml"),
        ("python", "requirements.txt"),
        ("rust", "Cargo.toml"),
        ("go", "go.mod"),
        ("dagger", "ci/src"),
    ]
    found: list[str] = []
    for label, rel in checks:
        if (repo_root / rel).exists() and label not in found:
            found.append(label)
    return found or ["unknown"]


def infer_feedback_strength(repo_root: Path) -> str:
    strong_signals = [
        repo_root / "ci/src",
        repo_root / ".githooks",
        repo_root / "scripts" / "check-agent-roster.py",
    ]
    strict_signals = [
        repo_root / "dagger.json",
        repo_root / "pyproject.toml",
        repo_root / "package-lock.json",
        repo_root / "bun.lock",
        repo_root / "Cargo.lock",
    ]
    score = sum(path.exists() for path in strong_signals) + sum(path.exists() for path in strict_signals)
    if score >= 4:
        return "strict"
    if score >= 2:
        return "strong"
    if score == 1:
        return "moderate"
    return "unknown"


def detect_local_gates(repo_root: Path) -> list[str]:
    gates: list[str] = []
    if (repo_root / "ci/src").exists():
        gates.append("dagger call check --source=.")
    if (repo_root / "Makefile").exists():
        gates.append("make test")
    if (repo_root / "package.json").exists():
        gates.append("npm test")
    return gates


def default_profile(repo_root: Path) -> dict[str, Any]:
    return {
        "version": 1,
        "generated_at": now_iso(),
        "profile": {
            "repo_root": str(repo_root),
            "detected_stack": detect_stack(repo_root),
            "stack_feedback_strength": infer_feedback_strength(repo_root),
        },
        "gates": {
            "local": detect_local_gates(repo_root),
            "ci": ["dagger call check --source=."] if (repo_root / "ci/src").exists() else [],
            "coverage": {"command": "", "threshold": ""},
        },
        "adr_policy": {
            "required_when": (
                "Decision is hard to reverse, surprising without context, "
                "and the result of a real tradeoff."
            ),
            "paths": ["docs/adr/"],
        },
        "infrastructure": {"manageability": "unknown", "surfaces": []},
        "module_boundaries": [],
        "mock_policy": "Mock only external boundaries; internal mocks are readiness regressions.",
        "observability": {"access": "unknown", "signals": []},
        "state_surfaces": [],
        "readiness_state": "unknown",
        "waivers": [],
    }


def load_profile(path: Path) -> dict[str, Any]:
    if not path.exists():
        raise ProfileError(f"{path}: missing readiness profile")
    with path.open(encoding="utf-8") as handle:
        data = yaml.safe_load(handle)
    if not isinstance(data, dict):
        raise ProfileError(f"{path}: profile must be a YAML mapping")
    return data


def write_profile(path: Path, data: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        yaml.safe_dump(data, handle, sort_keys=False)


def require_mapping(data: dict[str, Any], key: str) -> dict[str, Any]:
    value = data.get(key)
    if not isinstance(value, dict):
        raise ProfileError(f"{key}: must be a mapping")
    return value


def require_list(data: dict[str, Any], key: str) -> list[Any]:
    value = data.get(key)
    if not isinstance(value, list):
        raise ProfileError(f"{key}: must be a list")
    return value


def parse_expiry(value: object) -> date:
    if not isinstance(value, str) or not value.strip():
        raise ProfileError("waiver expires_on must be a YYYY-MM-DD string")
    try:
        return date.fromisoformat(value)
    except ValueError as error:
        raise ProfileError(f"waiver expires_on is not YYYY-MM-DD: {value}") from error


def validate_waiver(waiver: Any, today: date) -> None:
    if not isinstance(waiver, dict):
        raise ProfileError("waiver must be a mapping")
    for field in ("id", "scope", "reason", "adr"):
        if is_placeholder(waiver.get(field)):
            raise ProfileError(f"waiver {field} must be a non-placeholder string")
    expires_on = parse_expiry(waiver.get("expires_on"))
    if expires_on <= today:
        raise ProfileError(f"waiver {waiver.get('id', '<unknown>')}: expires_on must be in the future")


def validate_state_surface(surface: Any, today: date) -> None:
    if not isinstance(surface, dict):
        raise ProfileError("state_surfaces entry must be a mapping")
    for field in ("name", "system_of_record"):
        if is_placeholder(surface.get(field)):
            raise ProfileError(f"state_surfaces {field} must be a non-placeholder string")
    access = surface.get("agent_access")
    if access not in AGENT_ACCESS:
        raise ProfileError("state_surfaces agent_access is invalid")

    if access in HIDDEN_AGENT_ACCESS:
        if is_placeholder(surface.get("waiver")):
            raise ProfileError(
                f"state surface {surface.get('name', '<unknown>')}: "
                f"{access} requires a non-placeholder waiver"
            )
        expires_on = parse_expiry(surface.get("waiver_expires"))
        if expires_on <= today:
            raise ProfileError(
                f"state surface {surface.get('name', '<unknown>')}: waiver_expires must be in the future"
            )
        return

    if is_placeholder(surface.get("source_path")):
        raise ProfileError(
            f"state surface {surface.get('name', '<unknown>')}: source_path must be set"
        )
    if is_placeholder(surface.get("verification_command")):
        raise ProfileError(
            f"state surface {surface.get('name', '<unknown>')}: verification_command must be set"
        )
    command = surface.get("verification_command")
    try:
        command_parts = shlex.split(command)
    except ValueError as exc:
        raise ProfileError(
            f"state surface {surface.get('name', '<unknown>')}: verification_command is not shell-parseable"
        ) from exc
    if (
        not command_parts
        or "\n" in command
        or command_parts[0].lower() in PROSE_COMMAND_PREFIXES
        or command_parts[0].startswith(("http://", "https://"))
    ):
        raise ProfileError(
            f"state surface {surface.get('name', '<unknown>')}: verification_command must be command-shaped"
        )


def validate_profile(data: dict[str, Any], *, today: date | None = None) -> None:
    missing = REQUIRED_TOP_LEVEL - set(data)
    extra = set(data) - REQUIRED_TOP_LEVEL
    if missing:
        raise ProfileError(f"missing field(s): {sorted(missing)}")
    if extra:
        raise ProfileError(f"unknown field(s): {sorted(extra)}")
    if data.get("version") != 1:
        raise ProfileError("version must be 1")

    profile = require_mapping(data, "profile")
    if is_placeholder(profile.get("repo_root")):
        raise ProfileError("profile.repo_root must be set")
    if not isinstance(profile.get("detected_stack"), list) or not profile["detected_stack"]:
        raise ProfileError("profile.detected_stack must be a non-empty list")
    if profile.get("stack_feedback_strength") not in FEEDBACK_STRENGTH:
        raise ProfileError("profile.stack_feedback_strength is invalid")

    gates = require_mapping(data, "gates")
    require_list(gates, "local")
    require_list(gates, "ci")
    coverage = gates.get("coverage")
    if not isinstance(coverage, dict) or "command" not in coverage or "threshold" not in coverage:
        raise ProfileError("gates.coverage must include command and threshold")

    adr_policy = require_mapping(data, "adr_policy")
    if is_placeholder(adr_policy.get("required_when")):
        raise ProfileError("adr_policy.required_when must be set")
    require_list(adr_policy, "paths")

    infrastructure = require_mapping(data, "infrastructure")
    if infrastructure.get("manageability") not in INFRA_MANAGEABILITY:
        raise ProfileError("infrastructure.manageability is invalid")
    require_list(infrastructure, "surfaces")

    require_list(data, "module_boundaries")
    if is_placeholder(data.get("mock_policy")):
        raise ProfileError("mock_policy must be set")

    observability = require_mapping(data, "observability")
    if observability.get("access") not in OBSERVABILITY_ACCESS:
        raise ProfileError("observability.access is invalid")
    require_list(observability, "signals")

    effective_today = today or datetime.now(UTC).date()
    for surface in require_list(data, "state_surfaces"):
        validate_state_surface(surface, effective_today)

    if data.get("readiness_state") not in READINESS_STATES:
        raise ProfileError("readiness_state is invalid")

    for waiver in require_list(data, "waivers"):
        validate_waiver(waiver, effective_today)


def profile_summary(data: dict[str, Any]) -> str:
    profile = data["profile"]
    gates = data["gates"]
    hidden_surfaces = [
        surface
        for surface in data.get("state_surfaces", [])
        if surface.get("agent_access") in HIDDEN_AGENT_ACCESS
    ]
    return "\n".join(
        [
            "Agent readiness profile",
            f"- repo_root: {profile['repo_root']}",
            f"- detected_stack: {', '.join(profile['detected_stack'])}",
            f"- stack_feedback_strength: {profile['stack_feedback_strength']}",
            f"- readiness_state: {data['readiness_state']}",
            f"- local_gates: {', '.join(gates['local']) if gates['local'] else 'none'}",
            (
                f"- state_surfaces: {len(data.get('state_surfaces', []))} "
                f"(hidden_debt: {len(hidden_surfaces)}; "
                "remediation: meta/INTEGRATION_GUIDE.md)"
            ),
            f"- waivers: {len(data['waivers'])}",
        ]
    )


def command_create(args: argparse.Namespace) -> int:
    if args.profile.exists() and not args.force:
        raise ProfileError(f"{args.profile}: already exists; use --force to overwrite")
    data = default_profile(args.repo_root.resolve())
    validate_profile(data)
    write_profile(args.profile, data)
    print(f"created {args.profile}")
    return 0


def command_read(args: argparse.Namespace) -> int:
    data = load_profile(args.profile)
    validate_profile(data)
    print(profile_summary(data))
    return 0


def command_validate(args: argparse.Namespace) -> int:
    data = load_profile(args.profile)
    validate_profile(data)
    print(f"{args.profile}: valid")
    return 0


def command_update(args: argparse.Namespace) -> int:
    data = load_profile(args.profile)
    validate_profile(data)
    waiver = {
        "id": args.waiver_id,
        "scope": args.scope,
        "reason": args.reason,
        "expires_on": args.expires_on,
        "adr": args.adr,
    }
    validate_waiver(waiver, datetime.now(UTC).date())
    waivers = [entry for entry in data["waivers"] if entry.get("id") != args.waiver_id]
    waivers.append(waiver)
    data["waivers"] = sorted(waivers, key=lambda row: row["id"])
    if args.readiness_state:
        data["readiness_state"] = args.readiness_state
    validate_profile(data)
    write_profile(args.profile, data)
    print(f"updated {args.profile}: waiver {args.waiver_id}")
    return 0


def command_delete(args: argparse.Namespace) -> int:
    data = load_profile(args.profile)
    validate_profile(data)
    before = len(data["waivers"])
    data["waivers"] = [entry for entry in data["waivers"] if entry.get("id") != args.waiver_id]
    if len(data["waivers"]) == before:
        raise ProfileError(f"waiver not found: {args.waiver_id}")
    validate_profile(data)
    write_profile(args.profile, data)
    print(f"deleted waiver {args.waiver_id} from {args.profile}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path, default=DEFAULT_PROFILE)
    subparsers = parser.add_subparsers(dest="command", required=True)

    create = subparsers.add_parser("create")
    create.add_argument("--repo-root", type=Path, default=Path.cwd())
    create.add_argument("--force", action="store_true")
    create.set_defaults(func=command_create)

    read = subparsers.add_parser("read")
    read.set_defaults(func=command_read)

    validate = subparsers.add_parser("validate")
    validate.set_defaults(func=command_validate)

    update = subparsers.add_parser("update")
    update.add_argument("--waiver-id", required=True)
    update.add_argument("--scope", required=True)
    update.add_argument("--reason", required=True)
    update.add_argument("--expires-on", required=True)
    update.add_argument("--adr", required=True)
    update.add_argument("--readiness-state", choices=sorted(READINESS_STATES))
    update.set_defaults(func=command_update)

    delete = subparsers.add_parser("delete")
    delete.add_argument("--waiver-id", required=True)
    delete.set_defaults(func=command_delete)
    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        return args.func(args)
    except ProfileError as error:
        print(f"profile-crud: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
