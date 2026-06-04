from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
SCRIPT = ROOT / "skills" / "reflect" / "scripts" / "checkpoint.py"


def checkpoint(**overrides):
    data = {
        "topic": "load-bearing-decision",
        "source_refs": ["backlog.d/096-reflect-teach-back-checkpoints.md"],
        "question": "What decision did we make, what can fail, and what happens next?",
        "operator_restatement": "We keep this opt-in, refs-only, and continue after a pass.",
        "lead_verdict": "pass",
        "gaps": [],
        "next_action": "Continue the session.",
        "timestamp": "2026-01-01T00:00:00Z",
    }
    data.update(overrides)
    return data


class ReflectCheckpointTests(unittest.TestCase):
    def test_self_test_runs(self):
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--self-test"],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("reflect checkpoint self-test ok", result.stdout)

    def test_missing_restatement_fails_validation(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "checkpoint.json"
            path.write_text(json.dumps(checkpoint(operator_restatement="")), encoding="utf-8")
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "validate", str(path)],
                text=True,
                capture_output=True,
                check=False,
            )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("operator_restatement", result.stderr)

    def test_gate_is_noop_without_matching_packet_marker(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "partial.json"
            path.write_text(
                json.dumps(checkpoint(lead_verdict="partial", gaps=["Next action unclear."])),
                encoding="utf-8",
            )
            packet = root / "packet.md"
            packet.write_text("Comprehension-required: other-topic\n", encoding="utf-8")
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "validate",
                    str(path),
                    "--gate",
                    "load-bearing-decision",
                    "--packet",
                    str(packet),
                ],
                text=True,
                capture_output=True,
                check=False,
            )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_gate_is_noop_without_packet(self):
        result = subprocess.run(
            [
                sys.executable,
                str(SCRIPT),
                "validate",
                "--gate",
                "load-bearing-decision",
            ],
            text=True,
            capture_output=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_required_partial_gate_fails(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            path = root / "partial.json"
            path.write_text(
                json.dumps(checkpoint(lead_verdict="partial", gaps=["Next action unclear."])),
                encoding="utf-8",
            )
            packet = root / "packet.md"
            packet.write_text("Comprehension-required: load-bearing-decision\n", encoding="utf-8")
            result = subprocess.run(
                [
                    sys.executable,
                    str(SCRIPT),
                    "validate",
                    str(path),
                    "--gate",
                    "load-bearing-decision",
                    "--packet",
                    str(packet),
                ],
                text=True,
                capture_output=True,
                check=False,
            )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("lead_verdict pass", result.stderr)


if __name__ == "__main__":
    unittest.main()
