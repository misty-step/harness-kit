use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{Value, json};

use crate::claude_hooks::{destructive_command_guard, github_cli_guard, strip_quoted_content};

/// Designated secret files: reachable relative to `$HOME`, checked against
/// both the literal `~/...` form (a model may write the tilde verbatim) and
/// the `$HOME`-expanded absolute form. Extend this list as new flat
/// secret-bearing files are designated (harness-kit-913).
const SECRET_FILE_HOME_SUFFIXES: &[&str] = &[".secrets"];

/// Bash verbs whose first argument is commonly a file to dump to stdout.
/// Each is checked for a designated secret file appearing anywhere in the
/// argument list — not just as the first arg — so `grep KEY ~/.secrets`
/// (pattern before path) is caught, not just `cat ~/.secrets`.
const SECRET_READ_VERBS: &[&str] = &[
    "cat", "grep", "egrep", "fgrep", "head", "tail", "less", "more", "strings", "od", "awk",
    "hexdump", "xxd",
];

pub fn run_secrets_read_guard_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = secrets_read_guard(&input, &home_dir()) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

pub fn run_secrets_redaction_command_rewrite_from_stdin() -> Result<()> {
    let mut input = String::new();
    std::io::stdin()
        .read_to_string(&mut input)
        .context("failed to read stdin")?;
    if let Some(output) = secrets_redaction_command_rewrite(&input) {
        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}

/// Blocks direct-read commands (`cat`, `grep`, `head`, ...) against
/// designated secret files, while explicitly allowing `source`/`.` — the
/// sanctioned access pattern (shared AGENTS.md red lines). Root-caused by
/// the 2026-07-05 CANARY_API_KEY leak: a `grep` misfire against `~/.secrets`
/// printed a live key into a QMD-indexed transcript. This is a
/// command-string gate, not a filesystem permission: `source` and `cat` use
/// identical syscalls, so the block must happen pre-exec, at the tool-call
/// layer (harness-kit-913).
pub fn secrets_read_guard(input: &str, home: &Path) -> Option<Value> {
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
    let reason = secrets_read_reason(command, home)?;
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": format!(
                "BLOCKED: {reason}\n\nCommand: {command}\n\nUse `source ~/{}` (or `.`) instead — the sanctioned access pattern. Never cat/grep/head/tail a secret file; its value can land in this transcript, which is QMD-indexed and permanently searchable.",
                SECRET_FILE_HOME_SUFFIXES[0]
            ),
        }
    }))
}

fn secrets_read_reason(command: &str, home: &Path) -> Option<String> {
    let stripped = strip_quoted_content(command);
    // `source`/`.` are the sanctioned pattern — never block those, even if a
    // secret path also appears as a direct-read verb's argument elsewhere in
    // a compound command (e.g. `source ~/.secrets && cat notes.txt` is fine;
    // classification below is per verb-invocation, not per whole command).
    for secret_path in secret_file_paths(home) {
        for verb in SECRET_READ_VERBS {
            // Match the verb as a standalone command word (start of string,
            // or after a shell separator/pipe/subshell marker) followed
            // eventually by the secret path as a standalone argument word —
            // catches `grep KEY ~/.secrets` (path not first) and compound
            // commands (`cat ~/.secrets; true`), not just a bare prefix.
            let verb_pattern = format!(r"(?:^|[;&|`]|\$\()\s*{}\b", regex::escape(verb));
            let Some(verb_match) = Regex::new(&verb_pattern).unwrap().find(&stripped) else {
                continue;
            };
            let after_verb = &stripped[verb_match.end()..];
            let path_pattern = format!(
                "(?:^|[[:space:]'\"]){}(?:[[:space:]'\"]|$)",
                regex::escape(&secret_path)
            );
            // Only look within the same simple command (up to the next
            // separator), so `cat foo.txt; source ~/.secrets` isn't flagged.
            let segment_end = after_verb
                .find([';', '&', '|', '\n'])
                .unwrap_or(after_verb.len());
            let segment = &after_verb[..segment_end];
            if Regex::new(&path_pattern).unwrap().is_match(segment) {
                return Some(format!(
                    "Direct read of a designated secret file via `{verb}`. Its value can be printed into this transcript."
                ));
            }
        }
    }
    None
}

/// Both forms a model may write for a designated secret file: the literal
/// `~/name` shorthand (never shell-expanded before the policy sees it) and
/// the `$HOME`-expanded absolute path.
fn secret_file_paths(home: &Path) -> Vec<String> {
    SECRET_FILE_HOME_SUFFIXES
        .iter()
        .flat_map(|suffix| {
            vec![
                format!("~/{suffix}"),
                home.join(suffix).to_string_lossy().to_string(),
            ]
        })
        .collect()
}

fn home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// PreToolUse rewrite (harness-kit-915): wraps every Bash command so its own
/// stdout/stderr are captured to temp files, run through
/// `harness-kit-checks redact-stream`, and printed *before* Claude Code ever
/// captures the result as the tool response. Deliberately at PreToolUse, not
/// PostToolUse -- confirmed against the live Claude Code hooks docs that
/// PostToolUse's `updatedToolOutput` only changes what the model sees in
/// future context; the raw tool result (secret included) is already written
/// to the transcript JSONL and OTel telemetry by the time PostToolUse fires.
/// Rewriting the command itself, before Claude Code captures its result, is
/// the only way to keep a secret out of the transcript at rest.
///
/// Uses temp files, not `> >(process substitution)` -- live-verified the
/// process-substitution version is racy: `wait` (with or without explicit
/// PIDs, with or without an intervening `sleep`) does not reliably
/// synchronize with a `>(...)` reader before the parent command's own exit
/// path returns, so the redacted output was silently lost on every attempt
/// tried during this card's build. Temp files make the redaction step a
/// plain synchronous read with no backgrounded reader to race against.
///
/// Uses a real subshell `( ... )`, not a brace group `{ ... }` -- live-
/// verified the hard way: a brace group lets an `exit N` inside the wrapped
/// command terminate the *entire* rewritten script before the exit code is
/// captured and propagated, silently breaking exit-code semantics for any
/// wrapped command that calls `exit`. A subshell isolates that correctly.
///
/// Skips commands that already reference `redact-stream` (idempotent
/// against double-wrapping) and skips empty commands.
///
/// Correctness note verified against the live Claude Code hooks docs: hooks
/// matching the same event/matcher run in PARALLEL against the same
/// original input, not chained -- so array ordering does not decide which
/// hook "sees" a prior rewrite. That also means, if `destructive_command_
/// guard`/`secrets_read_guard` deny a command in one hook while this hook
/// concurrently returns a `modifiedToolInput` rewrite for the SAME command,
/// the merge behavior for two hooks in one matcher both emitting a decision
/// is genuinely undocumented (checked; not stated either way). Rather than
/// ship on that untested assumption, this function self-suppresses: it
/// calls the other Bash guards first and returns `None` (no-op) whenever
/// any of them would have denied/asked, so exactly one hook ever produces a
/// decision for a given command -- no concurrent-writer conflict is
/// possible regardless of how Claude Code actually resolves that case.
pub fn secrets_redaction_command_rewrite(input: &str) -> Option<Value> {
    let data: Value = serde_json::from_str(input).ok()?;
    if data.get("tool_name").and_then(Value::as_str) != Some("Bash") {
        return None;
    }
    let tool_input = data.get("tool_input").and_then(Value::as_object)?;
    let command = tool_input.get("command").and_then(Value::as_str)?;
    if command.is_empty() || command.contains("redact-stream") {
        return None;
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if destructive_command_guard(input, &cwd).is_some()
        || secrets_read_guard(input, &home_dir()).is_some()
        || github_cli_guard(input).is_some()
    {
        return None;
    }
    let escaped = command.replace('\'', r"'\''");
    let rewritten = format!(
        "__hk_out=$(mktemp); __hk_err=$(mktemp); \
         ( eval '{escaped}' ) > \"$__hk_out\" 2> \"$__hk_err\"; __hk_rc=$?; \
         harness-kit-checks redact-stream < \"$__hk_out\"; \
         harness-kit-checks redact-stream < \"$__hk_err\" >&2; \
         /usr/bin/trash \"$__hk_out\" \"$__hk_err\" 2>/dev/null; \
         exit $__hk_rc"
    );
    Some(json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "modifiedToolInput": {
                "command": rewritten,
                "description": tool_input
                    .get("description")
                    .and_then(Value::as_str)
                    .unwrap_or("Bash command (output redacted for secret shapes)"),
            }
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn secrets_guard_command(home: &Path, command: &str) -> Option<Value> {
        secrets_read_guard(
            &json!({"tool_name": "Bash", "tool_input": {"command": command}}).to_string(),
            home,
        )
    }

    #[test]
    fn secrets_guard_blocks_cat_and_grep_against_expanded_and_tilde_paths() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        for command in [
            "cat ~/.secrets",
            &format!("cat {}", home.join(".secrets").display()),
            "grep POWDER_API_KEY ~/.secrets",
        ] {
            let output = secrets_guard_command(home, command)
                .unwrap_or_else(|| panic!("expected block for: {command}"));
            assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "deny");
        }
    }

    #[test]
    fn secrets_guard_blocks_read_verb_inside_compound_command() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        // A simulated grep-misfire against ~/.secrets as part of a larger
        // command line, mirroring the 2026-07-05 CANARY_API_KEY leak shape.
        let output =
            secrets_guard_command(home, "echo debug; grep CANARY_API_KEY ~/.secrets | head -1")
                .unwrap();
        assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "deny");
        assert!(
            output["hookSpecificOutput"]["permissionDecisionReason"]
                .as_str()
                .unwrap()
                .contains("designated secret file")
        );
    }

    #[test]
    fn secrets_guard_allows_source_and_dot_of_the_same_file() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        for command in [
            "source ~/.secrets && exec powder-mcp",
            &format!(". {} && run-thing", home.join(".secrets").display()),
        ] {
            assert!(
                secrets_guard_command(home, command).is_none(),
                "expected allow for: {command}"
            );
        }
    }

    #[test]
    fn secrets_guard_ignores_unrelated_files_and_non_bash_tools() {
        let temp = TempDir::new().unwrap();
        let home = temp.path();
        assert!(secrets_guard_command(home, "cat README.md").is_none());
        assert!(secrets_guard_command(home, "grep TODO src/main.rs").is_none());
        assert!(
            secrets_read_guard(
                r#"{"tool_name":"Read","tool_input":{"file_path":"~/.secrets"}}"#,
                home,
            )
            .is_none()
        );
    }

    #[test]
    fn secrets_guard_blocks_the_exact_shape_that_leaked_powder_api_key_2026_07_06() {
        // Regression pin: during harness-kit-913's own Codex-side live
        // testing (2026-07-06), `grep POWDER_API_KEY ~/.secrets --` bypassed
        // Codex's argv-prefix execpolicy rule (grep's pattern arg sits
        // before the path, so a [verb, path]-shaped rule never matches) and
        // printed the live key into that transcript. Claude's hook uses a
        // full command-string search rather than fixed argv position, so it
        // must catch this exact shape — asserted here so it never silently
        // regresses.
        let temp = TempDir::new().unwrap();
        let output = secrets_guard_command(temp.path(), "grep POWDER_API_KEY ~/.secrets --")
            .expect("must block the exact command shape that caused a live leak");
        assert_eq!(output["hookSpecificOutput"]["permissionDecision"], "deny");
    }

    #[test]
    fn redaction_rewrite_wraps_bash_and_references_redact_stream() {
        let output = secrets_redaction_command_rewrite(
            r#"{"tool_name":"Bash","tool_input":{"command":"echo hello"}}"#,
        )
        .unwrap();
        let rewritten = output["hookSpecificOutput"]["modifiedToolInput"]["command"]
            .as_str()
            .unwrap();
        assert!(rewritten.contains("redact-stream"));
        assert!(rewritten.contains("echo hello"));
        // Uses a real subshell, not a brace group -- live-verified a brace
        // group lets a wrapped `exit N` terminate the whole rewritten
        // script before the exit code is captured.
        assert!(rewritten.contains("( eval"));
        assert!(!rewritten.contains("{ eval"));
    }

    #[test]
    fn redaction_rewrite_is_idempotent_and_ignores_non_bash_and_empty_commands() {
        assert!(
            secrets_redaction_command_rewrite(
                r#"{"tool_name":"Bash","tool_input":{"command":""}}"#
            )
            .is_none()
        );
        assert!(
            secrets_redaction_command_rewrite(
                r#"{"tool_name":"Read","tool_input":{"file_path":"x"}}"#
            )
            .is_none()
        );
        // Already-wrapped commands (containing redact-stream) are not
        // double-wrapped.
        assert!(
            secrets_redaction_command_rewrite(
                r#"{"tool_name":"Bash","tool_input":{"command":"echo x | harness-kit-checks redact-stream"}}"#
            )
            .is_none()
        );
    }

    #[test]
    fn redaction_rewrite_self_suppresses_when_another_bash_guard_would_deny() {
        // Whether Claude Code lets two hooks in the same matcher both emit a
        // decision for one command is genuinely undocumented (checked the
        // live docs). Rather than assume, this hook self-suppresses so it
        // never competes with destructive_command_guard / secrets_read_guard
        // / github_cli_guard for the same command.
        for command in [
            "rm README.md",   // destructive_command_guard
            "cat ~/.secrets", // secrets_read_guard
        ] {
            let input = format!(r#"{{"tool_name":"Bash","tool_input":{{"command":"{command}"}}}}"#);
            assert!(
                secrets_redaction_command_rewrite(&input).is_none(),
                "expected self-suppression for: {command}"
            );
        }
    }

    #[test]
    fn redaction_rewrite_preserves_single_quotes_in_the_original_command() {
        let output = secrets_redaction_command_rewrite(
            r#"{"tool_name":"Bash","tool_input":{"command":"echo 'hello world' && echo done"}}"#,
        )
        .unwrap();
        let rewritten = output["hookSpecificOutput"]["modifiedToolInput"]["command"]
            .as_str()
            .unwrap();
        // The escaped form must round-trip through a real shell without
        // corrupting the original command's quoting.
        assert!(rewritten.contains(r"'\''hello world'\''"));
    }
}
