import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CRUD_PATH = ROOT / "skills" / "skillify" / "scripts" / "skill-crud.py"


def load_crud():
    spec = importlib.util.spec_from_file_location("skill_crud", CRUD_PATH)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class SkillCrudTest(unittest.TestCase):
    def test_create_read_update_delete_validate_round_trip(self):
        crud = load_crud()
        with tempfile.TemporaryDirectory() as tmp:
            skills_root = Path(tmp) / "skills"
            created = crud.create_skill(
                skills_root,
                "demo-skill",
                "Demo skill for repeatable transcript extraction. Use when: \"demo skillify\". Trigger: /demo-skill.",
                "## Contract\n\nUse portable filesystem instructions with fallback commands.\n",
            )

            self.assertEqual(created["name"], "demo-skill")
            self.assertEqual(created["path"], str((skills_root / "demo-skill").resolve()))
            self.assertTrue((skills_root / "demo-skill" / "SKILL.md").exists())

            validation = crud.validate_skill(skills_root, "demo-skill")
            self.assertEqual(validation["status"], "valid")

            read = crud.read_skill(skills_root, "demo-skill")
            self.assertIn("SKILL.md", read["files"])
            self.assertIn("/demo-skill", read["skill_md"])

            updated = crud.update_skill(
                skills_root,
                "demo-skill",
                body="## Contract\n\nUpdated portable instructions with fallback commands.\n",
            )
            self.assertEqual(updated["status"], "updated")
            self.assertIn("Updated portable", crud.read_skill(skills_root, "demo-skill")["skill_md"])

            deleted = crud.delete_skill(skills_root, "demo-skill")
            self.assertEqual(deleted["status"], "deleted")
            self.assertFalse((skills_root / "demo-skill").exists())

    def test_rejects_path_traversal_and_harness_specific_ops_without_fallback(self):
        crud = load_crud()
        with tempfile.TemporaryDirectory() as tmp:
            skills_root = Path(tmp) / "skills"
            with self.assertRaises(ValueError):
                crud.create_skill(skills_root, "../escape", "bad", "bad")

            crud.create_skill(
                skills_root,
                "bad-skill",
                "Bad skill. Use when: \"bad skill\". Trigger: /bad-skill.",
                "Call SendUserMessage directly.",
            )
            with self.assertRaises(ValueError):
                crud.validate_skill(skills_root, "bad-skill")


if __name__ == "__main__":
    unittest.main()
