import json
import os
import shlex
import stat
import tempfile
import textwrap
import time
import unittest
from pathlib import Path

import yaml

import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[2] / "scripts" / "lib"))

from agent_roster import (  # noqa: E402
    ROSTER_PROVIDER_IDS,
    ReceiptValidationError,
    append_receipt,
    build_attempt_receipt,
    build_probe_receipts,
    dispatch_provider_lane,
    load_roster,
    summarize_receipts,
    resolve_roster_path,
    validate_receipt,
    validate_roster,
)

REPO_ROOT = Path(__file__).resolve().parents[2]


class RosterValidationTests(unittest.TestCase):
    def test_committed_roster_declares_required_providers(self) -> None:
        roster = load_roster(REPO_ROOT / ".spellbook/agents.yaml")

        validate_roster(roster)

        self.assertEqual(set(roster["providers"]), ROSTER_PROVIDER_IDS)
        self.assertEqual(roster["providers"]["codex"]["tier"], "primary")
        self.assertEqual(roster["providers"]["manual"]["kind"], "manual")

    def test_agy_print_flag_is_last_before_prompt(self) -> None:
        roster = load_roster(REPO_ROOT / ".spellbook/agents.yaml")

        for field in ("dispatch", "smoke"):
            parts = shlex.split(roster["providers"]["agy"][field])
            self.assertIn("--print", parts)
            self.assertIn("--print-timeout", parts)
            self.assertLess(parts.index("--print-timeout"), parts.index("--print"))
            self.assertEqual(parts[-1], "--print")

    def test_rejects_secret_like_command_values(self) -> None:
        roster = {
            "version": 1,
            "providers": {
                provider: {
                    "tier": "primary" if provider in {"codex", "claude", "pi"} else "conditional",
                    "kind": "cli",
                    "probe": "echo TOKEN=abc123",
                    "dispatch": "manual",
                    "output": "text",
                    "permissions": "default",
                    "worktree": "recommended",
                    "notes": "fixture",
                }
                for provider in ROSTER_PROVIDER_IDS
            },
        }
        roster["providers"]["manual"]["tier"] = "manual"
        roster["providers"]["manual"]["kind"] = "manual"

        with self.assertRaisesRegex(ValueError, "secret-like"):
            validate_roster(roster)

    def test_roster_resolution_uses_system_fallback_when_repo_roster_is_absent(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo = root / "repo"
            system_home = root / "system-spellbook"
            repo.mkdir()
            system_home.mkdir()
            system_roster = system_home / "agents.yaml"
            system_roster.write_text("version: 1\nproviders: {}\n")

            self.assertEqual(
                resolve_roster_path(repo=repo, system_home=system_home),
                system_roster,
            )

    def test_roster_resolution_prefers_repo_roster_over_system_roster(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            repo = root / "repo"
            system_home = root / "system-spellbook"
            local_dir = repo / ".spellbook"
            local_dir.mkdir(parents=True)
            system_home.mkdir()
            local_roster = local_dir / "agents.yaml"
            system_roster = system_home / "agents.yaml"
            local_roster.write_text("version: 1\nproviders: {}\n")
            system_roster.write_text("version: 1\nproviders: {}\n")

            self.assertEqual(
                resolve_roster_path(repo=repo, system_home=system_home),
                local_roster,
            )


class ReceiptTests(unittest.TestCase):
    def test_builds_valid_unavailable_probe_receipts_for_empty_path(self) -> None:
        roster = load_roster(REPO_ROOT / ".spellbook/agents.yaml")

        receipts = build_probe_receipts(
            roster,
            path_env="",
            lead_harness="codex",
            lead_provider="codex",
            input_ref="backlog.d/068-agent-provider-roster.md",
            objective="probe fixture",
        )

        self.assertTrue(receipts)
        automated = [r for r in receipts if r["provider_target"] != "manual"]
        self.assertTrue(automated)
        self.assertTrue(all(r["provider_status"] == "unavailable" for r in automated))
        self.assertTrue(all(r["attempt_status"] == "not_started" for r in automated))
        for receipt in receipts:
            validate_receipt(receipt)

    def test_probe_executes_side_effect_free_command(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            bin_dir = Path(tmp)
            tool = bin_dir / "fake-agent"
            tool.write_text("#!/usr/bin/env sh\nexit 7\n")
            tool.chmod(tool.stat().st_mode | stat.S_IXUSR)
            roster = {
                "version": 1,
                "providers": {
                    provider: {
                        "tier": (
                            "primary"
                            if provider in {"codex", "claude", "pi"}
                            else "conditional"
                        ),
                        "kind": "cli",
                        "probe": "fake-agent --version",
                        "dispatch": "fake-agent run",
                        "output": "text",
                        "permissions": "default",
                        "worktree": "recommended",
                        "notes": "fixture",
                    }
                    for provider in ROSTER_PROVIDER_IDS
                },
            }
            roster["providers"]["manual"]["tier"] = "manual"
            roster["providers"]["manual"]["kind"] = "manual"
            roster["providers"]["manual"]["probe"] = "manual"
            roster["providers"]["manual"]["dispatch"] = "manual"
            roster["providers"]["manual"]["output"] = "manual-summary"
            roster["providers"]["manual"]["worktree"] = "not_applicable"

            receipts = build_probe_receipts(
                roster,
                path_env=str(bin_dir),
                lead_harness="codex",
                lead_provider="codex",
                input_ref=".spellbook/agents.yaml",
                objective="probe fixture",
            )

        automated = [r for r in receipts if r["provider_target"] != "manual"]
        self.assertTrue(automated)
        self.assertTrue(all(r["provider_status"] == "error" for r in automated))

    def test_manual_and_cli_attempts_share_schema(self) -> None:
        cli = build_attempt_receipt(
            provider_target="codex",
            provider_status="available",
            attempt_status="succeeded",
            objective="implementation lane",
            input_ref="backlog.d/068-agent-provider-roster.md",
            evidence_refs=[".evidence/codex.txt"],
            lead_verdict="accepted",
            worktree_id="codex-lane",
        )
        manual = build_attempt_receipt(
            provider_target="manual",
            provider_status="manual",
            attempt_status="manual",
            objective="import external GUI notes",
            input_ref="backlog.d/068-agent-provider-roster.md",
            evidence_refs=[".evidence/manual.md"],
            lead_verdict="reference_only",
            worktree_id="manual",
        )

        self.assertEqual(set(cli), set(manual))
        validate_receipt(cli)
        validate_receipt(manual)

    def test_rejects_inline_transcript_evidence(self) -> None:
        with self.assertRaises(ReceiptValidationError):
            build_attempt_receipt(
                provider_target="codex",
                provider_status="available",
                attempt_status="succeeded",
                objective="bad evidence",
                input_ref="backlog.d/068-agent-provider-roster.md",
                evidence_refs=["The full transcript was copied here."],
                lead_verdict="pending",
                worktree_id="codex-lane",
            )

    def test_append_and_summarize_receipts(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "delegations.jsonl"
            append_receipt(
                path,
                build_attempt_receipt(
                    provider_target="codex",
                    provider_status="available",
                    attempt_status="succeeded",
                    objective="lane one",
                    input_ref="backlog.d/068-agent-provider-roster.md",
                    evidence_refs=[".evidence/codex.txt"],
                    lead_verdict="accepted",
                    worktree_id="wt-a",
                ),
            )
            append_receipt(
                path,
                build_attempt_receipt(
                    provider_target="claude",
                    provider_status="available",
                    attempt_status="rejected",
                    objective="lane two",
                    input_ref="backlog.d/068-agent-provider-roster.md",
                    evidence_refs=[".evidence/claude.txt"],
                    lead_verdict="rejected",
                    worktree_id="wt-b",
                ),
            )

            lines = path.read_text().splitlines()
            self.assertEqual(len(lines), 2)
            self.assertEqual(json.loads(lines[0])["provider_target"], "codex")

            summary = summarize_receipts(path)

        self.assertEqual(summary["providers"]["codex"]["succeeded"], 1)
        self.assertEqual(summary["providers"]["claude"]["rejected"], 1)
        self.assertEqual(summary["provider_statuses"]["codex"]["available"], 1)
        self.assertEqual(summary["lead_verdicts"]["accepted"], 1)
        self.assertEqual(summary["worktrees"]["wt-a"], 1)

    def test_summarize_receipts_filters_by_backlog_ref(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "delegations.jsonl"
            for backlog_ref, provider in (
                ("backlog.d/001-one.md", "codex"),
                ("backlog.d/002-two.md", "claude"),
            ):
                append_receipt(
                    path,
                    build_attempt_receipt(
                        provider_target=provider,
                        provider_status="available",
                        attempt_status="succeeded",
                        objective="lane",
                        input_ref=backlog_ref,
                        backlog_ref=backlog_ref,
                        evidence_refs=[".evidence/lane.txt"],
                        lead_verdict="accepted",
                        worktree_id=provider,
                    ),
                )

            summary = summarize_receipts(path, backlog_ref="backlog.d/001-one.md")

        self.assertEqual(summary["total"], 1)
        self.assertEqual(set(summary["providers"]), {"codex"})

    def test_dispatch_refuses_unavailable_provider_before_running(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            transcript_dir = root / "traces"
            receipt_path = root / "delegations.jsonl"
            roster = _fixture_roster("missing-agent --run", probe="missing-agent --version")

            receipt = dispatch_provider_lane(
                roster,
                provider_target="codex",
                prompt="hello",
                objective="unavailable provider fixture",
                input_ref="prompt.txt",
                backlog_ref="backlog.d/072-bounded-roster-lane-dispatch.md",
                transcript_dir=transcript_dir,
                receipt_output=receipt_path,
                timeout_s=1,
                grace_s=0.1,
                lead_harness="codex",
                lead_provider="codex",
                path_env="",
            )

            self.assertEqual(receipt["provider_status"], "unavailable")
            self.assertEqual(receipt["attempt_status"], "failed")
            self.assertFalse(transcript_dir.exists())
            self.assertEqual(summarize_receipts(receipt_path)["total"], 1)

    def test_dispatch_timeout_kills_process_group_and_records_transcript(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            bin_dir = root / "bin"
            bin_dir.mkdir()
            pid_file = root / "child.pid"
            fake = bin_dir / "fake-agent"
            fake.write_text(
                textwrap.dedent(
                    f"""\
                    #!{sys.executable}
                    import pathlib
                    import signal
                    import subprocess
                    import sys
                    import time
                    import warnings

                    warnings.filterwarnings("ignore", category=ResourceWarning)

                    if "--version" in sys.argv:
                        print("fake-agent 1.0")
                        raise SystemExit(0)

                    signal.signal(signal.SIGTERM, signal.SIG_IGN)
                    child = subprocess.Popen([
                        sys.executable,
                        "-c",
                        "import signal,time; signal.signal(signal.SIGTERM, signal.SIG_IGN); time.sleep(60)",
                    ])
                    pathlib.Path({str(pid_file)!r}).write_text(str(child.pid))
                    print("started child", child.pid, flush=True)
                    while True:
                        time.sleep(1)
                    """
                )
            )
            fake.chmod(fake.stat().st_mode | stat.S_IXUSR)
            receipt_path = root / "delegations.jsonl"
            transcript_dir = root / "traces"
            roster = _fixture_roster("fake-agent run", probe="fake-agent --version")

            started = time.monotonic()
            receipt = dispatch_provider_lane(
                roster,
                provider_target="codex",
                prompt="hello",
                objective="timeout fixture",
                input_ref="prompt.txt",
                backlog_ref="backlog.d/072-bounded-roster-lane-dispatch.md",
                transcript_dir=transcript_dir,
                receipt_output=receipt_path,
                timeout_s=0.2,
                grace_s=0.1,
                lead_harness="codex",
                lead_provider="codex",
                path_env=str(bin_dir),
            )
            elapsed = time.monotonic() - started

            self.assertLess(elapsed, 2)
            self.assertEqual(receipt["provider_status"], "error")
            self.assertEqual(receipt["attempt_status"], "failed")
            self.assertIn("timed out", receipt["summary"])
            self.assertEqual(len(receipt["evidence_refs"]), 1)
            transcript = Path(receipt["evidence_refs"][0])
            self.assertTrue(transcript.exists())
            self.assertIn("started child", transcript.read_text())
            self.assertEqual(summarize_receipts(receipt_path)["total"], 1)

            child_pid = int(pid_file.read_text())
            for _ in range(20):
                if not _pid_exists(child_pid):
                    break
                time.sleep(0.05)
            self.assertFalse(_pid_exists(child_pid))

    def test_dispatch_appends_prompt_after_agy_print_flag(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            bin_dir = root / "bin"
            bin_dir.mkdir()
            argv_path = root / "argv.json"
            fake = bin_dir / "fake-agy"
            fake.write_text(
                textwrap.dedent(
                    f"""\
                    #!{sys.executable}
                    import json
                    import pathlib
                    import sys

                    if "--help" in sys.argv:
                        print("Usage of agy")
                        raise SystemExit(0)

                    pathlib.Path({str(argv_path)!r}).write_text(json.dumps(sys.argv[1:]))
                    print(sys.argv[-1])
                    """
                )
            )
            fake.chmod(fake.stat().st_mode | stat.S_IXUSR)
            receipt_path = root / "delegations.jsonl"
            transcript_dir = root / "traces"
            roster = _fixture_roster(
                "fake-agy --dangerously-skip-permissions --print-timeout 10m --print",
                probe="fake-agy --help",
            )

            receipt = dispatch_provider_lane(
                roster,
                provider_target="agy",
                prompt="sentinel prompt",
                objective="agy argv fixture",
                input_ref="prompt.txt",
                transcript_dir=transcript_dir,
                receipt_output=receipt_path,
                timeout_s=1,
                grace_s=0.1,
                lead_harness="codex",
                lead_provider="codex",
                path_env=str(bin_dir),
            )

            self.assertEqual(receipt["attempt_status"], "succeeded")
            argv = json.loads(argv_path.read_text())
            self.assertEqual(argv[-2:], ["--print", "sentinel prompt"])
            self.assertLess(argv.index("--print-timeout"), argv.index("--print"))
            transcript = Path(receipt["evidence_refs"][0]).read_text()
            self.assertIn("sentinel prompt", transcript)


def _fixture_roster(dispatch: str, *, probe: str) -> dict:
    roster = {
        "version": 1,
        "providers": {
            provider: {
                "tier": "primary" if provider in {"codex", "claude", "pi"} else "conditional",
                "kind": "cli",
                "probe": probe,
                "dispatch": dispatch,
                "output": "text",
                "permissions": "default",
                "worktree": "recommended",
                "notes": "fixture",
            }
            for provider in ROSTER_PROVIDER_IDS
        },
    }
    roster["providers"]["manual"]["tier"] = "manual"
    roster["providers"]["manual"]["kind"] = "manual"
    roster["providers"]["manual"]["probe"] = "manual"
    roster["providers"]["manual"]["dispatch"] = "manual"
    roster["providers"]["manual"]["output"] = "manual-summary"
    roster["providers"]["manual"]["worktree"] = "not_applicable"
    return roster


def _pid_exists(pid: int) -> bool:
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    return True


class FixtureSyntaxTests(unittest.TestCase):
    def test_receipt_fixture_is_valid_jsonl(self) -> None:
        fixture = REPO_ROOT / ".spellbook/examples/delegation-receipt.jsonl"

        for line in fixture.read_text().splitlines():
            validate_receipt(json.loads(line))

    def test_roster_yaml_is_plain_mapping(self) -> None:
        data = yaml.safe_load((REPO_ROOT / ".spellbook/agents.yaml").read_text())

        self.assertIsInstance(data, dict)
        self.assertIn("providers", data)


if __name__ == "__main__":
    unittest.main()
