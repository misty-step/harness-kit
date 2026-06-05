use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    Sync,
    Check,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncOptions {
    pub repo_root: PathBuf,
    pub mode: SyncMode,
    pub allow_floating: bool,
    pub only_repo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyncReport {
    pub aliases: Vec<String>,
    pub changed: bool,
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEntry {
    pub repo: String,
    pub ref_name: String,
    pub pin: Option<String>,
    pub skills_path: String,
    pub include: Vec<String>,
    pub exclude: Vec<String>,
    pub alias_prefix: Option<String>,
    pub allow_floating: bool,
}

#[derive(Debug, Deserialize)]
struct Registry {
    sources: Option<Vec<RawSource>>,
}

#[derive(Debug, Deserialize)]
struct RawSource {
    repo: Option<String>,
    #[serde(rename = "ref")]
    ref_name: Option<String>,
    rev: Option<String>,
    pin: Option<String>,
    skills_path: Option<String>,
    include: Option<StringOrList>,
    exclude: Option<StringOrList>,
    alias_prefix: Option<String>,
    allow_floating: Option<bool>,
    default: Option<bool>,
    active: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
enum StringOrList {
    String(String),
    List(Vec<String>),
}

impl StringOrList {
    fn into_vec(self) -> Vec<String> {
        match self {
            Self::String(value) => vec![value],
            Self::List(values) => values,
        }
    }
}

pub fn run(options: &SyncOptions) -> Result<SyncReport> {
    ensure_command("git")?;
    let registry_path = options.repo_root.join("registry.yaml");
    let registry = parse_registry_file(&registry_path)?;
    let external_root = options.repo_root.join("skills/.external");
    let checkout_root = external_root.join("_checkouts");
    let mut state = SyncState::default();
    state.lines.push(format!(
        "sync-external [{}] -- reading {}",
        mode_name(options.mode),
        registry_path.display()
    ));

    fs::create_dir_all(&external_root)?;
    if registry.is_empty() {
        state
            .lines
            .push("no external sources declared in registry.yaml".to_string());
    }

    for source in &registry {
        if options
            .only_repo
            .as_ref()
            .is_some_and(|only| only != &source.repo)
        {
            continue;
        }
        sync_source(source, &external_root, &checkout_root, options, &mut state)?;
    }

    if options.only_repo.is_some() {
        state
            .lines
            .push("partial sync: skipping global orphan cleanup".to_string());
    } else {
        cleanup_orphans(&external_root, options.mode, &mut state)?;
    }
    cleanup_unused_checkouts(&checkout_root, &registry, options.mode, &mut state)?;

    if options.mode == SyncMode::Check {
        if state.changed {
            bail!(
                "registry drift: sync would change the tree. Run harness-kit-checks sync-external."
            );
        }
        state
            .lines
            .push("sync-external: clean (no changes needed)".to_string());
    } else {
        state.lines.push(format!(
            "sync-external: done ({} aliases installed)",
            state.aliases.len()
        ));
    }

    Ok(SyncReport {
        aliases: state.aliases,
        changed: state.changed,
        lines: state.lines,
    })
}

pub fn parse_registry_file(path: &Path) -> Result<Vec<SourceEntry>> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("registry not found or unreadable: {}", path.display()))?;
    parse_registry(&text)
}

pub fn parse_registry(text: &str) -> Result<Vec<SourceEntry>> {
    let registry: Registry =
        serde_yaml::from_str(text).context("malformed registry.yaml or unsupported schema")?;
    let mut entries = Vec::new();
    for (index, raw) in registry.sources.unwrap_or_default().into_iter().enumerate() {
        if raw.default.unwrap_or(false) || raw.active == Some(false) {
            continue;
        }
        let repo = raw
            .repo
            .filter(|repo| !repo.trim().is_empty())
            .ok_or_else(|| anyhow::anyhow!("registry.yaml sources[{index}] missing 'repo'"))?;
        entries.push(SourceEntry {
            repo,
            ref_name: raw
                .ref_name
                .or(raw.rev)
                .unwrap_or_else(|| "main".to_string()),
            pin: raw.pin.filter(|pin| !pin.trim().is_empty()),
            skills_path: raw.skills_path.unwrap_or_else(|| ".".to_string()),
            include: raw.include.map(StringOrList::into_vec).unwrap_or_default(),
            exclude: raw.exclude.map(StringOrList::into_vec).unwrap_or_default(),
            alias_prefix: raw.alias_prefix.filter(|prefix| !prefix.is_empty()),
            allow_floating: raw.allow_floating.unwrap_or(false),
        });
    }
    Ok(entries)
}

pub fn is_immutable_ref(ref_name: &str) -> bool {
    let sha = Regex::new(r"^[0-9a-f]{40}$").unwrap();
    let tag = Regex::new(r"^v?[0-9]+\.[0-9]+(\.[0-9]+)?(-[A-Za-z0-9.-]+)?$").unwrap();
    if sha.is_match(ref_name) || tag.is_match(ref_name) {
        return true;
    }
    !matches!(
        ref_name,
        "main" | "master" | "HEAD" | "develop" | "dev" | "trunk"
    )
}

pub fn slugify_repo(repo: &str) -> String {
    let mut output = String::new();
    let mut previous_underscore = false;
    for character in repo.replace('/', "_").chars() {
        let next = if character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | '-') {
            character
        } else {
            '_'
        };
        if next == '_' {
            if !previous_underscore {
                output.push('_');
            }
            previous_underscore = true;
        } else {
            output.push(next);
            previous_underscore = false;
        }
    }
    output.replace('_', "__")
}

pub fn self_test_partial_sync() -> Result<String> {
    let temp = tempfile::tempdir()?;
    let repo = temp.path().join("repo");
    let bin = temp.path().join("bin");
    fs::create_dir_all(repo.join("skills/.external/keep-me"))?;
    fs::create_dir_all(&bin)?;
    fs::write(
        repo.join("skills/.external/keep-me/SKILL.md"),
        r#"---
name: keep-me
description: Existing external skill that partial sync must preserve.
---

# Keep Me
"#,
    )?;
    fs::write(
        repo.join("registry.yaml"),
        r#"
sources:
  - repo: example/keep
    ref: main
    pin: a111111111111111111111111111111111111111
    skills_path: skills
    alias_prefix: keep-
    include: [keep-me]
  - repo: example/new
    ref: main
    pin: b222222222222222222222222222222222222222
    skills_path: skills
    alias_prefix: new-
    include: [new-skill]
"#,
    )?;
    let git = bin.join("git");
    fs::write(
        &git,
        r#"#!/usr/bin/env bash
set -euo pipefail

cmd="${1:-}"
case "$cmd" in
  --version)
    echo "git version 2.0.0"
    ;;
  clone)
    dest="${@: -1}"
    mkdir -p "$dest/.git" "$dest/skills/new-skill"
    cat > "$dest/skills/new-skill/SKILL.md" <<'DOC'
---
name: new-skill
description: New external skill fixture.
---

# New Skill
DOC
    ;;
  -C)
    dir="$2"
    sub="$3"
    case "$sub" in
      sparse-checkout|fetch|checkout)
        exit 0
        ;;
      ls-remote)
        ref="${@: -1}"
        case "$ref" in
          *b222222222222222222222222222222222222222*)
            printf '%s\t%s\n' "b222222222222222222222222222222222222222" "$ref"
            ;;
          *a111111111111111111111111111111111111111*)
            printf '%s\t%s\n' "a111111111111111111111111111111111111111" "$ref"
            ;;
        esac
        ;;
      *)
        echo "unexpected git -C $dir $sub" >&2
        exit 2
        ;;
    esac
    ;;
  *)
    echo "unexpected git $cmd" >&2
    exit 2
    ;;
esac
"#,
    )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&git, fs::Permissions::from_mode(0o755))?;
    }

    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let new_path = format!("{}:{}", bin.display(), old_path.to_string_lossy());
    unsafe {
        std::env::set_var("PATH", &new_path);
    }
    let result = run(&SyncOptions {
        repo_root: repo.clone(),
        mode: SyncMode::Sync,
        allow_floating: false,
        only_repo: Some("example/new".to_string()),
    });
    unsafe {
        std::env::set_var("PATH", old_path);
    }
    let report = result?;
    if !repo.join("skills/.external/keep-me/SKILL.md").is_file() {
        bail!("partial sync removed unrelated external skill");
    }
    if !repo
        .join("skills/.external/new-new-skill/SKILL.md")
        .is_file()
    {
        bail!("partial sync did not install requested external skill");
    }
    if !report
        .lines
        .iter()
        .any(|line| line == "partial sync: skipping global orphan cleanup")
    {
        bail!("partial sync should report skipped global orphan cleanup");
    }
    Ok("sync-external partial sync preserves unrelated externals".to_string())
}

pub fn discover_skills(root: &Path) -> Result<Vec<String>> {
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.metadata()?.is_dir()
            && entry.path().join("SKILL.md").is_file()
            && let Some(name) = entry.file_name().to_str()
        {
            skills.push(name.to_string());
        }
    }
    skills.sort();
    Ok(skills)
}

#[derive(Default)]
struct SyncState {
    aliases: Vec<String>,
    changed: bool,
    alias_to_source: HashMap<String, String>,
    lines: Vec<String>,
}

fn sync_source(
    source: &SourceEntry,
    external_root: &Path,
    checkout_root: &Path,
    options: &SyncOptions,
    state: &mut SyncState,
) -> Result<()> {
    state.lines.push(format!(
        "-> {} (ref={} pin={} path={})",
        source.repo,
        source.ref_name,
        source.pin.as_deref().unwrap_or("-"),
        source.skills_path
    ));

    if source.pin.is_none()
        && !is_immutable_ref(&source.ref_name)
        && !options.allow_floating
        && !source.allow_floating
    {
        bail!(
            "refusing floating ref '{}' for {} -- pin a sha/tag, set allow_floating: true, or pass --allow-floating",
            source.ref_name,
            source.repo
        );
    }

    let checkout_dir = ensure_checkout(checkout_root, &source.repo)?;
    set_sparse(&checkout_dir, &source.skills_path)?;
    let want_ref = source.pin.as_deref().unwrap_or(&source.ref_name);
    let sha = resolve_ref_to_sha(&checkout_dir, want_ref)?;
    checkout_sha(&checkout_dir, &sha)?;
    let skill_root = if source.skills_path == "." || source.skills_path.is_empty() {
        checkout_dir
    } else {
        checkout_dir.join(&source.skills_path)
    };
    let discovered = discover_skills(&skill_root)?;
    if discovered.is_empty() {
        bail!(
            "no skills found under {}/{} -- upstream layout change? Update skills_path.",
            source.repo,
            source.skills_path
        );
    }
    for skill in discovered {
        if !source.include.is_empty() && !source.include.iter().any(|item| item == &skill) {
            continue;
        }
        if source.exclude.iter().any(|item| item == &skill) {
            continue;
        }
        let alias = format!("{}{}", source.alias_prefix.as_deref().unwrap_or(""), skill);
        install_alias(
            &alias,
            &skill_root.join(&skill),
            &source.repo,
            &sha,
            external_root,
            options.mode,
            state,
        )?;
    }
    Ok(())
}

fn ensure_checkout(checkout_root: &Path, repo: &str) -> Result<PathBuf> {
    let dir = checkout_root.join(slugify_repo(repo));
    if !dir.join(".git").is_dir() {
        fs::create_dir_all(checkout_root)?;
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

fn set_sparse(dir: &Path, skills_path: &str) -> Result<()> {
    if skills_path == "." || skills_path.is_empty() {
        let _ = Command::new("git")
            .args(["-C"])
            .arg(dir)
            .args(["sparse-checkout", "disable"])
            .status();
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

fn resolve_ref_to_sha(dir: &Path, ref_name: &str) -> Result<String> {
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

fn checkout_sha(dir: &Path, sha: &str) -> Result<()> {
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

fn install_alias(
    alias: &str,
    src_path: &Path,
    repo: &str,
    sha: &str,
    external_root: &Path,
    mode: SyncMode,
    state: &mut SyncState,
) -> Result<()> {
    if let Some(existing) = state.alias_to_source.get(alias) {
        bail!(
            "alias collision: '{alias}' declared by both '{existing}' and '{repo}' -- set alias_prefix on the later source"
        );
    }
    state
        .alias_to_source
        .insert(alias.to_string(), repo.to_string());
    state.aliases.push(alias.to_string());

    let dest = external_root.join(alias);
    if dest.is_dir() && current_meta_sha(&dest)? == Some(sha.to_string()) {
        return Ok(());
    }
    state.changed = true;
    if mode == SyncMode::Check {
        state.lines.push(format!(
            "  would install/update: {alias} ({repo} @ {})",
            short_sha(sha)
        ));
        return Ok(());
    }
    if !src_path.is_dir() {
        bail!("source path missing: {}", src_path.display());
    }
    if dest.exists() {
        fs::remove_dir_all(&dest)?;
    }
    fs::create_dir_all(&dest)?;
    copy_dir(src_path, &dest)?;
    fs::write(
        dest.join(".sync-meta.json"),
        serde_json::to_string_pretty(&json!({
            "repo": repo,
            "sha": sha,
            "src_path_suffix": src_path.file_name().and_then(|name| name.to_str()).unwrap_or_default(),
            "fetched_at": DateTime::<Utc>::from(std::time::SystemTime::now()).format("%Y-%m-%dT%H:%M:%SZ").to_string(),
        }))? + "\n",
    )?;
    state.lines.push(format!(
        "  installed {alias} <- {repo}/{} @ {}",
        src_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
        short_sha(sha)
    ));
    Ok(())
}

fn current_meta_sha(dest: &Path) -> Result<Option<String>> {
    let path = dest.join(".sync-meta.json");
    if !path.is_file() {
        return Ok(None);
    }
    let value: Value = match serde_json::from_str(&fs::read_to_string(path)?) {
        Ok(value) => value,
        Err(_) => return Ok(None),
    };
    Ok(value
        .get("sha")
        .and_then(Value::as_str)
        .map(ToString::to_string))
}

fn cleanup_orphans(external_root: &Path, mode: SyncMode, state: &mut SyncState) -> Result<()> {
    if !external_root.is_dir() {
        return Ok(());
    }
    let declared = state.aliases.iter().cloned().collect::<BTreeSet<_>>();
    for entry in sorted_dirs(external_root)? {
        let base = entry
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        if base == "_checkouts" || declared.contains(&base) {
            continue;
        }
        state.changed = true;
        if mode == SyncMode::Check {
            state.lines.push(format!("  would remove orphan: {base}"));
        } else {
            fs::remove_dir_all(&entry)?;
            state.lines.push(format!("  removed orphan: {base}"));
        }
    }
    Ok(())
}

fn cleanup_unused_checkouts(
    checkout_root: &Path,
    sources: &[SourceEntry],
    mode: SyncMode,
    state: &mut SyncState,
) -> Result<()> {
    if !checkout_root.is_dir() {
        return Ok(());
    }
    let declared = sources
        .iter()
        .map(|source| slugify_repo(&source.repo))
        .collect::<BTreeSet<_>>();
    for entry in sorted_dirs(checkout_root)? {
        let base = entry
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        if declared.contains(&base) {
            continue;
        }
        state.changed = true;
        if mode == SyncMode::Check {
            state
                .lines
                .push(format!("  would remove unused checkout: {base}"));
        } else {
            fs::remove_dir_all(&entry)?;
            state
                .lines
                .push(format!("  removed unused checkout: {base}"));
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dest: &Path) -> Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name();
        if name == ".git" {
            continue;
        }
        let target = dest.join(name);
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            fs::create_dir_all(&target)?;
            copy_dir(&path, &target)?;
        } else if metadata.is_file() {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

fn sorted_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn run_checked(command: &mut Command, message: &str) -> Result<()> {
    let status = command.status().with_context(|| message.to_string())?;
    if !status.success() {
        bail!("{message}");
    }
    Ok(())
}

fn ensure_command(name: &str) -> Result<()> {
    let output = Command::new(name).arg("--version").output();
    if !output.is_ok_and(|output| output.status.success()) {
        bail!("{name} required");
    }
    Ok(())
}

fn short_sha(sha: &str) -> &str {
    sha.get(..7).unwrap_or(sha)
}

fn mode_name(mode: SyncMode) -> &'static str {
    match mode {
        SyncMode::Sync => "sync",
        SyncMode::Check => "check",
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn parses_registry_like_shell_and_skips_default_inactive_sources() {
        let entries = parse_registry(
            r#"
sources:
  - repo: local/repo
    default: true
  - repo: inactive/repo
    active: false
  - repo: upstream/skills
    ref: main
    skills_path: skills
    include: [one, two]
    exclude: skip
    alias_prefix: "up-"
    allow_floating: true
"#,
        )
        .unwrap();

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].repo, "upstream/skills");
        assert_eq!(entries[0].ref_name, "main");
        assert_eq!(entries[0].skills_path, "skills");
        assert_eq!(entries[0].include, vec!["one", "two"]);
        assert_eq!(entries[0].exclude, vec!["skip"]);
        assert_eq!(entries[0].alias_prefix.as_deref(), Some("up-"));
        assert!(entries[0].allow_floating);
    }

    #[test]
    fn immutable_ref_contract_matches_shell_rules() {
        assert!(is_immutable_ref("0123456789abcdef0123456789abcdef01234567"));
        assert!(is_immutable_ref("v1.2.3"));
        assert!(is_immutable_ref("1.2"));
        assert!(is_immutable_ref("release-candidate"));
        assert!(!is_immutable_ref("main"));
        assert!(!is_immutable_ref("HEAD"));
        assert!(!is_immutable_ref("trunk"));
    }

    #[test]
    fn discovers_skills_and_copies_hidden_files_without_git_metadata() {
        let temp = TempDir::new().unwrap();
        let source = temp.path().join("checkout/skills/demo");
        fs::create_dir_all(source.join(".git")).unwrap();
        fs::create_dir_all(source.join("references")).unwrap();
        fs::write(source.join("SKILL.md"), "demo\n").unwrap();
        fs::write(source.join(".hidden"), "hidden\n").unwrap();
        fs::write(source.join(".git/HEAD"), "ignored\n").unwrap();
        fs::write(source.join("references/a.md"), "a\n").unwrap();

        assert_eq!(
            discover_skills(&temp.path().join("checkout/skills")).unwrap(),
            vec!["demo"]
        );

        let dest = temp.path().join("external/demo");
        fs::create_dir_all(&dest).unwrap();
        copy_dir(&source, &dest).unwrap();

        assert!(dest.join("SKILL.md").is_file());
        assert!(dest.join(".hidden").is_file());
        assert!(dest.join("references/a.md").is_file());
        assert!(!dest.join(".git/HEAD").exists());
    }

    #[test]
    fn install_alias_check_mode_reports_drift_without_writing() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src/demo");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("SKILL.md"), "demo\n").unwrap();
        let external = temp.path().join("skills/.external");
        fs::create_dir_all(&external).unwrap();
        let mut state = SyncState::default();

        install_alias(
            "demo",
            &src,
            "org/repo",
            "0123456789abcdef0123456789abcdef01234567",
            &external,
            SyncMode::Check,
            &mut state,
        )
        .unwrap();

        assert!(state.changed);
        assert_eq!(state.aliases, vec!["demo"]);
        assert!(!external.join("demo/SKILL.md").exists());
        assert!(state.lines[0].contains("would install/update: demo (org/repo @ 0123456)"));
    }

    #[test]
    fn cleanup_orphans_removes_undeclared_aliases_and_keeps_checkouts() {
        let temp = TempDir::new().unwrap();
        let external = temp.path().join("skills/.external");
        fs::create_dir_all(external.join("declared")).unwrap();
        fs::create_dir_all(external.join("orphan")).unwrap();
        fs::create_dir_all(external.join("_checkouts/still-here")).unwrap();
        let mut state = SyncState {
            aliases: vec!["declared".to_string()],
            ..SyncState::default()
        };

        cleanup_orphans(&external, SyncMode::Sync, &mut state).unwrap();

        assert!(external.join("declared").is_dir());
        assert!(!external.join("orphan").exists());
        assert!(external.join("_checkouts/still-here").is_dir());
        assert!(state.changed);
    }
}
