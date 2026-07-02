//! Installs the `harness-kit-checks` binary into `~/.harness-kit/bin/`.
//!
//! Split out of `bootstrap.rs` (backlog.d/133) because freshness here is
//! surprisingly subtle: see `freshest_cli_build`'s doc comment.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::bootstrap::{blue, green};

/// Picks the freshest build to install. `env::current_exe()` is NOT a
/// reliable source of truth here: once `~/.harness-kit/bin/harness-kit-checks`
/// is on `PATH`, git hooks invoke it directly (see `.githooks/pre-commit`'s
/// `command -v harness-kit-checks` preference), so `bootstrap::run` executes
/// *as* the installed binary and `current_exe()` permanently points at the
/// install destination — even when a newer `cargo build` sits in `target/`
/// untouched. Prefer the repo's own fresh build artifacts; fall back to
/// `current_exe()` only when neither exists (e.g. a bare install with no
/// local `target/` directory).
fn freshest_cli_build(repo: &Path) -> Result<PathBuf> {
    let candidates = [
        repo.join("target/release/harness-kit-checks"),
        repo.join("target/debug/harness-kit-checks"),
    ];
    let mut newest: Option<(PathBuf, std::time::SystemTime)> = None;
    for candidate in candidates {
        let Ok(metadata) = fs::metadata(&candidate) else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if newest.as_ref().is_none_or(|(_, t)| modified > *t) {
            newest = Some((candidate, modified));
        }
    }
    match newest {
        Some((path, _)) => Ok(path),
        None => Ok(env::current_exe()?),
    }
}

fn file_sha256(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

#[cfg(unix)]
pub(crate) fn install_cli(repo: &Path, home: &Path, lines: &mut Vec<String>) -> Result<()> {
    lines.push(blue("Installing Rust CLI..."));
    let bin_dir = home.join(".harness-kit/bin");
    fs::create_dir_all(&bin_dir)?;
    let destination = bin_dir.join("harness-kit-checks");
    let source = freshest_cli_build(repo)?;

    // When the source IS the installed one (e.g. no local target/ build
    // exists and current_exe() falls back to the destination itself),
    // fs::copy(src, src) truncates it to zero bytes. Skip the self-copy.
    let same_file = match (source.canonicalize(), destination.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    // Content identity, not path identity: the installed binary staying
    // "current" must depend on whether its bytes match the freshest build,
    // never on which process happens to be executing this bootstrap run.
    let up_to_date = same_file
        || match (file_sha256(&source), file_sha256(&destination)) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        };
    if up_to_date {
        lines.push(green("    bin/harness-kit-checks (already current)"));
        lines.push(String::new());
        return Ok(());
    }
    // Copy via temp + rename so a concurrent invocation never sees a
    // half-written binary.
    let staging = bin_dir.join(".harness-kit-checks.tmp");
    fs::copy(&source, &staging)
        .with_context(|| format!("failed to stage {}", staging.display()))?;
    fs::rename(&staging, &destination)
        .with_context(|| format!("failed to install {}", destination.display()))?;
    lines.push(green("    bin/harness-kit-checks"));
    lines.push(String::new());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // backlog.d/133: install_cli must refresh the installed binary based on
    // build content, not on whether the *running* process happens to be the
    // installed copy — current_exe() is not a reliable freshness signal once
    // hooks invoke the installed binary directly.
    #[test]
    fn install_cli_refreshes_stale_installed_binary_from_fresh_build() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let home = tempfile::tempdir()?;

        fs::create_dir_all(repo.path().join("target/debug"))?;
        fs::write(
            repo.path().join("target/debug/harness-kit-checks"),
            b"fresh-build-content",
        )?;

        let bin_dir = home.path().join(".harness-kit/bin");
        fs::create_dir_all(&bin_dir)?;
        fs::write(
            bin_dir.join("harness-kit-checks"),
            b"stale-installed-content",
        )?;

        let mut lines = Vec::new();
        install_cli(repo.path(), home.path(), &mut lines)?;

        let installed = fs::read(bin_dir.join("harness-kit-checks"))?;
        assert_eq!(installed, b"fresh-build-content");
        assert!(
            lines
                .iter()
                .any(|line| line.contains("bin/harness-kit-checks"))
        );
        Ok(())
    }

    #[test]
    fn install_cli_skips_rewrite_when_content_already_matches() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let home = tempfile::tempdir()?;

        fs::create_dir_all(repo.path().join("target/debug"))?;
        fs::write(
            repo.path().join("target/debug/harness-kit-checks"),
            b"identical-content",
        )?;

        let bin_dir = home.path().join(".harness-kit/bin");
        fs::create_dir_all(&bin_dir)?;
        let destination = bin_dir.join("harness-kit-checks");
        fs::write(&destination, b"identical-content")?;
        let before = fs::metadata(&destination)?.modified()?;

        let mut lines = Vec::new();
        install_cli(repo.path(), home.path(), &mut lines)?;

        assert_eq!(fs::read(&destination)?, b"identical-content");
        assert!(lines.iter().any(|line| line.contains("already current")));
        // No rewrite happened at all (not just same content) — mtime is untouched.
        assert_eq!(fs::metadata(&destination)?.modified()?, before);
        Ok(())
    }

    #[test]
    fn install_cli_prefers_release_build_over_debug_when_newer() -> Result<()> {
        let repo = tempfile::tempdir()?;
        let home = tempfile::tempdir()?;

        fs::create_dir_all(repo.path().join("target/debug"))?;
        fs::write(
            repo.path().join("target/debug/harness-kit-checks"),
            b"stale-debug-build",
        )?;
        // Ensure the release build's mtime is observably newer than debug's.
        std::thread::sleep(std::time::Duration::from_millis(10));
        fs::create_dir_all(repo.path().join("target/release"))?;
        fs::write(
            repo.path().join("target/release/harness-kit-checks"),
            b"fresh-release-build",
        )?;

        let bin_dir = home.path().join(".harness-kit/bin");
        fs::create_dir_all(&bin_dir)?;
        fs::write(bin_dir.join("harness-kit-checks"), b"stale-installed")?;

        let mut lines = Vec::new();
        install_cli(repo.path(), home.path(), &mut lines)?;

        assert_eq!(
            fs::read(bin_dir.join("harness-kit-checks"))?,
            b"fresh-release-build"
        );
        Ok(())
    }
}
