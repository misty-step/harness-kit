import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]


class ReferenceQualityTests(unittest.TestCase):
    def read(self, relative_path: str) -> str:
        return (REPO_ROOT / relative_path).read_text()

    def test_pr_evidence_upload_uses_executable_pr_number_variable(self) -> None:
        content = self.read("skills/demo/references/pr-evidence-upload.md")

        self.assertIn("PR_NUMBER=123", content)
        self.assertNotIn("qa-evidence-pr-{NUMBER}", content)
        self.assertNotIn("gh pr comment {NUMBER}", content)
        self.assertIn('grep -F -- "qa-evidence-pr-${PR_NUMBER}"', content)

    def test_tts_narration_uses_jq_for_script_json(self) -> None:
        content = self.read("skills/demo/references/tts-narration.md")

        self.assertIn("jq -n --rawfile input /tmp/demo-slug/script.txt", content)
        self.assertNotIn('"input": "\'"$(cat /tmp/demo-slug/script.txt)"\'"', content)

    def test_reflect_grep_uses_end_of_options_before_pattern(self) -> None:
        content = self.read("skills/reflect/scripts/gather_evidence.sh")

        self.assertIn("grep -l -- 'dev.*server\\|\"dev\"'", content)

    def test_delegate_reference_conflict_bullet_is_complete(self) -> None:
        content = self.read("skills/research/references/delegate.md")

        self.assertIn("recommend which evidence to trust", content)
        self.assertIn("```text\nPhase 1", content)

    def test_deliver_receipt_uses_canonical_code_review_phase_name(self) -> None:
        content = self.read("skills/deliver/references/receipt.md")

        self.assertIn('"name": "code-review"', content)
        self.assertIn('"code-review: 2 blocking findings in auth.py"', content)
        self.assertIn("## State Lifecycle\n\n```text\n", content)
        self.assertNotIn('"name": "review"', content)

    def test_qa_grader_rejects_browser_and_accepts_cli_evidence(self) -> None:
        grader = REPO_ROOT / "skills/qa/evals/graders/check.sh"
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            passing = tmp_path / "passing.txt"
            passing.write_text(
                "CLI help covers malformed transcript evidence and tests pass\n"
            )
            failing = tmp_path / "failing.txt"
            failing.write_text(
                "CLI help covers malformed transcript evidence and tests pass; "
                "browser smoke mentioned\n"
            )

            ok = subprocess.run([str(grader), str(passing)], capture_output=True)
            bad = subprocess.run([str(grader), str(failing)], capture_output=True)

        self.assertEqual(ok.returncode, 0, ok.stderr.decode())
        self.assertNotEqual(bad.returncode, 0)


if __name__ == "__main__":
    unittest.main()
