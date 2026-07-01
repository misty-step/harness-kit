use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};

use crate::lint_gates::GateReport;

/// First-party skills that intentionally sit outside the eval-coverage
/// contract because they are not agent-invoked judgment skills.
const EXEMPT_SKILLS: &[&str] = &[];

pub fn check_eval_coverage(repo: &Path, now: DateTime<Utc>) -> Result<GateReport> {
    let skills_dir = repo.join("skills");
    if !skills_dir.exists() {
        return Ok(GateReport::success(
            "skills/ not present; skipping eval-coverage check.",
        ));
    }

    let mut missing = Vec::new();
    let mut expired = Vec::new();
    let mut covered = 0usize;

    for entry in fs::read_dir(&skills_dir)
        .with_context(|| format!("failed to read {}", skills_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if name.starts_with('.') || EXEMPT_SKILLS.contains(&name.as_str()) {
            continue;
        }
        if !path.join("SKILL.md").exists() {
            continue;
        }

        match eval_status(&path, now)? {
            EvalStatus::Covered => covered += 1,
            EvalStatus::Missing => missing.push(name),
            EvalStatus::ExpiredWaiver(expires) => {
                expired.push(format!("{name} (expired {expires})"))
            }
        }
    }

    if missing.is_empty() && expired.is_empty() {
        return Ok(GateReport::success(format!(
            "eval coverage: {covered} first-party skill(s) have an eval spec or live waiver."
        )));
    }

    let mut errors = vec![format!(
        "{} first-party skill(s) lack eval coverage:",
        missing.len() + expired.len()
    )];
    for name in &missing {
        errors.push(format!(
            "  {name}: no skills/{name}/evals/*.md and no skills/{name}/evals/WAIVER.md"
        ));
    }
    for name in &expired {
        errors.push(format!("  {name}: waiver expired"));
    }
    errors.push(String::new());
    errors.push(
        "Add an eval spec (copy skills/skill-eval/templates/eval-spec.md to \
         skills/<skill>/evals/<skill>-eval.md) or a time-boxed waiver \
         (skills/<skill>/evals/WAIVER.md with an `expires:` date)."
            .to_string(),
    );
    Ok(GateReport::failure(errors))
}

enum EvalStatus {
    Covered,
    Missing,
    ExpiredWaiver(String),
}

fn eval_status(skill_dir: &Path, now: DateTime<Utc>) -> Result<EvalStatus> {
    let evals_dir = skill_dir.join("evals");
    if !evals_dir.exists() {
        return Ok(EvalStatus::Missing);
    }

    let mut waiver_expiry: Option<String> = None;
    for entry in fs::read_dir(&evals_dir)
        .with_context(|| format!("failed to read {}", evals_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !file_name.ends_with(".md") {
            continue;
        }
        if file_name.eq_ignore_ascii_case("WAIVER.md") {
            let text = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            waiver_expiry = Some(parse_waiver_expiry(&text)?);
            continue;
        }
        // Any other non-empty markdown file under evals/ counts as a real spec.
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        if !text.trim().is_empty() {
            return Ok(EvalStatus::Covered);
        }
    }

    match waiver_expiry {
        Some(expires) => {
            let expires_date =
                NaiveDate::parse_from_str(&expires, "%Y-%m-%d").with_context(|| {
                    format!("{}: `expires:` is not YYYY-MM-DD", evals_dir.display())
                })?;
            if expires_date >= now.date_naive() {
                Ok(EvalStatus::Covered)
            } else {
                Ok(EvalStatus::ExpiredWaiver(expires))
            }
        }
        None => Ok(EvalStatus::Missing),
    }
}

fn parse_waiver_expiry(text: &str) -> Result<String> {
    for line in text.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("expires:") {
            return Ok(value.trim().to_string());
        }
    }
    anyhow::bail!("WAIVER.md is missing an `expires: YYYY-MM-DD` line")
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use tempfile::TempDir;

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 7, 1, 0, 0, 0).unwrap()
    }

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn skill_with_eval_spec_passes() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path();
        write(&repo.join("skills/foo/SKILL.md"), "---\nname: foo\n---\n");
        write(&repo.join("skills/foo/evals/foo-eval.md"), "# /foo eval\n");
        let report = check_eval_coverage(repo, now()).unwrap();
        assert!(report.errors.is_empty());
    }

    #[test]
    fn skill_with_live_waiver_passes() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path();
        write(&repo.join("skills/foo/SKILL.md"), "---\nname: foo\n---\n");
        write(
            &repo.join("skills/foo/evals/WAIVER.md"),
            "expires: 2026-12-31\nReason: no runnable claim yet.\n",
        );
        let report = check_eval_coverage(repo, now()).unwrap();
        assert!(report.errors.is_empty());
    }

    #[test]
    fn skill_with_expired_waiver_fails() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path();
        write(&repo.join("skills/foo/SKILL.md"), "---\nname: foo\n---\n");
        write(
            &repo.join("skills/foo/evals/WAIVER.md"),
            "expires: 2026-01-01\nReason: stale.\n",
        );
        let report = check_eval_coverage(repo, now()).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|line| line.contains("expired")));
    }

    #[test]
    fn skill_with_no_evals_dir_fails() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path();
        write(&repo.join("skills/foo/SKILL.md"), "---\nname: foo\n---\n");
        let report = check_eval_coverage(repo, now()).unwrap();
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|line| line.contains("foo")));
    }

    #[test]
    fn skill_with_empty_evals_dir_fails() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path();
        write(&repo.join("skills/foo/SKILL.md"), "---\nname: foo\n---\n");
        fs::create_dir_all(repo.join("skills/foo/evals")).unwrap();
        let report = check_eval_coverage(repo, now()).unwrap();
        assert!(!report.errors.is_empty());
    }

    #[test]
    fn external_skills_directory_is_ignored() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path();
        write(
            &repo.join("skills/.external/vendored/SKILL.md"),
            "---\nname: vendored\n---\n",
        );
        let report = check_eval_coverage(repo, now()).unwrap();
        assert!(report.errors.is_empty());
    }

    #[test]
    fn non_skill_directories_without_skill_md_are_ignored() {
        let temp = TempDir::new().unwrap();
        let repo = temp.path();
        fs::create_dir_all(repo.join("skills/not-a-skill")).unwrap();
        let report = check_eval_coverage(repo, now()).unwrap();
        assert!(report.errors.is_empty());
    }

    #[test]
    fn missing_skills_dir_is_ok() {
        let temp = TempDir::new().unwrap();
        let report = check_eval_coverage(temp.path(), now()).unwrap();
        assert!(report.errors.is_empty());
    }
}
