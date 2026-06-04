from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "scripts" / "mine-transcript-effectiveness.py"


def load_module():
    spec = importlib.util.spec_from_file_location("mine_transcript_effectiveness", SCRIPT)
    module = importlib.util.module_from_spec(spec)
    assert spec and spec.loader
    spec.loader.exec_module(module)
    return module


class TranscriptEffectivenessMiningTest(unittest.TestCase):
    def test_no_default_broad_ingestion(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--format", "json"],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("provide at least one --transcript", result.stderr)

    def test_report_redacts_and_joins_without_raw_excerpts(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            transcript = root / "session.jsonl"
            transcript.write_text(
                "\n".join(
                    [
                        json.dumps(
                            {
                                "type": "user",
                                "sessionId": "sess-091",
                                "gitBranch": "feat/091",
                                "cwd": "/tmp/harness-kit",
                                "message": {"role": "user", "content": "This is wrong, use /reflect instead."},
                            }
                        ),
                        json.dumps(
                            {
                                "type": "assistant",
                                "sessionId": "sess-091",
                                "backlog_ref": "091",
                                "work_id": "work-091",
                                "cwd": "/tmp/harness-kit",
                                "message": {
                                    "role": "assistant",
                                    "content": "Tool failed. Skill reflect succeeded. Authorization: Bearer sk-test_1234567890abcdef /Users/alice/project",
                                },
                            }
                        ),
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            skill_log = root / "skills.jsonl"
            skill_log.write_text(json.dumps({"session_id": "sess-091", "skill": "reflect", "project": "harness-kit"}) + "\n")
            work_ledger = root / "work.jsonl"
            work_ledger.write_text(json.dumps({"work_id": "work-091", "owning_skill": "deliver"}) + "\n")
            delegations = root / "delegations.jsonl"
            delegations.write_text(json.dumps({"backlog_ref": "091", "delegation_id": "del-091"}) + "\n")
            review_scores = root / "review.ndjson"
            review_scores.write_text(json.dumps({"branch": "feat/091", "correctness": 8}) + "\n")

            args = module.build_parser().parse_args(
                [
                    "--transcript",
                    str(transcript),
                    "--skill-log",
                    str(skill_log),
                    "--work-ledger",
                    str(work_ledger),
                    "--delegations",
                    str(delegations),
                    "--review-scores",
                    str(review_scores),
                    "--format",
                    "json",
                ]
            )
            report = module.build_report(args)
            rendered = module.render_markdown(report)

            self.assertEqual(report["categories"]["user_corrections"]["count"], 1)
            self.assertEqual(report["categories"]["successful_skill_usage"]["count"], 2)
            self.assertEqual(report["joins"]["skill_invocations"]["matched"], 1)
            self.assertEqual(report["joins"]["work_ledger"]["matched"], 1)
            self.assertEqual(report["joins"]["delegations"]["matched"], 1)
            self.assertEqual(report["joins"]["review_scores"]["matched"], 1)
            self.assertGreaterEqual(report["redaction_summary"]["redacted_segments"], 1)
            payload = json.dumps(report)
            self.assertNotIn("sk-test", payload)
            self.assertNotIn("Authorization: Bearer", rendered)
            self.assertNotIn("This is wrong", rendered)

    def test_unresolved_secret_like_content_fails_closed(self):
        module = load_module()
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            transcript = root / "unsafe.jsonl"
            transcript.write_text(
                json.dumps({"message": {"role": "user", "content": "private_customer_data"}}) + "\n",
                encoding="utf-8",
            )
            args = module.build_parser().parse_args(["--transcript", str(transcript)])
            with self.assertRaises(SystemExit):
                module.build_report(args)


if __name__ == "__main__":
    unittest.main()
