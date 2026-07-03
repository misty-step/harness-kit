use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};

use crate::mcp_registry::{CodexServer, Registry, Server, load_registry};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyOptions {
    pub repo: PathBuf,
    pub harness: String,
    pub profiles: Vec<String>,
    pub all_profiles: bool,
    pub project: PathBuf,
    pub codex_home: PathBuf,
    pub dry_run: bool,
    pub check_env: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApplyReport {
    pub harness: String,
    pub registry: PathBuf,
    pub config: PathBuf,
    pub project: PathBuf,
    pub profiles: Vec<String>,
    pub dry_run: bool,
    pub changed: bool,
    pub actions: Vec<PlanAction>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanAction {
    pub kind: ActionKind,
    pub server_id: String,
    pub codex_name: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionKind {
    Add,
    Update,
    Unchanged,
    Skip,
}

pub fn run_cli(args: &[String]) -> Result<String> {
    let options = parse_options(args)?;
    apply(&options).map(|report| report.render())
}

pub fn apply(options: &ApplyOptions) -> Result<ApplyReport> {
    if options.harness != "codex" {
        bail!("only --harness codex is supported for factory MCP materialization");
    }

    let registry = load_registry(&options.repo)?;
    let profiles = resolve_profiles(&registry, options)?;
    let selected_servers = selected_server_ids(&registry, &profiles);
    let config = options.codex_home.join("config.toml");
    let mut config_text = match fs::read_to_string(&config) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", config.display()));
        }
    };
    let mut actions = Vec::new();
    let mut changed = false;
    let project = normalized_path(&options.project);

    for server in &registry.servers {
        if !selected_servers.contains(&server.id) {
            continue;
        }
        let action = plan_server(server, &project, &config_text, options.check_env)?;
        if matches!(action.kind, ActionKind::Add | ActionKind::Update) && !options.dry_run {
            let codex = server
                .codex
                .as_ref()
                .context("ready server missing codex launcher")?;
            let block = render_codex_block(codex);
            config_text = upsert_mcp_table(&config_text, &codex.server_name, &block);
            changed = true;
        }
        actions.push(action);
    }

    if changed && !options.dry_run {
        if let Some(parent) = config.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(&config, config_text)
            .with_context(|| format!("failed to write {}", config.display()))?;
    }

    Ok(ApplyReport {
        harness: options.harness.clone(),
        registry: options.repo.join(".harness-kit/factory-mcps.yaml"),
        config,
        project,
        profiles,
        dry_run: options.dry_run,
        changed,
        actions,
    })
}

fn plan_server(
    server: &Server,
    project: &Path,
    config_text: &str,
    check_env: bool,
) -> Result<PlanAction> {
    let codex_name = server.codex.as_ref().map(|codex| codex.server_name.clone());
    if server.status != "available" {
        return Ok(skip(
            server,
            codex_name,
            server
                .reason
                .clone()
                .unwrap_or_else(|| format!("status is {}", server.status)),
        ));
    }
    if !scope_matches(server, project) {
        return Ok(skip(
            server,
            codex_name,
            format!("project {} is outside server scope", project.display()),
        ));
    }
    let Some(codex) = server.codex.as_ref() else {
        return Ok(skip(server, codex_name, "no Codex launcher declared"));
    };
    if check_env && !launcher_available(codex) {
        return Ok(skip(
            server,
            codex_name,
            format!(
                "launcher command '{}' is not available",
                launcher_label(codex)
            ),
        ));
    }
    if let Some(reason) = env_skip_reason(server, check_env) {
        return Ok(skip(server, codex_name, reason));
    }

    let block = render_codex_block(codex);
    let kind = match extract_mcp_table(config_text, &codex.server_name) {
        None => ActionKind::Add,
        Some(existing) if same_toml_block(existing, &block) => ActionKind::Unchanged,
        Some(_) => ActionKind::Update,
    };
    Ok(PlanAction {
        kind,
        server_id: server.id.clone(),
        codex_name,
        reason: None,
    })
}

fn skip(server: &Server, codex_name: Option<String>, reason: impl Into<String>) -> PlanAction {
    PlanAction {
        kind: ActionKind::Skip,
        server_id: server.id.clone(),
        codex_name,
        reason: Some(reason.into()),
    }
}

fn resolve_profiles(registry: &Registry, options: &ApplyOptions) -> Result<Vec<String>> {
    if options.all_profiles && !options.profiles.is_empty() {
        bail!("use either --all-profiles or one or more --profile flags, not both");
    }
    let known = registry
        .profiles
        .iter()
        .map(|profile| profile.id.as_str())
        .collect::<BTreeSet<_>>();
    let profiles = if options.all_profiles {
        registry
            .profiles
            .iter()
            .map(|profile| profile.id.clone())
            .collect::<Vec<_>>()
    } else {
        if options.profiles.is_empty() {
            bail!("factory MCP materialization requires --profile ID or --all-profiles");
        }
        options.profiles.clone()
    };
    for profile in &profiles {
        if !known.contains(profile.as_str()) {
            bail!("unknown factory MCP profile '{profile}'");
        }
    }
    Ok(profiles)
}

fn selected_server_ids(registry: &Registry, profiles: &[String]) -> BTreeSet<String> {
    let wanted = profiles.iter().map(String::as_str).collect::<BTreeSet<_>>();
    registry
        .profiles
        .iter()
        .filter(|profile| wanted.contains(profile.id.as_str()))
        .flat_map(|profile| profile.servers.iter().cloned())
        .collect()
}

fn scope_matches(server: &Server, project: &Path) -> bool {
    let Some(scope) = server.scope.as_ref() else {
        return true;
    };
    let project = path_for_match(project);
    let includes = scope.include_repo_globs.as_deref().unwrap_or(&[]);
    let excludes = scope.exclude_repo_globs.as_deref().unwrap_or(&[]);
    let included = includes.is_empty() || includes.iter().any(|glob| glob_matches(glob, &project));
    let excluded = excludes.iter().any(|glob| glob_matches(glob, &project));
    included && !excluded
}

fn glob_matches(glob: &str, path: &str) -> bool {
    let glob = glob.trim_end_matches('/');
    if glob == "*" {
        return true;
    }
    if let Some(prefix) = glob.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    path == glob
}

fn env_skip_reason(server: &Server, check_env: bool) -> Option<String> {
    let groups = server.required_env_any.as_ref()?;
    if groups.is_empty() {
        return None;
    }
    let sources = server
        .env_sources
        .as_ref()
        .map(|items| {
            items
                .iter()
                .map(|source| (source.name.as_str(), source.op_ref.as_str()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut group_reports = Vec::new();
    for group in groups {
        let missing = group
            .iter()
            .filter_map(|name| missing_env_reason(name, &sources, check_env))
            .collect::<Vec<_>>();
        if missing.is_empty() {
            return None;
        }
        group_reports.push(format!(
            "[{}] missing {}",
            group.join(", "),
            missing.join(", ")
        ));
    }
    Some(format!(
        "required_env_any unsatisfied; need one complete group: {}",
        group_reports.join("; ")
    ))
}

fn missing_env_reason(
    name: &str,
    sources: &BTreeMap<&str, &str>,
    check_env: bool,
) -> Option<String> {
    if env::var(name).is_ok_and(|value| !value.is_empty()) {
        return None;
    }
    let Some(op_ref) = sources.get(name) else {
        return Some(format!("{name} has no env value or env_source"));
    };
    if !op_ref.starts_with("op://Agents/") {
        return Some(format!("{name} env_source is not in the Agents vault"));
    }
    if !check_env || op_read_available(op_ref) {
        return None;
    }
    Some(format!("{name} env_source is unreadable"))
}

fn op_read_available(op_ref: &str) -> bool {
    Command::new("op")
        .args(["read", op_ref])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn launcher_available(codex: &CodexServer) -> bool {
    let Some(command) = codex.command.as_deref() else {
        return true;
    };
    let path = Path::new(command);
    if path.is_absolute() || command.contains('/') {
        return path.is_file();
    }
    env::var_os("PATH")
        .map(|paths| env::split_paths(&paths).any(|dir| dir.join(command).is_file()))
        .unwrap_or(false)
}

fn launcher_label(codex: &CodexServer) -> &str {
    codex
        .command
        .as_deref()
        .or(codex.url.as_deref())
        .unwrap_or("<missing>")
}

fn render_codex_block(codex: &CodexServer) -> String {
    let mut lines = vec![format!("[mcp_servers.{}]", codex.server_name)];
    if let Some(command) = codex.command.as_deref() {
        lines.push(format!("command = {}", quote_toml(command)));
    }
    if let Some(url) = codex.url.as_deref() {
        lines.push(format!("url = {}", quote_toml(url)));
    }
    if let Some(args) = codex.args.as_ref() {
        let args = args
            .iter()
            .map(|arg| quote_toml(arg))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!("args = [{args}]"));
    }
    lines.join("\n") + "\n"
}

fn quote_toml(value: &str) -> String {
    format!("{value:?}")
}

fn same_toml_block(left: &str, right: &str) -> bool {
    left.trim() == right.trim()
}

fn extract_mcp_table<'a>(text: &'a str, name: &str) -> Option<&'a str> {
    table_span(text, name).map(|(start, end)| &text[start..end])
}

fn upsert_mcp_table(text: &str, name: &str, block: &str) -> String {
    if let Some((start, end)) = table_span(text, name) {
        let mut output = String::with_capacity(text.len() + block.len());
        output.push_str(&text[..start]);
        output.push_str(block);
        output.push_str(&text[end..]);
        return output;
    }
    let mut output = text.to_string();
    if !output.is_empty() && !output.ends_with('\n') {
        output.push('\n');
    }
    if !output.is_empty() && !output.ends_with("\n\n") {
        output.push('\n');
    }
    output.push_str(block);
    output
}

fn table_span(text: &str, name: &str) -> Option<(usize, usize)> {
    let mut start = None;
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if is_table_header(trimmed) {
            match start {
                None if is_managed_mcp_header(trimmed, name) => start = Some(offset),
                Some(existing_start) if !is_managed_mcp_header(trimmed, name) => {
                    return Some((existing_start, offset));
                }
                _ => {}
            }
        }
        offset += line.len();
    }
    start.map(|existing_start| (existing_start, text.len()))
}

fn is_table_header(line: &str) -> bool {
    line.starts_with('[') && line.ends_with(']')
}

fn is_managed_mcp_header(line: &str, name: &str) -> bool {
    line == format!("[mcp_servers.{name}]") || line.starts_with(&format!("[mcp_servers.{name}."))
}

fn parse_options(args: &[String]) -> Result<ApplyOptions> {
    let mut repo = PathBuf::from(".");
    let mut harness = "codex".to_string();
    let mut profiles = Vec::new();
    let mut all_profiles = false;
    let mut project = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut codex_home = default_codex_home()?;
    let mut dry_run = false;
    let mut check_env = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                index += 1;
                repo = PathBuf::from(value(args, index, "--repo")?);
            }
            "--harness" => {
                index += 1;
                harness = value(args, index, "--harness")?;
            }
            "--profile" => {
                index += 1;
                profiles.push(value(args, index, "--profile")?);
            }
            "--all-profiles" => all_profiles = true,
            "--project" => {
                index += 1;
                project = PathBuf::from(value(args, index, "--project")?);
            }
            "--codex-home" => {
                index += 1;
                codex_home = PathBuf::from(value(args, index, "--codex-home")?);
            }
            "--dry-run" => dry_run = true,
            "--check-env" => check_env = Some(true),
            "--skip-env-check" => check_env = Some(false),
            "-h" | "--help" => bail!("{}", usage()),
            other => bail!("unknown apply-factory-mcps argument '{other}'\n{}", usage()),
        }
        index += 1;
    }

    Ok(ApplyOptions {
        repo,
        harness,
        profiles,
        all_profiles,
        project,
        codex_home,
        dry_run,
        check_env: check_env.unwrap_or(!dry_run),
    })
}

fn value(args: &[String], index: usize, flag: &str) -> Result<String> {
    args.get(index)
        .cloned()
        .with_context(|| format!("{flag} requires a value"))
}

fn default_codex_home() -> Result<PathBuf> {
    if let Some(home) = env::var_os("CODEX_HOME") {
        return Ok(PathBuf::from(home));
    }
    Ok(env::var_os("HOME")
        .map(PathBuf::from)
        .context("HOME must be set to infer CODEX_HOME")?
        .join(".codex"))
}

fn usage() -> &'static str {
    "usage: harness-kit-checks apply-factory-mcps --profile ID|--all-profiles [--repo PATH] [--harness codex] [--project PATH] [--codex-home PATH] [--dry-run] [--check-env|--skip-env-check]"
}

fn normalized_path(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}

fn path_for_match(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

impl ApplyReport {
    pub fn render(&self) -> String {
        let mut lines = vec![
            format!("factory MCP materializer: {}", self.harness),
            format!("registry: {}", self.registry.display()),
            format!("config: {}", self.config.display()),
            format!("project: {}", self.project.display()),
            format!("profiles: {}", self.profiles.join(", ")),
            format!("mode: {}", if self.dry_run { "dry-run" } else { "apply" }),
        ];
        for action in &self.actions {
            lines.push(action.render());
        }
        if !self.dry_run {
            lines.push(if self.changed {
                "result: wrote config".to_string()
            } else {
                "result: no changes".to_string()
            });
        }
        lines.join("\n")
    }
}

impl PlanAction {
    fn render(&self) -> String {
        let verb = match self.kind {
            ActionKind::Add => "ADD",
            ActionKind::Update => "UPDATE",
            ActionKind::Unchanged => "UNCHANGED",
            ActionKind::Skip => "SKIP",
        };
        let mut line = format!("{verb} {}", self.server_id);
        if let Some(name) = &self.codex_name
            && name != &self.server_id
        {
            line.push_str(&format!(" (codex: {name})"));
        }
        if let Some(reason) = &self.reason {
            line.push_str(&format!(": {reason}"));
        }
        line
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_registry(root: &Path) {
        fs::create_dir_all(root.join(".harness-kit")).unwrap();
        fs::write(
            root.join(".harness-kit/factory-mcps.yaml"),
            r#"
version: 1
servers:
  - id: canary
    app: Canary
    source_repo: misty-step/canary
    product_skill: misty-canary
    status: available
    scope:
      default_profiles: [global]
      include_repo_globs: ["*"]
      exclude_repo_globs: []
    codex:
      server_name: canary
      command: bash
      args: ["-lc", "exec canary"]
  - id: powder
    app: Powder
    source_repo: misty-step/powder
    product_skill: misty-powder
    status: available
    scope:
      default_profiles: [non-adminifi-non-r90]
      include_repo_globs: ["/Users/phaedrus/Development/**"]
      exclude_repo_globs:
        - "/Users/phaedrus/Development/adminifi/**"
        - "/Users/phaedrus/Development/r90/**"
    required_env_any:
      - [POWDER_API_BASE_URL, POWDER_API_KEY]
    env_sources:
      - name: POWDER_API_BASE_URL
        op_ref: op://Agents/POWDER_ENDPOINT/URL
      - name: POWDER_API_KEY
        op_ref: op://Agents/POWDER_API_KEY__bridge/credential
    codex:
      server_name: powder
      command: bash
      args: ["-lc", "exec powder"]
  - id: bitterblossom
    app: Bitterblossom
    source_repo: misty-step/bitterblossom
    product_skill: misty-bitterblossom
    status: available
    scope:
      default_profiles: [factory-ops]
      include_repo_globs: ["/Users/phaedrus/Development/**"]
      exclude_repo_globs: []
    codex:
      server_name: bitterblossom
      command: bash
      args: ["-lc", "exec bb"]
profiles:
  - id: global
    servers: [canary]
  - id: non-adminifi-non-r90
    servers: [powder]
  - id: factory-ops
    servers: [canary, powder, bitterblossom]
"#,
        )
        .unwrap();
    }

    fn options(repo: &Path, codex_home: &Path) -> ApplyOptions {
        ApplyOptions {
            repo: repo.to_path_buf(),
            harness: "codex".to_string(),
            profiles: Vec::new(),
            all_profiles: true,
            project: PathBuf::from("/Users/phaedrus/Development/harness-kit"),
            codex_home: codex_home.to_path_buf(),
            dry_run: true,
            check_env: false,
        }
    }

    #[test]
    fn dry_run_plans_profile_matched_adds_without_secret_values() {
        let repo = tempfile::tempdir().unwrap();
        let codex_home = tempfile::tempdir().unwrap();
        write_registry(repo.path());

        let report = apply(&options(repo.path(), codex_home.path())).unwrap();
        let rendered = report.render();

        assert!(rendered.contains("ADD canary"));
        assert!(rendered.contains("ADD powder"));
        assert!(rendered.contains("ADD bitterblossom"));
        assert!(!rendered.contains("exec powder"));
        assert!(!codex_home.path().join("config.toml").exists());
    }

    #[test]
    fn repo_scope_excludes_powder_for_adminifi_projects() {
        let repo = tempfile::tempdir().unwrap();
        let codex_home = tempfile::tempdir().unwrap();
        write_registry(repo.path());
        let mut opts = options(repo.path(), codex_home.path());
        opts.project = PathBuf::from("/Users/phaedrus/Development/adminifi/olympus");

        let report = apply(&opts).unwrap();
        let rendered = report.render();

        assert!(rendered.contains("ADD canary"));
        assert!(rendered.contains("SKIP powder"));
        assert!(rendered.contains("outside server scope"));
    }

    #[test]
    fn required_env_any_skips_without_env_or_sources() {
        let repo = tempfile::tempdir().unwrap();
        let codex_home = tempfile::tempdir().unwrap();
        fs::create_dir_all(repo.path().join(".harness-kit")).unwrap();
        fs::write(
            repo.path().join(".harness-kit/factory-mcps.yaml"),
            r#"
version: 1
servers:
  - id: powder
    app: Powder
    source_repo: misty-step/powder
    product_skill: misty-powder
    status: available
    scope:
      include_repo_globs: ["*"]
      exclude_repo_globs: []
    required_env_any:
      - [POWDER_TEST_MISSING_ONE, POWDER_TEST_MISSING_TWO]
    codex:
      server_name: powder
      command: bash
profiles:
  - id: non-adminifi-non-r90
    servers: [powder]
"#,
        )
        .unwrap();
        let mut opts = options(repo.path(), codex_home.path());
        opts.all_profiles = false;
        opts.profiles = vec!["non-adminifi-non-r90".to_string()];

        let report = apply(&opts).unwrap();
        let rendered = report.render();

        assert!(rendered.contains("SKIP powder"));
        assert!(rendered.contains("required_env_any unsatisfied"));
    }

    #[test]
    fn apply_replaces_matching_mcp_table_and_preserves_unrelated_tables() {
        let repo = tempfile::tempdir().unwrap();
        let codex_home = tempfile::tempdir().unwrap();
        write_registry(repo.path());
        fs::write(
            codex_home.path().join("config.toml"),
            r#"[mcp_servers.unrelated]
command = "node"
args = ["server.js"]

[mcp_servers.canary]
command = "old-canary"

[mcp_servers.canary.env]
TOKEN = "old"
"#,
        )
        .unwrap();
        let mut opts = options(repo.path(), codex_home.path());
        opts.dry_run = false;

        let report = apply(&opts).unwrap();
        let config = fs::read_to_string(codex_home.path().join("config.toml")).unwrap();

        assert!(report.changed);
        assert!(report.render().contains("UPDATE canary"));
        assert!(config.contains("[mcp_servers.unrelated]"));
        assert!(config.contains("[mcp_servers.canary]\ncommand = \"bash\""));
        assert!(!config.contains("old-canary"));
        assert!(!config.contains("[mcp_servers.canary.env]"));
    }

    #[test]
    fn second_apply_is_idempotent() {
        let repo = tempfile::tempdir().unwrap();
        let codex_home = tempfile::tempdir().unwrap();
        write_registry(repo.path());
        let mut opts = options(repo.path(), codex_home.path());
        opts.dry_run = false;

        apply(&opts).unwrap();
        let second = apply(&opts).unwrap();

        assert!(!second.changed);
        assert!(second.render().contains("UNCHANGED canary"));
        assert!(second.render().contains("UNCHANGED powder"));
    }
}
