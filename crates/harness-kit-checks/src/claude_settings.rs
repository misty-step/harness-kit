use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

const BARE_HOOK_PREFIX: &str = "harness-kit-checks claude-hook ";
const EXPECTED_GENERATED_HOOKS: &[&str] = &[
    "permission-auto-approve",
    "destructive-command-guard",
    "secrets-read-guard",
    "github-cli-guard",
    "time-context",
    "skill-invocation-tracker",
];

pub fn installed_cli_path(home: &Path) -> PathBuf {
    home.join(".harness-kit/bin/harness-kit-checks")
}

pub fn render_with_cli_path(settings_text: &str, cli_path: &Path) -> Result<String> {
    let mut value: Value =
        serde_json::from_str(settings_text).context("failed to parse Claude settings JSON")?;
    rewrite_hook_commands(&mut value, &shell_quote(cli_path));
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

pub fn validate_settings_file(settings_path: &Path) -> Result<Vec<String>> {
    let text = fs::read_to_string(settings_path)
        .with_context(|| format!("failed to read {}", settings_path.display()))?;
    validate_settings_text(&text)
        .with_context(|| format!("failed to validate {}", settings_path.display()))
}

pub fn install_rendered_settings(src: &Path, dest: &Path, installed_cli: &Path) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent)?;
    }
    let source =
        fs::read_to_string(src).with_context(|| format!("failed to read {}", src.display()))?;
    let rendered = render_with_cli_path(&source, installed_cli)?;
    fs::write(dest, rendered).with_context(|| format!("failed to write {}", dest.display()))?;
    let errors = validate_settings_file(dest)?;
    if !errors.is_empty() {
        anyhow::bail!(
            "generated Claude settings contain unresolvable harness-kit hook commands:\n{}",
            errors.join("\n")
        );
    }
    Ok(())
}

pub fn installed_cli_for_claude_dir(harness_dir: &Path) -> PathBuf {
    let home = harness_dir.parent().unwrap_or(harness_dir);
    installed_cli_path(home)
}

pub fn validate_settings_text(settings_text: &str) -> Result<Vec<String>> {
    let value: Value =
        serde_json::from_str(settings_text).context("failed to parse Claude settings JSON")?;
    let commands = collect_claude_hook_commands(&value);
    let mut errors = Vec::new();

    if commands.is_empty() {
        errors.push("Claude settings contain no harness-kit claude-hook commands".to_string());
        return Ok(errors);
    }

    let mut observed_hooks = BTreeSet::new();
    for command in &commands {
        if let Some(hook_name) = claude_hook_name(command) {
            observed_hooks.insert(hook_name.to_string());
        }
        match first_shell_word(command) {
            Some(binary) if command_binary_resolves(&binary) => {}
            Some(binary) => errors.push(format!(
                "Claude hook command is not resolvable: {command:?} (binary {binary:?} is missing or not executable)"
            )),
            None => errors.push(format!(
                "Claude hook command has no parseable binary: {command:?}"
            )),
        }
    }

    for expected in EXPECTED_GENERATED_HOOKS {
        if !observed_hooks.contains(*expected) {
            errors.push(format!("Claude hook command missing: {expected}"));
        }
    }

    Ok(errors)
}

fn rewrite_hook_commands(value: &mut Value, quoted_cli_path: &str) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(command)) = map.get_mut("command")
                && let Some(rest) = command.strip_prefix(BARE_HOOK_PREFIX)
            {
                *command = format!("{quoted_cli_path} claude-hook {rest}");
            }
            for child in map.values_mut() {
                rewrite_hook_commands(child, quoted_cli_path);
            }
        }
        Value::Array(items) => {
            for child in items {
                rewrite_hook_commands(child, quoted_cli_path);
            }
        }
        _ => {}
    }
}

fn collect_claude_hook_commands(value: &Value) -> Vec<String> {
    let mut commands = Vec::new();
    collect_claude_hook_commands_into(value, &mut commands);
    commands
}

fn collect_claude_hook_commands_into(value: &Value, commands: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(Value::String(command)) = map.get("command")
                && claude_hook_name(command).is_some()
            {
                commands.push(command.clone());
            }
            for child in map.values() {
                collect_claude_hook_commands_into(child, commands);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_claude_hook_commands_into(child, commands);
            }
        }
        _ => {}
    }
}

fn claude_hook_name(command: &str) -> Option<&str> {
    command
        .split_once("claude-hook ")
        .and_then(|(_, rest)| rest.split_whitespace().next())
}

fn command_binary_resolves(binary: &str) -> bool {
    let path = Path::new(binary);
    if binary.contains('/') {
        return path.is_absolute() && is_executable_file(path);
    }
    env::var_os("PATH")
        .map(|path_var| {
            env::split_paths(&path_var).any(|dir| is_executable_file(&dir.join(binary)))
        })
        .unwrap_or(false)
}

fn is_executable_file(path: &Path) -> bool {
    let Ok(metadata) = fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn shell_quote(path: &Path) -> String {
    let value = path.to_string_lossy();
    format!("'{}'", value.replace('\'', r#"'\''"#))
}

fn first_shell_word(command: &str) -> Option<String> {
    let mut chars = command.trim_start().chars().peekable();
    chars.peek()?;

    let mut word = String::new();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double => in_single = !in_single,
            '"' if !in_single => in_double = !in_double,
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    word.push(next);
                }
            }
            ch if ch.is_whitespace() && !in_single && !in_double => break,
            ch => word.push(ch),
        }
    }

    if in_single || in_double || word.is_empty() {
        None
    } else {
        Some(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_rewrites_bare_claude_hook_commands_to_installed_cli_path() -> Result<()> {
        let rendered = render_with_cli_path(
            r#"{
              "hooks": {
                "PreToolUse": [
                  {"hooks": [
                    {"command": "harness-kit-checks claude-hook permission-auto-approve"},
                    {"command": "harness-kit-checks claude-hook destructive-command-guard"},
                    {"command": "bash ~/.claude/statusline-command.sh"}
                  ]}
                ],
                "SessionStart": [{"hooks": [
                  {"command": "harness-kit-checks claude-hook time-context"}
                ]}],
                "PostToolUse": [{"hooks": [
                  {"command": "harness-kit-checks claude-hook skill-invocation-tracker"}
                ]}]
              }
            }"#,
            Path::new("/tmp/home with spaces/.harness-kit/bin/harness-kit-checks"),
        )?;

        assert!(!rendered.contains("\"harness-kit-checks claude-hook"));
        assert!(rendered.contains(
            "'/tmp/home with spaces/.harness-kit/bin/harness-kit-checks' claude-hook permission-auto-approve"
        ));
        assert!(rendered.contains("bash ~/.claude/statusline-command.sh"));
        Ok(())
    }

    #[test]
    fn validate_settings_fails_loud_for_missing_hook_binary() -> Result<()> {
        let errors = validate_settings_text(
            r#"{
              "hooks": {"SessionStart": [{"hooks": [
                {"command": "'/tmp/definitely-missing-harness-kit-checks' claude-hook time-context"}
              ]}]}
            }"#,
        )?;

        assert!(
            errors
                .iter()
                .any(|error| error.contains("is missing or not executable"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("permission-auto-approve"))
        );
        Ok(())
    }

    #[test]
    fn first_shell_word_handles_single_quote_escapes() {
        assert_eq!(
            first_shell_word(r#"'/tmp/O'\''Brien/harness-kit-checks' claude-hook time-context"#),
            Some("/tmp/O'Brien/harness-kit-checks".to_string())
        );
    }
}
