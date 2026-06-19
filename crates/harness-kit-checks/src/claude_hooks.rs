use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{Local, SecondsFormat, Utc};
use regex::Regex;
use serde_json::{Map, Value, json};
use wait_timeout::ChildExt;

const SAFE_BASH_COMMANDS: &[&str] = &[
    r"^ls\b",
    r"^cat\b",
    r"^head\b",
    r"^tail\b",
    r"^less\b",
    r"^more\b",
    r"^wc\b",
    r"^file\b",
    r"^stat\b",
    r"^du\b",
    r"^df\b",
    r"^tree\b",
    r"^find\b.*-print",
    r"^find\b.*-name",
    r"^find\b.*-type",
    r"^git\s+(status|log|diff|show|branch|remote|tag|stash\s+list)",
    r"^git\s+ls-",
    r"^git\s+rev-parse",
    r"^git\s+describe",
    r"^git\s+config\s+--get",
    r"^git\s+config\s+-l",
    r"^git\s+config\s+--list",
    r"^git\s+shortlog",
    r"^git\s+blame",
    r"^git\s+annotate",
    r"^git\s+worktree\s+list",
    r"^rg\b",
    r"^ag\b",
    r"^fd\b",
    r"^fzf\b",
    r"^jq\b",
    r"^yq\b",
    r"^bat\b",
    r"^eza?\b",
    r"^ast-grep\b",
    r"^tokei\b",
    r"^cloc\b",
    r"^scc\b",
    r"^npm\s+(list|ls|view|info|outdated|audit)",
    r"^pnpm\s+(list|ls|view|info|outdated|audit)",
    r"^yarn\s+(list|info|outdated|audit)",
    r"^pip\s+(list|show|freeze)",
    r"^cargo\s+(tree|metadata|pkgid)",
    r"^go\s+(list|mod\s+graph)",
    r"^uname\b",
    r"^whoami\b",
    r"^hostname\b",
    r"^pwd\b",
    r"^env\b",
    r"^printenv\b",
    r"^echo\s+\$",
    r"^which\b",
    r"^whereis\b",
    r"^type\b",
    r"^command\s+-v",
    r"^ps\b",
    r"^top\s+-l\s+1",
    r"^uptime\b",
    r"^date\b",
    r"^cal\b",
    r"^gh\s+(repo|issue|pr|release|workflow|run)\s+(view|list|status|diff)",
    r"^gh\s+api\s+.*-X\s+GET",
    r"^gh\s+api\s+[^-]*$",
    r"^gh\s+auth\s+status",
    r"^vercel\s+(list|ls|inspect|logs|env\s+ls)",
    r"^vercel\s+--help",
    r"^npx\s+convex\s+(env\s+list|dashboard|logs)",
];

const NEVER_APPROVE: &[&str] = &[
    r"rm\s",
    r"rmdir\s",
    r"unlink\s",
    r">\s",
    r">>\s",
    r"\|\s*tee\b",
    r"curl.*-[dXP]",
    r"wget\s",
    r"sudo\b",
    r"su\b",
    r"chmod\b",
    r"chown\b",
    r"chgrp\b",
    r"kill\b",
    r"pkill\b",
    r"killall\b",
];

const SAFE_GH_ISSUE_FIELDS: &[&str] = &[
    "title",
    "body",
    "comments",
    "author",
    "state",
    "labels",
    "assignees",
    "milestone",
    "number",
    "url",
    "createdAt",
    "updatedAt",
];

const DESTRUCTIVE_SUBSTRINGS: &[(&str, &str)] = &[
    (
        "git reset --hard",
        "Destroys all uncommitted work. Use 'git stash' first.",
    ),
    (
        "git push --force",
        "Overwrites remote history. Use '--force-with-lease' instead.",
    ),
    (
        "git push -f ",
        "Overwrites remote history. Use '--force-with-lease' instead.",
    ),
    ("git stash drop", "Permanently deletes stashed changes."),
    (
        "git stash clear",
        "Permanently deletes ALL stashed changes.",
    ),
    (
        "gh repo delete",
        "Permanently deletes repository. Extremely destructive.",
    ),
    ("gh issue delete", "Permanently deletes an issue."),
    (
        "gh repo archive",
        "Archives repository, making it read-only.",
    ),
];

const DANGEROUS_FLAGS: &[(&str, &str)] = &[
    (
        "--no-verify",
        "Skips git hooks. Hooks enforce quality gates.",
    ),
    (
        "--no-gpg-sign",
        "Skips commit signing. May violate repo policy.",
    ),
];

const DESTRUCTIVE_SAFE: &[&str] = &[
    "git checkout -b",
    "git checkout --orphan",
    "--force-with-lease",
    "--force-if-includes",
    "git merge --abort",
    "git reset --hard origin/",
];

const HEAVY_COMMANDS: &[&str] = &[
    "npm install",
    "pnpm install",
    "yarn install",
    "brew install",
    "brew upgrade",
    "docker build",
    "docker pull",
    "cargo build",
    "go build",
    "git clone",
    "npx create-",
    "pnpm create",
];

const DISK_WARN_THRESHOLD_GB: f64 = 20.0;
const DISK_BLOCK_THRESHOLD_GB: f64 = 5.0;

const ENV_SETTERS: &[&str] = &[
    "vercel env add",
    "vercel env set",
    "npx convex env set",
    "convex env set",
    "flyctl secrets set",
    "fly secrets set",
    "heroku config:set",
    "railway variables set",
    "netlify env:set",
    "wrangler secret put",
    "doppler secrets set",
    "infisical secrets set",
    "vault kv put",
    "aws ssm put-parameter",
    "gcloud secrets",
    "az keyvault secret set",
];

const TODO_NON_ACTIONABLE_PATTERNS: &[&str] = &[
    r"\bfuture\b",
    r"\bmaybe\b",
    r"\bconsider\b",
    r"\bpossibly\b",
    r"\beventually\b",
    r"\bsomeday\b",
    r"\bshould\s+probably\b",
    r"\bmight\s+want\b",
    r"\bcould\s+be\b",
    r"\bnice\s+to\s+have\b",
];

const EXCUSE_PATTERNS: &[&str] = &[
    r"pre-existing",
    r"not introduced by this (PR|branch|change)",
    r"not from this (PR|branch|change)",
    r"not caused by this (PR|branch|change)",
    r"exists on (master|main)",
    r"already (existed|present|broken) (on|in|before)",
    r"predates this",
    r"unrelated to (this|the) (PR|change)",
    r"outside the scope of this",
];

pub fn run_permission_auto_approve_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = permission_auto_approve(&input) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_time_context() -> Result<()> {
    println!("{}", serde_json::to_string(&time_context_message())?);
    Ok(())
}

pub fn run_disk_space_guard_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    match disk_space_guard(&input, free_space_gb(Path::new("/System/Volumes/Data"))) {
        DiskSpaceDecision::Json(output) => println!("{}", serde_json::to_string(&output)?),
        DiskSpaceDecision::Warning(message) => eprintln!("{message}"),
        DiskSpaceDecision::Silent => {}
    }
    Ok(())
}

pub fn run_destructive_command_guard_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = destructive_command_guard(&input, Path::new(".")) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_github_cli_guard_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = github_cli_guard(&input) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_block_master_push_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = block_master_push(&input, Path::new(".")) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_check_todo_quality_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    println!("{}", serde_json::to_string(&check_todo_quality(&input))?);
    Ok(())
}

pub fn run_codex_post_feedback_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = codex_post_feedback(&input, &delegation_state_file(parent_pid())) {
        for line in output {
            println!("{line}");
        }
    }
    Ok(())
}

pub fn run_codex_session_init() -> Result<()> {
    let output = codex_session_init(
        &delegation_state_file(parent_pid()),
        &default_delegation_config_path(),
        &env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    )?;
    println!("{output}");
    Ok(())
}

pub fn run_env_var_newline_guard_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = env_var_newline_guard(&input) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_exa_research_reminder() -> Result<()> {
    println!("{}", serde_json::to_string(&exa_research_reminder())?);
    Ok(())
}

pub fn run_exclusion_guard_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = exclusion_guard(&input) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_shaping_ripple_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(message) = shaping_ripple(&input) {
        eprint!("{message}");
        std::process::exit(2);
    }
    Ok(())
}

pub fn run_stop_quality_gate_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    match stop_quality_gate(
        &input,
        &env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    ) {
        StopQualityOutcome::AllowSilently => {}
        StopQualityOutcome::Passed { web_project } => {
            if web_project {
                println!("[stop-quality-gate] Web project detected with dev server running.");
                println!("Consider using Chrome MCP to verify UI changes visually.");
            }
            println!("[stop-quality-gate] All quality checks passed");
        }
        StopQualityOutcome::Failed {
            failed_check,
            output,
        } => {
            eprintln!("[stop-quality-gate] {failed_check} FAILED");
            eprintln!("\n{output}");
            eprintln!("\nFix these issues before completing.");
            std::process::exit(2);
        }
    }
    Ok(())
}

pub fn run_fix_what_you_touch_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = fix_what_you_touch(&input) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_portable_code_guard_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = portable_code_guard(&input) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_session_health_check() -> Result<()> {
    let warnings = collect_session_health_warnings();
    println!(
        "{}",
        serde_json::to_string(&session_health_output(warnings))?
    );
    Ok(())
}

pub fn run_skill_invocation_tracker_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    run_skill_invocation_tracker(&input, &default_skill_tracker_log_path());
    Ok(())
}

pub fn permission_auto_approve(input: &str) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    let tool_name = data
        .get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let tool_input = data
        .get("tool_input")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if is_safe_tool(tool_name, &tool_input) {
        Some(json!({
            "hookSpecificOutput": {
                "hookEventName": "PreToolUse",
                "permissionDecision": "allow",
                "permissionDecisionReason": format!("Auto-approved: {tool_name} is read-only"),
            }
        }))
    } else {
        None
    }
}

pub fn time_context_message() -> Value {
    let now = Local::now();
    let friendly_time = now.format("%A, %B %d, %Y at %-I:%M %p %Z").to_string();
    json!({
        "result": "continue",
        "message": format!("Current time: {friendly_time} ({})", now.to_rfc3339()),
    })
}

pub fn destructive_command_guard(input: &str, cwd: &Path) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    if data.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return None;
    }
    let command = data
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|input| input.get("command"))
        .and_then(Value::as_str)?;
    if command.is_empty() {
        return None;
    }
    let reason = destructive_command_reason(command, cwd)?;
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": format!("BLOCKED: {reason}\n\nCommand: {command}\n\nRun this yourself if truly needed."),
        }
    }))
}

#[derive(Debug, PartialEq)]
pub enum DiskSpaceDecision {
    Json(Value),
    Warning(String),
    Silent,
}

pub fn exclusion_guard(input: &str) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    let tool_name = data.get("tool_name").and_then(Value::as_str)?;
    if !matches!(tool_name, "Edit" | "Write" | "MultiEdit") {
        return None;
    }
    let tool_input = data
        .get("tool_input")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    for (file_path, content) in iter_edits(&tool_input) {
        if let Some(pattern_type) = detect_exclusion_pattern(&file_path, &content) {
            return Some(ask_output(&format!(
                "⚠️  Exclusion Pattern Detected: {pattern_type}\n\nBefore excluding, consider:\n□ Can the code be refactored to be testable?\n□ Can handler functions be exported and tested with mocks?\n□ Is this genuinely runtime-only code?\n□ Are there existing patterns in the codebase for testing similar code?\n\nIf exclusion is truly necessary, document WHY in a comment.\n\nProceed with this exclusion?"
            )));
        }
    }
    None
}

pub fn check_todo_quality(input: &str) -> Value {
    let Ok(data) = serde_json::from_str::<Value>(input) else {
        return json!({
            "continue": true,
            "systemMessage": "Hook error (non-blocking): invalid JSON",
        });
    };
    let mut response = json!({
        "continue": true,
        "suppressOutput": true,
    });
    let tool_name = data
        .get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !matches!(tool_name, "Edit" | "Write" | "MultiEdit") {
        return response;
    }
    let tool_input = data
        .get("tool_input")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let file_path = tool_input
        .get("file_path")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !file_path.contains("TODO.md") && !file_path.to_lowercase().contains("todo.md") {
        return response;
    }
    let new_content = match tool_name {
        "Edit" => tool_input
            .get("new_string")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        "Write" => tool_input
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        "MultiEdit" => tool_input
            .get("edits")
            .and_then(Value::as_array)
            .map(|edits| {
                edits
                    .iter()
                    .filter_map(|edit| edit.get("new_string").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .unwrap_or_default(),
        _ => String::new(),
    };
    let found = todo_non_actionable_matches(&new_content);
    if !found.is_empty() {
        response["systemMessage"] = json!(format!(
            "⚠️ TODO Quality Warning: Detected non-actionable language in TODO.md\n\nFound words/phrases: {}\n\nThe Torvalds Test: 'If it's not needed for this PR, it's not a TODO'\n\nTODOs should be:\n• Actionable - Clear steps that can be done now\n• Specific - No ambiguity about what needs doing\n• Current - Needed for active work, not 'someday' items\n\nConsider moving wishful items to BACKLOG.md instead.",
            found.join(", ")
        ));
    }
    response
}

pub fn codex_post_feedback(input: &str, state_file: &Path) -> Option<Vec<String>> {
    let data: Value = serde_json::from_str(input).ok()?;
    let tool_name = data
        .get("tool_name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !matches!(tool_name, "Edit" | "Write" | "MultiEdit" | "NotebookEdit") {
        return None;
    }
    let tool_input = data
        .get("tool_input")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let file_path = tool_input
        .get("file_path")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    let lines = count_edit_lines(&tool_input);
    let Some(state) = load_delegation_state(state_file) else {
        return Some(vec![format!("[codex] Edited {file_path} ({lines} lines)")]);
    };
    let files = state
        .get("files_touched")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let total_lines = state
        .get("total_lines_added")
        .and_then(Value::as_i64)
        .unwrap_or(lines as i64);
    let dirs = state
        .get("directories_touched")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    let new_files = state
        .get("new_files_created")
        .and_then(Value::as_i64)
        .unwrap_or_default();
    let mut stats = format!("{files} files | {total_lines} lines | {dirs} dirs");
    if new_files > 0 {
        stats.push_str(&format!(" | {new_files} new"));
    }
    let mut output = vec![format!("[codex] {file_path} (+{lines}) → Session: {stats}")];
    if total_lines >= 100 || files >= 5 {
        output.push(
            "[codex] 💡 Consider delegating further work to Codex CLI if this grows.".to_string(),
        );
    }
    Some(output)
}

pub fn codex_session_init(state_file: &Path, config_path: &Path, cwd: &Path) -> Result<String> {
    let state = json!({
        "files_touched": [],
        "new_files_created": 0,
        "total_lines_added": 0,
        "directories_touched": [],
    });
    if let Some(parent) = state_file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(state_file, serde_json::to_string_pretty(&state)?)
        .with_context(|| format!("failed to write {}", state_file.display()))?;
    let config = load_delegation_config(config_path);
    if !config
        .get("enabled")
        .and_then(Value::as_bool)
        .unwrap_or(true)
    {
        return Ok("[codex] Delegation enforcement disabled.".to_string());
    }
    if is_excluded_repo(cwd, &config) {
        return Ok("[codex] Excluded repo - delegation not enforced.".to_string());
    }
    Ok(
        "[codex] PATTERN: Codex first draft → You review → Ship. Don't investigate yourself."
            .to_string(),
    )
}

pub fn disk_space_guard(input: &str, free_gb: Option<f64>) -> DiskSpaceDecision {
    let Ok(data) = serde_json::from_str::<Value>(input) else {
        return DiskSpaceDecision::Silent;
    };
    if data.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return DiskSpaceDecision::Silent;
    }
    let Some(command) = data
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|input| input.get("command"))
        .and_then(Value::as_str)
    else {
        return DiskSpaceDecision::Silent;
    };
    if !is_heavy_command(command) {
        return DiskSpaceDecision::Silent;
    }
    let Some(free_gb) = free_gb else {
        return DiskSpaceDecision::Silent;
    };
    if free_gb < DISK_BLOCK_THRESHOLD_GB {
        return DiskSpaceDecision::Json(json!({
            "decision": "block",
            "reason": format!(
                "BLOCKED: Disk critically low ({free_gb:.1}GB free). Run 'cache-clean' alias before heavy operations."
            ),
        }));
    }
    if free_gb < DISK_WARN_THRESHOLD_GB {
        return DiskSpaceDecision::Warning(format!(
            "⚠️  Low disk space ({free_gb:.1}GB free). Consider running 'cache-clean' soon."
        ));
    }
    DiskSpaceDecision::Silent
}

pub fn exa_research_reminder() -> Value {
    json!({
        "decision": "warn",
        "reason": "STOP — WebSearch alone is not research. Use /research (no sub-command) which fans out to Exa, xAI, and delegated critique when relevant. WebSearch is a fallback, not a primary tool. If you're inside /research already, you MUST also launch a second evidence lane such as Exa, xAI, or a delegated critic — a single WebSearch does not satisfy the fanout requirement.",
    })
}

pub fn env_var_newline_guard(input: &str) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    if data.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return None;
    }
    let command = data
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|input| input.get("command"))
        .and_then(Value::as_str)?;
    let reason = env_var_newline_reason(command)?;
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": format!("BLOCKED: {reason}\n\nCommand: {command}"),
        }
    }))
}

pub fn shaping_ripple(input: &str) -> Option<String> {
    let data: Value = serde_json::from_str(input).ok()?;
    let file_path = data
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|input| input.get("file_path"))
        .and_then(Value::as_str)?;
    if !file_path.ends_with(".md") {
        return None;
    }
    let content = fs::read_to_string(file_path).ok()?;
    if !content.lines().take(5).any(|line| line == "shaping: true") {
        return None;
    }
    Some(
        "Ripple check:\n- Updated a Breadboard diagram? → Affordance tables are the source of truth. Update tables FIRST, then render to Mermaid\n- Changed Requirements? → update Fit Check + any Gaps, Open Questions by Part\n- Changed Shape (A, B...) Parts? → update Fit Check + any Gaps, Open Questions by Part\n- Changed Work Streams Detail? → update Work Streams Mermaid\n"
            .to_string(),
    )
}

#[derive(Debug, PartialEq)]
pub enum StopQualityOutcome {
    AllowSilently,
    Passed {
        web_project: bool,
    },
    Failed {
        failed_check: String,
        output: String,
    },
}

pub fn stop_quality_gate(input: &str, fallback_cwd: &Path) -> StopQualityOutcome {
    let cwd = serde_json::from_str::<Value>(input)
        .ok()
        .and_then(|data| data.get("cwd").and_then(Value::as_str).map(PathBuf::from))
        .unwrap_or_else(|| fallback_cwd.to_path_buf());
    let Some(project_type) = detect_project(&cwd) else {
        return StopQualityOutcome::AllowSilently;
    };
    match run_quality_checks(project_type, &cwd) {
        Ok(()) => StopQualityOutcome::Passed {
            web_project: is_running_web_project(&cwd),
        },
        Err((failed_check, output)) => StopQualityOutcome::Failed {
            failed_check,
            output,
        },
    }
}

pub fn portable_code_guard(input: &str) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    let tool_name = data.get("tool_name").and_then(Value::as_str)?;
    let tool_input = data
        .get("tool_input")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    if tool_name == "Bash"
        && let Some((issue, detail)) = portable_git_add_issue(&tool_input)
    {
        return Some(ask_output(&format!(
            "⚠️  Portability Issue: {issue}\n\n{detail}\n\nThis will break for other developers or bloat the repository.\n\nProceed anyway?"
        )));
    }
    if matches!(tool_name, "Edit" | "Write" | "MultiEdit") {
        for (file_path, content) in iter_edits(&tool_input) {
            if let Some((issue, detail)) = portable_content_issue(&file_path, &content) {
                return Some(ask_output(&format!(
                    "⚠️  Portability Issue: {issue}\n\n{detail}\n\nThis will break for other developers or bloat the repository.\n\nProceed anyway?"
                )));
            }
        }
    }
    None
}

pub fn fix_what_you_touch(input: &str) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    let command = data
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|input| input.get("command"))
        .and_then(Value::as_str)?;
    let reason = fix_what_you_touch_reason(command)?;
    Some(json!({
        "decision": "block",
        "reason": reason,
    }))
}

pub fn github_cli_guard(input: &str) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    if data.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return None;
    }
    let tool_input = data.get("tool_input").and_then(Value::as_object)?;
    let command = tool_input.get("command").and_then(Value::as_str)?;
    let transformed = transform_gh_issue_view(command)?;
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "modifiedToolInput": {
                "command": transformed,
                "description": tool_input
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("View GitHub issue"),
            }
        }
    }))
}

pub fn block_master_push(input: &str, cwd: &Path) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    if data.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return None;
    }
    let command = data
        .get("tool_input")
        .and_then(Value::as_object)
        .and_then(|input| input.get("command"))
        .and_then(Value::as_str)?;
    let reason = block_master_push_reason(command, cwd)?;
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": format!(
                "BLOCKED: {reason}\n\nCreate a feature branch and open a PR instead."
            ),
        }
    }))
}

pub fn session_health_output(warnings: Vec<String>) -> Value {
    if warnings.is_empty() {
        json!({})
    } else {
        json!({"message": format!("[codex] ⚠️ SYSTEM HEALTH:\n{}", warnings.join("\n"))})
    }
}

pub fn session_health_warnings(
    disk_pct: Option<u32>,
    swap_gb: Option<f64>,
    orphan_test_processes: usize,
    missing_hooks: Vec<String>,
) -> Vec<String> {
    let mut warnings = Vec::new();
    if disk_pct.is_some_and(|pct| pct >= 90) {
        warnings.push(format!(
            "Disk at {}% - consider running 'cache-clean'",
            disk_pct.unwrap()
        ));
    }
    if swap_gb.is_some_and(|gb| gb >= 15.0) {
        warnings.push(format!(
            "Swap at {:.1}GB - high memory pressure",
            swap_gb.unwrap()
        ));
    }
    if orphan_test_processes > 0 {
        warnings.push(format!(
            "Found {orphan_test_processes} vitest process(es) still running. Run: pkill -f vitest"
        ));
    }
    if !missing_hooks.is_empty() {
        let mut listed = missing_hooks
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");
        if missing_hooks.len() > 4 {
            listed.push_str(&format!(" (+{} more)", missing_hooks.len() - 4));
        }
        warnings.push(format!(
            "settings.json references missing hook scripts: {listed}. Remove stale entries before continuing."
        ));
    }
    warnings
}

fn destructive_command_reason(command: &str, cwd: &Path) -> Option<String> {
    for (flag, reason) in DANGEROUS_FLAGS {
        if command.contains(flag) {
            return Some((*reason).to_string());
        }
    }
    if DESTRUCTIVE_SAFE.iter().any(|safe| command.contains(safe)) {
        return None;
    }
    if Regex::new(r"^git\s+merge\s+\S+").unwrap().is_match(command)
        && current_branch(cwd).is_some_and(|branch| is_protected_branch(&branch))
    {
        let branch = current_branch(cwd).unwrap_or_default();
        return Some(format!(
            "Merging into {branch} is blocked. Create a PR instead."
        ));
    }
    if let Some(captures) = Regex::new(r"git\s+branch\s+-D\s+(.*)")
        .unwrap()
        .captures(command)
    {
        for branch in captures[1].split_whitespace() {
            if is_protected_branch(branch) {
                return Some(format!(
                    "Force-deleting {branch} is blocked. Protected branch."
                ));
            }
        }
    }

    let stripped = strip_quoted_content(command);
    if Regex::new(r"(?m)(^|[;&|`]|\$\()\s*rm\s")
        .unwrap()
        .is_match(&stripped)
    {
        return Some(
            "Use /usr/bin/trash instead. Moves to Trash (recoverable). Example: /usr/bin/trash file.txt"
                .to_string(),
        );
    }
    for (pattern, reason) in DESTRUCTIVE_SUBSTRINGS {
        if stripped.contains(pattern) {
            return Some((*reason).to_string());
        }
    }
    None
}

fn block_master_push_reason(command: &str, cwd: &Path) -> Option<String> {
    if !Regex::new(r"\bgit\s+push\b").unwrap().is_match(command) {
        return None;
    }
    if Regex::new(r"\bgit\s+push\b.*\s(--delete|-d)\s")
        .unwrap()
        .is_match(command)
    {
        return None;
    }
    if Regex::new(r"\bgit\s+push\b[^|;]*\s(master|main)(\s|$)")
        .unwrap()
        .is_match(command)
    {
        let parts = command.split_whitespace().collect::<Vec<_>>();
        if parts.contains(&"master") || parts.contains(&"main") {
            return Some("Direct push to master/main is prohibited.".to_string());
        }
    }

    let git_c = Regex::new(r"\bgit\s+-C\s+(\S+)")
        .unwrap()
        .captures(command)
        .and_then(|captures| captures.get(1).map(|path| PathBuf::from(path.as_str())));
    let branch_cwd = git_c.as_deref().unwrap_or(cwd);
    let branch = current_branch(branch_cwd)?;
    if is_protected_branch(&branch) {
        Some(format!(
            "Current branch is '{branch}' — direct push to master/main is prohibited."
        ))
    } else {
        None
    }
}

fn iter_edits(tool_input: &Map<String, Value>) -> Vec<(String, String)> {
    let mut edits = Vec::new();
    let file_path = tool_input
        .get("file_path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let content = tool_input
        .get("content")
        .or_else(|| tool_input.get("new_string"))
        .and_then(Value::as_str);
    if !file_path.is_empty() || content.is_some() {
        edits.push((file_path.clone(), content.unwrap_or_default().to_string()));
    }
    if let Some(items) = tool_input.get("edits").and_then(Value::as_array) {
        for edit in items {
            let edit_path = edit
                .get("file_path")
                .and_then(Value::as_str)
                .unwrap_or(&file_path)
                .to_string();
            let edit_content = edit
                .get("content")
                .or_else(|| edit.get("new_string"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string();
            edits.push((edit_path, edit_content));
        }
    }
    edits
}

fn ask_output(reason: &str) -> Value {
    json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "ask",
            "permissionDecisionReason": reason,
        }
    })
}

fn detect_exclusion_pattern(file_path: &str, content: &str) -> Option<&'static str> {
    if content.is_empty() {
        return None;
    }
    if Regex::new("(?i)(vitest|jest)\\.config")
        .unwrap()
        .is_match(file_path)
        && Regex::new("(?i)\\bexclude\\s*:").unwrap().is_match(content)
    {
        return Some("Coverage exclusion");
    }
    if Regex::new("(?i)eslint-disable(?:-next-line)?")
        .unwrap()
        .is_match(content)
    {
        return Some("ESLint disable");
    }
    if Regex::new("(?i)@ts-ignore").unwrap().is_match(content) {
        return Some("TypeScript ignore");
    }
    if Regex::new("(?i)@ts-expect-error")
        .unwrap()
        .is_match(content)
    {
        return Some("TypeScript expect-error");
    }
    if Regex::new(r"\bas\s+any\b").unwrap().is_match(content)
        || Regex::new(r":\s*any\b").unwrap().is_match(content)
    {
        return Some("TypeScript any");
    }
    if Regex::new(r"\.skip\s*\(").unwrap().is_match(content)
        || Regex::new(r"\bxit\s*\(").unwrap().is_match(content)
        || Regex::new(r"\bxdescribe\s*\(").unwrap().is_match(content)
    {
        return Some("Test skip");
    }
    None
}

fn todo_non_actionable_matches(content: &str) -> Vec<String> {
    let mut found = Vec::new();
    for pattern in TODO_NON_ACTIONABLE_PATTERNS {
        if let Some(matched) = Regex::new(&format!("(?i){pattern}")).unwrap().find(content) {
            let value = matched.as_str().to_string();
            if !found.contains(&value) {
                found.push(value);
            }
        }
    }
    found
}

fn count_edit_lines(tool_input: &Map<String, Value>) -> usize {
    let text = tool_input
        .get("new_string")
        .or_else(|| tool_input.get("content"))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if text.is_empty() {
        0
    } else {
        text.trim().split('\n').count()
    }
}

fn load_delegation_state(state_file: &Path) -> Option<Value> {
    serde_json::from_str(&fs::read_to_string(state_file).ok()?).ok()
}

fn load_delegation_config(config_path: &Path) -> Value {
    let default = || json!({"enabled": true, "exclusions": {"repositories": [], "patterns": []}});
    let Ok(contents) = fs::read_to_string(config_path) else {
        return default();
    };
    serde_json::from_str(&contents).unwrap_or_else(|_| default())
}

fn is_excluded_repo(cwd: &Path, config: &Value) -> bool {
    let cwd = cwd.to_string_lossy();
    let exclusions = config.get("exclusions").and_then(Value::as_object);
    if let Some(repositories) = exclusions
        .and_then(|exclusions| exclusions.get("repositories"))
        .and_then(Value::as_array)
        && repositories
            .iter()
            .filter_map(Value::as_str)
            .any(|repo| cwd.starts_with(repo))
    {
        return true;
    }
    if let Some(patterns) = exclusions
        .and_then(|exclusions| exclusions.get("patterns"))
        .and_then(Value::as_array)
        && patterns
            .iter()
            .filter_map(Value::as_str)
            .any(|pattern| wildcard_match(pattern, &cwd))
    {
        return true;
    }
    false
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let mut regex = String::from("^");
    for ch in pattern.chars() {
        match ch {
            '*' => regex.push_str(".*"),
            '?' => regex.push('.'),
            _ => regex.push_str(&regex::escape(&ch.to_string())),
        }
    }
    regex.push('$');
    Regex::new(&regex).is_ok_and(|regex| regex.is_match(value))
}

fn env_var_newline_reason(command: &str) -> Option<String> {
    if command.is_empty() {
        return None;
    }
    let captures = Regex::new(r"\becho\s+([^|]*)\|")
        .unwrap()
        .captures(command)?;
    let echo_args = captures
        .get(1)
        .map(|matched| matched.as_str().trim())
        .unwrap_or_default();
    if echo_args.starts_with("-n ")
        || echo_args == "-n"
        || echo_args.starts_with("-en ")
        || echo_args == "-en"
    {
        return None;
    }
    let command_lower = command.to_lowercase();
    for setter in ENV_SETTERS {
        if command_lower.contains(&setter.to_lowercase()) {
            let setter_cmd = setter.split_whitespace().next().unwrap_or(setter);
            return Some(format!(
                "`echo` adds a trailing newline that corrupts env vars.\n\nUse printf instead:\n  printf '%s' \"value\" | {setter_cmd} ...\n\nOr echo -n (bash-specific):\n  echo -n \"value\" | {setter_cmd} ..."
            ));
        }
    }
    None
}

fn portable_git_add_issue(tool_input: &Map<String, Value>) -> Option<(&'static str, &'static str)> {
    let command = tool_input
        .get("command")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if (command.contains("git add") || command.contains("git stage"))
        && Regex::new(r"packages/[^/]+/node_modules")
            .unwrap()
            .is_match(command)
    {
        Some((
            "Workspace node_modules in git",
            "Attempting to add workspace package node_modules to git.\nThese should be in .gitignore and installed via pnpm.",
        ))
    } else {
        None
    }
}

fn portable_content_issue(file_path: &str, content: &str) -> Option<(&'static str, String)> {
    if content.is_empty() || is_allowed_path_file(file_path) {
        return None;
    }
    if !is_shell_or_config(file_path) {
        return None;
    }
    if let Some(matched) = Regex::new(r"/Users/[a-zA-Z0-9_-]+/").unwrap().find(content) {
        return Some((
            "Hardcoded Home Path",
            format!(
                "Found machine-specific path: {}...\nOther developers have different home directories.",
                matched.as_str()
            ),
        ));
    }
    if let Some(matched) = Regex::new(r"C:\\Users\\[a-zA-Z0-9_-]+\\")
        .unwrap()
        .find(content)
    {
        return Some((
            "Hardcoded Windows Path",
            format!(
                "Found machine-specific path: {}...\nThis won't work on other machines.",
                matched.as_str()
            ),
        ));
    }
    None
}

fn is_shell_or_config(file_path: &str) -> bool {
    let suffix = Path::new(file_path)
        .extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| format!(".{extension}"))
        .unwrap_or_default();
    matches!(suffix.as_str(), ".sh" | ".bash" | ".zsh" | "")
        || ["lefthook", "husky", ".gitconfig", ".env"]
            .iter()
            .any(|needle| file_path.to_lowercase().contains(needle))
}

fn is_allowed_path_file(file_path: &str) -> bool {
    [".claude/hooks", "coverage/", ".next/", "dist/"]
        .iter()
        .any(|allowed| file_path.contains(allowed))
}

fn fix_what_you_touch_reason(command: &str) -> Option<String> {
    if !command.contains("gh pr comment") && !command.contains("gh api") {
        return None;
    }
    if command.contains("--body-file") || command.contains("--body-file=") {
        return None;
    }
    if command.contains("gh api") && !command.contains("/comments") {
        return None;
    }
    let combined = Regex::new(&format!("(?i){}", EXCUSE_PATTERNS.join("|"))).unwrap();
    if combined.is_match(command) {
        return Some(
            "BLOCKED: You diagnosed a broken thing and are excusing it instead of fixing it. Origin is irrelevant — fix what you touch. Fix the problem first, then comment about the fix."
                .to_string(),
        );
    }
    if Regex::new("(?i)not (a )?block(er|ing)")
        .unwrap()
        .is_match(command)
        && Regex::new("(?i)fail(ure|ing|ed)")
            .unwrap()
            .is_match(command)
        && !Regex::new(r"(#\d+|github\.com/.+/issues/\d+)")
            .unwrap()
            .is_match(command)
    {
        return Some(
            "BLOCKED: A failing check IS a blocker. Fix it, or file a tracking issue and link it (e.g., 'tracked in #123')."
                .to_string(),
        );
    }
    None
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ProjectType {
    Node,
    Python,
    Rust,
    Go,
}

fn detect_project(cwd: &Path) -> Option<ProjectType> {
    if cwd.join("package.json").exists() {
        Some(ProjectType::Node)
    } else if cwd.join("pyproject.toml").exists() {
        Some(ProjectType::Python)
    } else if cwd.join("Cargo.toml").exists() {
        Some(ProjectType::Rust)
    } else if cwd.join("go.mod").exists() {
        Some(ProjectType::Go)
    } else {
        None
    }
}

fn quality_checks_for_project(project_type: ProjectType) -> Vec<(&'static str, Vec<&'static str>)> {
    match project_type {
        ProjectType::Node => vec![
            ("Type check", vec!["pnpm", "tsc", "--noEmit"]),
            ("Lint", vec!["pnpm", "lint"]),
            ("Test", vec!["pnpm", "test"]),
        ],
        ProjectType::Python => vec![
            ("Type check", vec!["pyright"]),
            ("Lint", vec!["ruff", "check", "."]),
            ("Test", vec!["pytest", "-x", "--tb=short"]),
        ],
        ProjectType::Rust => vec![
            ("Check", vec!["cargo", "check"]),
            ("Clippy", vec!["cargo", "clippy", "--", "-D", "warnings"]),
            ("Test", vec!["cargo", "test"]),
        ],
        ProjectType::Go => vec![
            ("Vet", vec!["go", "vet", "./..."]),
            ("Test", vec!["go", "test", "-v", "./..."]),
        ],
    }
}

fn run_quality_checks(
    project_type: ProjectType,
    cwd: &Path,
) -> std::result::Result<(), (String, String)> {
    for (name, command) in quality_checks_for_project(project_type) {
        if !has_command(command[0]) {
            continue;
        }
        let mut child = Command::new(command[0])
            .args(&command[1..])
            .current_dir(cwd)
            .env("CI", "true")
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|error| (name.to_string(), error.to_string()))?;
        match child
            .wait_timeout(Duration::from_secs(120))
            .map_err(|error| (name.to_string(), error.to_string()))?
        {
            Some(status) if status.success() => {}
            Some(_) => {
                let output = child
                    .wait_with_output()
                    .map_err(|error| (name.to_string(), error.to_string()))?;
                let combined = format!(
                    "{}{}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                return Err((name.to_string(), combined.trim().to_string()));
            }
            None => {
                let _ = child.kill();
                let _ = child.wait();
                return Err((name.to_string(), format!("{name} timed out after 120s")));
            }
        }
    }
    Ok(())
}

fn has_command(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .is_ok_and(|output| output.status.success())
}

fn is_running_web_project(cwd: &Path) -> bool {
    if !cwd.join("package.json").exists() {
        return false;
    }
    if !["next.config.js", "next.config.ts", "next.config.mjs"]
        .iter()
        .any(|file| cwd.join(file).exists())
    {
        return false;
    }
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], 3000)),
        Duration::from_secs(2),
    )
    .is_ok()
}

fn strip_quoted_content(command: &str) -> String {
    let mut result = String::new();
    let mut chars = command.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(ch) = chars.next() {
        if ch == '\\' && chars.peek().is_some() {
            if !in_single && !in_double {
                result.push(ch);
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            } else {
                let _ = chars.next();
            }
            continue;
        }
        if ch == '"' && !in_single {
            in_double = !in_double;
            result.push(ch);
        } else if ch == '\'' && !in_double {
            in_single = !in_single;
            result.push(ch);
        } else if !in_single && !in_double {
            result.push(ch);
        }
    }
    result
}

fn is_heavy_command(command: &str) -> bool {
    let command = command.to_lowercase();
    HEAVY_COMMANDS.iter().any(|heavy| command.contains(heavy))
}

fn free_space_gb(path: &Path) -> Option<f64> {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;

    let c_path = CString::new(path.as_os_str().as_bytes()).ok()?;
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    // SAFETY: c_path is a valid NUL-terminated path and stat points to writable memory.
    let result = unsafe { libc::statvfs(c_path.as_ptr(), stat.as_mut_ptr()) };
    if result != 0 {
        return None;
    }
    // SAFETY: statvfs returned success, so stat has been initialized.
    let stat = unsafe { stat.assume_init() };
    let free_bytes = stat.f_bavail as f64 * stat.f_frsize as f64;
    Some(free_bytes / 1024_f64.powi(3))
}

fn collect_session_health_warnings() -> Vec<String> {
    session_health_warnings(
        disk_percent(),
        swap_gb(),
        count_orphan_test_processes(),
        find_missing_hook_targets(&default_claude_settings_path()),
    )
}

fn disk_percent() -> Option<u32> {
    let output = Command::new("df")
        .args(["-h", "/System/Volumes/Data"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines().skip(1) {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() >= 5 {
            return parts[4].trim_end_matches('%').parse().ok();
        }
    }
    None
}

fn swap_gb() -> Option<f64> {
    let output = Command::new("sysctl").arg("vm.swapusage").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let used = stdout.split("used = ").nth(1)?.split_whitespace().next()?;
    Some(used.trim_end_matches('M').parse::<f64>().ok()? / 1024.0)
}

fn count_orphan_test_processes() -> usize {
    let output = Command::new("pgrep").args(["-lf", "vitest"]).output();
    let Ok(output) = output else {
        return 0;
    };
    if !output.status.success() {
        return 0;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| {
            let lower = line.to_lowercase();
            lower.contains("vitest") && !lower.contains("pgrep")
        })
        .count()
}

fn find_missing_hook_targets(settings_path: &Path) -> Vec<String> {
    let Ok(contents) = fs::read_to_string(settings_path) else {
        return Vec::new();
    };
    let Ok(settings) = serde_json::from_str::<Value>(&contents) else {
        return Vec::new();
    };
    let mut missing = Vec::new();
    let Some(hooks) = settings.get("hooks").and_then(Value::as_object) else {
        return Vec::new();
    };
    for groups in hooks.values().filter_map(Value::as_array) {
        for group in groups {
            let Some(hooks) = group.get("hooks").and_then(Value::as_array) else {
                continue;
            };
            for hook in hooks {
                let command = hook
                    .get("command")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                for token in command.split_whitespace() {
                    if let Some(rest) = token.strip_prefix("~/.claude/hooks/") {
                        let target = default_home_dir().join(".claude/hooks").join(rest);
                        if !target.exists() {
                            missing.push(rest.to_string());
                        }
                        break;
                    }
                }
            }
        }
    }
    missing.sort();
    missing.dedup();
    missing
}

fn default_claude_settings_path() -> PathBuf {
    default_home_dir().join(".claude/settings.json")
}

fn default_delegation_config_path() -> PathBuf {
    default_home_dir().join(".claude/config/delegation-enforcement.json")
}

fn default_home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn delegation_state_file(ppid: u32) -> PathBuf {
    PathBuf::from(format!("/tmp/claude-delegation-{ppid}.json"))
}

fn parent_pid() -> u32 {
    // SAFETY: getppid takes no pointers and has no preconditions.
    unsafe { libc::getppid() as u32 }
}

fn current_branch(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(cwd)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn is_protected_branch(branch: &str) -> bool {
    matches!(branch, "main" | "master")
}

fn transform_gh_issue_view(command: &str) -> Option<String> {
    let command = command.trim();
    let captures = Regex::new(r"^gh\s+issue\s+view\s+(\d+|[A-Za-z0-9_/-]+#\d+)(.*)$")
        .unwrap()
        .captures(command)?;
    let issue_ref = captures.get(1)?.as_str();
    let flags = captures.get(2).map(|m| m.as_str().trim()).unwrap_or("");
    if flags.contains("--json") || flags.contains("--web") || flags.contains("-w") {
        return None;
    }
    let remaining = flags.replace("--comments", "").trim().to_string();
    let fields = SAFE_GH_ISSUE_FIELDS.join(",");
    if remaining.is_empty() {
        Some(format!("gh issue view {issue_ref} --json {fields}"))
    } else {
        Some(format!(
            "gh issue view {issue_ref} {remaining} --json {fields}"
        ))
    }
}

pub fn run_skill_invocation_tracker(input: &str, log_path: &Path) {
    let Ok(data) = serde_json::from_str::<Value>(input) else {
        return;
    };
    let Some(entry) = build_skill_invocation_entry(&data) else {
        return;
    };
    let _ = append_jsonl(log_path, &entry);
}

fn is_safe_tool(tool_name: &str, tool_input: &Map<String, Value>) -> bool {
    match tool_name {
        "Read" | "Glob" | "Grep" | "LS" | "WebFetch" | "WebSearch" => true,
        "Bash" => tool_input
            .get("command")
            .and_then(Value::as_str)
            .is_some_and(is_safe_bash),
        "Task" => tool_input
            .get("subagent_type")
            .and_then(Value::as_str)
            .is_some_and(|subagent| matches!(subagent, "Explore" | "Plan")),
        _ => false,
    }
}

fn is_safe_bash(command: &str) -> bool {
    for pattern in NEVER_APPROVE {
        if Regex::new(&format!("(?i){pattern}"))
            .expect("never approve regex compiles")
            .is_match(command)
        {
            return false;
        }
    }
    let trimmed = command.trim();
    SAFE_BASH_COMMANDS.iter().any(|pattern| {
        Regex::new(&format!("(?i){pattern}"))
            .expect("safe bash regex compiles")
            .is_match(trimmed)
    })
}

pub fn build_skill_invocation_entry(data: &Value) -> Option<Value> {
    if data.get("tool_name").and_then(Value::as_str) != Some("Skill") {
        return None;
    }
    let tool_input = data.get("tool_input").and_then(Value::as_object)?;
    let skill = tool_input.get("skill").and_then(Value::as_str)?;
    if skill.is_empty() {
        return None;
    }

    let cwd = data.get("cwd").and_then(Value::as_str).unwrap_or_default();
    let project = Path::new(cwd)
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_default();

    let mut entry = Map::new();
    entry.insert(
        "schema_version".to_string(),
        data.get("schema_version")
            .cloned()
            .unwrap_or_else(|| json!(2)),
    );
    entry.insert(
        "event_type".to_string(),
        data.get("event_type")
            .cloned()
            .unwrap_or_else(|| json!("skill_invocation")),
    );
    entry.insert(
        "ts".to_string(),
        json!(Utc::now().to_rfc3339_opts(SecondsFormat::Micros, true)),
    );
    entry.insert(
        "harness".to_string(),
        data.get("harness")
            .cloned()
            .unwrap_or_else(|| json!("claude")),
    );
    entry.insert(
        "source_protocol".to_string(),
        data.get("source_protocol")
            .cloned()
            .unwrap_or_else(|| json!("post_tool_use")),
    );
    entry.insert("skill".to_string(), json!(skill));
    entry.insert(
        "args".to_string(),
        tool_input.get("args").cloned().unwrap_or_else(|| json!("")),
    );
    entry.insert(
        "session_id".to_string(),
        data.get("session_id").cloned().unwrap_or_else(|| json!("")),
    );
    entry.insert("cwd".to_string(), json!(cwd));
    entry.insert("project".to_string(), json!(project));

    for field in ["model_id", "outcome", "duration_ms", "usage"] {
        if let Some(value) = data.get(field) {
            entry.insert(field.to_string(), value.clone());
        }
    }

    Some(Value::Object(entry))
}

fn default_skill_tracker_log_path() -> PathBuf {
    if let Some(path) = env::var_os("SKILL_TRACKER_LOG_PATH") {
        return PathBuf::from(path);
    }
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".claude/skill-invocations.jsonl")
}

fn append_jsonl(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(value)?)
        .with_context(|| format!("failed to append {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn read_rows(path: &Path) -> Vec<Value> {
        fs::read_to_string(path)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn skill_invocation_appends_jsonl_entry() {
        let temp = TempDir::new().unwrap();
        let log = temp.path().join("skill-invocations.jsonl");
        run_skill_invocation_tracker(
            r#"{"tool_name":"Skill","tool_input":{"skill":"commit","args":"-m fix"},"session_id":"abc","cwd":"/tmp/myproject"}"#,
            &log,
        );
        let rows = read_rows(&log);
        assert_eq!(rows.len(), 1);
        let entry = &rows[0];
        assert_eq!(entry["skill"], "commit");
        assert_eq!(entry["args"], "-m fix");
        assert_eq!(entry["session_id"], "abc");
        assert_eq!(entry["cwd"], "/tmp/myproject");
        assert_eq!(entry["project"], "myproject");
        assert_eq!(entry["harness"], "claude");
        assert_eq!(entry["schema_version"], 2);
        assert_eq!(entry["event_type"], "skill_invocation");
        assert_eq!(entry["source_protocol"], "post_tool_use");
        assert!(entry.get("ts").is_some());
    }

    #[test]
    fn optional_usage_fields_pass_through_when_available() {
        let temp = TempDir::new().unwrap();
        let log = temp.path().join("skill-invocations.jsonl");
        run_skill_invocation_tracker(
            r#"{"tool_name":"Skill","tool_input":{"skill":"qa","args":""},"session_id":"abc","cwd":"/tmp/myproject","model_id":"claude-opus-4-8","outcome":"succeeded","duration_ms":1200,"usage":{"input_tokens":10,"output_tokens":5,"total_tokens":15,"cost_usd":0.001,"cost_source":"provider_reported"}}"#,
            &log,
        );
        let entry = read_rows(&log).remove(0);
        assert_eq!(entry["model_id"], "claude-opus-4-8");
        assert_eq!(entry["outcome"], "succeeded");
        assert_eq!(entry["duration_ms"], 1200);
        assert_eq!(entry["usage"]["total_tokens"], 15);
    }

    #[test]
    fn non_skill_invalid_and_empty_skill_inputs_are_ignored() {
        let temp = TempDir::new().unwrap();
        let log = temp.path().join("skill-invocations.jsonl");
        run_skill_invocation_tracker(
            r#"{"tool_name":"Bash","tool_input":{"command":"ls"}}"#,
            &log,
        );
        run_skill_invocation_tracker("", &log);
        run_skill_invocation_tracker("not json at all", &log);
        run_skill_invocation_tracker(
            r#"{"tool_name":"Skill","tool_input":{"skill":"","args":""},"session_id":"abc","cwd":"/tmp/myproject"}"#,
            &log,
        );
        assert!(!log.exists());
    }

    #[test]
    fn multiple_invocations_append() {
        let temp = TempDir::new().unwrap();
        let log = temp.path().join("skill-invocations.jsonl");
        for skill in ["commit", "review", "investigate"] {
            run_skill_invocation_tracker(
                &format!(
                    r#"{{"tool_name":"Skill","tool_input":{{"skill":"{skill}","args":""}},"session_id":"sess1","cwd":"/tmp/proj"}}"#
                ),
                &log,
            );
        }
        let skills: Vec<String> = read_rows(&log)
            .into_iter()
            .map(|row| row["skill"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(skills, ["commit", "review", "investigate"]);
    }

    #[test]
    fn permission_auto_approve_allows_read_only_tools() {
        let output = permission_auto_approve(
            r#"{"tool_name":"Read","tool_input":{"file_path":"README.md"}}"#,
        )
        .unwrap();
        assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "allow");
        assert_eq!(output["hookSpecificOutput"]["hookEventName"], "PreToolUse");
    }

    #[test]
    fn permission_auto_approve_allows_safe_bash_and_blocks_mutating_bash() {
        let safe = permission_auto_approve(
            r#"{"tool_name":"Bash","tool_input":{"command":"git status --short"}}"#,
        )
        .unwrap();
        assert_eq!(safe["hookSpecificOutput"]["permissionDecision"], "allow");
        assert!(
            permission_auto_approve(
                r#"{"tool_name":"Bash","tool_input":{"command":"cat README.md > out.txt"}}"#
            )
            .is_none()
        );
        assert!(
            permission_auto_approve(
                r#"{"tool_name":"Bash","tool_input":{"command":"rm README.md"}}"#
            )
            .is_none()
        );
    }

    #[test]
    fn permission_auto_approve_handles_invalid_json_silently() {
        assert!(permission_auto_approve("not json").is_none());
    }

    #[test]
    fn permission_auto_approve_allows_explore_and_plan_tasks_only() {
        assert!(
            permission_auto_approve(
                r#"{"tool_name":"Task","tool_input":{"subagent_type":"Explore"}}"#
            )
            .is_some()
        );
        assert!(
            permission_auto_approve(
                r#"{"tool_name":"Task","tool_input":{"subagent_type":"Build"}}"#
            )
            .is_none()
        );
    }

    #[test]
    fn time_context_message_has_continue_result_and_current_time() {
        let output = time_context_message();
        assert_eq!(output["result"], "continue");
        assert!(
            output["message"]
                .as_str()
                .unwrap()
                .contains("Current time:")
        );
    }

    #[test]
    fn destructive_guard_blocks_reset_rm_and_dangerous_flags() {
        let temp = TempDir::new().unwrap();
        for command in ["git reset --hard", "rm README.md", "git commit --no-verify"] {
            let output = destructive_command_guard(
                &format!(r#"{{"tool_name":"Bash","tool_input":{{"command":"{command}"}}}}"#),
                temp.path(),
            )
            .unwrap();
            assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "deny");
        }
    }

    #[test]
    fn destructive_guard_ignores_rm_inside_quotes_and_allows_safe_force_with_lease() {
        let temp = TempDir::new().unwrap();
        assert!(
            destructive_command_guard(
                r#"{"tool_name":"Bash","tool_input":{"command":"git commit -m \"rm all files\"}}"#,
                temp.path(),
            )
            .is_none()
        );
        assert!(
            destructive_command_guard(
                r#"{"tool_name":"Bash","tool_input":{"command":"git push --force-with-lease"}}"#,
                temp.path(),
            )
            .is_none()
        );
    }

    #[test]
    fn destructive_guard_blocks_protected_branch_delete() {
        let temp = TempDir::new().unwrap();
        let output = destructive_command_guard(
            r#"{"tool_name":"Bash","tool_input":{"command":"git branch -D feature main"}}"#,
            temp.path(),
        )
        .unwrap();
        assert!(
            output["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .contains("Force-deleting main")
        );
    }

    #[test]
    fn disk_space_guard_blocks_warns_and_ignores_like_python_hook() {
        let input = r#"{"tool_name":"Bash","tool_input":{"command":"cargo build --workspace"}}"#;
        let blocked = disk_space_guard(input, Some(4.2));
        let DiskSpaceDecision::Json(output) = blocked else {
            panic!("expected JSON block decision");
        };
        assert_eq!(output["decision"], "block");
        assert!(output["reason"].as_str().unwrap().contains("4.2GB free"));

        let warned = disk_space_guard(input, Some(12.0));
        let DiskSpaceDecision::Warning(message) = warned else {
            panic!("expected warning decision");
        };
        assert!(message.contains("12.0GB free"));

        assert_eq!(
            disk_space_guard(
                r#"{"tool_name":"Bash","tool_input":{"command":"git status --short"}}"#,
                Some(1.0),
            ),
            DiskSpaceDecision::Silent
        );
        assert_eq!(
            disk_space_guard(
                r#"{"tool_name":"Read","tool_input":{"file_path":"README.md"}}"#,
                Some(1.0)
            ),
            DiskSpaceDecision::Silent
        );
        assert_eq!(
            disk_space_guard("not json", Some(1.0)),
            DiskSpaceDecision::Silent
        );
        assert_eq!(disk_space_guard(input, None), DiskSpaceDecision::Silent);
    }

    #[test]
    fn block_master_push_denies_explicit_and_current_branch_pushes() {
        let temp = TempDir::new().unwrap();
        let explicit = block_master_push(
            r#"{"tool_name":"Bash","tool_input":{"command":"git push origin main"}}"#,
            temp.path(),
        )
        .unwrap();
        assert_eq!(
            explicit["hookSpecificOutput"]["permissionDecisionReason"],
            "BLOCKED: Direct push to master/main is prohibited.\n\nCreate a feature branch and open a PR instead."
        );

        Command::new("git")
            .args(["init", "-b", "main"])
            .current_dir(temp.path())
            .output()
            .unwrap();
        let current = block_master_push(
            r#"{"tool_name":"Bash","tool_input":{"command":"git push"}}"#,
            temp.path(),
        )
        .unwrap();
        assert!(
            current["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .contains("Current branch is 'main'")
        );
    }

    #[test]
    fn block_master_push_allows_deletes_feature_names_and_non_bash() {
        let temp = TempDir::new().unwrap();
        assert!(
            block_master_push(
                r#"{"tool_name":"Bash","tool_input":{"command":"git push origin --delete main"}}"#,
                temp.path(),
            )
            .is_none()
        );
        assert!(
            block_master_push(
                r#"{"tool_name":"Bash","tool_input":{"command":"git push origin cx/hotfix-master-compile"}}"#,
                temp.path(),
            )
            .is_none()
        );
        assert!(
            block_master_push(
                r#"{"tool_name":"Read","tool_input":{"file_path":"README.md"}}"#,
                temp.path(),
            )
            .is_none()
        );
        assert!(block_master_push("not json", temp.path()).is_none());
    }

    #[test]
    fn session_health_formats_warnings_and_missing_hooks_like_python_hook() {
        let warnings = session_health_warnings(
            Some(97),
            Some(15.25),
            2,
            vec![
                "a.py".to_string(),
                "b.py".to_string(),
                "c.py".to_string(),
                "d.py".to_string(),
                "e.py".to_string(),
            ],
        );
        assert_eq!(warnings.len(), 4);
        assert_eq!(warnings[0], "Disk at 97% - consider running 'cache-clean'");
        assert_eq!(warnings[1], "Swap at 15.2GB - high memory pressure");
        assert_eq!(
            warnings[2],
            "Found 2 vitest process(es) still running. Run: pkill -f vitest"
        );
        assert!(warnings[3].contains("a.py, b.py, c.py, d.py (+1 more)"));

        let output = session_health_output(warnings);
        assert!(
            output["message"]
                .as_str()
                .unwrap()
                .starts_with("[codex] ⚠️ SYSTEM HEALTH:\nDisk at 97%")
        );
        assert_eq!(session_health_output(Vec::new()), json!({}));
    }

    #[test]
    fn missing_hook_targets_are_sorted_unique_and_ignore_rust_commands() {
        let temp = TempDir::new().unwrap();
        let settings = temp.path().join("settings.json");
        fs::write(
            &settings,
            r#"{
              "hooks": {
                "PreToolUse": [
                  {"hooks": [
                    {"command": "python3 ~/.claude/hooks/missing.py"},
                    {"command": "python3 ~/.claude/hooks/missing.py"},
                    {"command": "harness-kit-checks claude-hook time-context"},
                    {"command": "bash ~/.claude/hooks/also-missing.sh"}
                  ]}
                ]
              }
            }"#,
        )
        .unwrap();
        assert_eq!(
            find_missing_hook_targets(&settings),
            vec!["also-missing.sh".to_string(), "missing.py".to_string()]
        );
    }

    #[test]
    fn exclusion_guard_asks_for_exclusions_any_and_skips() {
        for (input, expected) in [
            (
                r#"{"tool_name":"Write","tool_input":{"file_path":"vitest.config.ts","content":"export default { coverage: { exclude: ['src/x.ts'] } }"}}"#,
                "Coverage exclusion",
            ),
            (
                r#"{"tool_name":"Edit","tool_input":{"file_path":"src/a.ts","new_string":"// eslint-disable-next-line\nconst x = 1"}}"#,
                "ESLint disable",
            ),
            (
                r#"{"tool_name":"Edit","tool_input":{"file_path":"src/a.ts","new_string":"const x = value as any"}}"#,
                "TypeScript any",
            ),
            (
                r#"{"tool_name":"MultiEdit","tool_input":{"file_path":"test.spec.ts","edits":[{"new_string":"describe.skip('x', () => {})"}]}}"#,
                "Test skip",
            ),
        ] {
            let output = exclusion_guard(input).unwrap();
            assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "ask");
            assert!(
                output["hookSpecificOutput"]["permissionDecisionReason"]
                    .as_str()
                    .unwrap()
                    .contains(expected)
            );
        }
        assert!(exclusion_guard("not json").is_none());
        assert!(exclusion_guard(r#"{"tool_name":"Read","tool_input":{}}"#).is_none());
    }

    #[test]
    fn check_todo_quality_warns_non_actionable_todo_language() {
        let output = check_todo_quality(
            r#"{"tool_name":"Write","tool_input":{"file_path":"TODO.md","content":"Maybe consider this someday."}}"#,
        );
        assert_eq!(output["continue"], true);
        assert_eq!(output["suppressOutput"], true);
        let message = output["systemMessage"].as_str().unwrap();
        assert!(message.contains("TODO Quality Warning"));
        assert!(message.contains("Maybe"));
        assert!(message.contains("consider"));
        assert!(message.contains("someday"));

        let clean = check_todo_quality(
            r#"{"tool_name":"Write","tool_input":{"file_path":"TODO.md","content":"- [ ] Run cargo test"}}"#,
        );
        assert!(clean.get("systemMessage").is_none());
        assert_eq!(
            check_todo_quality("not json")["systemMessage"],
            "Hook error (non-blocking): invalid JSON"
        );
    }

    #[test]
    fn env_var_newline_guard_blocks_echo_into_env_setter_only() {
        let output = env_var_newline_guard(
            r#"{"tool_name":"Bash","tool_input":{"command":"echo \"secret\" | vercel env add API_KEY production"}}"#,
        )
        .unwrap();
        assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "deny");
        assert!(
            output["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .contains("printf '%s'")
        );

        assert!(
            env_var_newline_guard(
                r#"{"tool_name":"Bash","tool_input":{"command":"echo -n \"secret\" | vercel env add API_KEY production"}}"#
            )
            .is_none()
        );
        assert!(
            env_var_newline_guard(
                r#"{"tool_name":"Bash","tool_input":{"command":"echo \"secret\" | cat"}}"#
            )
            .is_none()
        );
        assert!(env_var_newline_guard("not json").is_none());
    }

    #[test]
    fn portable_code_guard_asks_for_home_paths_and_node_modules() {
        let home = portable_code_guard(
            r#"{"tool_name":"Write","tool_input":{"file_path":"scripts/run.sh","content":"cd /Users/alice/project"}}"#,
        )
        .unwrap();
        assert_eq!(home["hookSpecificOutput"]["permissionDecision"], "ask");
        assert!(
            home["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .contains("Hardcoded Home Path")
        );

        let node_modules = portable_code_guard(
            r#"{"tool_name":"Bash","tool_input":{"command":"git add packages/app/node_modules/lib/index.js"}}"#,
        )
        .unwrap();
        assert!(
            node_modules["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .contains("Workspace node_modules in git")
        );

        assert!(
            portable_code_guard(
                r#"{"tool_name":"Write","tool_input":{"file_path":".claude/hooks/x.sh","content":"cd /Users/alice/project"}}"#
            )
            .is_none()
        );
        assert!(
            portable_code_guard(
                r#"{"tool_name":"Write","tool_input":{"file_path":"src/app.ts","content":"const path = '/Users/alice/project'"}}"#
            )
            .is_none()
        );
    }

    #[test]
    fn fix_what_you_touch_blocks_excuses_and_untracked_not_blocker_comments() {
        let excuse = fix_what_you_touch(
            r#"{"tool_name":"Bash","tool_input":{"command":"gh pr comment 1 --body 'This is pre-existing and not introduced by this PR'"}}"#,
        )
        .unwrap();
        assert_eq!(excuse["decision"], "block");
        assert!(
            excuse["reason"]
                .as_str()
                .unwrap()
                .contains("Origin is irrelevant")
        );

        let not_blocker = fix_what_you_touch(
            r#"{"tool_name":"Bash","tool_input":{"command":"gh api repos/o/r/issues/1/comments -f body='Test failed but not a blocker'"}}"#,
        )
        .unwrap();
        assert!(
            not_blocker["reason"]
                .as_str()
                .unwrap()
                .contains("A failing check IS a blocker")
        );

        assert!(
            fix_what_you_touch(
                r#"{"tool_name":"Bash","tool_input":{"command":"gh pr comment 1 --body-file /tmp/body.md"}}"#
            )
            .is_none()
        );
        assert!(
            fix_what_you_touch(
                r#"{"tool_name":"Bash","tool_input":{"command":"gh pr comment 1 --body 'Test failed, tracked in #123'"}}"#
            )
            .is_none()
        );
        assert!(fix_what_you_touch("not json").is_none());
    }

    #[test]
    fn codex_post_feedback_reports_first_edit_and_session_stats() {
        let temp = TempDir::new().unwrap();
        let state = temp.path().join("state.json");
        let first = codex_post_feedback(
            r#"{"tool_name":"Write","tool_input":{"file_path":"src/lib.rs","content":"a\nb\n"}}"#,
            &state,
        )
        .unwrap();
        assert_eq!(first, vec!["[codex] Edited src/lib.rs (2 lines)"]);

        fs::write(
            &state,
            r#"{"files_touched":["a","b","c","d","e"],"total_lines_added":120,"directories_touched":["src"],"new_files_created":2}"#,
        )
        .unwrap();
        let summary = codex_post_feedback(
            r#"{"tool_name":"Edit","tool_input":{"file_path":"src/lib.rs","new_string":"one"}}"#,
            &state,
        )
        .unwrap();
        assert_eq!(
            summary[0],
            "[codex] src/lib.rs (+1) → Session: 5 files | 120 lines | 1 dirs | 2 new"
        );
        assert!(summary[1].contains("Consider delegating"));
        assert!(codex_post_feedback("not json", &state).is_none());
    }

    #[test]
    fn codex_session_init_resets_state_and_respects_config() {
        let temp = TempDir::new().unwrap();
        let state = temp.path().join("state.json");
        let config = temp.path().join("config.json");
        let cwd = temp.path().join("repo");
        fs::create_dir(&cwd).unwrap();

        let message = codex_session_init(&state, &config, &cwd).unwrap();
        assert!(message.contains("Codex first draft"));
        let written: Value = serde_json::from_str(&fs::read_to_string(&state).unwrap()).unwrap();
        assert_eq!(written["new_files_created"], 0);

        fs::write(&config, r#"{"enabled": false}"#).unwrap();
        assert_eq!(
            codex_session_init(&state, &config, &cwd).unwrap(),
            "[codex] Delegation enforcement disabled."
        );

        fs::write(
            &config,
            format!(
                r#"{{"enabled": true, "exclusions": {{"repositories": ["{}"], "patterns": []}}}}"#,
                cwd.display()
            ),
        )
        .unwrap();
        assert_eq!(
            codex_session_init(&state, &config, &cwd).unwrap(),
            "[codex] Excluded repo - delegation not enforced."
        );
    }

    #[test]
    fn exa_research_reminder_warns_with_research_fanout() {
        let output = exa_research_reminder();
        assert_eq!(output["decision"], "warn");
        assert!(output["reason"].as_str().unwrap().contains("/research"));
        assert!(output["reason"].as_str().unwrap().contains("Exa"));
    }

    #[test]
    fn shaping_ripple_triggers_for_shaping_markdown_only() {
        let temp = TempDir::new().unwrap();
        let shaped = temp.path().join("packet.md");
        fs::write(&shaped, "---\nshaping: true\n---\nbody").unwrap();
        let plain = temp.path().join("plain.md");
        fs::write(&plain, "# Plain").unwrap();
        let message = shaping_ripple(&format!(
            r#"{{"tool_input":{{"file_path":"{}"}}}}"#,
            shaped.display()
        ))
        .unwrap();
        assert!(message.contains("Ripple check:"));
        assert!(
            shaping_ripple(&format!(
                r#"{{"tool_input":{{"file_path":"{}"}}}}"#,
                plain.display()
            ))
            .is_none()
        );
        assert!(shaping_ripple(r#"{"tool_input":{"file_path":"src/lib.rs"}}"#).is_none());
        assert!(shaping_ripple("not json").is_none());
    }

    #[test]
    fn stop_quality_gate_detects_projects_and_check_order() {
        let temp = TempDir::new().unwrap();
        assert_eq!(detect_project(temp.path()), None);
        fs::write(temp.path().join("go.mod"), "module example").unwrap();
        assert_eq!(detect_project(temp.path()), Some(ProjectType::Go));
        fs::write(temp.path().join("Cargo.toml"), "[package]\nname='x'").unwrap();
        assert_eq!(detect_project(temp.path()), Some(ProjectType::Rust));
        fs::write(temp.path().join("pyproject.toml"), "[project]\nname='x'").unwrap();
        assert_eq!(detect_project(temp.path()), Some(ProjectType::Python));
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        assert_eq!(detect_project(temp.path()), Some(ProjectType::Node));

        assert_eq!(
            quality_checks_for_project(ProjectType::Rust),
            vec![
                ("Check", vec!["cargo", "check"]),
                ("Clippy", vec!["cargo", "clippy", "--", "-D", "warnings"]),
                ("Test", vec!["cargo", "test"]),
            ]
        );
        assert_eq!(
            quality_checks_for_project(ProjectType::Node)[0],
            ("Type check", vec!["pnpm", "tsc", "--noEmit"])
        );
    }

    #[test]
    fn stop_quality_gate_allows_silently_for_unknown_project() {
        let temp = TempDir::new().unwrap();
        assert_eq!(
            stop_quality_gate(r#"{"cwd":"/does/not/matter"}"#, temp.path()),
            StopQualityOutcome::AllowSilently
        );
        assert!(!is_running_web_project(temp.path()));
        fs::write(temp.path().join("next.config.js"), "").unwrap();
        assert!(!is_running_web_project(temp.path()));
    }

    #[test]
    fn github_cli_guard_transforms_issue_view_without_json() {
        let output = github_cli_guard(
            r#"{"tool_name":"Bash","tool_input":{"command":"gh issue view 123 --comments","description":"Issue"}}"#,
        )
        .unwrap();
        let command = output["hookSpecificOutput"]["modifiedToolInput"]["command"]
            .as_str()
            .unwrap();
        assert!(command.starts_with("gh issue view 123 --json "));
        assert!(command.contains("title,body,comments"));
        assert_eq!(
            output["hookSpecificOutput"]["modifiedToolInput"]["description"],
            "Issue"
        );
    }

    #[test]
    fn github_cli_guard_leaves_json_web_and_non_issue_commands_alone() {
        assert!(
            github_cli_guard(
                r#"{"tool_name":"Bash","tool_input":{"command":"gh issue view 123 --json title"}}"#
            )
            .is_none()
        );
        assert!(
            github_cli_guard(
                r#"{"tool_name":"Bash","tool_input":{"command":"gh issue view 123 --web"}}"#
            )
            .is_none()
        );
        assert!(
            github_cli_guard(r#"{"tool_name":"Bash","tool_input":{"command":"gh pr view 1"}}"#)
                .is_none()
        );
    }
}
