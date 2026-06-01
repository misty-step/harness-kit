import importlib.util
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CHECK_FRONTMATTER_PATH = REPO_ROOT / "scripts" / "check-frontmatter.py"


def _load_check_frontmatter_module():
    spec = importlib.util.spec_from_file_location(
        "check_frontmatter", CHECK_FRONTMATTER_PATH
    )
    assert spec is not None
    assert spec.loader is not None
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class TriggerContractTests(unittest.TestCase):
    def test_explicit_triggers_strip_alias_notes(self) -> None:
        checker = _load_check_frontmatter_module()
        description = """\
Use when: "commit and push".
Trigger: /yeet, /ship-local (alias).
"""

        self.assertEqual(
            checker.explicit_triggers(description), ["/yeet", "/ship-local"]
        )

    def test_use_when_phrases_are_trigger_claims(self) -> None:
        checker = _load_check_frontmatter_module()
        description = """\
Use when: "ship it", "finish this ticket".
Trigger: /ship.
"""

        self.assertEqual(
            checker.trigger_claims(description),
            ["/ship", "ship it", "finish this ticket"],
        )

    def test_use_when_phrases_ignore_nonrouting_quotes(self) -> None:
        checker = _load_check_frontmatter_module()
        description = """\
End-to-end "ship it to the remote" is descriptive prose.
Use when: "yeet", "commit and push".
Trigger: /yeet.
"""

        self.assertEqual(
            checker.trigger_claims(description), ["/yeet", "yeet", "commit and push"]
        )

    def test_collision_key_unifies_slash_hyphen_and_phrase_forms(self) -> None:
        checker = _load_check_frontmatter_module()

        self.assertEqual(checker.collision_key("/ship-it"), "ship it")
        self.assertEqual(checker.collision_key("ship it"), "ship it")

    def test_trigger_contract_warns_on_missing_trigger(self) -> None:
        checker = _load_check_frontmatter_module()

        errors, warnings = checker.check_trigger_contracts(
            [
                (
                    "skills/example/SKILL.md",
                    {"description": 'Use when: "example".'},
                )
            ]
        )

        self.assertEqual(errors, [])
        self.assertEqual(
            warnings,
            ["skills/example/SKILL.md: missing Trigger definition in description"],
        )

    def test_trigger_contract_rejects_duplicate_claims(self) -> None:
        checker = _load_check_frontmatter_module()

        errors, warnings = checker.check_trigger_contracts(
            [
                (
                    "skills/ship/SKILL.md",
                    {"description": 'Use when: "ship it".\nTrigger: /ship.'},
                ),
                (
                    "skills/yeet/SKILL.md",
                    {"description": 'Use when: "ship it".\nTrigger: /yeet.'},
                ),
            ]
        )

        self.assertEqual(warnings, [])
        self.assertEqual(
            errors,
            [
                "trigger claim collision 'ship it': "
                "skills/ship/SKILL.md, skills/yeet/SKILL.md"
            ],
        )

    def test_trigger_contract_rejects_slash_phrase_collisions(self) -> None:
        checker = _load_check_frontmatter_module()

        errors, warnings = checker.check_trigger_contracts(
            [
                (
                    "skills/deploy/SKILL.md",
                    {"description": 'Use when: "deploy".\nTrigger: /ship-it.'},
                ),
                (
                    "skills/ship/SKILL.md",
                    {"description": 'Use when: "ship it".\nTrigger: /ship.'},
                ),
            ]
        )

        self.assertEqual(warnings, [])
        self.assertEqual(
            errors,
            [
                "trigger claim collision 'ship it': "
                "skills/deploy/SKILL.md, skills/ship/SKILL.md"
            ],
        )

    def test_frontmatter_parser_ignores_body_horizontal_rules(self) -> None:
        checker = _load_check_frontmatter_module()
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "SKILL.md"
            path.write_text(
                """\
---
name: demo
description: |
  Use when: "demo".
  Trigger: /demo.
---

# Body

---

Body horizontal rule.
"""
            )

            self.assertEqual(checker.check_frontmatter(path), [])
            self.assertEqual(checker.load_frontmatter(path)["name"], "demo")


if __name__ == "__main__":
    unittest.main()
