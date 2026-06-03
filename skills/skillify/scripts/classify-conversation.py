#!/usr/bin/env python3
"""Classify whether a parsed conversation is skill-worthy using roster lanes."""

from __future__ import annotations

import argparse
import json
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

import yaml


def load_roster(path: Path) -> dict[str, Any]:
    with path.open() as handle:
        roster = yaml.safe_load(handle)
    if not isinstance(roster, dict):
        raise ValueError("roster must be a YAML mapping")
    return roster


def select_providers(
    roster: dict[str, Any],
    *,
    requested: list[str] | None = None,
    minimum: int = 2,
) -> list[str]:
    providers = roster.get("providers", roster)
    if not isinstance(providers, dict):
        raise ValueError("roster providers must be a mapping")
    candidates = requested or list(providers)
    selected = []
    for provider_id in candidates:
        provider = providers.get(provider_id)
        if not isinstance(provider, dict):
            continue
        if provider.get("kind") == "manual" or provider.get("tier") in {"manual", "disabled"}:
            continue
        selected.append(provider_id)
        if len(selected) == minimum:
            break
    if len(selected) < minimum:
        raise ValueError(f"need at least {minimum} non-manual providers")
    return selected


def build_prompt(packet: dict[str, Any]) -> str:
    return (
        "Role: skillify classifier.\n"
        "Objective: evaluate whether this conversation contains a novel, repeatable, "
        "portable workflow worth turning into a first-party Harness Kit skill.\n"
        "Return JSON with fields: skill_worthy, confidence, suggested_name, "
        "novelty_reason, repeatability_reason, portability_risk.\n\n"
        + json.dumps(packet, sort_keys=True, indent=2)
    )


def build_dispatch_commands(
    *,
    repo_root: Path,
    prompt_file: Path,
    providers: list[str],
    input_ref: str,
    backlog_ref: str,
    timeout_s: int,
) -> list[str]:
    commands = []
    dispatch = repo_root / "scripts" / "dispatch-agent.py"
    for provider in providers:
        command = [
            sys.executable,
            str(dispatch),
            "--provider-target",
            provider,
            "--objective",
            "skillify novelty and repeatability classification",
            "--input-ref",
            input_ref,
            "--prompt-file",
            str(prompt_file),
            "--backlog-ref",
            backlog_ref,
            "--timeout-s",
            str(timeout_s),
        ]
        commands.append(" ".join(shlex.quote(part) for part in command))
    return commands


def classify(
    *,
    packet_path: Path,
    roster_path: Path,
    repo_root: Path,
    providers: list[str] | None,
    dry_run: bool,
    timeout_s: int,
) -> dict[str, Any]:
    packet = json.loads(packet_path.read_text())
    selected = select_providers(load_roster(roster_path), requested=providers)
    prompt = build_prompt(packet)
    with tempfile.NamedTemporaryFile("w", suffix="-skillify-prompt.md", delete=False) as handle:
        handle.write(prompt)
        prompt_file = Path(handle.name)
    commands = build_dispatch_commands(
        repo_root=repo_root,
        prompt_file=prompt_file,
        providers=selected,
        input_ref=str(packet_path),
        backlog_ref="075",
        timeout_s=timeout_s,
    )
    if dry_run:
        return {"status": "dry_run", "providers": selected, "commands": commands}
    receipts = []
    for command in commands:
        completed = subprocess.run(command, shell=True, check=False, text=True, capture_output=True)
        if completed.returncode != 0:
            raise RuntimeError(completed.stderr or completed.stdout or f"dispatch failed: {command}")
        receipts.append(json.loads(completed.stdout.splitlines()[-1]))
    return {"status": "classified", "providers": selected, "receipts": receipts}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("packet", type=Path)
    parser.add_argument("--roster", type=Path, default=Path(".harness-kit/agents.yaml"))
    parser.add_argument("--repo-root", type=Path, default=Path("."))
    parser.add_argument("--provider", action="append", dest="providers")
    parser.add_argument("--dry-run", action="store_true")
    parser.add_argument("--timeout-s", type=int, default=120)
    args = parser.parse_args(argv)
    try:
        result = classify(
            packet_path=args.packet,
            roster_path=args.roster,
            repo_root=args.repo_root,
            providers=args.providers,
            dry_run=args.dry_run,
            timeout_s=args.timeout_s,
        )
    except (ValueError, RuntimeError, json.JSONDecodeError) as error:
        print(str(error), file=sys.stderr)
        return 1
    print(json.dumps(result, sort_keys=True, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
