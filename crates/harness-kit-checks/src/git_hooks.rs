use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::{bootstrap, docs_site, generate_index, lint_gates, verdicts};

pub fn run_pre_commit(repo: &Path) -> Result<String> {
    let staged = staged_paths(repo)?;
    let deliver_hits = deliver_state_hits(&staged);
    if !deliver_hits.is_empty() {
        bail!(
            "refusing to commit agent-written /deliver state files:\n{}\n\nThese files are rewritten by /deliver and must never be human-edited.\nIf /deliver is stuck, use:\n  /deliver --resume <ulid>\n  /deliver --abandon <ulid>",
            deliver_hits
                .iter()
                .map(|path| format!("  {path}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let mut lines = Vec::new();
    let mut changed = false;
    if skill_changed(&staged) {
        changed = true;
        lines.push("[pre-commit] Skill content changed - regenerating index.yaml".to_string());
        generate_index::write_index(repo, chrono::Utc::now())?;
        git(repo, &["add", "index.yaml"])?;
    }

    if docs_input_changed(&staged) {
        changed = true;
        lines.push("[pre-commit] Docs source changed - regenerating docs/site".to_string());
        let site = docs_site::default_site(repo);
        docs_site::build_site(repo, &site)?;
        git(repo, &["add", "docs/site"])?;

        lines.push("[pre-commit] Verifying generated docs/site".to_string());
        docs_site::validate_site(&docs_site::CheckOptions {
            repo_root: repo.to_path_buf(),
            site,
            compare_to_rebuild: true,
        })?;
    }

    if changed {
        lines.push("[pre-commit] Verifying shared-root install wording".to_string());
        let report = lint_gates::check_harness_install_paths(repo)?;
        if !report.errors.is_empty() {
            bail!("{}", report.errors.join("\n"));
        }
    }

    Ok(lines.join("\n"))
}

pub fn run_pre_push(repo: &Path) -> Result<String> {
    if !repo.join("dagger.json").exists() {
        return Ok(String::new());
    }

    let mut lines = Vec::new();
    if !command_exists("dagger") {
        return Ok("pre-push: dagger not installed, skipping CI gates".to_string());
    }
    if !docker_info_ok()? {
        return Ok("pre-push: Docker not running, skipping CI gates".to_string());
    }

    lines.extend(in_flight_deliver_warnings(repo)?);
    lines.push("pre-push: running dagger call check...".to_string());
    if !run_command(repo, "dagger", &["call", "check"])? {
        bail!("pre-push: CI gates failed. Fix issues before pushing.");
    }
    Ok(lines.join("\n"))
}

pub fn run_pre_merge_commit(repo: &Path) -> Result<String> {
    run_pre_merge_commit_with_env(repo, env_map())
}

pub fn run_post_commit(repo: &Path) -> Result<String> {
    let changed = git_output(repo, &["diff", "--name-only", "HEAD~1", "HEAD"]).unwrap_or_default();
    if !bootstrap_relevant_changed(changed.lines()) {
        return Ok(String::new());
    }
    let output = bootstrap::run(&bootstrap::BootstrapOptions::from_env(Some(
        repo.to_path_buf(),
    ))?)?;
    Ok(format!(
        "[post-commit] Harness Kit harness content changed - re-linking\n{output}"
    ))
}

pub fn run_post_merge(repo: &Path) -> Result<String> {
    bootstrap::run(&bootstrap::BootstrapOptions::from_env(Some(
        repo.to_path_buf(),
    ))?)
}

pub fn run_post_rewrite(repo: &Path, args: &[String]) -> Result<String> {
    if args.first().map(String::as_str) != Some("rebase") {
        return Ok(String::new());
    }
    run_post_merge(repo)
}

fn run_pre_merge_commit_with_env(repo: &Path, env: BTreeMap<String, String>) -> Result<String> {
    let mut lines = Vec::new();
    if env.get("HARNESS_KIT_NO_REVIEW").map(String::as_str) == Some("1") {
        lines
            .push("pre-merge-commit: bypassing verdict gate (HARNESS_KIT_NO_REVIEW=1)".to_string());
    } else if let Some(branch) = merge_topic_branch(&env) {
        match verdicts::check_landable(repo, &branch)? {
            verdicts::Landable::Yes => {}
            verdicts::Landable::No => bail!(
                "pre-merge-commit: no valid verdict for '{branch}'.\n  Run /code-review first, or bypass with:\n  HARNESS_KIT_NO_REVIEW=1 git merge --no-ff \"{branch}\""
            ),
            verdicts::Landable::DontShip => bail!(
                "pre-merge-commit: verdict is 'dont-ship' for '{branch}'.\n  Address review findings and re-run /code-review."
            ),
        }
    } else {
        lines.push(
            "pre-merge-commit: cannot determine topic branch - allowing verdict gate".to_string(),
        );
    }

    match current_branch(repo)?.as_deref() {
        Some("master" | "main") => {}
        _ => return Ok(lines.join("\n")),
    }
    if !repo.join("dagger.json").exists() {
        return Ok(lines.join("\n"));
    }
    if env.get("HARNESS_KIT_NO_DAGGER").map(String::as_str) == Some("1") {
        lines.push("pre-merge-commit: bypassing Dagger gate (HARNESS_KIT_NO_DAGGER=1)".to_string());
        return Ok(lines.join("\n"));
    }

    if !command_exists("dagger") {
        bail!(
            "pre-merge-commit: dagger not installed; cannot merge without Dagger gate.\n  Install dagger or bypass explicitly with HARNESS_KIT_NO_DAGGER=1."
        );
    }
    if !command_exists("docker") || !docker_info_ok()? {
        bail!(
            "pre-merge-commit: Docker is unavailable; cannot run Dagger gate.\n  Start Docker or bypass explicitly with HARNESS_KIT_NO_DAGGER=1."
        );
    }

    lines.push("pre-merge-commit: running dagger call check --source=.".to_string());
    if !run_command(repo, "dagger", &["call", "check", "--source=."])? {
        bail!(
            "pre-merge-commit: dagger call check --source=. failed.\n  Run /ci or dagger call check --source=. before merging.\n  Bypass explicitly with HARNESS_KIT_NO_DAGGER=1."
        );
    }
    Ok(lines.join("\n"))
}

fn staged_paths(repo: &Path) -> Result<Vec<String>> {
    let output = git_output(repo, &["diff", "--cached", "--name-only"])?;
    Ok(output.lines().map(str::to_string).collect())
}

fn deliver_state_hits(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter(|path| {
            path.starts_with(".harness-kit/deliver/")
                && (path.ends_with("/state.json") || path.ends_with("/receipt.json"))
        })
        .cloned()
        .collect()
}

fn skill_changed(paths: &[String]) -> bool {
    paths.iter().any(|path| {
        (path.starts_with("skills/") && path.ends_with("/SKILL.md"))
            || (path.starts_with("agents/") && path.ends_with(".md"))
    })
}

fn docs_input_changed(paths: &[String]) -> bool {
    paths.iter().any(|path| {
        (path.starts_with("skills/") && path.ends_with("/SKILL.md"))
            || (path.starts_with("agents/") && path.ends_with(".md"))
            || path == "AGENTS.md"
            || path == "harnesses/shared/AGENTS.md"
            || path.starts_with("docs/copy/")
            || path == "docs/positioning.md"
            || (path.starts_with("backlog.d/") && path.ends_with(".md"))
            || path == "bootstrap.sh"
            || path == "ci/src/harness_kit_ci/main.py"
            || path.starts_with("crates/harness-kit-checks/src/docs_site.rs")
            || path.starts_with("crates/harness-kit-checks/src/generate_index.rs")
    })
}

fn bootstrap_relevant_changed<'a>(paths: impl IntoIterator<Item = &'a str>) -> bool {
    paths.into_iter().any(|path| {
        path.starts_with("skills/")
            || path.starts_with("agents/")
            || path.starts_with("harnesses/")
            || path.starts_with(".githooks/")
            || path == "bootstrap.sh"
    })
}

fn merge_topic_branch(env: &BTreeMap<String, String>) -> Option<String> {
    if let Some(action) = env.get("GIT_REFLOG_ACTION")
        && let Some(branch) = action.strip_prefix("merge ")
        && !branch.is_empty()
    {
        return Some(branch.to_string());
    }
    env.iter()
        .find_map(|(key, value)| key.starts_with("GITHEAD_").then_some(value))
        .filter(|branch| !branch.is_empty())
        .cloned()
}

fn in_flight_deliver_warnings(repo: &Path) -> Result<Vec<String>> {
    let root = repo.join(".harness-kit/deliver");
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut warnings = Vec::new();
    for state in state_files(&root)? {
        let Ok(text) = fs::read_to_string(&state) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(status) = value.get("status").and_then(Value::as_str) else {
            continue;
        };
        if status == "merge_ready" {
            continue;
        }
        let ulid = state
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        warnings.push(format!(
            "pre-push: in-flight /deliver run detected (ulid={ulid}, status={status})"
        ));
        warnings.push(format!("  -> /deliver --resume {ulid}    # finish the run"));
        warnings.push(format!(
            "  -> /deliver --abandon {ulid}   # clear state, keep branch"
        ));
    }
    Ok(warnings)
}

fn state_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for run in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let run = run?;
        if !run.file_type()?.is_dir() {
            continue;
        }
        let state = run.path().join("state.json");
        if state.is_file() {
            files.push(state);
        }
    }
    files.sort();
    Ok(files)
}

fn current_branch(repo: &Path) -> Result<Option<String>> {
    match git_output(repo, &["symbolic-ref", "--quiet", "--short", "HEAD"]) {
        Ok(output) => Ok(Some(output.trim().to_string()).filter(|branch| !branch.is_empty())),
        Err(_) => Ok(None),
    }
}

fn env_map() -> BTreeMap<String, String> {
    std::env::vars().collect()
}

fn command_exists(command: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| is_executable(dir.join(command)))
}

fn is_executable(path: PathBuf) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn docker_info_ok() -> Result<bool> {
    run_command(Path::new("."), "docker", &["info"])
}

fn run_command(repo: &Path, command: &str, args: &[&str]) -> Result<bool> {
    let status = Command::new(command)
        .args(args)
        .current_dir(repo)
        .status()
        .with_context(|| format!("failed to run {command} {}", args.join(" ")))?;
    Ok(status.success())
}

fn git(repo: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !status.success() {
        bail!("git {} failed with {status}", args.join(" "));
    }
    Ok(())
}

fn git_output(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detects_deliver_state_files() {
        let paths = vec![
            ".harness-kit/deliver/01/state.json".to_string(),
            ".harness-kit/deliver/01/receipt.json".to_string(),
            ".harness-kit/traces/delegations.jsonl".to_string(),
        ];
        assert_eq!(
            deliver_state_hits(&paths),
            vec![
                ".harness-kit/deliver/01/state.json".to_string(),
                ".harness-kit/deliver/01/receipt.json".to_string()
            ]
        );
    }

    #[test]
    fn classifies_index_and_docs_inputs() {
        assert!(skill_changed(&["skills/ci/SKILL.md".to_string()]));
        assert!(skill_changed(&["agents/a11y-auditor.md".to_string()]));
        assert!(!skill_changed(&[
            "skills/ci/references/audit.md".to_string()
        ]));

        assert!(docs_input_changed(
            &["backlog.d/090-example.md".to_string()]
        ));
        assert!(docs_input_changed(&[
            "crates/harness-kit-checks/src/docs_site.rs".to_string()
        ]));
        assert!(docs_input_changed(&[
            "ci/src/harness_kit_ci/main.py".to_string()
        ]));
        assert!(!docs_input_changed(&["README.md".to_string()]));
    }

    #[test]
    fn detects_bootstrap_relevant_changes() {
        assert!(bootstrap_relevant_changed(["skills/ci/SKILL.md"]));
        assert!(bootstrap_relevant_changed(["harnesses/shared/AGENTS.md"]));
        assert!(bootstrap_relevant_changed([".githooks/pre-push"]));
        assert!(bootstrap_relevant_changed(["bootstrap.sh"]));
        assert!(!bootstrap_relevant_changed(["README.md"]));
    }

    #[test]
    fn extracts_merge_topic_branch_from_git_env() {
        let mut env = BTreeMap::new();
        env.insert("GIT_REFLOG_ACTION".to_string(), "merge feat-x".to_string());
        assert_eq!(merge_topic_branch(&env), Some("feat-x".to_string()));

        env.insert("GIT_REFLOG_ACTION".to_string(), "commit".to_string());
        env.insert("GITHEAD_abc123".to_string(), "feat-y".to_string());
        assert_eq!(merge_topic_branch(&env), Some("feat-y".to_string()));

        env.clear();
        assert_eq!(merge_topic_branch(&env), None);
    }

    #[test]
    fn reads_in_flight_deliver_warnings_without_python() {
        let temp = TempDir::new().unwrap();
        let deliver = temp.path().join(".harness-kit/deliver");
        fs::create_dir_all(deliver.join("01")).unwrap();
        fs::create_dir_all(deliver.join("02")).unwrap();
        fs::write(deliver.join("01/state.json"), r#"{"status":"running"}"#).unwrap();
        fs::write(deliver.join("02/state.json"), r#"{"status":"merge_ready"}"#).unwrap();

        let warnings = in_flight_deliver_warnings(temp.path()).unwrap();
        assert_eq!(
            warnings,
            vec![
                "pre-push: in-flight /deliver run detected (ulid=01, status=running)".to_string(),
                "  -> /deliver --resume 01    # finish the run".to_string(),
                "  -> /deliver --abandon 01   # clear state, keep branch".to_string()
            ]
        );
    }
}
