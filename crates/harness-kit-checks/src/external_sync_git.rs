//! Git plumbing for `external_sync`: clone/sparse-checkout/resolve/checkout
//! against an upstream repo cache. Split out of `external_sync.rs`
//! (backlog.d/122 item 3) purely to keep that file under the god-file
//! ceiling — registry parsing, staging, and cleanup stay there; the raw git
//! subprocess calls live here.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use regex::Regex;

use super::slugify_repo;

pub(super) fn ensure_checkout(checkout_root: &Path, repo: &str) -> Result<PathBuf> {
    let dir = checkout_root.join(slugify_repo(repo));
    if !dir.join(".git").is_dir() {
        std::fs::create_dir_all(checkout_root)?;
        let url = format!("https://github.com/{repo}.git");
        run_checked(
            Command::new("git")
                .args(["clone", "--filter=blob:none", "--sparse", &url])
                .arg(&dir),
            &format!("clone failed: {url} (unreachable or auth required)"),
        )?;
    }
    Ok(dir)
}

pub(super) fn set_sparse(dir: &Path, skills_path: &str) -> Result<()> {
    if skills_path == "." || skills_path.is_empty() {
        // A root-level skill (`stage_root_skill`) needs only SKILL.md + a
        // license file, both top-level. `ensure_checkout`'s clone already
        // sets cone-mode sparse-checkout to top-level-files-only by default
        // (`git clone --sparse`'s own behavior) — disabling it here, as this
        // branch used to, actively undoes that and checks out the entire
        // upstream working tree (verified live: for a Rust app source, this
        // pulled the whole crate tree into the gitignored `_checkouts`
        // cache). `sparse-checkout set` with no path args re-asserts
        // top-level-only explicitly and idempotently, regardless of
        // whatever sparse state a prior sync of this same cached checkout
        // (possibly for a different skills_path) left behind.
        run_checked(
            Command::new("git")
                .args(["-C"])
                .arg(dir)
                .args(["sparse-checkout", "set"]),
            &format!("sparse-checkout failed in {} for root skill", dir.display()),
        )?;
    } else {
        run_checked(
            Command::new("git")
                .args(["-C"])
                .arg(dir)
                .args(["sparse-checkout", "set", skills_path]),
            &format!(
                "sparse-checkout failed in {} for {skills_path}",
                dir.display()
            ),
        )?;
    }
    Ok(())
}

pub(super) fn resolve_ref_to_sha(dir: &Path, ref_name: &str) -> Result<String> {
    if Regex::new(r"^[0-9a-f]{40}$").unwrap().is_match(ref_name) {
        return Ok(ref_name.to_string());
    }
    for pattern in [
        format!("refs/tags/{ref_name}"),
        format!("refs/heads/{ref_name}"),
        ref_name.to_string(),
    ] {
        let output = Command::new("git")
            .args(["-C"])
            .arg(dir)
            .args(["ls-remote", "origin", &pattern])
            .output()
            .with_context(|| format!("failed to resolve ref '{ref_name}' in {}", dir.display()))?;
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Some(sha) = stdout
                .lines()
                .filter_map(|line| line.split_whitespace().next())
                .find(|sha| !sha.is_empty())
            {
                return Ok(sha.to_string());
            }
        }
    }
    bail!("cannot resolve ref '{ref_name}' in {}", dir.display());
}

pub(super) fn checkout_sha(dir: &Path, sha: &str) -> Result<()> {
    let shallow = Command::new("git")
        .args(["-C"])
        .arg(dir)
        .args(["fetch", "--depth=1", "--filter=blob:none", "origin", sha])
        .status();
    let fetched = shallow.is_ok_and(|status| status.success())
        || Command::new("git")
            .args(["-C"])
            .arg(dir)
            .args(["fetch", "--filter=blob:none", "origin"])
            .status()
            .is_ok_and(|status| status.success());
    if !fetched {
        bail!("fetch failed in {}", dir.display());
    }
    let checked_out = Command::new("git")
        .args(["-C"])
        .arg(dir)
        .args(["checkout", "--quiet", sha])
        .status()
        .is_ok_and(|status| status.success())
        || Command::new("git")
            .args(["-C"])
            .arg(dir)
            .args(["checkout", "--quiet", "-B", "harness-kit-sync", sha])
            .status()
            .is_ok_and(|status| status.success());
    if !checked_out {
        bail!("checkout {sha} failed in {}", dir.display());
    }
    Ok(())
}

fn run_checked(command: &mut Command, message: &str) -> Result<()> {
    let status = command.status().with_context(|| message.to_string())?;
    if !status.success() {
        bail!("{message}");
    }
    Ok(())
}
