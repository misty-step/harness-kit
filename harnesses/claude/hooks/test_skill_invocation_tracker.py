#!/usr/bin/env python3
"""Tests for skill-invocation-tracker.py PostToolUse hook."""

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HOOK_PATH = Path(__file__).parent / "skill-invocation-tracker.py"


class TestSkillInvocationTracker(unittest.TestCase):
    """Test the skill invocation tracker hook."""

    def setUp(self):
        """Create a temp file for the JSONL log."""
        self.tmpdir = tempfile.mkdtemp()
        self.log_path = os.path.join(self.tmpdir, "skill-invocations.jsonl")

    def tearDown(self):
        """Clean up temp files."""
        if os.path.exists(self.log_path):
            os.unlink(self.log_path)
        os.rmdir(self.tmpdir)

    def _run_hook(self, stdin_data: str) -> subprocess.CompletedProcess:
        """Run the hook with given stdin, overriding LOG_PATH via env."""
        env = os.environ.copy()
        env["SKILL_TRACKER_LOG_PATH"] = self.log_path
        return subprocess.run(
            [sys.executable, str(HOOK_PATH)],
            input=stdin_data,
            capture_output=True,
            text=True,
            env=env,
        )

    def test_skill_invocation_appends_jsonl_entry(self):
        """A Skill tool_name with valid input appends one JSONL line."""
        payload = json.dumps({
            "tool_name": "Skill",
            "tool_input": {"skill": "commit", "args": "-m fix"},
            "session_id": "abc",
            "cwd": "/tmp/myproject",
        })
        result = self._run_hook(payload)

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")

        with open(self.log_path) as f:
            lines = f.readlines()
        self.assertEqual(len(lines), 1)

        entry = json.loads(lines[0])
        self.assertEqual(entry["skill"], "commit")
        self.assertEqual(entry["args"], "-m fix")
        self.assertEqual(entry["session_id"], "abc")
        self.assertEqual(entry["cwd"], "/tmp/myproject")
        self.assertEqual(entry["project"], "myproject")
        self.assertEqual(entry["harness"], "claude")
        self.assertIn("ts", entry)

    def test_optional_usage_fields_pass_through_when_available(self):
        """Optional model, outcome, duration, and usage fields are preserved."""
        payload = json.dumps({
            "tool_name": "Skill",
            "tool_input": {"skill": "qa", "args": ""},
            "session_id": "abc",
            "cwd": "/tmp/myproject",
            "model_id": "claude-opus-4-7",
            "outcome": "succeeded",
            "duration_ms": 1200,
            "usage": {
                "input_tokens": 10,
                "output_tokens": 5,
                "total_tokens": 15,
                "cost_usd": 0.001,
                "cost_source": "provider_reported",
            },
        })
        result = self._run_hook(payload)

        self.assertEqual(result.returncode, 0)
        with open(self.log_path) as f:
            entry = json.loads(f.readline())
        self.assertEqual(entry["model_id"], "claude-opus-4-7")
        self.assertEqual(entry["outcome"], "succeeded")
        self.assertEqual(entry["duration_ms"], 1200)
        self.assertEqual(entry["usage"]["total_tokens"], 15)

    def test_non_skill_tool_ignored(self):
        """Non-Skill tool_name exits 0 with no output and no log entry."""
        payload = json.dumps({
            "tool_name": "Bash",
            "tool_input": {"command": "ls"},
        })
        result = self._run_hook(payload)

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertFalse(os.path.exists(self.log_path))

    def test_empty_stdin_exits_gracefully(self):
        """Empty stdin exits 0 with no output."""
        result = self._run_hook("")

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertFalse(os.path.exists(self.log_path))

    def test_invalid_json_exits_gracefully(self):
        """Invalid JSON exits 0 with no output."""
        result = self._run_hook("not json at all")

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertFalse(os.path.exists(self.log_path))

    def test_skill_without_skill_name_ignored(self):
        """Skill tool_name but empty skill field exits 0 with no log."""
        payload = json.dumps({
            "tool_name": "Skill",
            "tool_input": {"skill": "", "args": ""},
            "session_id": "abc",
            "cwd": "/tmp/myproject",
        })
        result = self._run_hook(payload)

        self.assertEqual(result.returncode, 0)
        self.assertEqual(result.stdout, "")
        self.assertFalse(os.path.exists(self.log_path))

    def test_multiple_invocations_append(self):
        """Multiple Skill invocations append multiple lines."""
        for skill_name in ["commit", "review", "investigate"]:
            payload = json.dumps({
                "tool_name": "Skill",
                "tool_input": {"skill": skill_name, "args": ""},
                "session_id": "sess1",
                "cwd": "/tmp/proj",
            })
            result = self._run_hook(payload)
            self.assertEqual(result.returncode, 0)

        with open(self.log_path) as f:
            lines = f.readlines()
        self.assertEqual(len(lines), 3)

        skills = [json.loads(line)["skill"] for line in lines]
        self.assertEqual(skills, ["commit", "review", "investigate"])

    def test_no_stdout_ever(self):
        """Hook must never produce stdout -- it's passive telemetry."""
        payload = json.dumps({
            "tool_name": "Skill",
            "tool_input": {"skill": "test", "args": ""},
            "session_id": "x",
            "cwd": "/tmp/p",
        })
        result = self._run_hook(payload)
        self.assertEqual(result.stdout, "")


if __name__ == "__main__":
    unittest.main()
