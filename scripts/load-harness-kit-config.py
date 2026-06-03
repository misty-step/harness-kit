#!/usr/bin/env python3
"""Load and validate repo-local .harness-kit config as normalized JSON."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

try:
    import yaml
except ImportError:  # pragma: no cover - exercised by operators without PyYAML.
    yaml = None


CONFIGS = ("deploy", "monitor", "flywheel")
DURATION_RE = re.compile(r"^\s*(\d+)\s*([smhd])\s*$")
DURATION_SECONDS = {"s": 1, "m": 60, "h": 3600, "d": 86400}
TARGETS = {"fly", "vercel", "cloudflare", "aws", "s3", "docker", "k8s", "custom"}
SIGNAL_SOURCES = {"datadog", "prometheus", "grafana", "loki", "logs", "custom"}


class ConfigError(Exception):
    """User-actionable config error."""


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate .harness-kit/<name>.yaml and print normalized JSON."
    )
    parser.add_argument("name", choices=CONFIGS)
    parser.add_argument("--repo", default=".", help="Path in the target repo.")
    parser.add_argument("--config", help="Explicit config path.")
    parser.add_argument(
        "--optional",
        action="store_true",
        help="Print {} and exit 0 when the config file is missing.",
    )
    return parser.parse_args()


def repo_root(repo: str) -> Path:
    candidate = Path(repo).expanduser().resolve()
    if not candidate.exists():
        raise ConfigError(f"--repo path does not exist: {candidate}")
    result = subprocess.run(
        ["git", "-C", str(candidate), "rev-parse", "--show-toplevel"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
    )
    if result.returncode != 0:
        detail = result.stderr.strip() or "not a git repository"
        raise ConfigError(f"unable to resolve repo root from {candidate}: {detail}")
    return Path(result.stdout.strip()).resolve()


def config_path(name: str, root: Path, explicit: str | None) -> Path:
    if explicit:
        return Path(explicit).expanduser().resolve()
    return root / ".harness-kit" / f"{name}.yaml"


def load_yaml(path: Path) -> dict[str, Any]:
    if yaml is None:
        raise ConfigError("PyYAML is required to parse .harness-kit config")
    try:
        payload = yaml.safe_load(path.read_text(encoding="utf-8"))
    except yaml.YAMLError as exc:
        raise ConfigError(f"{path}: YAML parse error: {exc}") from exc
    except OSError as exc:
        raise ConfigError(f"{path}: unable to read file: {exc}") from exc
    if payload is None:
        payload = {}
    if not isinstance(payload, dict):
        raise ConfigError(f"{path}: expected top-level mapping")
    return payload


def unknown(path: Path, data: dict[str, Any], allowed: set[str], where: str) -> None:
    extra = sorted(set(data) - allowed)
    if extra:
        raise ConfigError(f"{path}: unknown key at {where}: {', '.join(extra)}")


def require(path: Path, data: dict[str, Any], key: str, where: str) -> None:
    if key not in data:
        raise ConfigError(f"{path}: missing required key at {where}: {key}")


def require_schema(path: Path, data: dict[str, Any]) -> None:
    require(path, data, "schema_version", "<root>")
    if data["schema_version"] != 1:
        raise ConfigError(f"{path}: schema_version must be 1")


def require_string(path: Path, data: dict[str, Any], key: str, where: str) -> None:
    if key in data and (not isinstance(data[key], str) or data[key] == ""):
        raise ConfigError(f"{path}: {where}.{key} must be a non-empty string")


def require_bool(path: Path, data: dict[str, Any], key: str, where: str) -> None:
    if key in data and not isinstance(data[key], bool):
        raise ConfigError(f"{path}: {where}.{key} must be boolean")


def require_int_min(path: Path, data: dict[str, Any], key: str, minimum: int, where: str) -> None:
    if key in data and (not isinstance(data[key], int) or data[key] < minimum):
        raise ConfigError(f"{path}: {where}.{key} must be integer >= {minimum}")


def require_url(path: Path, data: dict[str, Any], key: str, where: str) -> None:
    require_string(path, data, key, where)
    if key in data and not re.match(r"^https?://", data[key]):
        raise ConfigError(f"{path}: {where}.{key} must be an http(s) URL")


def duration_seconds(field: str, value: Any) -> int:
    if not isinstance(value, str):
        raise ConfigError(f"{field}: expected duration string like 30s, 5m, 1h")
    match = DURATION_RE.match(value)
    if not match:
        raise ConfigError(
            f"{field}: invalid duration '{value}' (expected <number><unit>, units: s, m, h, d)"
        )
    return int(match.group(1)) * DURATION_SECONDS[match.group(2)]


def normalize_duration(data: dict[str, Any], field: str) -> None:
    if field in data:
        data[f"{field}_seconds"] = duration_seconds(field, data[field])


def validate_deploy(path: Path, data: dict[str, Any]) -> None:
    allowed = {
        "schema_version",
        "target",
        "app",
        "envs",
        "healthcheck",
        "rollback_grace_seconds",
        "idempotent",
        "deploy_cmd",
        "current_sha_cmd",
        "rollback_handle_cmd",
        "rollback_cmd",
    }
    unknown(path, data, allowed, "<root>")
    require_schema(path, data)
    require(path, data, "target", "<root>")
    if data["target"] not in TARGETS:
        raise ConfigError(f"{path}: target is unsupported: {data['target']}")
    for key in allowed - {"schema_version", "envs", "rollback_grace_seconds", "idempotent"}:
        require_string(path, data, key, "<root>")
    require_url(path, data, "healthcheck", "<root>")
    require_int_min(path, data, "rollback_grace_seconds", 1, "<root>")
    require_bool(path, data, "idempotent", "<root>")
    if data["target"] == "custom":
        for key in ("deploy_cmd", "current_sha_cmd", "rollback_handle_cmd", "rollback_cmd"):
            require(path, data, key, "<root>")
    envs = data.get("envs")
    if envs is not None:
        if not isinstance(envs, dict) or not envs:
            raise ConfigError(f"{path}: envs must be a non-empty mapping")
        env_allowed = {"app", "healthcheck", "rollback_grace_seconds", "require_ci_green"}
        for env_name, env_cfg in envs.items():
            if not isinstance(env_name, str) or not env_name:
                raise ConfigError(f"{path}: env names must be non-empty strings")
            if not isinstance(env_cfg, dict):
                raise ConfigError(f"{path}: envs.{env_name} must be a mapping")
            where = f"envs.{env_name}"
            unknown(path, env_cfg, env_allowed, where)
            require_string(path, env_cfg, "app", where)
            require_url(path, env_cfg, "healthcheck", where)
            require_int_min(path, env_cfg, "rollback_grace_seconds", 1, where)
            require_bool(path, env_cfg, "require_ci_green", where)


def validate_monitor(path: Path, data: dict[str, Any]) -> None:
    allowed = {"schema_version", "grace_window", "poll_interval", "observability", "healthcheck", "signals"}
    unknown(path, data, allowed, "<root>")
    require_schema(path, data)
    if not any(key in data for key in ("observability", "healthcheck", "signals")):
        raise ConfigError(f"{path}: monitor config needs observability, healthcheck, or signals")
    for field in ("grace_window", "poll_interval"):
        require_string(path, data, field, "<root>")
        normalize_duration(data, field)
    healthcheck = data.get("healthcheck")
    if healthcheck is not None:
        if not isinstance(healthcheck, dict):
            raise ConfigError(f"{path}: healthcheck must be a mapping")
        unknown(path, healthcheck, {"url", "expected_status", "hard_fail_on_5xx"}, "healthcheck")
        require(path, healthcheck, "url", "healthcheck")
        require_url(path, healthcheck, "url", "healthcheck")
        require_int_min(path, healthcheck, "expected_status", 100, "healthcheck")
        if healthcheck.get("expected_status", 100) > 599:
            raise ConfigError(f"{path}: healthcheck.expected_status must be <= 599")
        require_bool(path, healthcheck, "hard_fail_on_5xx", "healthcheck")
    observability = data.get("observability")
    if observability is not None:
        if not isinstance(observability, dict):
            raise ConfigError(f"{path}: observability must be a mapping")
        allowed_obs = {
            "delegation_receipts",
            "workflow_events",
            "evidence_dirs",
            "local_logs",
            "benchmark_outputs",
            "release_smoke",
            "analytics_coverage",
        }
        unknown(path, observability, allowed_obs, "observability")
        for key in ("evidence_dirs", "local_logs", "benchmark_outputs", "release_smoke"):
            if key in observability and (
                not isinstance(observability[key], list)
                or not all(isinstance(item, str) and item for item in observability[key])
            ):
                raise ConfigError(f"{path}: observability.{key} must be a list of strings")
        for key in allowed_obs - {"evidence_dirs", "local_logs", "benchmark_outputs", "release_smoke"}:
            require_string(path, observability, key, "observability")
    signals = data.get("signals")
    if signals is not None:
        if not isinstance(signals, list) or not signals:
            raise ConfigError(f"{path}: signals must be a non-empty list")
        allowed_signal = {"name", "source", "query", "url", "threshold", "jq", "hard_fail"}
        for index, signal in enumerate(signals):
            where = f"signals.{index}"
            if not isinstance(signal, dict):
                raise ConfigError(f"{path}: {where} must be a mapping")
            unknown(path, signal, allowed_signal, where)
            for key in ("name", "source", "threshold"):
                require(path, signal, key, where)
            if "query" not in signal and "url" not in signal:
                raise ConfigError(f"{path}: {where} needs query or url")
            for key in allowed_signal - {"hard_fail"}:
                require_string(path, signal, key, where)
            require_url(path, signal, "url", where)
            require_bool(path, signal, "hard_fail", where)
            if signal["source"] not in SIGNAL_SOURCES:
                raise ConfigError(f"{path}: {where}.source is unsupported: {signal['source']}")


def validate_flywheel(path: Path, data: dict[str, Any]) -> None:
    allowed = {
        "schema_version",
        "cadence",
        "max_cycles",
        "budget_tokens",
        "backlog_includes",
        "stop_on_monitor_alert",
        "stop_on_phase_failed",
        "stop_on_budget_exhausted",
    }
    unknown(path, data, allowed, "<root>")
    require_schema(path, data)
    normalize_duration(data, "cadence")
    for key in ("max_cycles", "budget_tokens"):
        require_int_min(path, data, key, 1, "<root>")
    for key in ("stop_on_monitor_alert", "stop_on_phase_failed", "stop_on_budget_exhausted"):
        require_bool(path, data, key, "<root>")
    includes = data.get("backlog_includes")
    if includes is not None:
        if not isinstance(includes, list) or not includes:
            raise ConfigError(f"{path}: backlog_includes must be a non-empty list")
        if not all(isinstance(item, str) and item for item in includes):
            raise ConfigError(f"{path}: backlog_includes items must be non-empty strings")


def validate(name: str, path: Path, data: dict[str, Any]) -> dict[str, Any]:
    normalized = dict(data)
    if name == "deploy":
        validate_deploy(path, normalized)
    elif name == "monitor":
        validate_monitor(path, normalized)
    elif name == "flywheel":
        validate_flywheel(path, normalized)
    return normalized


def main() -> int:
    args = parse_args()
    root = repo_root(args.repo)
    path = config_path(args.name, root, args.config)
    if not path.exists():
        if args.optional:
            print("{}")
            return 0
        print(
            f"ERROR: missing config file: {path} "
            f"(create {root / '.harness-kit' / (args.name + '.yaml')})",
            file=sys.stderr,
        )
        return 2
    try:
        payload = validate(args.name, path, load_yaml(path))
    except ConfigError as exc:
        print(f"ERROR: {exc}", file=sys.stderr)
        return 1
    print(json.dumps(payload, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
