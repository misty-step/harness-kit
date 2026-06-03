import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CLASSIFY_PATH = ROOT / "skills" / "skillify" / "scripts" / "classify-conversation.py"


def load_classifier():
    spec = importlib.util.spec_from_file_location("classify_conversation", CLASSIFY_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SkillifyClassifyTest(unittest.TestCase):
    def test_selects_two_non_manual_providers_and_builds_dispatch_commands(self):
        classifier = load_classifier()
        roster = {
            "providers": {
                "codex": {"tier": "primary", "kind": "cli"},
                "pi": {"tier": "primary", "kind": "cli"},
                "manual": {"tier": "manual", "kind": "manual"},
                "grok-build": {"tier": "disabled", "kind": "cli"},
            }
        }
        providers = classifier.select_providers(roster, requested=["codex", "pi"], minimum=2)
        self.assertEqual(providers, ["codex", "pi"])

        with tempfile.TemporaryDirectory() as tmp:
            commands = classifier.build_dispatch_commands(
                repo_root=ROOT,
                prompt_file=Path(tmp) / "prompt.md",
                providers=providers,
                input_ref="packet.json",
                backlog_ref="075",
                timeout_s=30,
            )

        self.assertEqual(len(commands), 2)
        self.assertTrue(all("scripts/dispatch-agent.py" in command for command in commands))
        self.assertTrue(all("--backlog-ref 075" in command for command in commands))


if __name__ == "__main__":
    unittest.main()
