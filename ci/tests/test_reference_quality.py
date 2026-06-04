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

        self.assertIn('jq -n --rawfile input "$EVIDENCE_DIR/script.txt"', content)
        self.assertNotIn('"input": "\'"$(cat "$EVIDENCE_DIR/script.txt")"\'"', content)

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

    def test_qa_per_commit_grader_self_test(self) -> None:
        grader = REPO_ROOT / "skills/qa/evals/graders/check-per-commit-lane.py"
        result = subprocess.run(
            ["python3", str(grader), "self-test"],
            capture_output=True,
            cwd=REPO_ROOT,
        )

        self.assertEqual(result.returncode, 0, result.stderr.decode())

    def test_qa_per_commit_grader_modes_cover_fixtures(self) -> None:
        grader = REPO_ROOT / "skills/qa/evals/graders/check-per-commit-lane.py"
        browser_case = self.read(
            "skills/qa/evals/cases/commit-browser-missing-selector.md"
        )
        cli_case = self.read("skills/qa/evals/cases/commit-cli-non-browser.md")

        self.assertIn("Upgrade plan button is visible", browser_case)
        self.assertIn("Status is `fail` or `inconclusive`, not `pass`", browser_case)
        self.assertIn("CLI/library-shaped change", cli_case)
        self.assertIn("Does not force Playwright", cli_case)

        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            browser_ok = tmp_path / "browser-ok.txt"
            browser_ok.write_text(
                "Status: fail\n"
                "Tool: browser\n"
                "Route: http://127.0.0.1:3000/billing\n"
                "Evidence: .evidence/feat-billing/2026-06-04/browser.png\n"
                "Transcript: .evidence/feat-billing/2026-06-04/route-selection.md\n"
                "Report: .evidence/feat-billing/2026-06-04/qa-report.md\n"
                "Assertion: Upgrade plan button missing.\n"
            )
            browser_bad = tmp_path / "browser-bad.txt"
            browser_bad.write_text(
                "Status: pass\n"
                "Tool: browser\n"
                "Route: http://127.0.0.1:3000/billing\n"
                "Evidence: .evidence/feat-billing/2026-06-04/browser.png\n"
                "Transcript: .evidence/feat-billing/2026-06-04/route-selection.md\n"
                "Report: .evidence/feat-billing/2026-06-04/qa-report.md\n"
                "Assertion: Upgrade plan button missing.\n"
            )
            cli_ok = tmp_path / "cli-ok.txt"
            cli_ok.write_text(
                "App shape: CLI\n"
                "Commands: acme --help; acme render input.yaml; missing-file check\n"
                "Evidence: .evidence/feat-cli/2026-06-04/terminal-transcript.txt\n"
            )
            cli_bad = tmp_path / "cli-bad.txt"
            cli_bad.write_text(
                "App shape: CLI\n"
                "Use browser-use and capture a screenshot.\n"
            )

            browser_ok_result = subprocess.run(
                ["python3", str(grader), "browser-missing-selector", str(browser_ok)],
                capture_output=True,
                cwd=REPO_ROOT,
            )
            browser_bad_result = subprocess.run(
                ["python3", str(grader), "browser-missing-selector", str(browser_bad)],
                capture_output=True,
                cwd=REPO_ROOT,
            )
            cli_ok_result = subprocess.run(
                ["python3", str(grader), "non-browser", str(cli_ok)],
                capture_output=True,
                cwd=REPO_ROOT,
            )
            cli_bad_result = subprocess.run(
                ["python3", str(grader), "non-browser", str(cli_bad)],
                capture_output=True,
                cwd=REPO_ROOT,
            )

        self.assertEqual(browser_ok_result.returncode, 0, browser_ok_result.stderr.decode())
        self.assertNotEqual(browser_bad_result.returncode, 0)
        self.assertEqual(cli_ok_result.returncode, 0, cli_ok_result.stderr.decode())
        self.assertNotEqual(cli_bad_result.returncode, 0)


if __name__ == "__main__":
    unittest.main()
