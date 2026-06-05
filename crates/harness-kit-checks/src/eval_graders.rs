use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalGrader {
    CodeReviewEntrypoint,
    CodeReviewRepoFit,
    CreateRepoSkill,
    QaBrowserMissingSelector,
    QaCliSmoke,
    QaNonBrowser,
}

impl EvalGrader {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "code-review-entrypoint" => Some(Self::CodeReviewEntrypoint),
            "code-review-repo-fit" => Some(Self::CodeReviewRepoFit),
            "create-repo-skill" => Some(Self::CreateRepoSkill),
            "qa-browser-missing-selector" => Some(Self::QaBrowserMissingSelector),
            "qa-cli-smoke" => Some(Self::QaCliSmoke),
            "qa-non-browser" => Some(Self::QaNonBrowser),
            _ => None,
        }
    }
}

pub fn grade(grader: EvalGrader, path: &Path) -> Result<String> {
    let text =
        fs::read_to_string(path).map_err(|error| anyhow::anyhow!("{}: {error}", path.display()))?;
    grade_text(grader, &text)
}

pub fn grade_text(grader: EvalGrader, text: &str) -> Result<String> {
    match grader {
        EvalGrader::CodeReviewEntrypoint => code_review_entrypoint(text),
        EvalGrader::CodeReviewRepoFit => code_review_repo_fit(text),
        EvalGrader::CreateRepoSkill => create_repo_skill(text),
        EvalGrader::QaBrowserMissingSelector => qa_browser_missing_selector(text),
        EvalGrader::QaCliSmoke => qa_cli_smoke(text),
        EvalGrader::QaNonBrowser => qa_non_browser(text),
    }
}

pub fn self_test() -> Result<String> {
    for (grader, passing, failing) in [
        (
            EvalGrader::CodeReviewEntrypoint,
            "Unverified entrypoint on runtime path blocks ship: scripts/import-users.py needs executable path evidence.",
            "Ship it; README looks fine.",
        ),
        (
            EvalGrader::CodeReviewRepoFit,
            "Blocking: structurally valid scaffold lacks repo-fit and live repo proof. Run python3 -m example_tool --help.",
            "Looks structurally valid.",
        ),
        (
            EvalGrader::CreateRepoSkill,
            "Persona value evidence report eval .agents/skills Completion Gate Residual",
            "Persona value evidence report eval .agents/skills Completion Gate TODO",
        ),
        (
            EvalGrader::QaCliSmoke,
            "CLI command --help malformed missing transcript evidence tests pass go test",
            "Use browser and playwright for the CLI command.",
        ),
        (
            EvalGrader::QaBrowserMissingSelector,
            "Status: inconclusive\nTool: browser\nRoute: http://127.0.0.1:3000/billing\nEvidence: .evidence/feat-billing/2026-06-04/browser.png\nTranscript: .evidence/feat-billing/2026-06-04/route-selection.md\nReport: .evidence/feat-billing/2026-06-04/qa-report.md\nAssertion: Upgrade plan button missing.\nFollow-up: route fixes through /deliver --polish-only.",
            "Status: pass\nTool: browser\nRoute: http://127.0.0.1:3000/billing\nEvidence: .evidence/feat-billing/2026-06-04/browser.png\nTranscript: .evidence/feat-billing/2026-06-04/route-selection.md\nReport: .evidence/feat-billing/2026-06-04/qa-report.md\nAssertion: Upgrade plan button missing.",
        ),
        (
            EvalGrader::QaNonBrowser,
            "App shape: CLI\nCommands: acme --help; acme render input.yaml; missing-file check\nEvidence: .evidence/feat-cli/2026-06-04/terminal-transcript.txt",
            "App shape: CLI\nUse Playwright and capture a screenshot for acme render input.yaml.",
        ),
    ] {
        grade_text(grader, passing)?;
        expect_failure(grader, failing)?;
    }
    Ok("PASS: eval graders self-test".to_string())
}

fn code_review_entrypoint(text: &str) -> Result<String> {
    require(text, "unverified")?;
    require(text, "entrypoint|runtime path|executable path")?;
    require(text, "block|blocking|not ship|don't ship")?;
    require(text, "scripts/import-users\\.py")?;
    Ok("PASS: code-review output blocks unverified entrypoint".to_string())
}

fn code_review_repo_fit(text: &str) -> Result<String> {
    require(text, "structural|frontmatter|scaffold")?;
    require(text, "repo-fit|repo fit|live repo")?;
    require(text, "python3 -m example_tool --help")?;
    require(text, "block|blocking|not ship|don't ship")?;
    Ok("PASS: code-review output blocks structurally valid but non-repo-fit work".to_string())
}

fn create_repo_skill(text: &str) -> Result<String> {
    for phrase in [
        "persona",
        "value",
        "evidence",
        "report",
        "eval",
        ".agents/skills",
        "Completion Gate",
        "Residual",
    ] {
        require(text, &regex::escape(phrase))?;
    }
    if matches(text, r"TODO|\[fill in\]|your-app") {
        bail!("placeholder text remains");
    }
    Ok("generated repo skill shape looks concrete".to_string())
}

fn qa_cli_smoke(text: &str) -> Result<String> {
    require(text, "CLI|command")?;
    require(text, "help|--help")?;
    require(text, "malformed|missing")?;
    require(text, "transcript|evidence")?;
    require(text, "tests pass|go test")?;
    if matches(text, "playwright|browser") {
        bail!("candidate reached for browser tooling on a CLI repo");
    }
    Ok("PASS: qa output routes to CLI smoke evidence".to_string())
}

fn qa_browser_missing_selector(text: &str) -> Result<String> {
    if matches(text, r#"\bstatus\s*[:=]\s*[`'"]?pass[`'"]?"#) {
        bail!("missing expected element serialized as pass");
    }
    if !matches(text, r#"\bstatus\s*[:=]\s*[`'"]?(fail|inconclusive)[`'"]?"#) {
        bail!("missing fail or inconclusive status");
    }
    require(text, r"http://127\.0\.0\.1:3000/billing|/billing\b")?;
    require(text, r"\b(browser|browser-use|playwright|chrome)\b")?;
    require(text, r"\.evidence/feat-billing/2026-06-04/browser\.png")?;
    require(text, r"route-selection\.md")?;
    require(text, r"qa-report\.md")?;
    require(text, r"Upgrade plan button|expected button|missing")?;
    if matches(text, r"\b(open|create|push)\b[^.\n]{0,80}\bfix PR\b") {
        bail!("forbidden autonomous fix PR");
    }
    Ok("PASS: qa-browser-missing-selector".to_string())
}

fn qa_non_browser(text: &str) -> Result<String> {
    require(text, r"\b(CLI|command|shell|library|hybrid)\b")?;
    require(
        text,
        r"(--help|README|acme render|missing[- ]file|malformed)",
    )?;
    require(text, r"\.evidence/|transcript")?;
    if matches(text, r"\b(playwright|browser-use|webvnc|screenshot)\b") {
        bail!("forbidden forced browser tooling");
    }
    Ok("PASS: qa-non-browser".to_string())
}

fn expect_failure(grader: EvalGrader, text: &str) -> Result<()> {
    match grade_text(grader, text) {
        Ok(_) => bail!("expected eval grader fixture to fail"),
        Err(_) => Ok(()),
    }
}

fn require(text: &str, pattern: &str) -> Result<()> {
    if matches(text, pattern) {
        Ok(())
    } else {
        bail!("missing required pattern: {pattern}")
    }
}

fn matches(text: &str, pattern: &str) -> bool {
    Regex::new(&format!("(?i){pattern}"))
        .expect("eval grader regex must compile")
        .is_match(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::{Path, PathBuf};

    fn repo_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .ancestors()
            .nth(2)
            .expect("crate is nested under crates/harness-kit-checks")
            .to_path_buf()
    }

    fn read_repo_file(relative_path: &str) -> String {
        fs::read_to_string(repo_root().join(relative_path))
            .expect("repo fixture should be readable")
    }

    #[test]
    fn self_test_contract_passes() {
        assert_eq!(self_test().unwrap(), "PASS: eval graders self-test");
    }

    #[test]
    fn create_repo_skill_rejects_placeholders() {
        let error = grade_text(
            EvalGrader::CreateRepoSkill,
            "persona value evidence report eval .agents/skills Completion Gate Residual your-app",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(error, "placeholder text remains");
    }

    #[test]
    fn qa_cli_smoke_rejects_browser_tooling() {
        let error = grade_text(
            EvalGrader::QaCliSmoke,
            "CLI command --help malformed missing transcript evidence tests pass browser",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(error, "candidate reached for browser tooling on a CLI repo");
    }

    #[test]
    fn qa_browser_missing_selector_rejects_pass_status() {
        let error = grade_text(
            EvalGrader::QaBrowserMissingSelector,
            "Status: pass\nTool: browser\nRoute: /billing\nEvidence: .evidence/feat-billing/2026-06-04/browser.png\nTranscript: route-selection.md\nReport: qa-report.md\nAssertion: Upgrade plan button missing.",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(error, "missing expected element serialized as pass");
    }

    #[test]
    fn qa_non_browser_rejects_forced_browser_tooling() {
        let error = grade_text(
            EvalGrader::QaNonBrowser,
            "App shape: CLI\nCommands: acme --help\nEvidence: transcript\nUse playwright screenshot.",
        )
        .unwrap_err()
        .to_string();

        assert_eq!(error, "forbidden forced browser tooling");
    }

    #[test]
    fn demo_references_keep_executable_shell_examples() {
        let pr_evidence = read_repo_file("skills/demo/references/pr-evidence-upload.md");
        assert!(pr_evidence.contains("PR_NUMBER=123"));
        assert!(!pr_evidence.contains("qa-evidence-pr-{NUMBER}"));
        assert!(!pr_evidence.contains("gh pr comment {NUMBER}"));
        assert!(pr_evidence.contains(r#"grep -F -- "qa-evidence-pr-${PR_NUMBER}""#));

        let tts = read_repo_file("skills/demo/references/tts-narration.md");
        assert!(tts.contains(r#"jq -n --rawfile input "$EVIDENCE_DIR/script.txt""#));
        assert!(!tts.contains(r#""input": "'"$(cat "$EVIDENCE_DIR/script.txt")"'"#));
    }

    #[test]
    fn delegate_and_deliver_references_keep_canonical_contract_text() {
        let delegate = read_repo_file("skills/research/references/delegate.md");
        assert!(delegate.contains("recommend which evidence to trust"));
        assert!(delegate.contains("```text\nPhase 1"));

        let receipt = read_repo_file("skills/deliver/references/receipt.md");
        assert!(receipt.contains(r#""name": "code-review""#));
        assert!(receipt.contains(r#""code-review: 2 blocking findings in auth.py""#));
        assert!(receipt.contains("## State Lifecycle\n\n```text\n"));
        assert!(!receipt.contains(r#""name": "review""#));
    }

    #[test]
    fn qa_grader_docs_and_text_contracts_cover_cli_smoke() {
        let grader_doc = read_repo_file("skills/qa/evals/graders/qa-cli-smoke.md");
        assert!(grader_doc.contains("eval-grader qa-cli-smoke"));

        assert!(
            grade_text(
                EvalGrader::QaCliSmoke,
                "CLI help covers malformed transcript evidence and tests pass",
            )
            .is_ok()
        );
        assert!(grade_text(
            EvalGrader::QaCliSmoke,
            "CLI help covers malformed transcript evidence and tests pass; browser smoke mentioned",
        )
        .is_err());
    }

    #[test]
    fn qa_per_commit_grader_docs_and_modes_cover_fixtures() {
        let grader_doc = read_repo_file("skills/qa/evals/graders/qa-per-commit.md");
        assert!(grader_doc.contains("qa-browser-missing-selector"));
        assert!(grader_doc.contains("qa-non-browser"));

        let browser_case =
            read_repo_file("skills/qa/evals/cases/commit-browser-missing-selector.md");
        assert!(browser_case.contains("Upgrade plan button is visible"));
        assert!(browser_case.contains("Status is `fail` or `inconclusive`, not `pass`"));

        let cli_case = read_repo_file("skills/qa/evals/cases/commit-cli-non-browser.md");
        assert!(cli_case.contains("CLI/library-shaped change"));
        assert!(cli_case.contains("Does not force Playwright"));

        assert!(grade_text(
            EvalGrader::QaBrowserMissingSelector,
            "Status: fail\nTool: browser\nRoute: http://127.0.0.1:3000/billing\nEvidence: .evidence/feat-billing/2026-06-04/browser.png\nTranscript: .evidence/feat-billing/2026-06-04/route-selection.md\nReport: .evidence/feat-billing/2026-06-04/qa-report.md\nAssertion: Upgrade plan button missing.\n",
        )
        .is_ok());
        assert!(grade_text(
            EvalGrader::QaBrowserMissingSelector,
            "Status: pass\nTool: browser\nRoute: http://127.0.0.1:3000/billing\nEvidence: .evidence/feat-billing/2026-06-04/browser.png\nTranscript: .evidence/feat-billing/2026-06-04/route-selection.md\nReport: .evidence/feat-billing/2026-06-04/qa-report.md\nAssertion: Upgrade plan button missing.\n",
        )
        .is_err());
        assert!(grade_text(
            EvalGrader::QaNonBrowser,
            "App shape: CLI\nCommands: acme --help; acme render input.yaml; missing-file check\nEvidence: .evidence/feat-cli/2026-06-04/terminal-transcript.txt\n",
        )
        .is_ok());
        assert!(
            grade_text(
                EvalGrader::QaNonBrowser,
                "App shape: CLI\nUse browser-use and capture a screenshot.\n",
            )
            .is_err()
        );
    }
}
