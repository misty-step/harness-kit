use std::fs;
use std::path::Path;

use anyhow::Result;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvalSuiteReport {
    pub checked: usize,
    pub errors: Vec<String>,
}

impl EvalSuiteReport {
    pub fn ensure_success(&self) -> Result<()> {
        if self.errors.is_empty() {
            Ok(())
        } else {
            anyhow::bail!("skill eval suite validation failed")
        }
    }
}

pub fn check_repo(repo: &Path) -> Result<EvalSuiteReport> {
    let skills_root = repo.join("skills");
    if !skills_root.exists() {
        return Ok(EvalSuiteReport {
            checked: 0,
            errors: vec!["skills/ not found".to_string()],
        });
    }

    let mut evals_dirs = Vec::new();
    for entry in fs::read_dir(&skills_root)? {
        let entry = entry?;
        let evals = entry.path().join("evals");
        if evals.is_dir() {
            evals_dirs.push(evals);
        }
    }
    evals_dirs.sort();

    let mut errors = Vec::new();
    for evals_dir in &evals_dirs {
        let skill = evals_dir
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");

        if !evals_dir.join("README.md").is_file() {
            errors.push(format!("skills/{skill}/evals: missing README.md"));
        }

        let cases = evals_dir.join("cases");
        if !cases.is_dir() || !has_direct_file(&cases)? {
            errors.push(format!(
                "skills/{skill}/evals: missing at least one case file"
            ));
        }

        let graders = evals_dir.join("graders");
        if !graders.is_dir() || !has_direct_file(&graders)? {
            errors.push(format!("skills/{skill}/evals: missing at least one grader"));
        }
    }

    Ok(EvalSuiteReport {
        checked: evals_dirs.len(),
        errors,
    })
}

pub fn format_success(report: &EvalSuiteReport) -> String {
    format!("OK: {} skill eval suite(s) valid", report.checked)
}

fn has_direct_file(path: &Path) -> Result<bool> {
    for child in fs::read_dir(path)? {
        if child?.path().is_file() {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_missing_skills_root() {
        let temp = tempfile::tempdir().unwrap();

        let report = check_repo(temp.path()).unwrap();

        assert_eq!(
            report,
            EvalSuiteReport {
                checked: 0,
                errors: vec!["skills/ not found".to_string()],
            }
        );
    }

    #[test]
    fn accepts_valid_eval_suites() {
        let temp = tempfile::tempdir().unwrap();
        create_suite(temp.path(), "qa", true, true, true);
        create_suite(temp.path(), "design", true, true, true);

        let report = check_repo(temp.path()).unwrap();

        assert_eq!(report.checked, 2);
        assert!(report.errors.is_empty());
        assert_eq!(format_success(&report), "OK: 2 skill eval suite(s) valid");
    }

    #[test]
    fn reports_each_missing_required_part() {
        let temp = tempfile::tempdir().unwrap();
        create_suite(temp.path(), "qa", false, false, false);

        let report = check_repo(temp.path()).unwrap();

        assert_eq!(
            report.errors,
            vec![
                "skills/qa/evals: missing README.md",
                "skills/qa/evals: missing at least one case file",
                "skills/qa/evals: missing at least one grader",
            ]
        );
    }

    #[test]
    fn nested_files_do_not_satisfy_direct_file_requirement() {
        let temp = tempfile::tempdir().unwrap();
        let evals = temp.path().join("skills/qa/evals");
        fs::create_dir_all(evals.join("cases/nested")).unwrap();
        fs::create_dir_all(evals.join("graders/nested")).unwrap();
        fs::write(evals.join("README.md"), "# evals\n").unwrap();
        fs::write(evals.join("cases/nested/case.md"), "case\n").unwrap();
        fs::write(evals.join("graders/nested/check.sh"), "check\n").unwrap();

        let report = check_repo(temp.path()).unwrap();

        assert_eq!(
            report.errors,
            vec![
                "skills/qa/evals: missing at least one case file",
                "skills/qa/evals: missing at least one grader",
            ]
        );
    }

    #[test]
    fn error_order_is_deterministic_by_skill_name() {
        let temp = tempfile::tempdir().unwrap();
        create_suite(temp.path(), "qa", false, true, true);
        create_suite(temp.path(), "code-review", false, true, true);

        let report = check_repo(temp.path()).unwrap();

        assert_eq!(
            report.errors,
            vec![
                "skills/code-review/evals: missing README.md",
                "skills/qa/evals: missing README.md",
            ]
        );
    }

    fn create_suite(root: &Path, skill: &str, readme: bool, case_file: bool, grader_file: bool) {
        let evals = root.join("skills").join(skill).join("evals");
        fs::create_dir_all(evals.join("cases")).unwrap();
        fs::create_dir_all(evals.join("graders")).unwrap();
        if readme {
            fs::write(evals.join("README.md"), "# evals\n").unwrap();
        }
        if case_file {
            fs::write(evals.join("cases/case.md"), "case\n").unwrap();
        }
        if grader_file {
            fs::write(evals.join("graders/check.sh"), "check\n").unwrap();
        }
    }
}
