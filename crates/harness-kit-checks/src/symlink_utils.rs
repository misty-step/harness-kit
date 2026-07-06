//! Symlink management primitives shared across `bootstrap`'s harness-linking
//! functions, split out to keep `bootstrap.rs` under its god-file ceiling
//! (`check-godfiles`) after harness-kit-914's codex_execpolicy wiring pushed
//! it over -- per gates-never-lowered doctrine, the fix is the split the
//! gate names, not a raised ceiling. Owns the mechanics of creating,
//! replacing, and pruning symlinks; `bootstrap.rs` owns the policy of what
//! gets linked where.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

#[cfg(unix)]
use std::os::unix::fs::symlink;

#[cfg(unix)]
pub(crate) fn cleanup_symlinks_under_prefix(
    dir: &Path,
    prefix: &Path,
    expected: &[String],
    lines: &mut Vec<String>,
) -> Result<()> {
    fs::create_dir_all(dir)?;
    let expected = expected.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if !is_symlink(&path) {
            continue;
        }
        let base = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if expected.contains(base) {
            continue;
        }
        let target = fs::read_link(&path).unwrap_or_default();
        // Remove links we own: into this checkout, into any other Harness
        // Kit checkout (old clone or worktree), or dangling. Links into
        // non-Harness-Kit locations are user-owned and preserved.
        let stale = target.starts_with(prefix)
            || !path.exists()
            || points_into_harness_kit_checkout(&target);
        if stale {
            fs::remove_file(&path)?;
            lines.push(crate::bootstrap::green(format!("    removed stale {base}")));
        }
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn points_into_harness_kit_checkout(target: &Path) -> bool {
    // Managed link targets look like <checkout>/{skills,agents}/…; prompt
    // targets are recognized here only so bootstrap can prune retired links.
    // Walk ancestors and look for the checkout markers.
    target.ancestors().skip(1).any(|ancestor| {
        ancestor.join("harnesses").is_dir()
            && ancestor.join("bootstrap.sh").is_file()
            && ancestor.join("skills").is_dir()
    })
}

#[cfg(unix)]
pub(crate) fn link_if_present(
    src: &Path,
    dest: &Path,
    label: &str,
    lines: &mut Vec<String>,
) -> Result<()> {
    if src.exists() {
        link_or_replace(src, dest)?;
        lines.push(crate::bootstrap::green(format!("    {label}")));
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn link_or_replace(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    remove_any(dest)?;
    symlink(src, dest)
        .with_context(|| format!("failed to symlink {} -> {}", dest.display(), src.display()))
}

pub(crate) fn remove_any(path: &Path) -> Result<()> {
    if is_symlink(path) || path.is_file() {
        fs::remove_file(path)?;
    } else if path.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub(crate) fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

pub(crate) fn is_harness_kit_agents_target(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.ends_with("/harness-kit/agents") || text.contains("/harness-kit/agents/")
}

#[cfg(test)]
mod tests {
    use super::*;

    // backlog.d/114: a pre-rewrite bootstrap symlinked ~/.claude/hooks/* into
    // whatever worktree it ran from; when that worktree was deleted, the
    // links went dangling and were never cleaned up because nothing pruned
    // that directory. Self-healing must remove them on the next bootstrap
    // run without touching links that still resolve.
    #[test]
    fn cleanup_symlinks_under_prefix_prunes_dangling_hook_links() -> Result<()> {
        let hooks_dir = tempfile::tempdir()?;
        let deleted_worktree_target = hooks_dir.path().join("gone-forever.py");
        // Never created — simulates a target whose worktree was removed.
        #[cfg(unix)]
        symlink(&deleted_worktree_target, hooks_dir.path().join("stale.py"))?;

        let live_target = hooks_dir.path().join("still-here.py");
        fs::write(&live_target, "# still a real file")?;
        #[cfg(unix)]
        symlink(&live_target, hooks_dir.path().join("live.py"))?;

        let mut lines = Vec::new();
        cleanup_symlinks_under_prefix(
            hooks_dir.path(),
            Path::new("/does/not/matter"),
            &[],
            &mut lines,
        )?;

        assert!(!hooks_dir.path().join("stale.py").exists());
        assert!(
            fs::symlink_metadata(hooks_dir.path().join("stale.py")).is_err(),
            "dangling symlink itself must be removed, not just unresolvable"
        );
        assert!(hooks_dir.path().join("live.py").exists());
        Ok(())
    }
}
