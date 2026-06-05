use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use serde_yaml::Value as YamlValue;
use tempfile::Builder;

pub fn load_roster(path: &Path) -> Result<YamlValue> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let roster: YamlValue = serde_yaml::from_str(&text)?;
    if !roster.is_mapping() {
        bail!("roster must be a YAML mapping");
    }
    Ok(roster)
}

pub fn select_providers(
    roster: &YamlValue,
    requested: Option<&[String]>,
    minimum: usize,
) -> Result<Vec<String>> {
    let root = roster
        .as_mapping()
        .expect("load_roster and tests pass mappings");
    let providers = root
        .get(YamlValue::String("providers".to_string()))
        .unwrap_or(roster);
    let Some(providers) = providers.as_mapping() else {
        bail!("roster providers must be a mapping");
    };
    let candidates: Vec<String> = requested.map_or_else(
        || {
            providers
                .keys()
                .filter_map(YamlValue::as_str)
                .map(ToString::to_string)
                .collect()
        },
        |requested| requested.to_vec(),
    );
    let mut selected = Vec::new();
    for provider_id in candidates {
        let Some(provider) = providers
            .get(YamlValue::String(provider_id.clone()))
            .and_then(YamlValue::as_mapping)
        else {
            continue;
        };
        let kind = provider
            .get(YamlValue::String("kind".to_string()))
            .and_then(YamlValue::as_str);
        let tier = provider
            .get(YamlValue::String("tier".to_string()))
            .and_then(YamlValue::as_str);
        if kind == Some("manual") || matches!(tier, Some("manual" | "disabled")) {
            continue;
        }
        selected.push(provider_id);
        if selected.len() == minimum {
            break;
        }
    }
    if selected.len() < minimum {
        bail!("need at least {minimum} non-manual providers");
    }
    Ok(selected)
}

pub fn build_prompt(packet: &Value) -> Result<String> {
    Ok(format!(
        "Role: skillify classifier.\n\
Objective: evaluate whether this conversation contains a novel, repeatable, portable workflow worth turning into a first-party Harness Kit skill.\n\
Return JSON with fields: skill_worthy, confidence, suggested_name, novelty_reason, repeatability_reason, portability_risk.\n\n{}",
        serde_json::to_string_pretty(packet)?
    ))
}

pub fn build_dispatch_commands(
    _repo_root: &Path,
    prompt_file: &Path,
    providers: &[String],
    input_ref: &str,
    backlog_ref: &str,
    timeout_s: u64,
) -> Vec<String> {
    providers
        .iter()
        .map(|provider| {
            [
                "cargo".to_string(),
                "run".to_string(),
                "--locked".to_string(),
                "-p".to_string(),
                "harness-kit-checks".to_string(),
                "--".to_string(),
                "dispatch-agent".to_string(),
                "--provider-target".to_string(),
                provider.clone(),
                "--objective".to_string(),
                "skillify novelty and repeatability classification".to_string(),
                "--input-ref".to_string(),
                input_ref.to_string(),
                "--prompt-file".to_string(),
                prompt_file.display().to_string(),
                "--backlog-ref".to_string(),
                backlog_ref.to_string(),
                "--timeout-s".to_string(),
                timeout_s.to_string(),
            ]
            .iter()
            .map(|part| shell_quote(part))
            .collect::<Vec<_>>()
            .join(" ")
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassifyOptions {
    pub packet_path: PathBuf,
    pub roster_path: PathBuf,
    pub repo_root: PathBuf,
    pub providers: Vec<String>,
    pub dry_run: bool,
    pub timeout_s: u64,
}

pub fn classify(options: &ClassifyOptions) -> Result<Value> {
    let packet: Value = serde_json::from_str(&fs::read_to_string(&options.packet_path)?)?;
    let requested = (!options.providers.is_empty()).then_some(options.providers.as_slice());
    let selected = select_providers(&load_roster(&options.roster_path)?, requested, 2)?;
    let prompt = build_prompt(&packet)?;
    let mut prompt_file = Builder::new().suffix("-skillify-prompt.md").tempfile()?;
    prompt_file.write_all(prompt.as_bytes())?;
    let (_file, prompt_path) = prompt_file.keep()?;
    let commands = build_dispatch_commands(
        &options.repo_root,
        &prompt_path,
        &selected,
        &options.packet_path.display().to_string(),
        "075",
        options.timeout_s,
    );
    if options.dry_run {
        return Ok(json!({"status": "dry_run", "providers": selected, "commands": commands}));
    }
    let mut receipts = Vec::new();
    for command in &commands {
        let completed = Command::new("sh").arg("-c").arg(command).output()?;
        if !completed.status.success() {
            let stderr = String::from_utf8_lossy(&completed.stderr);
            let stdout = String::from_utf8_lossy(&completed.stdout);
            let message = if !stderr.is_empty() {
                stderr.to_string()
            } else if !stdout.is_empty() {
                stdout.to_string()
            } else {
                format!("dispatch failed: {command}")
            };
            bail!("{}", message.trim_end());
        }
        let stdout = String::from_utf8_lossy(&completed.stdout);
        let Some(last_line) = stdout.lines().last() else {
            bail!("dispatch returned no receipt");
        };
        receipts.push(serde_json::from_str::<Value>(last_line)?);
    }
    Ok(json!({"status": "classified", "providers": selected, "receipts": receipts}))
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }
    if value
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || "_@%+=:,./-".contains(character))
    {
        return value.to_string();
    }
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use serde_yaml::Mapping;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn selects_two_non_manual_providers_and_builds_dispatch_commands() {
        let roster = serde_yaml::from_str(
            r#"
providers:
  codex:
    tier: primary
    kind: cli
  pi:
    tier: primary
    kind: cli
  manual:
    tier: manual
    kind: manual
  grok-build:
    tier: disabled
    kind: cli
"#,
        )
        .unwrap();
        let requested = vec!["codex".to_string(), "pi".to_string()];
        let providers = select_providers(&roster, Some(&requested), 2).unwrap();
        assert_eq!(providers, vec!["codex", "pi"]);

        let tmp = TempDir::new().unwrap();
        let commands = build_dispatch_commands(
            Path::new("."),
            &tmp.path().join("prompt.md"),
            &providers,
            "packet.json",
            "075",
            30,
        );

        assert_eq!(commands.len(), 2);
        assert!(
            commands
                .iter()
                .all(|command| command.contains("harness-kit-checks -- dispatch-agent"))
        );
        assert!(
            commands
                .iter()
                .all(|command| command.contains("--backlog-ref 075"))
        );
    }

    #[test]
    fn rejects_roster_without_two_non_manual_providers() {
        let mut roster = Mapping::new();
        roster.insert(
            YamlValue::String("providers".to_string()),
            serde_yaml::from_str("manual:\n  tier: manual\n  kind: manual\n").unwrap(),
        );
        let error = select_providers(&YamlValue::Mapping(roster), None, 2).unwrap_err();

        assert_eq!(error.to_string(), "need at least 2 non-manual providers");
    }
}
