import sys
from pathlib import Path
from tempfile import TemporaryDirectory
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src" / "harness_kit_ci"))

from heal_support import snapshot_delta  # noqa: E402


class SnapshotDeltaTests(unittest.TestCase):
    def test_reports_only_paths_changed_after_snapshot(self) -> None:
        with TemporaryDirectory() as before_dir, TemporaryDirectory() as after_dir:
            before = Path(before_dir)
            after = Path(after_dir)

            (before / "same.txt").write_text("same\n")
            (after / "same.txt").write_text("same\n")

            (before / "changed.txt").write_text("before\n")
            (after / "changed.txt").write_text("after\n")

            (before / "removed.txt").write_text("gone\n")
            (after / "added.txt").write_text("new\n")

            stage, remove = snapshot_delta(before, after)

            self.assertEqual(stage, ["added.txt", "changed.txt"])
            self.assertEqual(remove, ["removed.txt"])

    def test_ignores_dotenv_and_git_metadata(self) -> None:
        with TemporaryDirectory() as before_dir, TemporaryDirectory() as after_dir:
            before = Path(before_dir)
            after = Path(after_dir)

            (before / ".env").write_text("before\n")
            (after / ".env").write_text("after\n")

            (before / ".git").mkdir()
            (before / ".git" / "HEAD").write_text("ref: refs/heads/main\n")
            (after / ".git").mkdir()
            (after / ".git" / "HEAD").write_text("ref: refs/heads/heal\n")

            stage, remove = snapshot_delta(before, after)

            self.assertEqual(stage, [])
            self.assertEqual(remove, [])


if __name__ == "__main__":
    unittest.main()
