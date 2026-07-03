use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

#[cfg(unix)]
use std::os::unix::fs::symlink;

const ALLOWED_GLOBAL_AGENTS: &[&str] = &["a11y-auditor", "a11y-fixer", "a11y-critic"];
const RETIRED_GLOBAL_AGENTS: &[&str] = &[
    "beck",
    "builder",
    "carmack",
    "cooper",
    "critic",
    "grug",
    "ousterhout",
    "planner",
];
const RETIRED_PROMPTS: &[&str] = &[
    "critique.md",
    "orient.md",
    "reflect.md",
    "ship.md",
    "yeet.md",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapOptions {
    pub repo: PathBuf,
    pub home: PathBuf,
    /// Role-scoped bundle name (backlog.d/130). `None` installs the full
    /// catalog — the default, unchanged behavior.
    pub bundle: Option<String>,
    /// Report the projected skill count and description-byte estimate for
    /// the selected scope (bundle or full catalog) without touching the
    /// filesystem.
    pub dry_run: bool,
}

impl BootstrapOptions {
    pub fn from_env(repo: Option<PathBuf>) -> Result<Self> {
        let repo = repo
            .or_else(|| env::var_os("HARNESS_KIT_DIR").map(PathBuf::from))
            .unwrap_or(env::current_dir()?);
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME must be set")?;
        Ok(Self {
            repo,
            home,
            bundle: None,
            dry_run: false,
        })
    }
}

pub fn run(options: &BootstrapOptions) -> Result<String> {
    #[cfg(not(unix))]
    bail!("bootstrap requires Unix symlink support");

    #[cfg(unix)]
    run_unix(options)
}

#[cfg(unix)]
fn run_unix(options: &BootstrapOptions) -> Result<String> {
    let repo = options
        .repo
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {}", options.repo.display()))?;
    ensure_checkout(&repo)?;

    let all_skills = discover_skills(&repo)?;
    let all_externals = discover_external_skills(&repo)?;
    let agents = discover_agents(&repo)?;
    if all_skills.is_empty() {
        bail!("No skills found");
    }

    let (skills, externals) = match &options.bundle {
        Some(name) => crate::bundles::resolve_bundle(&repo, name, &all_skills, &all_externals)?,
        None => (all_skills.clone(), all_externals.clone()),
    };

    if options.dry_run {
        return crate::bundles::dry_run_report(
            &repo,
            options.bundle.as_deref(),
            &all_skills,
            &all_externals,
            &skills,
            &externals,
        );
    }

    let mut lines = vec![
        blue("Harness Kit Bootstrap"),
        blue(format!("Local checkout detected: {}", repo.display())),
        blue("Mode: symlink"),
        String::new(),
    ];
    if is_disposable_worktree_path(&repo) {
        lines.push(yellow(format!(
            "WARNING: bootstrapping from a disposable worktree ({}) — installed \
             links will point here and go dangling once this worktree is \
             removed. Prefer running bootstrap from your canonical checkout.",
            repo.display()
        )));
        lines.push(String::new());
    }
    if let Some(name) = &options.bundle {
        lines.push(blue(format!("Bundle: {name}")));
        lines.push(String::new());
    }
    install_system_roster(&repo, &options.home, &mut lines)?;
    install_factory_mcp_registry(&repo, &options.home, &mut lines)?;
    crate::cli_install::install_cli(&repo, &options.home, &mut lines)?;

    let mut installed = 0usize;
    for harness in [
        "claude",
        "codex",
        "pi",
        "antigravity-cli",
        "antigravity-ide",
        "antigravity",
    ] {
        let harness_dir = harness_dir(&options.home, harness);
        if !harness_dir.is_dir() && !command_exists(harness_command(harness)) {
            continue;
        }
        fs::create_dir_all(&harness_dir)?;
        lines.push(blue(format!("Detected: {harness}")));
        link_harness(
            &repo,
            harness,
            &harness_dir,
            &skills,
            &externals,
            &agents,
            &mut lines,
        )?;
        installed += 1;
        lines.push(String::new());
    }

    if installed == 0 {
        lines.push(yellow("No agent harnesses detected."));
        lines.push(yellow("Installing to ~/.claude/ as default."));
        let claude = options.home.join(".claude");
        fs::create_dir_all(&claude)?;
        link_harness(
            &repo, "claude", &claude, &skills, &externals, &agents, &mut lines,
        )?;
        installed = 1;
    }

    set_hooks_path(&repo, &mut lines)?;
    lines.push(green(format!(
        "Done. Installed to {installed} harness(es)."
    )));
    lines.push(String::new());
    lines.push(blue(format!(
        "Skills ({}): {}",
        skills.len(),
        skills.join(" ")
    )));
    if !agents.is_empty() {
        lines.push(blue(format!(
            "Agents ({}): {}",
            agents.len(),
            agents.join(" ")
        )));
    }
    match &options.bundle {
        Some(name) => lines.push(blue(format!(
            "'{name}' bundle skills are installed system-wide for each detected harness."
        ))),
        None => lines.push(blue(
            "All first-party and declared external skills are installed system-wide for each detected harness.",
        )),
    }
    lines.push(String::new());
    lines.push(blue(format!(
        "Mode: symlink (edits in {} propagate instantly)",
        repo.display()
    )));
    Ok(lines.join("\n"))
}

fn ensure_checkout(repo: &Path) -> Result<()> {
    if repo.join("skills").is_dir() && repo.join("harnesses").is_dir() {
        Ok(())
    } else {
        bail!("{} is not a Harness Kit checkout", repo.display())
    }
}

pub(crate) fn discover_skills(repo: &Path) -> Result<Vec<String>> {
    let mut skills = BTreeSet::new();
    for entry in fs::read_dir(repo.join("skills"))? {
        let path = entry?.path();
        if path.is_dir()
            && path.join("SKILL.md").is_file()
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            skills.insert(name.to_string());
        }
    }
    Ok(skills.into_iter().collect())
}

fn discover_agents(repo: &Path) -> Result<Vec<String>> {
    Ok(ALLOWED_GLOBAL_AGENTS
        .iter()
        .filter(|agent| repo.join("agents").join(format!("{agent}.md")).is_file())
        .map(|agent| (*agent).to_string())
        .collect())
}

pub(crate) fn discover_external_skills(repo: &Path) -> Result<Vec<String>> {
    let external = repo.join("skills/.external");
    let mut skills = BTreeSet::new();
    if external.is_dir() {
        for entry in fs::read_dir(&external)? {
            let path = entry?.path();
            if path.is_dir()
                && path.join("SKILL.md").is_file()
                && let Some(name) = path.file_name().and_then(|name| name.to_str())
            {
                skills.insert(name.to_string());
            }
        }
    }
    Ok(skills.into_iter().collect())
}

#[cfg(unix)]
fn install_system_roster(repo: &Path, home: &Path, lines: &mut Vec<String>) -> Result<()> {
    lines.push(blue("Installing system roster..."));
    link_or_replace(
        &repo.join(".harness-kit/agents.yaml"),
        &home.join(".harness-kit/agents.yaml"),
    )?;
    lines.push(green("    agents.yaml"));
    link_or_replace(
        &repo.join(".harness-kit/agents.yaml"),
        &home.join(".spellbook/agents.yaml"),
    )?;
    lines.push(green("    legacy agents.yaml"));
    cleanup_retired_system_examples(repo, home, lines)?;
    lines.push(String::new());
    Ok(())
}

#[cfg(unix)]
fn cleanup_retired_system_examples(
    repo: &Path,
    home: &Path,
    lines: &mut Vec<String>,
) -> Result<()> {
    let path = home.join(".harness-kit/examples");
    if !is_symlink(&path) {
        return Ok(());
    }
    let target = fs::read_link(&path).unwrap_or_default();
    let retired_source = repo.join(".harness-kit/examples");
    let owned = target.starts_with(&retired_source)
        || !path.exists()
        || points_into_harness_kit_checkout(&target);
    if owned {
        fs::remove_file(&path)?;
        lines.push(green("    removed retired examples/"));
    }
    Ok(())
}

#[cfg(unix)]
fn install_factory_mcp_registry(repo: &Path, home: &Path, lines: &mut Vec<String>) -> Result<()> {
    let src = repo.join(".harness-kit/factory-mcps.yaml");
    if !src.is_file() {
        return Ok(());
    }
    lines.push(blue("Installing factory MCP registry..."));
    link_or_replace(&src, &home.join(".harness-kit/factory-mcps.yaml"))?;
    lines.push(green("    factory-mcps.yaml"));
    link_or_replace(&src, &home.join(".spellbook/factory-mcps.yaml"))?;
    lines.push(green("    legacy factory-mcps.yaml"));
    lines.push(String::new());
    Ok(())
}

#[cfg(unix)]
fn link_harness(
    repo: &Path,
    harness: &str,
    harness_dir: &Path,
    skills: &[String],
    externals: &[String],
    agents: &[String],
    lines: &mut Vec<String>,
) -> Result<()> {
    let skills_dir = harness_dir.join("skills");
    fs::create_dir_all(&skills_dir)?;
    lines.push(blue("  Linking skills..."));
    let mut all_expected = skills.to_vec();
    all_expected.extend(externals.iter().cloned());
    cleanup_symlinks_under_prefix(&skills_dir, &repo.join("skills"), &all_expected, lines)?;
    for skill in skills {
        link_or_replace(&repo.join("skills").join(skill), &skills_dir.join(skill))?;
        lines.push(green(format!("    {skill}")));
    }
    for alias in externals {
        link_or_replace(
            &repo.join("skills/.external").join(alias),
            &skills_dir.join(alias),
        )?;
    }
    if !externals.is_empty() {
        lines.push(green(format!(
            "    + {} vendored externals",
            externals.len()
        )));
    }

    cleanup_retired_prompt_links(repo, harness, harness_dir, lines)?;

    let agents_dir = harness_dir.join("agents");
    lines.push(blue("  Linking agents..."));
    if prepare_agents_dir(&agents_dir, &repo.join("agents"), lines)? {
        cleanup_retired_agents(&agents_dir, repo, lines)?;
        let expected = agents
            .iter()
            .map(|agent| format!("{agent}.md"))
            .collect::<Vec<_>>();
        cleanup_symlinks_under_prefix(&agents_dir, &repo.join("agents"), &expected, lines)?;
        for agent in agents {
            link_or_replace(
                &repo.join("agents").join(format!("{agent}.md")),
                &agents_dir.join(format!("{agent}.md")),
            )?;
            lines.push(green(format!("    {agent}")));
        }
    }

    lines.push(blue("  Linking harness config..."));
    match harness {
        "claude" => {
            link_if_present(
                &repo.join("harnesses/shared/AGENTS.md"),
                &harness_dir.join("CLAUDE.md"),
                "CLAUDE.md (← shared AGENTS.md)",
                lines,
            )?;
            // `harnesses/claude/hooks/` no longer ships .py hooks (all moved
            // into the `harness-kit-checks claude-hook` binary), so there is
            // nothing left to link here. Self-heal any dangling symlinks a
            // pre-rewrite bootstrap left in ~/.claude/hooks/ instead
            // (backlog.d/114) rather than re-linking a source that is gone.
            cleanup_symlinks_under_prefix(
                &harness_dir.join("hooks"),
                &repo.join("harnesses/claude/hooks"),
                &[],
                lines,
            )?;
            copy_if_present(
                &repo.join("harnesses/claude/settings.json"),
                &harness_dir.join("settings.json"),
                "settings.json (copied)",
                lines,
            )?;
        }
        "codex" => {
            link_if_present(
                &repo.join("harnesses/codex/config.toml"),
                &harness_dir.join("config/config.toml"),
                "config.toml",
                lines,
            )?;
            link_if_present(
                &repo.join("harnesses/shared/AGENTS.md"),
                &harness_dir.join("AGENTS.md"),
                "AGENTS.md (← shared)",
                lines,
            )?;
        }
        "pi" => {
            link_if_present(
                &repo.join("harnesses/shared/AGENTS.md"),
                &harness_dir.join("agent/AGENTS.md"),
                "AGENTS.md (← shared)",
                lines,
            )?;
            link_if_present(
                &repo.join("harnesses/pi/settings.json"),
                &harness_dir.join("settings.json"),
                "settings.json",
                lines,
            )?;
        }
        "antigravity-cli" | "antigravity-ide" | "antigravity" => {
            link_if_present(
                &repo.join("harnesses/shared/AGENTS.md"),
                &harness_dir.join("AGENTS.md"),
                "AGENTS.md (← shared)",
                lines,
            )?;
        }
        _ => {}
    }
    Ok(())
}

#[cfg(unix)]
fn cleanup_retired_prompt_links(
    repo: &Path,
    harness: &str,
    harness_dir: &Path,
    lines: &mut Vec<String>,
) -> Result<()> {
    let Some(prompts_dir) = (match harness {
        "claude" => Some(harness_dir.join("commands")),
        "codex" | "pi" => Some(harness_dir.join("prompts")),
        _ => None,
    }) else {
        return Ok(());
    };
    if !prompts_dir.is_dir() {
        return Ok(());
    }

    let retired_source = repo.join("prompts");
    let mut printed_header = false;
    for prompt in RETIRED_PROMPTS {
        let path = prompts_dir.join(prompt);
        if !is_symlink(&path) {
            continue;
        }
        let target = fs::read_link(&path).unwrap_or_default();
        let owned = target.starts_with(&retired_source)
            || !path.exists()
            || points_into_harness_kit_checkout(&target);
        if owned {
            if !printed_header {
                lines.push(blue("  Pruning retired prompt links..."));
                printed_header = true;
            }
            fs::remove_file(&path)?;
            lines.push(green(format!("    removed retired {prompt}")));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn prepare_agents_dir(dir: &Path, source_agents: &Path, lines: &mut Vec<String>) -> Result<bool> {
    if is_symlink(dir) {
        let target = fs::read_link(dir).unwrap_or_default();
        if target == source_agents || is_harness_kit_agents_target(&target) {
            fs::remove_file(dir)?;
            lines.push(green("    removed stale agents/ parent symlink"));
        } else {
            lines.push(yellow(
                "    agents/ is a user-owned symlink; leaving agents unchanged",
            ));
            return Ok(false);
        }
    } else if dir.exists() && !dir.is_dir() {
        lines.push(yellow(
            "    agents/ is not a directory; leaving agents unchanged",
        ));
        return Ok(false);
    }
    fs::create_dir_all(dir)?;
    Ok(true)
}

#[cfg(unix)]
fn cleanup_retired_agents(dir: &Path, repo: &Path, lines: &mut Vec<String>) -> Result<()> {
    for agent in RETIRED_GLOBAL_AGENTS {
        let dest = dir.join(format!("{agent}.md"));
        if !dest.exists() && !is_symlink(&dest) {
            continue;
        }
        if is_symlink(&dest)
            && is_harness_kit_agents_target(&fs::read_link(&dest).unwrap_or_default())
        {
            fs::remove_file(&dest)?;
            lines.push(green(format!("    removed retired agent {agent}")));
        } else if dest.is_file() && repo.join("agents").join(format!("{agent}.md")).is_file() {
            let src = repo.join("agents").join(format!("{agent}.md"));
            if fs::read(&src)? == fs::read(&dest)? {
                fs::remove_file(&dest)?;
                lines.push(green(format!("    removed retired copied agent {agent}")));
            } else {
                lines.push(yellow(format!("    preserving user-owned agent {agent}")));
            }
        } else {
            lines.push(yellow(format!("    preserving user-owned agent {agent}")));
        }
    }
    Ok(())
}

#[cfg(unix)]
fn cleanup_symlinks_under_prefix(
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
            lines.push(green(format!("    removed stale {base}")));
        }
    }
    Ok(())
}

/// Detects a checkout living under a disposable agent worktree
/// (`.../.codex/worktrees/...`) rather than a durable local clone —
/// backlog.d/114, found live 2026-06-17 when an old bootstrap symlinked
/// installed skills/hooks into a worktree that was later deleted, leaving 23
/// dangling links behind. Portable pattern match (adjacent path components),
/// not a hardcoded machine-specific path.
fn is_disposable_worktree_path(path: &Path) -> bool {
    let components: Vec<_> = path.components().collect();
    components
        .windows(2)
        .any(|pair| pair[0].as_os_str() == ".codex" && pair[1].as_os_str() == "worktrees")
}

#[cfg(unix)]
fn points_into_harness_kit_checkout(target: &Path) -> bool {
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
fn link_if_present(src: &Path, dest: &Path, label: &str, lines: &mut Vec<String>) -> Result<()> {
    if src.exists() {
        link_or_replace(src, dest)?;
        lines.push(green(format!("    {label}")));
    }
    Ok(())
}

fn copy_if_present(src: &Path, dest: &Path, label: &str, lines: &mut Vec<String>) -> Result<()> {
    if src.exists() {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dest)?;
        lines.push(green(format!("    {label}")));
    }
    Ok(())
}

#[cfg(unix)]
fn link_or_replace(src: &Path, dest: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    remove_any(dest)?;
    symlink(src, dest)
        .with_context(|| format!("failed to symlink {} -> {}", dest.display(), src.display()))
}

fn remove_any(path: &Path) -> Result<()> {
    if is_symlink(path) || path.is_file() {
        fs::remove_file(path)?;
    } else if path.is_dir() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn is_symlink(path: &Path) -> bool {
    fs::symlink_metadata(path)
        .map(|metadata| metadata.file_type().is_symlink())
        .unwrap_or(false)
}

fn is_harness_kit_agents_target(path: &Path) -> bool {
    let text = path.to_string_lossy();
    text.ends_with("/harness-kit/agents") || text.contains("/harness-kit/agents/")
}

fn harness_dir(home: &Path, harness: &str) -> PathBuf {
    match harness {
        "antigravity-cli" => home.join(".gemini/antigravity-cli"),
        "antigravity-ide" => home.join(".gemini/antigravity-ide"),
        "antigravity" => home.join(".gemini/antigravity"),
        other => home.join(format!(".{other}")),
    }
}

fn harness_command(harness: &str) -> &str {
    match harness {
        "antigravity-cli" => "agy",
        other => other,
    }
}

fn command_exists(command: &str) -> bool {
    env::var_os("PATH")
        .map(|path| env::split_paths(&path).any(|dir| dir.join(command).is_file()))
        .unwrap_or(false)
}

fn set_hooks_path(repo: &Path, lines: &mut Vec<String>) -> Result<()> {
    if !repo.join(".githooks").is_dir() || !command_exists("git") {
        return Ok(());
    }
    let current = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["config", "core.hooksPath"])
        .output()
        .context("failed to inspect core.hooksPath")?;
    if String::from_utf8_lossy(&current.stdout).trim() != ".githooks" {
        let status = Command::new("git")
            .arg("-C")
            .arg(repo)
            .args(["config", "core.hooksPath", ".githooks"])
            .status()
            .context("failed to set core.hooksPath")?;
        if status.success() {
            lines.push(blue("Set core.hooksPath → .githooks"));
        }
    }
    Ok(())
}

pub(crate) fn blue(message: impl AsRef<str>) -> String {
    format!("\x1b[0;34m{}\x1b[0m", message.as_ref())
}

pub(crate) fn green(message: impl AsRef<str>) -> String {
    format!("\x1b[0;32m{}\x1b[0m", message.as_ref())
}

fn yellow(message: impl AsRef<str>) -> String {
    format!("\x1b[0;33m{}\x1b[0m", message.as_ref())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovers_only_allowlisted_agents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        fs::create_dir_all(temp.path().join("skills/demo"))?;
        fs::write(
            temp.path().join("skills/demo/SKILL.md"),
            "---\nname: demo\n---\n",
        )?;
        fs::create_dir_all(temp.path().join("agents"))?;
        fs::create_dir_all(temp.path().join("harnesses"))?;
        for agent in ["a11y-auditor", "a11y-fixer", "a11y-critic", "critic"] {
            fs::write(temp.path().join("agents").join(format!("{agent}.md")), "")?;
        }
        assert_eq!(
            discover_agents(temp.path())?,
            vec!["a11y-auditor", "a11y-fixer", "a11y-critic"]
        );
        Ok(())
    }

    #[test]
    fn detects_disposable_codex_worktree_paths() {
        assert!(is_disposable_worktree_path(Path::new(
            "/Users/anyone/.codex/worktrees/ed05/harness-kit"
        )));
        assert!(!is_disposable_worktree_path(Path::new(
            "/Users/anyone/Development/harness-kit"
        )));
        // ".codex" and "worktrees" must be adjacent, not merely both present.
        assert!(!is_disposable_worktree_path(Path::new(
            "/Users/anyone/.codex/config/worktrees-backup"
        )));
    }

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
