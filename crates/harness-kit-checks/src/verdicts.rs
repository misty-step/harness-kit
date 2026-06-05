use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde_json::Value;

pub const VERDICTS_REF_PREFIX: &str = "refs/verdicts";
const REQUIRED_FIELDS: &[&str] = &["branch", "verdict", "sha", "date"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landable {
    Yes,
    No,
    DontShip,
}

pub fn write(repo: &Path, branch: &str, json: &str) -> Result<()> {
    validate_json(json).context("verdict_write: invalid JSON or missing required fields")?;
    let blob_sha = git_with_stdin(repo, &["hash-object", "-w", "--stdin"], json)?
        .trim()
        .to_string();
    git(repo, &["update-ref", &verdict_ref(branch), &blob_sha]).map(|_| ())
}

pub fn read(repo: &Path, branch: &str) -> Result<String> {
    let reference = verdict_ref(branch);
    git(repo, &["rev-parse", "--verify", &reference])?;
    git(repo, &["cat-file", "-p", &reference])
}

pub fn validate(repo: &Path, branch: &str) -> Result<()> {
    let target = format!("{branch}^{{commit}}");
    let head_sha = git(repo, &["rev-parse", "--verify", &target])
        .map(|output| output.trim().to_string())
        .map_err(|_| anyhow::anyhow!("verdict_validate: could not resolve target '{branch}'"))?;
    let json = read(repo, branch)?;
    let verdict_sha = parse_field(&json, "sha")?;
    if verdict_sha == head_sha {
        Ok(())
    } else {
        bail!("verdict is stale for {branch}")
    }
}

pub fn check_landable(repo: &Path, branch: &str) -> Result<Landable> {
    if validate(repo, branch).is_err() {
        return Ok(Landable::No);
    }
    let json = read(repo, branch)?;
    match parse_field(&json, "verdict")?.as_str() {
        "ship" | "conditional" => Ok(Landable::Yes),
        "dont-ship" => Ok(Landable::DontShip),
        _ => Ok(Landable::No),
    }
}

pub fn delete(repo: &Path, branch: &str) -> Result<()> {
    git(repo, &["update-ref", "-d", &verdict_ref(branch)]).map(|_| ())
}

pub fn list(repo: &Path) -> Result<String> {
    git(
        repo,
        &[
            "for-each-ref",
            &format!("{VERDICTS_REF_PREFIX}/"),
            "--format=%(refname:short)",
        ],
    )
}

pub fn push(repo: &Path, remote: &str) -> Result<()> {
    let refs = git(
        repo,
        &[
            "for-each-ref",
            "--format=%(refname)",
            &format!("{VERDICTS_REF_PREFIX}/"),
        ],
    )?;
    if refs.trim().is_empty() {
        return Ok(());
    }
    git(
        repo,
        &[
            "push",
            remote,
            &format!("{VERDICTS_REF_PREFIX}/*:{VERDICTS_REF_PREFIX}/*"),
        ],
    )
    .map(|_| ())
}

pub fn fetch(repo: &Path, remote: &str) -> Result<()> {
    let remote_refs = git(
        repo,
        &[
            "ls-remote",
            "--refs",
            remote,
            &format!("{VERDICTS_REF_PREFIX}/*"),
        ],
    )?;
    if remote_refs.trim().is_empty() {
        return Ok(());
    }
    git(
        repo,
        &[
            "fetch",
            remote,
            &format!("{VERDICTS_REF_PREFIX}/*:{VERDICTS_REF_PREFIX}/*"),
        ],
    )
    .map(|_| ())
}

pub fn validate_json(json: &str) -> Result<()> {
    let value: Value = serde_json::from_str(json)?;
    let Some(object) = value.as_object() else {
        bail!("verdict JSON must be an object");
    };
    for field in REQUIRED_FIELDS {
        if !object.contains_key(*field) {
            bail!("missing required verdict field: {field}");
        }
    }
    Ok(())
}

fn parse_field(json: &str, field: &str) -> Result<String> {
    let value: Value = serde_json::from_str(json)?;
    value
        .get(field)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .with_context(|| format!("verdict JSON field must be a string: {field}"))
}

fn verdict_ref(branch: &str) -> String {
    format!("{VERDICTS_REF_PREFIX}/{branch}")
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
    child
        .stdin
        .as_mut()
        .context("failed to open git stdin")?
        .write_all(stdin.as_bytes())
        .context("failed to write git stdin")?;
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
            std::fs::create_dir_all(temp.path().join(".empty-hooks")).unwrap();
            run_git(
                temp.path(),
                &["commit", "--allow-empty", "-m", "initial", "-q"],
            );
            run_git(temp.path(), &["checkout", "-b", "feat-foo", "-q"]);
            run_git(
                temp.path(),
                &["commit", "--allow-empty", "-m", "feat commit", "-q"],
            );
            Self { temp }
        }

        fn path(&self) -> &Path {
            self.temp.path()
        }

        fn head(&self) -> String {
            git(self.path(), &["rev-parse", "HEAD"])
                .unwrap()
                .trim()
                .to_string()
        }

        fn json(&self, branch: &str, verdict: &str) -> String {
            format!(
                "{{\"branch\":\"{branch}\",\"base\":\"master\",\"verdict\":\"{verdict}\",\"reviewers\":[\"critic\"],\"scores\":{{\"correctness\":8}},\"sha\":\"{}\",\"date\":\"2026-04-06T15:00:00Z\"}}",
                self.head()
            )
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
    fn write_creates_ref() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "ship")).unwrap();
        assert!(
            git(
                repo.path(),
                &["rev-parse", "--verify", "refs/verdicts/feat-foo"]
            )
            .is_ok()
        );
    }

    #[test]
    fn read_returns_exact_json() {
        let repo = Repo::new();
        let json = repo.json("feat-foo", "ship");
        write(repo.path(), "feat-foo", &json).unwrap();
        assert_eq!(read(repo.path(), "feat-foo").unwrap(), json);
    }

    #[test]
    fn read_nonexistent_fails() {
        let repo = Repo::new();
        assert!(read(repo.path(), "no-such-branch").is_err());
    }

    #[test]
    fn validate_passes_when_sha_matches() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "ship")).unwrap();
        validate(repo.path(), "feat-foo").unwrap();
    }

    #[test]
    fn validate_fails_when_sha_stale() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "ship")).unwrap();
        run_git(
            repo.path(),
            &["commit", "--allow-empty", "-m", "post-review commit", "-q"],
        );
        assert!(validate(repo.path(), "feat-foo").is_err());
    }

    #[test]
    fn validate_fails_without_verdict() {
        let repo = Repo::new();
        run_git(repo.path(), &["branch", "no-verdict-branch"]);
        assert!(validate(repo.path(), "no-verdict-branch").is_err());
    }

    #[test]
    fn validate_explains_unresolved_branch_without_verdict() {
        let repo = Repo::new();
        let error = validate(repo.path(), "no-verdict-branch")
            .unwrap_err()
            .to_string();
        assert!(error.contains("could not resolve target 'no-verdict-branch'"));
    }

    #[test]
    fn validate_fails_for_unresolved_branch_even_when_sha_matches_head() {
        let repo = Repo::new();
        write(
            repo.path(),
            "missing-branch",
            &repo.json("missing-branch", "ship"),
        )
        .unwrap();
        let error = validate(repo.path(), "missing-branch")
            .unwrap_err()
            .to_string();
        assert!(error.contains("could not resolve target 'missing-branch'"));
    }

    #[test]
    fn delete_removes_ref() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "ship")).unwrap();
        delete(repo.path(), "feat-foo").unwrap();
        assert!(read(repo.path(), "feat-foo").is_err());
    }

    #[test]
    fn list_shows_verdicts() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "ship")).unwrap();
        assert_eq!(list(repo.path()).unwrap().trim(), "verdicts/feat-foo");
    }

    #[test]
    fn write_rejects_invalid_json() {
        let repo = Repo::new();
        let error = write(repo.path(), "feat-foo", "not json")
            .unwrap_err()
            .to_string();
        assert!(error.contains("verdict_write: invalid JSON or missing required fields"));
    }

    #[test]
    fn write_rejects_missing_fields() {
        let repo = Repo::new();
        let error = write(
            repo.path(),
            "feat-foo",
            "{\"branch\":\"feat-foo\",\"verdict\":\"ship\"}",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("verdict_write: invalid JSON or missing required fields"));
    }

    #[test]
    fn write_preserves_json_bytes_with_edge_characters() {
        let repo = Repo::new();
        let json = format!(
            "{{\"branch\":\"feat-foo\",\"base\":\"master\",\"verdict\":\"ship\",\"reviewers\":[\"critic\"],\"scores\":{{\"correctness\":8}},\"sha\":\"{}\",\"date\":\"2026-04-06T15:00:00Z\",\"note\":\"edge chars: \\\"quote\\\" \\\\backslash $dollar [brackets]\"}}",
            repo.head()
        );
        write(repo.path(), "feat-foo", &json).unwrap();
        let actual_size = git(repo.path(), &["cat-file", "-s", "refs/verdicts/feat-foo"]).unwrap();
        assert_eq!(actual_size.trim().parse::<usize>().unwrap(), json.len());
    }

    #[test]
    fn check_landable_passes_ship() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "ship")).unwrap();
        assert_eq!(
            check_landable(repo.path(), "feat-foo").unwrap(),
            Landable::Yes
        );
    }

    #[test]
    fn check_landable_passes_conditional() {
        let repo = Repo::new();
        write(
            repo.path(),
            "feat-foo",
            &repo.json("feat-foo", "conditional"),
        )
        .unwrap();
        assert_eq!(
            check_landable(repo.path(), "feat-foo").unwrap(),
            Landable::Yes
        );
    }

    #[test]
    fn check_landable_rejects_dont_ship() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "dont-ship")).unwrap();
        assert_eq!(
            check_landable(repo.path(), "feat-foo").unwrap(),
            Landable::DontShip
        );
    }

    #[test]
    fn check_landable_rejects_missing() {
        let repo = Repo::new();
        assert_eq!(
            check_landable(repo.path(), "no-verdict-branch").unwrap(),
            Landable::No
        );
    }

    #[test]
    fn check_landable_rejects_stale() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "ship")).unwrap();
        run_git(
            repo.path(),
            &["commit", "--allow-empty", "-m", "post-review", "-q"],
        );
        assert_eq!(
            check_landable(repo.path(), "feat-foo").unwrap(),
            Landable::No
        );
    }

    #[test]
    fn check_landable_rejects_unknown_verdict() {
        let repo = Repo::new();
        write(repo.path(), "feat-foo", &repo.json("feat-foo", "unknown")).unwrap();
        assert_eq!(
            check_landable(repo.path(), "feat-foo").unwrap(),
            Landable::No
        );
    }

    #[test]
    fn push_fetch_syncs_between_clones() {
        let temp = TempDir::new().unwrap();
        let remote = temp.path().join("remote.git");
        let clone_a = temp.path().join("clone-a");
        let clone_b = temp.path().join("clone-b");
        run_git(
            temp.path(),
            &["init", "--bare", "-q", remote.to_str().unwrap()],
        );
        run_git(
            temp.path(),
            &[
                "clone",
                "-q",
                remote.to_str().unwrap(),
                clone_a.to_str().unwrap(),
            ],
        );
        run_git(&clone_a, &["checkout", "-q", "-b", "master"]);
        run_git(&clone_a, &["config", "user.name", "Test User"]);
        run_git(&clone_a, &["config", "user.email", "test@example.com"]);
        run_git(
            &clone_a,
            &["commit", "--allow-empty", "-m", "initial", "-q"],
        );
        run_git(&clone_a, &["push", "-q", "-u", "origin", "master"]);
        run_git(&clone_a, &["checkout", "-b", "feat-sync", "-q"]);
        run_git(
            &clone_a,
            &["commit", "--allow-empty", "-m", "feat sync", "-q"],
        );
        run_git(&clone_a, &["push", "-q", "-u", "origin", "feat-sync"]);
        run_git(
            temp.path(),
            &[
                "clone",
                "-q",
                remote.to_str().unwrap(),
                clone_b.to_str().unwrap(),
            ],
        );

        let sha = git(&clone_a, &["rev-parse", "HEAD"])
            .unwrap()
            .trim()
            .to_string();
        let json = format!(
            "{{\"branch\":\"feat-sync\",\"base\":\"master\",\"verdict\":\"ship\",\"reviewers\":[\"critic\"],\"scores\":{{}},\"sha\":\"{sha}\",\"date\":\"2026-04-06T15:00:00Z\"}}"
        );
        write(&clone_a, "feat-sync", &json).unwrap();
        push(&clone_a, "origin").unwrap();
        fetch(&clone_b, "origin").unwrap();
        assert_eq!(read(&clone_b, "feat-sync").unwrap(), json);
    }

    #[test]
    fn push_no_verdicts_is_noop() {
        let repo = Repo::new();
        run_git(repo.path(), &["init", "--bare", "-q", "noop.git"]);
        run_git(repo.path(), &["remote", "add", "origin", "noop.git"]);
        push(repo.path(), "origin").unwrap();
    }

    #[test]
    fn fetch_no_remote_verdicts_is_noop() {
        let repo = Repo::new();
        run_git(
            repo.path(),
            &["init", "--bare", "-q", "no-remote-verdicts.git"],
        );
        run_git(
            repo.path(),
            &["remote", "add", "origin", "no-remote-verdicts.git"],
        );
        fetch(repo.path(), "origin").unwrap();
    }

    #[test]
    fn fetch_missing_remote_fails() {
        let repo = Repo::new();
        assert!(fetch(repo.path(), "missing-remote").is_err());
    }
}
