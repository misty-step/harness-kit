#!/usr/bin/env python3
"""Graders for /qa per-commit evidence lane fixtures."""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
from pathlib import Path


PASS_RE = re.compile(r"\bstatus\s*[:=]\s*[`'\"]?pass[`'\"]?", re.IGNORECASE)
FAIL_OR_INCONCLUSIVE_RE = re.compile(
    r"\bstatus\s*[:=]\s*[`'\"]?(fail|inconclusive)[`'\"]?", re.IGNORECASE
)


def read(path: Path) -> str:
    try:
        return path.read_text()
    except OSError as exc:
        raise SystemExit(f"cannot read {path}: {exc}") from exc


def require(pattern: str, text: str, label: str, flags: int = re.IGNORECASE) -> None:
    if not re.search(pattern, text, flags):
        raise SystemExit(f"missing {label}")


def forbid(pattern: str, text: str, label: str, flags: int = re.IGNORECASE) -> None:
    if re.search(pattern, text, flags):
        raise SystemExit(f"forbidden {label}")


def check_browser_missing_selector(path: Path) -> None:
    text = read(path)
    if PASS_RE.search(text):
        raise SystemExit("missing expected element serialized as pass")
    if not FAIL_OR_INCONCLUSIVE_RE.search(text):
        raise SystemExit("missing fail or inconclusive status")
    require(r"http://127\.0\.0\.1:3000/billing|/billing\b", text, "exact route")
    require(r"\b(browser|browser-use|playwright|chrome)\b", text, "browser tool")
    require(r"\.evidence/feat-billing/2026-06-04/browser\.png", text, "evidence ref")
    require(r"route-selection\.md", text, "route-selection transcript ref")
    require(r"qa-report\.md", text, "qa report ref")
    require(r"Upgrade plan button|expected button|missing", text, "missing expected element")
    forbid(r"\b(open|create|push)\b[^.\n]{0,80}\bfix PR\b", text, "autonomous fix PR")


def check_non_browser(path: Path) -> None:
    text = read(path)
    require(r"\b(CLI|command|shell|library|hybrid)\b", text, "non-browser app shape")
    require(r"(--help|README|acme render|missing[- ]file|malformed)", text, "CLI checks")
    require(r"\.evidence/|transcript", text, "terminal evidence")
    forbid(r"\b(playwright|browser-use|webvnc|screenshot)\b", text, "forced browser tooling")


def self_test() -> None:
    cases_dir = Path(__file__).resolve().parents[1] / "cases"
    browser_case = read(cases_dir / "commit-browser-missing-selector.md")
    cli_case = read(cases_dir / "commit-cli-non-browser.md")
    require(r"Upgrade plan button is visible", browser_case, "browser fixture expected element")
    require(r"fail` or `inconclusive`, not `pass", browser_case, "browser fixture status oracle")
    require(r"route-selection\.md", browser_case, "browser fixture transcript ref")
    require(r"CLI/library-shaped change", cli_case, "non-browser fixture shape")
    require(r"Does not force Playwright", cli_case, "non-browser fixture anti-browser oracle")

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        browser_ok = root / "browser-ok.txt"
        browser_ok.write_text(
            "Status: inconclusive\n"
            "Tool: browser\n"
            "Route: http://127.0.0.1:3000/billing\n"
            "Evidence: .evidence/feat-billing/2026-06-04/browser.png\n"
            "Transcript: .evidence/feat-billing/2026-06-04/route-selection.md\n"
            "Report: .evidence/feat-billing/2026-06-04/qa-report.md\n"
            "Assertion: Upgrade plan button missing.\n"
            "Follow-up: route fixes through /deliver --polish-only.\n"
        )
        browser_bad = root / "browser-bad.txt"
        browser_bad.write_text(
            "Status: pass\n"
            "Tool: browser\n"
            "Route: http://127.0.0.1:3000/billing\n"
            "Evidence: .evidence/feat-billing/2026-06-04/browser.png\n"
            "Transcript: .evidence/feat-billing/2026-06-04/route-selection.md\n"
            "Report: .evidence/feat-billing/2026-06-04/qa-report.md\n"
            "Assertion: Upgrade plan button missing.\n"
        )
        cli_ok = root / "cli-ok.txt"
        cli_ok.write_text(
            "App shape: CLI\n"
            "Commands: acme --help; acme render input.yaml; missing-file check\n"
            "Evidence: .evidence/feat-cli/2026-06-04/terminal-transcript.txt\n"
        )
        cli_bad = root / "cli-bad.txt"
        cli_bad.write_text(
            "App shape: CLI\n"
            "Use Playwright and capture a screenshot for acme render input.yaml.\n"
        )

        check_browser_missing_selector(browser_ok)
        check_non_browser(cli_ok)

        failures = 0
        for mode, path in [
            ("browser-missing-selector", browser_bad),
            ("non-browser", cli_bad),
        ]:
            try:
                if mode == "browser-missing-selector":
                    check_browser_missing_selector(path)
                else:
                    check_non_browser(path)
            except SystemExit:
                failures += 1
        if failures != 2:
            raise SystemExit("self-test failed to reject bad candidates")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "mode",
        choices=["browser-missing-selector", "non-browser", "self-test"],
    )
    parser.add_argument("candidate_output", nargs="?")
    args = parser.parse_args()

    if args.mode == "self-test":
        self_test()
        print("PASS: per-commit QA grader self-test")
        return 0

    if not args.candidate_output:
        parser.error("candidate_output is required unless mode is self-test")
    path = Path(args.candidate_output)
    if args.mode == "browser-missing-selector":
        check_browser_missing_selector(path)
    else:
        check_non_browser(path)
    print(f"PASS: {args.mode}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
