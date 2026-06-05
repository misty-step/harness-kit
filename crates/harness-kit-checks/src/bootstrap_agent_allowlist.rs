use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

#[cfg(unix)]
use std::os::unix::fs::symlink;

const ALLOWED_AGENTS: &[&str] = &["a11y-auditor", "a11y-critic", "a11y-fixer"];
const RETIRED_AGENTS: &[&str] = &[
    "beck", "builder", "carmack", "cooper", "critic", "grug", "planner",
];

pub fn run(repo: &Path) -> Result<String> {
    #[cfg(not(unix))]
    bail!("bootstrap agent allowlist smoke requires Unix symlink support");

    #[cfg(unix)]
    run_unix(repo)
}

#[cfg(unix)]
fn run_unix(repo: &Path) -> Result<String> {
    let repo = repo.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize Harness Kit repo path {}",
            repo.display()
        )
    })?;
    let temp = tempfile::tempdir().context("failed to create temporary directory")?;
    let home = temp.path().join("home");
    let claude_agents = home.join(".claude/agents");
    let codex_root = home.join(".codex");
    fs::create_dir_all(&claude_agents)?;
    fs::create_dir_all(&codex_root)?;

    fs::write(claude_agents.join("ousterhout.md"), "user owned\n")?;
    symlink(
        repo.join("agents/critic.md"),
        claude_agents.join("critic.md"),
    )?;
    symlink(repo.join("agents"), codex_root.join("agents"))?;

    let python = python_executable()?;
    let bin = temp.path().join("bin");
    fs::create_dir_all(&bin)?;
    symlink(python, bin.join("python3"))?;
    symlink(
        std::env::current_exe().context("failed to locate current executable")?,
        bin.join("harness-kit-checks"),
    )?;
    let path = format!(
        "{}:/usr/bin:/bin:/usr/sbin:/sbin:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );

    run_bootstrap(&repo, &home, &path)?;
    let second = run_bootstrap(&repo, &home, &path)?;
    let launcher = run_bootstrap_launcher(&repo, &home, &path)?;

    for harness in [home.join(".claude"), home.join(".codex")] {
        let agents = harness.join("agents");
        ensure_dir_not_parent_symlink(&agents)?;
        for agent in ALLOWED_AGENTS {
            assert_symlink_to(
                &agents.join(format!("{agent}.md")),
                &repo.join(format!("agents/{agent}.md")),
            )?;
        }
        for retired in RETIRED_AGENTS {
            assert_not_exists(&agents.join(format!("{retired}.md")))?;
        }
    }

    assert_exists(&claude_agents.join("ousterhout.md"))?;
    let ousterhout = fs::read_to_string(claude_agents.join("ousterhout.md"))?;
    if ousterhout != "user owned\n" {
        bail!("user-owned retired agent was modified");
    }
    assert_not_exists(&home.join(".codex/agents/ousterhout.md"))?;

    if !second.contains("Agents (3):") || !launcher.contains("Agents (3):") {
        bail!("bootstrap summary should report only three global agents");
    }
    Ok("bootstrap agent allowlist ok".to_string())
}

fn python_executable() -> Result<PathBuf> {
    let output = Command::new("python3")
        .args(["-c", "import sys; print(sys.executable)"])
        .output()
        .context("failed to locate python3")?;
    if !output.status.success() {
        bail!(
            "python3 lookup failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        bail!("python3 lookup returned an empty path");
    }
    Ok(PathBuf::from(path))
}

fn run_bootstrap(repo: &Path, home: &Path, path: &str) -> Result<String> {
    let output =
        Command::new(std::env::current_exe().context("failed to locate current executable")?)
            .arg("bootstrap")
            .arg("--repo")
            .arg(repo)
            .env("HARNESS_KIT_DIR", repo)
            .env("HOME", home)
            .env("PATH", path)
            .current_dir(repo)
            .output()
            .context("failed to run harness-kit-checks bootstrap")?;
    if !output.status.success() {
        bail!(
            "harness-kit-checks bootstrap failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_bootstrap_launcher(repo: &Path, home: &Path, path: &str) -> Result<String> {
    let output = Command::new("bash")
        .arg(repo.join("bootstrap.sh"))
        .env("HARNESS_KIT_DIR", repo)
        .env("HOME", home)
        .env("PATH", path)
        .current_dir(repo)
        .output()
        .context("failed to run bootstrap.sh launcher")?;
    if !output.status.success() {
        bail!(
            "bootstrap.sh launcher failed: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn ensure_dir_not_parent_symlink(path: &Path) -> Result<()> {
    assert_exists(path)?;
    if !path.is_dir() {
        bail!("expected agents directory: {}", path.display());
    }
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        bail!(
            "agents directory should not be a parent symlink: {}",
            path.display()
        );
    }
    Ok(())
}

fn assert_exists(path: &Path) -> Result<()> {
    if path.exists() || fs::symlink_metadata(path).is_ok() {
        Ok(())
    } else {
        bail!("missing expected path: {}", path.display())
    }
}

fn assert_not_exists(path: &Path) -> Result<()> {
    if !path.exists() && fs::symlink_metadata(path).is_err() {
        Ok(())
    } else {
        bail!("unexpected path exists: {}", path.display())
    }
}

#[cfg(unix)]
fn assert_symlink_to(path: &Path, target: &Path) -> Result<()> {
    assert_exists(path)?;
    if !fs::symlink_metadata(path)?.file_type().is_symlink() {
        bail!("expected symlink: {}", path.display());
    }
    let actual = fs::read_link(path)?;
    if actual != target {
        bail!(
            "unexpected symlink target for {}: {}",
            path.display(),
            actual.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "integration smoke runs bootstrap.sh; covered by test-bootstrap-agent-allowlist gate"]
    fn bootstrap_agent_allowlist_contract_matches_shell_smoke() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .expect("crate must live under crates/harness-kit-checks");

        assert_eq!(run(repo).unwrap(), "bootstrap agent allowlist ok");
    }

    #[test]
    fn assert_exists_accepts_files_and_symlinks() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("file.md");
        let link = temp.path().join("link.md");
        fs::write(&file, "x").unwrap();
        #[cfg(unix)]
        symlink(&file, &link).unwrap();

        assert_exists(&file).unwrap();
        #[cfg(unix)]
        assert_exists(&link).unwrap();
        assert!(assert_exists(&temp.path().join("missing.md")).is_err());
    }

    #[test]
    fn assert_not_exists_rejects_files_and_symlinks() {
        let temp = tempfile::tempdir().unwrap();
        let file = temp.path().join("file.md");
        let link = temp.path().join("link.md");
        fs::write(&file, "x").unwrap();
        #[cfg(unix)]
        symlink(&file, &link).unwrap();

        assert!(assert_not_exists(&file).is_err());
        #[cfg(unix)]
        assert!(assert_not_exists(&link).is_err());
        assert_not_exists(&temp.path().join("missing.md")).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn assert_symlink_to_checks_exact_target() {
        let temp = tempfile::tempdir().unwrap();
        let target = temp.path().join("target.md");
        let wrong = temp.path().join("wrong.md");
        let link = temp.path().join("link.md");
        fs::write(&target, "target").unwrap();
        fs::write(&wrong, "wrong").unwrap();
        symlink(&target, &link).unwrap();

        assert_symlink_to(&link, &target).unwrap();
        assert!(assert_symlink_to(&link, &wrong).is_err());
    }
}
