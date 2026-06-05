use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::Utc;

pub fn branch_slug(repo: &Path, branch: Option<&str>) -> Result<String> {
    let branch = match branch {
        Some(branch) if !branch.is_empty() => branch.to_string(),
        _ => git_branch(repo)?,
    };
    Ok(slugify_branch(&branch))
}

pub fn evidence_date() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

pub fn evidence_dir(repo: &Path, branch: Option<&str>, day: Option<&str>) -> Result<String> {
    let slug = branch_slug(repo, branch)?;
    if slug.is_empty() {
        bail!("evidence_dir: empty branch slug");
    }
    let day = day
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(evidence_date);
    Ok(format!(".evidence/{slug}/{day}/"))
}

pub fn evidence_dir_create(repo: &Path, branch: Option<&str>, day: Option<&str>) -> Result<String> {
    let dir = evidence_dir(repo, branch, day)?;
    fs::create_dir_all(repo.join(&dir)).with_context(|| format!("failed to create {dir}"))?;
    Ok(dir)
}

pub fn evidence_trailer(
    repo: &Path,
    dir: Option<&str>,
    branch: Option<&str>,
    day: Option<&str>,
) -> Result<Option<String>> {
    let dir = match dir {
        Some(dir) if !dir.is_empty() => dir.to_string(),
        _ => evidence_dir(repo, branch, day)?,
    };
    if directory_has_entries(&repo.join(&dir))? {
        Ok(Some(format!("QA-Evidence: {dir}")))
    } else {
        Ok(None)
    }
}

fn slugify_branch(branch: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for byte in branch.bytes() {
        let keep = byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-');
        if keep {
            slug.push(byte as char);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    slug.trim_matches('-').to_string()
}

fn directory_has_entries(path: &Path) -> Result<bool> {
    if !path.is_dir() {
        return Ok(false);
    }
    Ok(fs::read_dir(path)
        .with_context(|| format!("failed to read {}", path.display()))?
        .next()
        .transpose()?
        .is_some())
}

fn git_branch(repo: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .context("failed to run git rev-parse --abbrev-ref HEAD")?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }
    bail!("failed to determine current branch")
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;
    use tempfile::TempDir;

    struct Repo {
        temp: TempDir,
    }

    impl Repo {
        fn new() -> Self {
            let temp = TempDir::new().unwrap();
            run_git(temp.path(), &["init", "-q"]);
            run_git(temp.path(), &["checkout", "-q", "-b", "master"]);
            run_git(temp.path(), &["config", "core.hooksPath", ".empty-hooks"]);
            run_git(temp.path(), &["config", "user.name", "Test User"]);
            run_git(temp.path(), &["config", "user.email", "test@example.com"]);
            fs::create_dir_all(temp.path().join(".empty-hooks")).unwrap();
            run_git(
                temp.path(),
                &["commit", "--allow-empty", "-m", "initial", "-q"],
            );
            run_git(
                temp.path(),
                &["checkout", "-b", "feat/024-offline-evidence", "-q"],
            );
            Self { temp }
        }

        fn path(&self) -> &Path {
            self.temp.path()
        }
    }

    fn run_git(repo: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo)
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    #[test]
    fn branch_slug_normalizes_slashes() {
        let repo = Repo::new();
        assert_eq!(
            branch_slug(repo.path(), None).unwrap(),
            "feat-024-offline-evidence"
        );
    }

    #[test]
    fn branch_slug_collapses_and_trims_replacements() {
        let repo = Repo::new();
        assert_eq!(
            branch_slug(repo.path(), Some("/feature//x!!")).unwrap(),
            "feature-x"
        );
    }

    #[test]
    fn evidence_dir_uses_branch_and_date() {
        let repo = Repo::new();
        assert_eq!(
            evidence_dir(repo.path(), Some("feature/x"), Some("2026-06-02")).unwrap(),
            ".evidence/feature-x/2026-06-02/"
        );
    }

    #[test]
    fn evidence_dir_create_makes_directory() {
        let repo = Repo::new();
        let dir = evidence_dir_create(repo.path(), Some("feature/x"), Some("2026-06-02")).unwrap();
        assert_eq!(dir, ".evidence/feature-x/2026-06-02/");
        assert!(repo.path().join(dir).is_dir());
    }

    #[test]
    fn evidence_trailer_skips_empty_dir() {
        let repo = Repo::new();
        let dir = evidence_dir_create(repo.path(), Some("feature/x"), Some("2026-06-02")).unwrap();
        assert_eq!(
            evidence_trailer(repo.path(), Some(&dir), None, None).unwrap(),
            None
        );
    }

    #[test]
    fn evidence_trailer_prints_non_empty_dir() {
        let repo = Repo::new();
        let dir = evidence_dir_create(repo.path(), Some("feature/x"), Some("2026-06-02")).unwrap();
        fs::write(repo.path().join(&dir).join("qa-report.md"), "proof\n").unwrap();
        assert_eq!(
            evidence_trailer(repo.path(), Some(&dir), None, None).unwrap(),
            Some("QA-Evidence: .evidence/feature-x/2026-06-02/".to_string())
        );
    }
}
