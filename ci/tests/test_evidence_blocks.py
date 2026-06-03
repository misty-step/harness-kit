import sys
import textwrap
import unittest
import importlib.util
from pathlib import Path

SCRIPT_PATH = Path(__file__).resolve().parents[2] / "scripts" / "check-evidence-blocks.py"
SPEC = importlib.util.spec_from_file_location("evidence_blocks", SCRIPT_PATH)
assert SPEC is not None
evidence_blocks = importlib.util.module_from_spec(SPEC)
sys.modules["evidence_blocks"] = evidence_blocks
assert SPEC.loader is not None
SPEC.loader.exec_module(evidence_blocks)


class EvidenceBlockTests(unittest.TestCase):
    def test_valid_completion_gate_accepts_required_fields(self) -> None:
        text = textwrap.dedent(
            """
            ## Completion Gate
            - Evidence that proves it: focused test fails before the fix and passes after.
            - Exact command/path/route exercised: pytest tests/test_login.py -q.
            - Residual risk: OAuth provider outage path remains unverified.
            """
        )

        blocks = evidence_blocks.parse_evidence_blocks(Path("fixture.md"), text)

        self.assertEqual(evidence_blocks.check_block(blocks[0]), [])

    def test_completion_gate_rejects_missing_required_field(self) -> None:
        text = textwrap.dedent(
            """
            ## Completion Gate
            - Evidence that proves it: smoke test output.
            - Residual risk: none beyond provider outage.
            """
        )

        [block] = evidence_blocks.parse_evidence_blocks(Path("fixture.md"), text)
        errors = evidence_blocks.check_block(block)

        self.assertTrue(
            any("Exact command/path/route exercised" in error for error in errors)
        )

    def test_completion_gate_rejects_blank_and_placeholder_values(self) -> None:
        text = textwrap.dedent(
            """
            ## Completion Gate
            - Evidence that proves it: TBD
            - Exact command/path/route exercised:
            - Residual risk: <unknown>
            """
        )

        [block] = evidence_blocks.parse_evidence_blocks(Path("fixture.md"), text)
        errors = evidence_blocks.check_block(block)

        self.assertEqual(len(errors), 3)
        self.assertTrue(all("placeholder" in error for error in errors))

    def test_valid_acceptance_evidence_accepts_contract_fields(self) -> None:
        text = textwrap.dedent(
            """
            ## Acceptance Evidence
            - Acceptance source: docs/spec.md plus fixtures/auth.json.
            - Evidence that proves it: mutated fixture failed the acceptance path.
            - Exact command/path/route exercised: npm run test:e2e -- auth.
            - Oracle / acceptance artifact hash: sha256:abc123 for fixtures/auth.json.
            - Contract-change acknowledgment: no acceptance contract changed.
            - Residual risk: browser-specific layout remains out of scope.
            """
        )

        [block] = evidence_blocks.parse_evidence_blocks(Path("fixture.md"), text)

        self.assertEqual(evidence_blocks.check_block(block), [])

    def test_outer_heading_collects_inner_fenced_template_once(self) -> None:
        text = textwrap.dedent(
            """
            ## Completion Gate

            Every report includes:

            ```markdown
            ## Completion Gate
            - Evidence that proves it: command output copied into the brief.
            - Exact command/path/route exercised: dagger call check --source=.
            - Residual risk: none known.
            ```

            ## Gotchas
            """
        )

        blocks = evidence_blocks.parse_evidence_blocks(Path("fixture.md"), text)

        self.assertEqual(len(blocks), 1)
        self.assertEqual(evidence_blocks.check_block(blocks[0]), [])

    def test_h3_completion_gate_terminates_at_next_h3(self) -> None:
        text = textwrap.dedent(
            """
            ## Report

            ### Completion Gate
            - Evidence that proves it: command output copied into the report.
            - Exact command/path/route exercised: python3 scripts/check.py.
            - Residual risk: none known.

            ### Residual Risks
            - Follow-up:
            """
        )

        blocks = evidence_blocks.parse_evidence_blocks(Path("fixture.md"), text)

        self.assertEqual(len(blocks), 1)
        self.assertNotIn("Follow-up", blocks[0].fields)
        self.assertEqual(evidence_blocks.check_block(blocks[0]), [])


if __name__ == "__main__":
    unittest.main()
