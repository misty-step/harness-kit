use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

const TRAILER_KEYS: &[&str] = &["Closes-backlog", "Ships-backlog", "Refs-backlog"];
const CLOSING_KEYS: &[&str] = &["Closes-backlog", "Ships-backlog"];

pub fn trailer_keys() -> &'static [&'static str] {
    TRAILER_KEYS
}

pub fn closing_keys() -> &'static [&'static str] {
    CLOSING_KEYS
}

pub fn ids_from_commit(repo: &Path, commit: &str) -> Result<Vec<String>> {
    let message = git(repo, &["log", "-1", "--format=%B", commit])?;
    ids_from_message(repo, &message)
}

pub fn ids_from_range(repo: &Path, range: &str) -> Result<Vec<String>> {
    let shas = git(repo, &["rev-list", range])?;
    let mut ids = BTreeSet::new();
    for sha in shas.split_whitespace() {
        if let Ok(commit_ids) = ids_from_commit(repo, sha) {
            ids.extend(commit_ids);
        }
    }
    if ids.is_empty() {
        bail!("no backlog closing trailers found in range")
    }
    Ok(ids.into_iter().collect())
}

pub fn file_for_id(cwd: &Path, id: &str) -> Result<Option<PathBuf>> {
    validate_id(id).with_context(|| format!("backlog_file_for_id: invalid ID '{id}'"))?;
    let root = repo_root(cwd).context("backlog_file_for_id: not in a git repo")?;
    if let Some(name) = first_match(&root.join("backlog.d"), id)? {
        return Ok(Some(PathBuf::from("backlog.d").join(name)));
    }
    if let Some(name) = first_match(&root.join("backlog.d/_done"), id)? {
        return Ok(Some(PathBuf::from("backlog.d/_done").join(name)));
    }
    Ok(None)
}

pub fn archive(cwd: &Path, id: &str) -> Result<()> {
    validate_id(id).with_context(|| format!("backlog_archive: invalid ID '{id}'"))?;
    let root = repo_root(cwd).context("backlog_archive: not in a git repo")?;
    let active_matches = all_matches(&root.join("backlog.d"), id)?;
    let done_match = first_match(&root.join("backlog.d/_done"), id)?;

    if active_matches.is_empty() && done_match.is_some() {
        return Ok(());
    }
    if active_matches.is_empty() {
        bail!("backlog_archive: no ticket file found for ID '{id}'");
    }

    fs::create_dir_all(root.join("backlog.d/_done")).with_context(|| {
        format!(
            "backlog_archive: could not create archive directory under '{}'",
            root.display()
        )
    })?;
    for active_match in active_matches {
        let source = format!("backlog.d/{active_match}");
        let target = format!("backlog.d/_done/{active_match}");
        git(&root, &["mv", &source, &target]).with_context(|| {
            format!("backlog_archive: could not move {source} into backlog.d/_done/")
        })?;
    }
    Ok(())
}

pub fn ids_from_message(repo: &Path, message: &str) -> Result<Vec<String>> {
    let parsed = git_with_stdin(
        repo,
        &["interpret-trailers", "--parse", "--no-divider"],
        message,
    )?;
    let mut ids = BTreeSet::new();
    for line in parsed.lines() {
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        if !CLOSING_KEYS.contains(&key.trim()) {
            continue;
        }
        let id: String = value.chars().filter(|ch| !ch.is_whitespace()).collect();
        if id.chars().all(|ch| ch.is_ascii_digit()) && !id.is_empty() {
            ids.insert(id);
        }
    }
    if ids.is_empty() {
        bail!("no backlog closing trailers found")
    }
    Ok(ids.into_iter().collect())
}

fn validate_id(id: &str) -> Result<()> {
    if id.chars().all(|ch| ch.is_ascii_digit()) && !id.is_empty() {
        Ok(())
    } else {
        bail!("invalid backlog ID")
    }
}

fn repo_root(cwd: &Path) -> Result<PathBuf> {
    let root = git(cwd, &["rev-parse", "--show-toplevel"])?;
    Ok(PathBuf::from(root.trim()))
}

fn first_match(dir: &Path, id: &str) -> Result<Option<String>> {
    Ok(all_matches(dir, id)?.into_iter().next())
}

fn all_matches(dir: &Path, id: &str) -> Result<Vec<String>> {
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let prefix = format!("{id}-");
    let mut matches = Vec::new();
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with(&prefix) && name.ends_with(".md") {
            matches.push(name);
        }
    }
    matches.sort();
    Ok(matches)
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        bail!("git {} failed with {}", args.join(" "), output.status);
    }
    bail!("{stderr}")
}

fn git_with_stdin(repo: &Path, args: &[&str], stdin: &str) -> Result<String> {
    let mut child = Command::new("git")
        .args(args)
        .current_dir(repo)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if let Some(child_stdin) = child.stdin.as_mut() {
        use std::io::Write;
        child_stdin
            .write_all(stdin.as_bytes())
            .context("failed to write git stdin")?;
    }
    let output = child
        .wait_with_output()
        .with_context(|| format!("failed to wait for git {}", args.join(" ")))?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if stderr.is_empty() {
        bail!("git {} failed with {}", args.join(" "), output.status);
    }
    bail!("{stderr}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct Fixture {
        temp: TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = TempDir::new().unwrap();
            run_git(temp.path(), &["init", "-q"]);
            run_git(temp.path(), &["checkout", "-q", "-b", "master"]);
            run_git(temp.path(), &["config", "core.hooksPath", ".empty-hooks"]);
            run_git(temp.path(), &["config", "user.name", "Test User"]);
            run_git(temp.path(), &["config", "user.email", "test@example.com"]);
            fs::create_dir_all(temp.path().join(".empty-hooks")).unwrap();
            fs::create_dir_all(temp.path().join("backlog.d/_done")).unwrap();
            fs::write(
                temp.path().join("backlog.d/031-active-ticket.md"),
                "# BACKLOG-031: Active ticket\nStatus: ready\n",
            )
            .unwrap();
            fs::write(
                temp.path().join("backlog.d/042-another-active.md"),
                "# BACKLOG-042: Another active\nStatus: ready\n",
            )
            .unwrap();
            fs::write(
                temp.path().join("backlog.d/_done/007-archived-ticket.md"),
                "# BACKLOG-007: Already archived\nStatus: done\n",
            )
            .unwrap();
            run_git(temp.path(), &["add", "-A"]);
            run_git(temp.path(), &["commit", "-m", "initial", "-q"]);
            Self { temp }
        }

        fn path(&self) -> &Path {
            self.temp.path()
        }

        fn commit_with_trailers(&self, subject: &str, trailers: &str) {
            let message = format!("{subject}\n\nbody line\n\n{trailers}\n");
            run_git(
                self.path(),
                &["commit", "--allow-empty", "-q", "-m", &message],
            );
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
    fn trailer_keys_lists_all_three() {
        assert_eq!(
            trailer_keys(),
            ["Closes-backlog", "Ships-backlog", "Refs-backlog"]
        );
    }

    #[test]
    fn closing_keys_excludes_refs() {
        assert_eq!(closing_keys(), ["Closes-backlog", "Ships-backlog"]);
    }

    #[test]
    fn ids_from_commit_extracts_multiple_closing_trailers() {
        let fixture = Fixture::new();
        fixture.commit_with_trailers(
            "feat: thing",
            "Closes-backlog: 031\nShips-backlog: 042\nRefs-backlog: 099",
        );
        assert_eq!(
            ids_from_commit(fixture.path(), "HEAD").unwrap(),
            ["031", "042"]
        );
    }

    #[test]
    fn ids_from_commit_deduplicates() {
        let fixture = Fixture::new();
        fixture.commit_with_trailers("feat: dup", "Closes-backlog: 031\nShips-backlog: 031");
        assert_eq!(ids_from_commit(fixture.path(), "HEAD").unwrap(), ["031"]);
    }

    #[test]
    fn ids_from_commit_no_trailers_fails() {
        let fixture = Fixture::new();
        run_git(
            fixture.path(),
            &[
                "commit",
                "--allow-empty",
                "-q",
                "-m",
                "chore: no trailers here",
            ],
        );
        assert!(ids_from_commit(fixture.path(), "HEAD").is_err());
    }

    #[test]
    fn ids_from_commit_ignores_refs_only() {
        let fixture = Fixture::new();
        fixture.commit_with_trailers("chore: reference only", "Refs-backlog: 031");
        assert!(ids_from_commit(fixture.path(), "HEAD").is_err());
    }

    #[test]
    fn ids_from_range_deduplicates_across_commits() {
        let fixture = Fixture::new();
        let base = git(fixture.path(), &["rev-parse", "HEAD"]).unwrap();
        fixture.commit_with_trailers("feat: one", "Closes-backlog: 031");
        fixture.commit_with_trailers("feat: two", "Ships-backlog: 042\nCloses-backlog: 031");
        let range = format!("{}..HEAD", base.trim());
        assert_eq!(
            ids_from_range(fixture.path(), &range).unwrap(),
            ["031", "042"]
        );
    }

    #[test]
    fn ids_from_range_empty_fails() {
        let fixture = Fixture::new();
        let head = git(fixture.path(), &["rev-parse", "HEAD"]).unwrap();
        run_git(
            fixture.path(),
            &["commit", "--allow-empty", "-q", "-m", "chore: no trailers"],
        );
        let range = format!("{}..HEAD", head.trim());
        assert!(ids_from_range(fixture.path(), &range).is_err());
    }

    #[test]
    fn file_for_id_finds_active_then_archived() {
        let fixture = Fixture::new();
        assert_eq!(
            file_for_id(fixture.path(), "031").unwrap().unwrap(),
            PathBuf::from("backlog.d/031-active-ticket.md")
        );
        assert_eq!(
            file_for_id(fixture.path(), "007").unwrap().unwrap(),
            PathBuf::from("backlog.d/_done/007-archived-ticket.md")
        );
    }

    #[test]
    fn file_for_id_unknown_or_invalid_fails() {
        let fixture = Fixture::new();
        assert!(file_for_id(fixture.path(), "999").unwrap().is_none());
        let error = file_for_id(fixture.path(), "abc").unwrap_err().to_string();
        assert!(error.contains("backlog_file_for_id: invalid ID 'abc'"));
    }

    #[test]
    fn file_for_id_works_from_subdirectory() {
        let fixture = Fixture::new();
        let nested = fixture.path().join("nested/deeper");
        fs::create_dir_all(&nested).unwrap();
        assert_eq!(
            file_for_id(&nested, "031").unwrap().unwrap(),
            PathBuf::from("backlog.d/031-active-ticket.md")
        );
    }

    #[test]
    fn archive_moves_active_ticket_and_stages_rename() {
        let fixture = Fixture::new();
        archive(fixture.path(), "031").unwrap();
        assert!(
            fixture
                .path()
                .join("backlog.d/_done/031-active-ticket.md")
                .is_file()
        );
        assert!(
            !fixture
                .path()
                .join("backlog.d/031-active-ticket.md")
                .exists()
        );
        let staged = git(fixture.path(), &["diff", "--cached", "--name-status"]).unwrap();
        assert!(staged.contains("031-active-ticket.md"));
        assert!(staged.lines().any(|line| line.starts_with('R')));
    }

    #[test]
    fn archive_moves_all_matching_ticket_files() {
        let fixture = Fixture::new();
        fs::write(
            fixture.path().join("backlog.d/031-active-ticket.ctx.md"),
            "# Context packet\n",
        )
        .unwrap();
        run_git(
            fixture.path(),
            &["add", "backlog.d/031-active-ticket.ctx.md"],
        );
        run_git(
            fixture.path(),
            &["commit", "-q", "-m", "add context packet"],
        );
        archive(fixture.path(), "031").unwrap();
        assert!(
            fixture
                .path()
                .join("backlog.d/_done/031-active-ticket.md")
                .is_file()
        );
        assert!(
            fixture
                .path()
                .join("backlog.d/_done/031-active-ticket.ctx.md")
                .is_file()
        );
        assert!(
            !fixture
                .path()
                .join("backlog.d/031-active-ticket.md")
                .exists()
        );
        assert!(
            !fixture
                .path()
                .join("backlog.d/031-active-ticket.ctx.md")
                .exists()
        );
    }

    #[test]
    fn archive_is_idempotent_for_already_done() {
        let fixture = Fixture::new();
        archive(fixture.path(), "007").unwrap();
    }

    #[test]
    fn archive_is_idempotent_after_archiving_and_committing() {
        let fixture = Fixture::new();
        archive(fixture.path(), "031").unwrap();
        run_git(fixture.path(), &["commit", "-q", "-m", "archive 031"]);
        archive(fixture.path(), "031").unwrap();
    }

    #[test]
    fn archive_unknown_or_invalid_fails() {
        let fixture = Fixture::new();
        let unknown = archive(fixture.path(), "999").unwrap_err().to_string();
        assert!(unknown.contains("no ticket file found"));
        let invalid = archive(fixture.path(), "abc").unwrap_err().to_string();
        assert!(invalid.contains("backlog_archive: invalid ID 'abc'"));
    }

    #[test]
    fn archive_fails_outside_git_repo() {
        let temp = TempDir::new().unwrap();
        let error = archive(temp.path(), "031").unwrap_err().to_string();
        assert!(error.contains("backlog_archive: not in a git repo"));
    }
}
