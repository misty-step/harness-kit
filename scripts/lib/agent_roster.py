"""Agent-provider roster and delegation receipt validation."""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import signal
import subprocess
import time
import uuid
from collections import Counter, defaultdict
from datetime import UTC, datetime
from pathlib import Path
from typing import Any

import yaml

ROSTER_PROVIDER_IDS = {
    "codex",
    "claude",
    "pi",
    "agy",
    "cursor-agent",
    "grok-build",
    "manual",
}
RETIRED_RECEIPT_PROVIDER_IDS = {"opencode"}
RECEIPT_PROVIDER_IDS = ROSTER_PROVIDER_IDS | RETIRED_RECEIPT_PROVIDER_IDS

VALID_TIERS = {"primary", "conditional", "manual", "disabled"}
VALID_KINDS = {"cli", "bench", "manual"}
VALID_OUTPUTS = {"json", "stream-json", "text", "patch-ref", "manual-summary"}
VALID_WORKTREE = {"required", "recommended", "not_applicable"}
VALID_PROVIDER_STATUS = {"available", "unavailable", "error", "partial", "manual"}
VALID_ATTEMPT_STATUS = {
    "not_started",
    "running",
    "succeeded",
    "failed",
    "rejected",
    "superseded",
    "manual",
}
VALID_LEAD_VERDICTS = {
    "accepted",
    "partially_accepted",
    "rejected",
    "reference_only",
    "pending",
}

REQUIRED_PROVIDER_FIELDS = {
    "tier",
    "kind",
    "probe",
    "dispatch",
    "output",
    "permissions",
    "worktree",
    "notes",
}
REQUIRED_RECEIPT_FIELDS = {
    "schema_version",
    "delegation_id",
    "created_at",
    "repo_root",
    "worktree_id",
    "lead_harness",
    "lead_provider",
    "backlog_ref",
    "objective",
    "input_ref",
    "provider_target",
    "provider_status",
    "attempt_status",
    "evidence_refs",
    "summary",
    "lead_verdict",
    "redactions_applied",
}

SECRET_RE = re.compile(
    r"(?i)(api[_-]?key|token|secret|password|bearer|xai_api_key|exa_api_key|anthropic_api_key)"
)
SHELL_META_RE = re.compile(r"[;&|`$<>]")
INLINE_EVIDENCE_RE = re.compile(r"\s")


class ReceiptValidationError(ValueError):
    """Raised when a delegation receipt row is malformed."""


def now_iso() -> str:
    return datetime.now(UTC).replace(microsecond=0).isoformat().replace("+00:00", "Z")


def repo_root() -> Path:
    completed = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        check=False,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        text=True,
    )
    if completed.returncode == 0 and completed.stdout.strip():
        return Path(completed.stdout.strip())
    return Path.cwd()


def load_roster(path: Path) -> dict[str, Any]:
    with path.open() as handle:
        roster = yaml.safe_load(handle)
    if not isinstance(roster, dict):
        raise ValueError(f"{path} must contain a YAML mapping.")
    return roster


def validate_roster(roster: dict[str, Any]) -> None:
    if roster.get("version") != 1:
        raise ValueError("roster version must be 1.")

    providers = roster.get("providers")
    if not isinstance(providers, dict):
        raise ValueError("roster must define providers mapping.")

    missing = ROSTER_PROVIDER_IDS - set(providers)
    extra = set(providers) - ROSTER_PROVIDER_IDS
    if missing:
        raise ValueError(f"roster missing providers: {', '.join(sorted(missing))}")
    if extra:
        raise ValueError(f"roster contains unknown providers: {', '.join(sorted(extra))}")

    for provider_id, provider in providers.items():
        if not isinstance(provider, dict):
            raise ValueError(f"{provider_id}: provider entry must be a mapping.")
        missing_fields = REQUIRED_PROVIDER_FIELDS - set(provider)
        if missing_fields:
            raise ValueError(
                f"{provider_id}: missing fields: {', '.join(sorted(missing_fields))}"
            )
        _validate_enum(provider_id, provider, "tier", VALID_TIERS)
        _validate_enum(provider_id, provider, "kind", VALID_KINDS)
        _validate_enum(provider_id, provider, "output", VALID_OUTPUTS)
        _validate_enum(provider_id, provider, "worktree", VALID_WORKTREE)
        if provider_id == "manual":
            if provider["kind"] != "manual" or provider["tier"] != "manual":
                raise ValueError("manual provider must use tier=manual and kind=manual.")
        for field in ("probe", "dispatch", "permissions", "notes"):
            value = provider.get(field)
            if not isinstance(value, str) or not value.strip():
                raise ValueError(f"{provider_id}: {field} must be a non-empty string.")
            if SECRET_RE.search(value):
                raise ValueError(f"{provider_id}: {field} contains secret-like text.")
            if field in {"probe", "dispatch"} and SHELL_META_RE.search(value):
                raise ValueError(f"{provider_id}: {field} contains shell metacharacters.")
        variants = provider.get("model_variants")
        if variants is not None:
            if not isinstance(variants, dict):
                raise ValueError(f"{provider_id}: model_variants must be a mapping.")
            for name, model in variants.items():
                if not isinstance(name, str) or not name.strip():
                    raise ValueError(f"{provider_id}: model_variants keys must be non-empty strings.")
                if not isinstance(model, str) or not model.strip():
                    raise ValueError(f"{provider_id}: model_variants values must be non-empty strings.")
                if SECRET_RE.search(model):
                    raise ValueError(f"{provider_id}: model_variants contains secret-like text.")


def _validate_enum(
    provider_id: str,
    provider: dict[str, Any],
    field: str,
    valid_values: set[str],
) -> None:
    value = provider.get(field)
    if value not in valid_values:
        allowed = ", ".join(sorted(valid_values))
        raise ValueError(f"{provider_id}: {field} must be one of: {allowed}.")


def build_probe_receipts(
    roster: dict[str, Any],
    *,
    path_env: str | None,
    lead_harness: str,
    lead_provider: str,
    input_ref: str,
    objective: str,
    backlog_ref: str = "",
) -> list[dict[str, Any]]:
    validate_roster(roster)
    providers = roster["providers"]
    return [
        build_attempt_receipt(
            provider_target=provider_id,
            provider_status=_probe_status(provider_id, provider, path_env),
            attempt_status="not_started",
            objective=objective,
            input_ref=input_ref,
            evidence_refs=[],
            lead_verdict="pending",
            worktree_id=Path.cwd().name,
            backlog_ref=backlog_ref,
            lead_harness=lead_harness,
            lead_provider=lead_provider,
            summary=f"probe: {provider_id}",
        )
        for provider_id, provider in providers.items()
    ]


def _probe_status(provider_id: str, provider: dict[str, Any], path_env: str | None) -> str:
    if provider_id == "manual" or provider["kind"] == "manual":
        return "manual"

    try:
        command = shlex.split(provider["probe"])
    except ValueError:
        return "error"
    if not command:
        return "error"
    binary = command[0]
    search_path = path_env if path_env is not None else os.environ.get("PATH", "")
    if not shutil.which(binary, path=search_path):
        return "unavailable"
    env = os.environ.copy()
    env["PATH"] = search_path
    try:
        completed = subprocess.run(
            command,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            env=env,
            timeout=5,
        )
    except (OSError, subprocess.TimeoutExpired):
        return "error"
    if completed.returncode != 0:
        return "error"
    return "available"


def dispatch_provider_lane(
    roster: dict[str, Any],
    *,
    provider_target: str,
    prompt: str,
    objective: str,
    input_ref: str,
    transcript_dir: Path,
    receipt_output: Path,
    timeout_s: float,
    grace_s: float,
    lead_harness: str,
    lead_provider: str,
    backlog_ref: str = "",
    path_env: str | None = None,
    model_override: str | None = None,
) -> dict[str, Any]:
    """Run one provider command with process-group cleanup and receipt capture."""
    validate_roster(roster)
    if provider_target not in roster["providers"]:
        raise ValueError(f"unknown provider target: {provider_target}")

    provider = roster["providers"][provider_target]
    worktree_id = Path.cwd().name
    if provider["kind"] == "manual":
        receipt = build_attempt_receipt(
            provider_target=provider_target,
            provider_status="manual",
            attempt_status="manual",
            objective=objective,
            input_ref=input_ref,
            evidence_refs=[],
            lead_verdict="reference_only",
            worktree_id=worktree_id,
            backlog_ref=backlog_ref,
            lead_harness=lead_harness,
            lead_provider=lead_provider,
            summary="manual provider cannot be dispatched by CLI",
        )
        append_receipt(receipt_output, receipt)
        return receipt

    provider_status = _probe_status(provider_target, provider, path_env)
    if provider_status != "available":
        receipt = build_attempt_receipt(
            provider_target=provider_target,
            provider_status=provider_status,
            attempt_status="failed",
            objective=objective,
            input_ref=input_ref,
            evidence_refs=[],
            lead_verdict="rejected",
            worktree_id=worktree_id,
            backlog_ref=backlog_ref,
            lead_harness=lead_harness,
            lead_provider=lead_provider,
            summary=f"provider probe was {provider_status}; dispatch skipped",
        )
        append_receipt(receipt_output, receipt)
        return receipt

    command = _dispatch_command(provider, prompt, model_override=model_override)
    transcript = _transcript_path(transcript_dir, provider_target)
    transcript.parent.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    if path_env is not None:
        env["PATH"] = path_env

    timed_out = False
    with transcript.open("w") as output:
        process = subprocess.Popen(
            command,
            stdout=output,
            stderr=subprocess.STDOUT,
            text=True,
            env=env,
            start_new_session=True,
        )
        try:
            returncode = process.wait(timeout=timeout_s)
        except subprocess.TimeoutExpired:
            timed_out = True
            cleanup_note = _terminate_process_group(process.pid, grace_s)
            try:
                returncode = process.wait(timeout=1)
            except subprocess.TimeoutExpired:
                returncode = -signal.SIGKILL

    if timed_out:
        provider_status = "error"
        attempt_status = "failed"
        lead_verdict = "rejected"
        summary = f"provider dispatch timed out after {timeout_s:g}s; {cleanup_note}"
    elif returncode == 0:
        provider_status = "available"
        attempt_status = "succeeded"
        lead_verdict = "pending"
        summary = "provider dispatch exited 0"
        if model_override:
            summary += f"; model_override={_resolve_model_override(provider, model_override)}"
    else:
        provider_status = "available"
        attempt_status = "failed"
        lead_verdict = "rejected"
        summary = f"provider dispatch exited {returncode}"

    receipt = build_attempt_receipt(
        provider_target=provider_target,
        provider_status=provider_status,
        attempt_status=attempt_status,
        objective=objective,
        input_ref=input_ref,
        evidence_refs=[str(transcript)],
        lead_verdict=lead_verdict,
        worktree_id=worktree_id,
        backlog_ref=backlog_ref,
        lead_harness=lead_harness,
        lead_provider=lead_provider,
        summary=summary,
    )
    append_receipt(receipt_output, receipt)
    return receipt


def _resolve_model_override(provider: dict[str, Any], model_override: str) -> str:
    variants = provider.get("model_variants")
    if isinstance(variants, dict) and model_override in variants:
        return str(variants[model_override])
    return model_override


def _dispatch_command(
    provider: dict[str, Any],
    prompt: str,
    *,
    model_override: str | None = None,
) -> list[str]:
    command = shlex.split(provider["dispatch"])
    if model_override:
        model = _resolve_model_override(provider, model_override)
        if "--model" in command:
            model_index = command.index("--model")
            if model_index == len(command) - 1:
                raise ValueError("dispatch command has --model without a value.")
            command[model_index + 1] = model.removeprefix("openrouter/")
        else:
            command.extend(["--model", model.removeprefix("openrouter/")])
    return command + [prompt]


def _transcript_path(transcript_dir: Path, provider_target: str) -> Path:
    stamp = datetime.now(UTC).strftime("%Y%m%dT%H%M%S.%fZ")
    safe_provider = re.sub(r"[^A-Za-z0-9_.-]", "_", provider_target)
    return transcript_dir / f"{stamp}-{safe_provider}-{uuid.uuid4().hex[:8]}.txt"


def _terminate_process_group(pid: int, grace_s: float) -> str:
    try:
        pgid = os.getpgid(pid)
    except ProcessLookupError:
        return "process exited before cleanup"
    try:
        os.killpg(pgid, signal.SIGTERM)
    except ProcessLookupError:
        return "process group exited before SIGTERM"
    except PermissionError:
        return "permission denied during SIGTERM cleanup"
    if grace_s > 0:
        time.sleep(grace_s)
    try:
        os.killpg(pgid, signal.SIGKILL)
    except ProcessLookupError:
        return "process group exited after SIGTERM"
    except PermissionError:
        return "permission denied during SIGKILL cleanup"
    return "process group killed"


def build_attempt_receipt(
    *,
    provider_target: str,
    provider_status: str,
    attempt_status: str,
    objective: str,
    input_ref: str,
    evidence_refs: list[str],
    lead_verdict: str,
    worktree_id: str,
    backlog_ref: str = "",
    lead_harness: str = "unknown",
    lead_provider: str = "unknown",
    summary: str = "",
) -> dict[str, Any]:
    receipt = {
        "schema_version": 1,
        "delegation_id": str(uuid.uuid4()),
        "created_at": now_iso(),
        "repo_root": str(repo_root()),
        "worktree_id": worktree_id,
        "lead_harness": lead_harness,
        "lead_provider": lead_provider,
        "backlog_ref": backlog_ref,
        "objective": objective,
        "input_ref": input_ref,
        "provider_target": provider_target,
        "provider_status": provider_status,
        "attempt_status": attempt_status,
        "evidence_refs": evidence_refs,
        "summary": summary,
        "lead_verdict": lead_verdict,
        "redactions_applied": [],
    }
    validate_receipt(receipt)
    return receipt


def validate_receipt(receipt: dict[str, Any]) -> None:
    missing = REQUIRED_RECEIPT_FIELDS - set(receipt)
    if missing:
        raise ReceiptValidationError(f"receipt missing fields: {', '.join(sorted(missing))}")
    extra = set(receipt) - REQUIRED_RECEIPT_FIELDS
    if extra:
        raise ReceiptValidationError(f"receipt has unknown fields: {', '.join(sorted(extra))}")
    if receipt["schema_version"] != 1:
        raise ReceiptValidationError("receipt schema_version must be 1.")
    try:
        uuid.UUID(str(receipt["delegation_id"]))
    except ValueError as error:
        raise ReceiptValidationError("receipt delegation_id must be a UUID.") from error
    if not isinstance(receipt["redactions_applied"], list):
        raise ReceiptValidationError("receipt redactions_applied must be a list.")
    if receipt["provider_target"] not in RECEIPT_PROVIDER_IDS:
        raise ReceiptValidationError("receipt provider_target is not a known provider id.")
    if receipt["provider_status"] not in VALID_PROVIDER_STATUS:
        raise ReceiptValidationError("receipt provider_status is invalid.")
    if receipt["attempt_status"] not in VALID_ATTEMPT_STATUS:
        raise ReceiptValidationError("receipt attempt_status is invalid.")
    if receipt["lead_verdict"] not in VALID_LEAD_VERDICTS:
        raise ReceiptValidationError("receipt lead_verdict is invalid.")
    if not isinstance(receipt["evidence_refs"], list):
        raise ReceiptValidationError("receipt evidence_refs must be a list.")
    for ref in receipt["evidence_refs"]:
        if not isinstance(ref, str) or not ref:
            raise ReceiptValidationError("receipt evidence_refs must contain strings.")
        if INLINE_EVIDENCE_RE.search(ref):
            raise ReceiptValidationError("receipt evidence_refs must be paths or ids only.")
        if SECRET_RE.search(ref):
            raise ReceiptValidationError("receipt evidence_refs contain secret-like text.")
    for field in ("objective", "summary", "input_ref", "backlog_ref"):
        value = receipt.get(field, "")
        if isinstance(value, str) and SECRET_RE.search(value):
            raise ReceiptValidationError(f"receipt {field} contains secret-like text.")


def append_receipt(path: Path, receipt: dict[str, Any]) -> None:
    validate_receipt(receipt)
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a") as handle:
        handle.write(json.dumps(receipt, sort_keys=True) + "\n")


def read_receipts(path: Path) -> list[dict[str, Any]]:
    if not path.exists():
        return []
    receipts = []
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        if not line.strip():
            continue
        try:
            receipt = json.loads(line)
        except json.JSONDecodeError as error:
            raise ReceiptValidationError(f"{path}:{line_number}: invalid JSON") from error
        validate_receipt(receipt)
        receipts.append(receipt)
    return receipts


def summarize_receipts(path: Path, backlog_ref: str = "") -> dict[str, Any]:
    receipts = read_receipts(path)
    if backlog_ref:
        receipts = [receipt for receipt in receipts if receipt["backlog_ref"] == backlog_ref]
    provider_counts: dict[str, Counter[str]] = defaultdict(Counter)
    provider_statuses: dict[str, Counter[str]] = defaultdict(Counter)
    verdicts: Counter[str] = Counter()
    worktrees: Counter[str] = Counter()
    for receipt in receipts:
        provider_counts[receipt["provider_target"]][receipt["attempt_status"]] += 1
        provider_statuses[receipt["provider_target"]][receipt["provider_status"]] += 1
        verdicts[receipt["lead_verdict"]] += 1
        worktrees[receipt["worktree_id"]] += 1
    return {
        "total": len(receipts),
        "backlog_ref": backlog_ref,
        "providers": {provider: dict(counts) for provider, counts in provider_counts.items()},
        "provider_statuses": {
            provider: dict(counts) for provider, counts in provider_statuses.items()
        },
        "lead_verdicts": dict(verdicts),
        "worktrees": dict(worktrees),
    }


def system_harness_kit_dir() -> Path:
    configured = os.environ.get("HARNESS_KIT_HOME")
    if configured:
        return Path(configured).expanduser()
    return Path.home() / ".harness-kit"


def resolve_roster_path(
    *,
    repo: Path | None = None,
    system_home: Path | None = None,
    configured: str | None = None,
) -> Path:
    configured = (
        configured
        or os.environ.get("HARNESS_KIT_ROSTER")
        or os.environ.get("HARNESS_KIT_ROSTER_PATH")
    )
    if configured:
        return Path(configured).expanduser()

    local = (repo if repo is not None else repo_root()) / ".harness-kit" / "agents.yaml"
    if local.exists():
        return local

    system = (system_home if system_home is not None else system_harness_kit_dir()) / "agents.yaml"
    if system.exists():
        return system

    return local


def default_roster_path() -> Path:
    return resolve_roster_path()


def default_receipt_path() -> Path:
    configured = (
        os.environ.get("HARNESS_KIT_RECEIPTS")
        or os.environ.get("HARNESS_KIT_RECEIPT_PATH")
    )
    if configured:
        return Path(configured).expanduser()

    return repo_root() / ".harness-kit" / "traces" / "delegations.jsonl"


def parse_common_args(description: str) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("--roster", type=Path, default=default_roster_path())
    parser.add_argument("--receipt-output", type=Path, default=default_receipt_path())
    return parser
