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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BootstrapOptions {
    pub repo: PathBuf,
    pub home: PathBuf,
}

impl BootstrapOptions {
    pub fn from_env(repo: Option<PathBuf>) -> Result<Self> {
        let repo = repo
            .or_else(|| env::var_os("HARNESS_KIT_DIR").map(PathBuf::from))
            .unwrap_or(env::current_dir()?);
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .context("HOME must be set")?;
        Ok(Self { repo, home })
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

    let skills = discover_skills(&repo)?;
    let agents = discover_agents(&repo)?;
    if skills.is_empty() {
        bail!("No skills found");
    }
    if agents.is_empty() {
        bail!("No agents found");
    }

    let mut lines = vec![
        blue("Harness Kit Bootstrap"),
        blue(format!("Local checkout detected: {}", repo.display())),
        blue("Mode: symlink"),
        String::new(),
    ];
    install_system_roster(&repo, &options.home, &mut lines)?;
    install_cli(&options.home, &mut lines)?;

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
        link_harness(&repo, harness, &harness_dir, &skills, &agents, &mut lines)?;
        installed += 1;
        lines.push(String::new());
    }

    if installed == 0 {
        lines.push(yellow("No agent harnesses detected."));
        lines.push(yellow("Installing to ~/.claude/ as default."));
        let claude = options.home.join(".claude");
        fs::create_dir_all(&claude)?;
        link_harness(&repo, "claude", &claude, &skills, &agents, &mut lines)?;
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
    lines.push(blue(format!(
        "Agents ({}): {}",
        agents.len(),
        agents.join(" ")
    )));
    lines.push(blue(
        "All first-party skills are installed system-wide for each detected harness.",
    ));
    lines.push(String::new());
    lines.push(blue(format!(
        "Mode: symlink (edits in {} propagate instantly)",
        repo.display()
    )));
    Ok(lines.join("\n"))
}

fn ensure_checkout(repo: &Path) -> Result<()> {
    if repo.join("skills").is_dir()
        && repo.join("agents").is_dir()
        && repo.join("harnesses").is_dir()
    {
        Ok(())
    } else {
        bail!("{} is not a Harness Kit checkout", repo.display())
    }
}

fn discover_skills(repo: &Path) -> Result<Vec<String>> {
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
    link_or_replace(
        &repo.join(".harness-kit/examples"),
        &home.join(".harness-kit/examples"),
    )?;
    lines.push(green("    examples/"));
    lines.push(String::new());
    Ok(())
}

#[cfg(unix)]
fn install_cli(home: &Path, lines: &mut Vec<String>) -> Result<()> {
    lines.push(blue("Installing Rust CLI..."));
    let bin_dir = home.join(".harness-kit/bin");
    fs::create_dir_all(&bin_dir)?;
    let destination = bin_dir.join("harness-kit-checks");
    let current = env::current_exe()?;
    // When the running binary IS the installed one (invoked via PATH or a
    // symlink into bin_dir), fs::copy(src, src) truncates it to zero bytes.
    // Skip the self-copy; there is nothing newer to install.
    let same_file = match (current.canonicalize(), destination.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    };
    if same_file {
        lines.push(green("    bin/harness-kit-checks (already current)"));
        lines.push(String::new());
        return Ok(());
    }
    // Copy via temp + rename so a concurrent invocation never sees a
    // half-written binary.
    let staging = bin_dir.join(".harness-kit-checks.tmp");
    fs::copy(&current, &staging)
        .with_context(|| format!("failed to stage {}", staging.display()))?;
    fs::rename(&staging, &destination)
        .with_context(|| format!("failed to install {}", destination.display()))?;
    lines.push(green("    bin/harness-kit-checks"));
    lines.push(String::new());
    Ok(())
}

#[cfg(unix)]
fn link_harness(
    repo: &Path,
    harness: &str,
    harness_dir: &Path,
    skills: &[String],
    agents: &[String],
    lines: &mut Vec<String>,
) -> Result<()> {
    let skills_dir = harness_dir.join("skills");
    fs::create_dir_all(&skills_dir)?;
    lines.push(blue("  Linking skills..."));
    cleanup_symlinks_under_prefix(&skills_dir, &repo.join("skills"), skills, lines)?;
    for skill in skills {
        link_or_replace(&repo.join("skills").join(skill), &skills_dir.join(skill))?;
        lines.push(green(format!("    {skill}")));
    }

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
            link_dir_entries_if_present(
                &repo.join("harnesses/claude/hooks"),
                &harness_dir.join("hooks"),
                "hooks/",
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
        let target = fs::read_link(&path).unwrap_or_default();
        if target.starts_with(prefix) {
            let base = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            if !expected.contains(base) {
                fs::remove_file(&path)?;
                lines.push(green(format!("    removed stale {base}")));
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn link_dir_entries_if_present(
    src_dir: &Path,
    dest_dir: &Path,
    label: &str,
    lines: &mut Vec<String>,
) -> Result<()> {
    if !src_dir.is_dir() {
        return Ok(());
    }
    fs::create_dir_all(dest_dir)?;
    for entry in fs::read_dir(src_dir)? {
        let src = entry?.path();
        link_or_replace(&src, &dest_dir.join(src.file_name().unwrap_or_default()))?;
    }
    lines.push(green(format!("    {label}")));
    Ok(())
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

fn blue(message: impl AsRef<str>) -> String {
    format!("\x1b[0;34m{}\x1b[0m", message.as_ref())
}

fn green(message: impl AsRef<str>) -> String {
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
}
