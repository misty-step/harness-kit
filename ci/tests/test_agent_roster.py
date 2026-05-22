import json
import tempfile
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
    load_roster,
    summarize_receipts,
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
        self.assertEqual(summary["lead_verdicts"]["accepted"], 1)


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
