import importlib.util
import json
import os
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
PARSE_PATH = ROOT / "skills" / "skillify" / "scripts" / "parse-transcript.py"


def load_parser():
    spec = importlib.util.spec_from_file_location("parse_transcript", PARSE_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SkillifyParseTranscriptTest(unittest.TestCase):
    def test_parses_claude_jsonl_into_instruction_packet(self):
        parser = load_parser()
        rows = [
            {"type": "user", "message": {"role": "user", "content": "Make this workflow reusable."}},
            {
                "type": "assistant",
                "message": {
                    "role": "assistant",
                    "content": [
                        {"type": "text", "text": "Use a local SKILL.md and avoid harness-only tools."}
                    ],
                },
            },
            {"type": "tool_result", "content": "ignored"},
            "not json",
        ]
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "claude.jsonl"
            path.write_text("\n".join(json.dumps(row) if not isinstance(row, str) else row for row in rows))

            packet = parser.parse_transcript(path)

        self.assertEqual(packet["source"], str(path))
        self.assertEqual(packet["evidence"]["turn_count"], 2)
        self.assertEqual(packet["evidence"]["malformed_count"], 1)
        self.assertEqual(packet["turns"][0]["role"], "user")
        self.assertIn("local SKILL.md", packet["candidate_instructions"][0])

    def test_from_current_selects_latest_transcript_for_project(self):
        parser = load_parser()
        with tempfile.TemporaryDirectory() as tmp:
            projects_dir = Path(tmp) / "projects"
            current_project = projects_dir / "-Users-phaedrus-Development-harness-kit"
            other_project = projects_dir / "-Users-phaedrus-Development-other"
            current_project.mkdir(parents=True)
            other_project.mkdir(parents=True)
            older = current_project / "older.jsonl"
            newer = current_project / "newer.jsonl"
            unrelated = other_project / "unrelated.jsonl"
            older.write_text(json.dumps({"type": "user", "message": {"role": "user", "content": "old"}}))
            newer.write_text(json.dumps({"type": "user", "message": {"role": "user", "content": "new"}}))
            unrelated.write_text(json.dumps({"type": "user", "message": {"role": "user", "content": "other"}}))
            os.utime(older, (1, 1))
            os.utime(unrelated, (3, 3))
            os.utime(newer, (2, 2))

            resolved = parser.resolve_transcript(
                None,
                from_current=True,
                projects_dir=projects_dir,
                cwd=Path("/Users/phaedrus/Development/harness-kit"),
            )

        self.assertEqual(resolved.name, "newer.jsonl")

    def test_from_current_rejects_unrelated_project_transcript(self):
        parser = load_parser()
        with tempfile.TemporaryDirectory() as tmp:
            projects_dir = Path(tmp) / "projects"
            other_project = projects_dir / "-Users-phaedrus-Development-other"
            other_project.mkdir(parents=True)
            (other_project / "unrelated.jsonl").write_text(
                json.dumps({"type": "user", "message": {"role": "user", "content": "other"}})
            )

            with self.assertRaises(FileNotFoundError):
                parser.resolve_transcript(
                    None,
                    from_current=True,
                    projects_dir=projects_dir,
                    cwd=Path("/Users/phaedrus/Development/harness-kit"),
                )


if __name__ == "__main__":
    unittest.main()
