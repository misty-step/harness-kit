use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::lint_gates::GateReport;

#[derive(Debug, Deserialize)]
struct Registry {
    version: u32,
    servers: Vec<Server>,
    profiles: Vec<Profile>,
}

#[derive(Debug, Deserialize)]
struct Server {
    id: String,
    app: String,
    source_repo: String,
    product_skill: String,
    status: String,
    reason: Option<String>,
    codex: Option<CodexServer>,
}

#[derive(Debug, Deserialize)]
struct CodexServer {
    server_name: String,
    command: Option<String>,
    url: Option<String>,
    args: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct Profile {
    id: String,
    servers: Vec<String>,
}

pub fn check_repo(repo: &Path) -> Result<GateReport> {
    let path = repo.join(".harness-kit/factory-mcps.yaml");
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let registry: Registry = serde_yaml::from_str(&text)
        .with_context(|| format!("invalid factory MCP registry: {}", path.display()))?;

    let mut errors = Vec::new();
    if registry.version != 1 {
        errors.push(format!(
            "factory MCP registry version must be 1, got {}",
            registry.version
        ));
    }
    if registry.servers.is_empty() {
        errors.push("factory MCP registry must declare at least one server".to_string());
    }

    let mut server_ids = BTreeSet::new();
    let mut codex_names = BTreeSet::<String>::new();
    for server in &registry.servers {
        if server.id.trim().is_empty() {
            errors.push("server id must not be empty".to_string());
            continue;
        }
        if !server_ids.insert(server.id.as_str()) {
            errors.push(format!("duplicate server id '{}'", server.id));
        }
        if server.app.trim().is_empty() {
            errors.push(format!("server '{}' must name an app", server.id));
        }
        if !server.source_repo.contains('/') {
            errors.push(format!(
                "server '{}' source_repo must be an owner/repo path",
                server.id
            ));
        }
        if !skill_exists(repo, &server.product_skill) {
            errors.push(format!(
                "server '{}' product_skill '{}' is not installed",
                server.id, server.product_skill
            ));
        }
        match server.status.as_str() {
            "available" => validate_available_server(server, &mut errors, &mut codex_names),
            "not_applicable" => {
                if server.reason.as_deref().unwrap_or("").trim().is_empty() {
                    errors.push(format!(
                        "server '{}' is not_applicable but does not explain why",
                        server.id
                    ));
                }
            }
            other => errors.push(format!(
                "server '{}' has unsupported status '{}'",
                server.id, other
            )),
        }
    }

    let known = registry
        .servers
        .iter()
        .map(|server| server.id.as_str())
        .collect::<BTreeSet<_>>();
    for profile in &registry.profiles {
        if profile.id.trim().is_empty() {
            errors.push("profile id must not be empty".to_string());
        }
        for server in &profile.servers {
            if !known.contains(server.as_str()) {
                errors.push(format!(
                    "profile '{}' references unknown server '{}'",
                    profile.id, server
                ));
            }
        }
    }

    if errors.is_empty() {
        Ok(GateReport::success(format!(
            "Factory MCP registry valid: {} servers, {} profiles.",
            registry.servers.len(),
            registry.profiles.len()
        )))
    } else {
        Ok(GateReport::failure(errors))
    }
}

fn validate_available_server(
    server: &Server,
    errors: &mut Vec<String>,
    codex_names: &mut BTreeSet<String>,
) {
    let Some(codex) = server.codex.as_ref() else {
        errors.push(format!(
            "available server '{}' must declare a codex launcher",
            server.id
        ));
        return;
    };
    if codex.server_name.trim().is_empty() {
        errors.push(format!("server '{}' codex.server_name is empty", server.id));
    } else if !codex_names.insert(codex.server_name.clone()) {
        errors.push(format!(
            "duplicate codex server_name '{}'",
            codex.server_name
        ));
    }
    let has_command = codex
        .command
        .as_deref()
        .is_some_and(|command| !command.trim().is_empty());
    let has_url = codex
        .url
        .as_deref()
        .is_some_and(|url| !url.trim().is_empty());
    if has_command == has_url {
        errors.push(format!(
            "server '{}' codex launcher must declare exactly one of command or url",
            server.id
        ));
    }
    if has_command && codex.args.as_ref().is_some_and(Vec::is_empty) {
        errors.push(format!(
            "server '{}' codex args should be omitted rather than empty",
            server.id
        ));
    }
}

fn skill_exists(repo: &Path, skill: &str) -> bool {
    repo.join("skills").join(skill).join("SKILL.md").is_file()
        || repo
            .join("skills/.external")
            .join(skill)
            .join("SKILL.md")
            .is_file()
}
