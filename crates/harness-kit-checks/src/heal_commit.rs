use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::TempDir;

use crate::heal_support;

pub fn run(args: &[String], repo_root: &Path) -> Result<String> {
    touch_env(repo_root)?;
    ensure_empty_index(repo_root)?;
    let before = TempDir::new().context("failed to create snapshot dir")?;
    let stage_plan = tempfile::NamedTempFile::new().context("failed to create stage plan")?;
    copy_snapshot(repo_root, before.path())?;

    let check_output = run_dagger(repo_root, &["call", "check"], false)?;
    let gate = heal_support::first_failed_gate(&check_output);
    let Some(gate) = gate else {
        print!("{check_output}");
        if check_output_has_failure_status(&check_output) {
            bail!("dagger check failed without a parseable gate");
        }
        return Ok(String::new());
    };
    let branch = heal_support::repair_branch_name(&gate);
    let commit_message = heal_support::repair_commit_message(&gate);

    let mut heal_args = vec![
        "call".to_string(),
        "--allow-llm".to_string(),
        "all".to_string(),
        "-o".to_string(),
        ".".to_string(),
        "heal".to_string(),
    ];
    heal_args.extend(args.iter().cloned());
    let heal_refs = heal_args.iter().map(String::as_str).collect::<Vec<_>>();
    run_dagger(repo_root, &heal_refs, true)?;
    run_dagger(repo_root, &["call", "check"], true)?;
    run_checked(
        Command::new("git")
            .args(["switch", "-c", &branch])
            .current_dir(repo_root),
        "git switch",
    )?;

    let (stage, remove) = heal_support::snapshot_delta(before.path(), repo_root)?;
    let mut plan = String::new();
    for path in &remove {
        plan.push_str(&format!("D\t{path}\n"));
    }
    for path in &stage {
        plan.push_str(&format!("S\t{path}\n"));
    }
    fs::write(stage_plan.path(), plan)?;

    for path in remove.iter().chain(stage.iter()) {
        apply_snapshot_patch(repo_root, before.path(), path)?;
    }
    if git_diff_cached_quiet(repo_root)? {
        bail!("heal produced no commit-ready diff");
    }
    run_checked(
        Command::new("git")
            .args(["commit", "-m", &commit_message])
            .current_dir(repo_root),
        "git commit",
    )?;
    let message = format!("Healed {gate} on {branch}");
    println!("{message}");
    Ok(message)
}

fn touch_env(repo_root: &Path) -> Result<()> {
    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(repo_root.join(".env"))
        .with_context(|| "failed to touch .env")?;
    Ok(())
}

fn ensure_empty_index(repo_root: &Path) -> Result<()> {
    if !git_diff_cached_quiet(repo_root)? {
        bail!("heal requires an empty index; unstage or commit existing staged changes first");
    }
    Ok(())
}

fn git_diff_cached_quiet(repo_root: &Path) -> Result<bool> {
    let status = Command::new("git")
        .args(["diff", "--cached", "--quiet"])
        .current_dir(repo_root)
        .status()
        .context("failed to run git diff --cached --quiet")?;
    Ok(status.success())
}

fn copy_snapshot(repo_root: &Path, destination: &Path) -> Result<()> {
    copy_dir_filtered(repo_root, destination, &|path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| matches!(name, ".git" | ".env"))
    })
}

fn apply_snapshot_patch(repo_root: &Path, before_root: &Path, path: &str) -> Result<()> {
    let before_path = before_root.join(path);
    let after_path = repo_root.join(path);
    let patch = tempfile::NamedTempFile::new().context("failed to create patch file")?;

    let mut command = Command::new("diff");
    command.arg("-u");
    command.args(["--label", &format!("a/{path}")]);
    command.args(["--label", &format!("b/{path}")]);
    command.arg("--");
    if before_path.exists() {
        command.arg(&before_path);
    } else {
        command.arg("/dev/null");
    }
    if after_path.exists() {
        command.arg(&after_path);
    } else {
        command.arg("/dev/null");
    }
    let output = command.output().context("failed to compute repair patch")?;
    if !output.status.success() && output.status.code() != Some(1) {
        bail!("failed to compute a repair patch for {path}");
    }
    if output.stdout.is_empty() {
        return Ok(());
    }
    fs::write(patch.path(), output.stdout)?;
    let status = Command::new("git")
        .args(["apply", "--cached", "--whitespace=nowarn"])
        .arg(patch.path())
        .current_dir(repo_root)
        .status()
        .context("failed to run git apply")?;
    if !status.success() {
        bail!(
            "heal cannot stage {path} without including pre-existing edits; commit the repair manually"
        );
    }
    Ok(())
}

fn run_dagger(repo_root: &Path, args: &[&str], require_success: bool) -> Result<String> {
    let output = Command::new("dagger")
        .args(args)
        .env(
            "DAGGER_NO_NAG",
            std::env::var("DAGGER_NO_NAG").unwrap_or_else(|_| "1".to_string()),
        )
        .current_dir(repo_root)
        .output()
        .with_context(|| "failed to run dagger")?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if require_success && !output.status.success() {
        bail!("{combined}");
    }
    Ok(combined)
}

fn check_output_has_failure_status(output: &str) -> bool {
    output.lines().any(|line| line.contains("  FAIL  "))
}

fn run_checked(command: &mut Command, label: &str) -> Result<()> {
    let output = command
        .output()
        .with_context(|| format!("failed to run {label}"))?;
    if !output.status.success() {
        bail!(
            "{label} failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn copy_dir_filtered(
    source: &Path,
    destination: &Path,
    skip: &impl Fn(&Path) -> bool,
) -> Result<()> {
    if skip(source) {
        return Ok(());
    }
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let source_path = entry?.path();
        if skip(&source_path) {
            continue;
        }
        let destination_path = destination.join(
            source_path
                .file_name()
                .context("source path missing file name")?,
        );
        if source_path.is_dir() {
            copy_dir_filtered(&source_path, &destination_path, skip)?;
        } else {
            fs::copy(&source_path, destination_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_output_failure_status_detects_fail_lines() {
        assert!(check_output_has_failure_status("  FAIL  lint-shell\n"));
        assert!(!check_output_has_failure_status("  PASS  lint-shell\n"));
    }

    #[test]
    fn snapshot_copy_excludes_git_and_env() {
        let source = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        fs::write(source.path().join("file.txt"), "ok\n").unwrap();
        fs::write(source.path().join(".env"), "secret\n").unwrap();
        fs::create_dir(source.path().join(".git")).unwrap();
        fs::write(source.path().join(".git/HEAD"), "ref\n").unwrap();
        copy_snapshot(source.path(), dest.path()).unwrap();
        assert!(dest.path().join("file.txt").exists());
        assert!(!dest.path().join(".env").exists());
        assert!(!dest.path().join(".git").exists());
    }
}
