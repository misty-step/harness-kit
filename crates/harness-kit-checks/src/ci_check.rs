use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::{docs_site, frontmatter, generate_index, lint_gates};

pub fn run(repo: &Path) -> Result<Vec<String>> {
    let repo = repo.canonicalize().unwrap_or_else(|_| repo.to_path_buf());
    let mut lines = Vec::new();

    gate(&mut lines, "lint-yaml", || lint_yaml(&repo))?;
    gate(&mut lines, "lint-shell", || lint_shell(&repo))?;
    gate(&mut lines, "check-frontmatter", || {
        frontmatter::check_repo(&repo)?.ensure_success()
    })?;
    gate(&mut lines, "check-index-drift", || {
        generate_index::check_drift(&repo, chrono::Utc::now())
    })?;
    gate(&mut lines, "check-docs-site", || {
        docs_site::validate_site(&docs_site::CheckOptions {
            repo_root: repo.to_path_buf(),
            site: docs_site::default_site(&repo),
            compare_to_rebuild: true,
        })
    })?;
    gate_report(&mut lines, "check-vendored-copies", || {
        lint_gates::check_vendored_copies(&repo)
    })?;
    gate_report(&mut lines, "check-exclusions", || {
        lint_gates::check_exclusions(&repo)
    })?;
    gate_report(&mut lines, "check-conflict-markers", || {
        lint_gates::check_conflict_markers(&repo)
    })?;
    gate_report(&mut lines, "check-portable-paths", || {
        lint_gates::check_portable_paths(&repo)
    })?;
    gate_report(&mut lines, "check-harness-install-paths", || {
        lint_gates::check_harness_install_paths(&repo)
    })?;
    gate_report(&mut lines, "check-no-claims", || {
        lint_gates::check_no_claims(&repo)
    })?;
    gate(&mut lines, "test-sync-external-partial", || {
        let _ = crate::external_sync::self_test_partial_sync()?;
        Ok(())
    })?;
    gate(&mut lines, "bun-test-research", || {
        run_command_in(&repo.join("skills/research"), "bun", &["test"])
    })?;
    gate(&mut lines, "cargo-fmt", || {
        run_command(&repo, "cargo", &["fmt", "--all", "--check"])
    })?;
    gate(&mut lines, "cargo-test", || {
        run_command(&repo, "cargo", &["test", "--workspace", "--locked"])
    })?;
    gate(&mut lines, "cargo-clippy", || {
        run_command(
            &repo,
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--locked",
                "--",
                "-D",
                "warnings",
            ],
        )
    })?;

    lines.push("check: passed".to_string());
    Ok(lines)
}

fn gate<F>(lines: &mut Vec<String>, name: &str, f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    f().with_context(|| format!("{name} failed"))?;
    lines.push(format!("PASS {name}"));
    Ok(())
}

fn gate_report<F>(lines: &mut Vec<String>, name: &str, f: F) -> Result<()>
where
    F: FnOnce() -> Result<lint_gates::GateReport>,
{
    let report = f().with_context(|| format!("{name} failed"))?;
    if !report.errors.is_empty() {
        bail!("{name} failed:\n{}", report.errors.join("\n"));
    }
    lines.push(format!("PASS {name}"));
    Ok(())
}

fn lint_yaml(repo: &Path) -> Result<()> {
    for path in files_with_extensions(repo, &["yaml", "yml"])? {
        let relative = relative_slash(repo, &path);
        if relative.starts_with("ci/") {
            continue;
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let _: serde_yaml::Value = serde_yaml::from_str(&text)
            .with_context(|| format!("invalid YAML in {}", path.display()))?;
    }
    Ok(())
}

fn lint_shell(repo: &Path) -> Result<()> {
    if !command_exists("shellcheck") {
        return Ok(());
    }
    let mut paths = Vec::new();
    for path in files_with_extensions(repo, &["sh"])? {
        let relative = relative_slash(repo, &path);
        if !relative.starts_with("ci/") {
            paths.push(path);
        }
    }
    if paths.is_empty() {
        return Ok(());
    }
    let args = paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let arg_refs = args.iter().map(String::as_str).collect::<Vec<_>>();
    run_command(repo, "shellcheck", &arg_refs)
}

fn files_with_extensions(repo: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_files(repo, repo, extensions, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_files(
    root: &Path,
    dir: &Path,
    extensions: &[&str],
    paths: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        let relative = relative_slash(root, &path);
        if entry.file_type()?.is_dir() {
            if should_skip_dir(&relative) {
                continue;
            }
            collect_files(root, &path, extensions, paths)?;
        } else if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extensions.contains(&extension))
        {
            paths.push(path);
        }
    }
    Ok(())
}

fn should_skip_dir(relative: &str) -> bool {
    matches!(
        relative,
        ".git"
            | "target"
            | ".venv"
            | "__pycache__"
            | "node_modules"
            | "skills/.external"
            | ".harness-kit/tmp"
    )
}

fn run_command(repo: &Path, command: &str, args: &[&str]) -> Result<()> {
    run_command_in(repo, command, args)
}

fn run_command_in(cwd: &Path, command: &str, args: &[&str]) -> Result<()> {
    let output = Command::new(command)
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run {command} {}", args.join(" ")))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    bail!(
        "{} {} failed\n{}{}",
        command,
        args.join(" "),
        stdout,
        stderr
    )
}

fn command_exists(command: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(command).is_file())
}

fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
