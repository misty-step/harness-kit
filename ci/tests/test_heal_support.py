import sys
from pathlib import Path
import unittest

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "src" / "harness_kit_ci"))

from heal_support import (  # noqa: E402
    HEALABLE_GATES,
    GateFailure,
    first_failed_gate,
    parse_check_failures,
    repair_branch_name,
    repair_commit_message,
    select_healable_failure,
)


class ParseCheckFailuresTests(unittest.TestCase):
    def test_returns_all_failed_gate_details(self) -> None:
        summary = """Harness Kit CI Results
========================================
  PASS  lint-yaml
  FAIL  lint-shell
         first detail
         second detail
  FAIL  check-frontmatter
         missing field
========================================
1 passed, 2 failed
"""

        failures = parse_check_failures(summary)

        self.assertEqual(
            failures,
            [
                GateFailure(name="lint-shell", detail="first detail\nsecond detail"),
                GateFailure(name="check-frontmatter", detail="missing field"),
            ],
        )

    def test_uses_default_detail_when_stderr_missing(self) -> None:
        summary = """Harness Kit CI Results
========================================
  FAIL  lint-python
========================================
0 passed, 1 failed
"""

        failures = parse_check_failures(summary)

        self.assertEqual(
            failures,
            [GateFailure(name="lint-python", detail="No stderr captured.")],
        )

    def test_first_failed_gate_returns_first_failure_name(self) -> None:
        summary = """Harness Kit CI Results
========================================
  PASS  lint-yaml
  FAIL  lint-shell
         first detail
  FAIL  lint-python
         second detail
========================================
1 passed, 2 failed
"""

        self.assertEqual(first_failed_gate(summary), "lint-shell")

    def test_first_failed_gate_returns_none_when_summary_has_no_failures(self) -> None:
        summary = """Harness Kit CI Results
========================================
  PASS  lint-yaml
========================================
1 passed, 0 failed
"""

        self.assertIsNone(first_failed_gate(summary))


class SelectHealableFailureTests(unittest.TestCase):
    def test_accepts_single_supported_failure(self) -> None:
        failure = GateFailure(name="lint-yaml", detail="bad yaml")

        selected = select_healable_failure([failure])

        self.assertEqual(selected, failure)

    def test_rejects_no_failures(self) -> None:
        with self.assertRaisesRegex(ValueError, "at least one failing gate"):
            select_healable_failure([])

    def test_rejects_unsupported_gate(self) -> None:
        with self.assertRaisesRegex(ValueError, "Supported gates"):
            select_healable_failure([GateFailure(name="test-bun", detail="boom")])

    def test_rejects_multiple_failures(self) -> None:
        failures = [
            GateFailure(name="lint-shell", detail="bad shell"),
            GateFailure(name="lint-python", detail="bad python"),
        ]

        with self.assertRaisesRegex(ValueError, "one failing gate at a time"):
            select_healable_failure(failures)


class RepairMetadataTests(unittest.TestCase):
    def test_commit_message_is_semantic(self) -> None:
        self.assertEqual(repair_commit_message("lint-shell"), "ci: heal lint-shell")

    def test_branch_name_includes_gate_slug(self) -> None:
        branch = repair_branch_name("Check Frontmatter")

        self.assertRegex(branch, r"^heal/check-frontmatter-\d{14}$")

    def test_healable_gate_list_covers_expected_gates(self) -> None:
        self.assertEqual(
            set(HEALABLE_GATES),
            {"lint-yaml", "lint-shell", "lint-python", "check-frontmatter"},
        )


if __name__ == "__main__":
    unittest.main()
