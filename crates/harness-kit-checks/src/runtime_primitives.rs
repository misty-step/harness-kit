use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

const CLAUDE_HOOKS: &[&str] = &[
    "permission-auto-approve",
    "destructive-command-guard",
    "github-cli-guard",
    "time-context",
    "skill-invocation-tracker",
];

pub fn run(repo: &Path) -> Result<String> {
    check_claude_settings(repo)?;
    check_skill_invocation_protocols(repo)?;
    Ok("runtime primitives valid".to_string())
}

fn check_claude_settings(repo: &Path) -> Result<()> {
    let path = repo.join("harnesses/claude/settings.json");
    let value: Value = serde_json::from_str(
        &fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", path.display()))?;
    let hooks = value
        .get("hooks")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("{}: missing hooks object", path.display()))?;
    let valid_hooks: BTreeSet<_> = CLAUDE_HOOKS.iter().copied().collect();
    let mut checked = 0usize;
    for (event, groups) in hooks {
        let groups = groups
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("{}: hooks.{event} must be a list", path.display()))?;
        for group in groups {
            let group = group.as_object().ok_or_else(|| {
                anyhow::anyhow!("{}: hooks.{event} entries must be objects", path.display())
            })?;
            let entries = group
                .get("hooks")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    anyhow::anyhow!("{}: hooks.{event}.hooks must be a list", path.display())
                })?;
            for entry in entries {
                let entry = entry.as_object().ok_or_else(|| {
                    anyhow::anyhow!("{}: hook entry must be an object", path.display())
                })?;
                let command = entry.get("command").and_then(Value::as_str).unwrap_or("");
                let Some(hook) = rust_hook_command(command) else {
                    continue;
                };
                checked += 1;
                if !valid_hooks.contains(hook) {
                    bail!("{}: unknown claude-hook command {hook}", path.display());
                }
            }
        }
    }
    if checked == 0 {
        bail!(
            "{}: no harness-kit-checks claude-hook commands validated",
            path.display()
        );
    }
    Ok(())
}

fn rust_hook_command(command: &str) -> Option<&str> {
    let mut words = command.split_whitespace();
    while let Some(word) = words.next() {
        if word == "harness-kit-checks" && words.next() == Some("claude-hook") {
            return words.next();
        }
    }
    None
}

fn check_skill_invocation_protocols(repo: &Path) -> Result<()> {
    let path = repo.join(".harness-kit/examples/skill-invocations.jsonl");
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut rows = 0usize;
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        rows += 1;
        let record: Value = serde_json::from_str(line)
            .with_context(|| format!("{}:{}: invalid JSON", path.display(), index + 1))?;
        let harness = record.get("harness").and_then(Value::as_str).unwrap_or("");
        let protocol = record
            .get("source_protocol")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !allowed_protocol(harness, protocol) {
            bail!(
                "{}:{}: {harness}/{protocol} is not a verified live hook, import, or explicit fixture protocol",
                path.display(),
                index + 1
            );
        }
    }
    if rows == 0 {
        bail!("{}: no skill invocation fixture rows", path.display());
    }
    Ok(())
}

fn allowed_protocol(harness: &str, protocol: &str) -> bool {
    match protocol {
        "external_import" | "manual_fixture" => true,
        "post_tool_use" => harness == "claude",
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn rejects_unverified_protocols() {
        assert!(allowed_protocol("claude", "post_tool_use"));
        assert!(allowed_protocol("codex", "manual_fixture"));
        assert!(!allowed_protocol("codex", "post_tool_use"));
    }

    #[test]
    fn validates_runtime_fixture_contract() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("harnesses/claude")).unwrap();
        fs::create_dir_all(temp.path().join(".harness-kit/examples")).unwrap();
        fs::write(
            temp.path().join("harnesses/claude/settings.json"),
            r#"{"hooks":{"PostToolUse":[{"hooks":[{"command":"harness-kit-checks claude-hook skill-invocation-tracker"}]}]}}"#,
        )
        .unwrap();
        fs::write(
            temp.path()
                .join(".harness-kit/examples/skill-invocations.jsonl"),
            r#"{"harness":"claude","source_protocol":"post_tool_use"}
{"harness":"codex","source_protocol":"manual_fixture"}
"#,
        )
        .unwrap();

        assert_eq!(run(temp.path()).unwrap(), "runtime primitives valid");
    }
}
