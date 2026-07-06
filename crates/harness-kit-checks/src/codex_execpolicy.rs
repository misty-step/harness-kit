//! Idempotently converges the harness-kit-owned block of Codex CLI execpolicy
//! rules (`~/.codex/rules/default.rules`) that forbids the bare, no-argument
//! form of direct-read commands against designated secret files
//! (harness-kit-913). Codex has no PreToolUse-equivalent hook;
//! `prefix_rule(..., decision="forbidden")` entries in this `.rules` file are
//! the actual pre-exec interception point (confirmed live: a forbidden
//! prefix rule rejects the matching command before it runs, even under
//! `approval_policy = "never"` / `sandbox_mode = "danger-full-access"`).
//!
//! STATED LIMITATIONS — verified live, not assumed. `prefix_rule` matches a
//! literal, fixed-position leading argv sequence; the DSL has no wildcard or
//! "contains" primitive (`pattern=["grep", "*", path]` was tested and does
//! NOT match anything — `*` is a literal token, not a glob).
//!
//! 1. **Verbs whose canonical shape puts the path at a variable position are
//!    not covered.** `grep`/`egrep`/`fgrep`/`awk` take a pattern/program
//!    before the file (`grep PATTERN FILE`), so a rule keyed on
//!    `[verb, path]` never matches real usage — it would be dead code
//!    creating false confidence, so those verbs are deliberately excluded
//!    from `SECRET_READ_VERBS` below rather than shipped as a no-op rule.
//!    (Live-verified the hard way: this exact gap let a real POWDER_API_KEY
//!    leak into a transcript during this card's own testing — the model ran
//!    `grep POWDER_API_KEY ~/.secrets --`, which a `[verb, path]`-shaped rule
//!    cannot match because the pattern argument sits before the path.)
//! 2. **A leading flag on a covered verb also bypasses its rule** — `cat -n
//!    ~/.secrets` shifts the path to position 2, past a rule anchored at
//!    position 1. Confirmed live. This means even the covered verbs below
//!    only reliably block the bare `verb path` invocation, not `verb [any
//!    flag] path`.
//! 3. **Cannot see inside a quoted shell string** — `bash -c 'cat
//!    ~/.secrets'` bypasses every rule below; the top-level argv is
//!    `["bash", "-c", "..."]`, never `["cat", ...]`. Confirmed live.
//!
//! Given these, this module is real but narrow defense in depth (it catches
//! the literal bare-invocation misfire shape), not a general guarantee the
//! way Claude Code's full command-string regex hook
//! (`harness_kit_hooks::claude_hooks::secrets_read_guard`) is. Codex's actual
//! backstop for the argument-position and shell-wrapper gaps is
//! harness-kit-915 (transcript-level redaction) — do not represent this
//! module as closing the class, only the bare-invocation slice of it.
//!
//! The file is otherwise hand-curated by the operator/agents (accumulated
//! per-repo allow rules) — this module must never replace the whole file,
//! only converge one clearly marked block within it, matching the
//! `powder_mcp_bootstrap` upsert idiom used for `~/.codex/config.toml`.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

const BEGIN_MARKER: &str = "# BEGIN harness-kit-913 secrets-read-guard (generated, do not hand-edit this block)";
const END_MARKER: &str = "# END harness-kit-913 secrets-read-guard";

/// Designated secret files, relative to `$HOME` — kept in lockstep with
/// `harness_kit_hooks::claude_hooks::SECRET_FILE_HOME_SUFFIXES`. Extend both
/// lists together when a new flat secret-bearing file is designated.
const SECRET_FILE_HOME_SUFFIXES: &[&str] = &[".secrets"];

/// Verbs whose canonical invocation puts the target file immediately after
/// the verb (`verb FILE`), so a `[verb, path]` prefix rule can match real
/// usage. Deliberately excludes `grep`/`egrep`/`fgrep`/`awk` — see module
/// docs, limitation 1.
const SECRET_READ_VERBS: &[&str] = &["cat", "head", "tail", "less", "more", "strings", "od", "hexdump", "xxd"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Updated,
    Unchanged,
}

/// Converge the harness-kit-owned block in `path` (typically
/// `~/.codex/rules/default.rules`). Creates the file (and its parent
/// directory) if absent. Preserves every other line verbatim.
pub fn ensure(path: &Path) -> Result<Status> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error).with_context(|| format!("failed to read {}", path.display())),
    };
    let desired = desired_block();
    if is_converged(&text, &desired) {
        return Ok(Status::Unchanged);
    }
    let mut next = remove_block(&text);
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    if !next.is_empty() {
        next.push('\n');
    }
    next.push_str(&desired);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, next).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(Status::Updated)
}

fn is_converged(text: &str, desired: &str) -> bool {
    match block_span(text) {
        Some((start, end)) => text[start..end].trim_end() == desired.trim_end(),
        None => false,
    }
}

fn remove_block(text: &str) -> String {
    let Some((start, end)) = block_span(text) else {
        return text.to_string();
    };
    let mut out = String::with_capacity(text.len());
    out.push_str(&text[..start]);
    out.push_str(&text[end..]);
    trim_excess_blank_lines(&out)
}

/// Byte span `[start, end)` of the existing marker block, including a
/// trailing newline, or `None` if no (or an unterminated) block is present.
fn block_span(text: &str) -> Option<(usize, usize)> {
    let start = text.find(BEGIN_MARKER)?;
    let after_begin = start + BEGIN_MARKER.len();
    let end_marker_pos = text[after_begin..].find(END_MARKER)?;
    let mut end = after_begin + end_marker_pos + END_MARKER.len();
    if let Some(rest) = text[end..].strip_prefix('\n') {
        end = text.len() - rest.len();
    }
    Some((start, end))
}

fn trim_excess_blank_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank_count = 0usize;
    for line in text.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    out.trim_end_matches('\n').to_string()
}

fn desired_block() -> String {
    let mut lines = vec![BEGIN_MARKER.to_string()];
    lines.push(
        "# Blocks the BARE invocation (verb + path, no flags/pattern) of direct-read commands"
            .to_string(),
    );
    lines.push(
        "# against designated secret files (harness-kit-913). NOT a general guarantee -- a"
            .to_string(),
    );
    lines.push(
        "# leading flag (`cat -n ~/.secrets`), a `bash -c` wrapper, or a verb whose path"
            .to_string(),
    );
    lines.push(
        "# position varies (grep/awk: pattern comes before the file) all bypass this. See"
            .to_string(),
    );
    lines.push(
        "# crate::codex_execpolicy module docs for the live-verified limitations and why"
            .to_string(),
    );
    lines.push(
        "# harness-kit-915 (transcript redaction) is the real backstop on Codex.".to_string(),
    );
    for suffix in SECRET_FILE_HOME_SUFFIXES {
        for verb in SECRET_READ_VERBS {
            for form in [format!("~/{suffix}"), format!("$HOME/{suffix}")] {
                lines.push(format!(
                    "prefix_rule(pattern=[{verb:?}, {form:?}], decision=\"forbidden\")"
                ));
            }
        }
    }
    lines.push(END_MARKER.to_string());
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn creates_file_with_block_when_absent() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("rules/default.rules");
        assert_eq!(ensure(&path).unwrap(), Status::Updated);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains(BEGIN_MARKER));
        assert!(text.contains(END_MARKER));
        assert!(text.contains(r#"prefix_rule(pattern=["cat", "~/.secrets"], decision="forbidden")"#));
        assert!(text.contains(
            r#"prefix_rule(pattern=["head", "$HOME/.secrets"], decision="forbidden")"#
        ));
        // grep/awk deliberately excluded — see module docs, limitation 1: a
        // [verb, path] rule can never match their real invocation shape
        // (pattern/program comes before the file), so shipping one would be
        // dead code creating false confidence.
        assert!(!text.contains("\"grep\""));
        assert!(!text.contains("\"awk\""));
    }

    #[test]
    fn second_run_is_unchanged() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("default.rules");
        assert_eq!(ensure(&path).unwrap(), Status::Updated);
        assert_eq!(ensure(&path).unwrap(), Status::Unchanged);
    }

    #[test]
    fn preserves_hand_curated_rules_before_and_after_the_block() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("default.rules");
        fs::write(
            &path,
            "prefix_rule(pattern=[\"rg\"], decision=\"allow\")\nprefix_rule(pattern=[\"gh\"], decision=\"allow\")\n",
        )
        .unwrap();
        assert_eq!(ensure(&path).unwrap(), Status::Updated);
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains(r#"prefix_rule(pattern=["rg"], decision="allow")"#));
        assert!(text.contains(r#"prefix_rule(pattern=["gh"], decision="allow")"#));
        assert!(text.contains(BEGIN_MARKER));

        // Appending again after more hand-curated rules converges cleanly
        // rather than duplicating the block.
        let mut appended = text.clone();
        appended.push_str("prefix_rule(pattern=[\"pnpm\", \"install\"], decision=\"allow\")\n");
        fs::write(&path, appended).unwrap();
        assert_eq!(ensure(&path).unwrap(), Status::Unchanged);
        let final_text = fs::read_to_string(&path).unwrap();
        assert_eq!(final_text.matches(BEGIN_MARKER).count(), 1);
        assert!(final_text.contains("pnpm"));
    }

    #[test]
    fn replaces_stale_block_without_touching_surrounding_rules() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("default.rules");
        fs::write(
            &path,
            format!(
                "prefix_rule(pattern=[\"rg\"], decision=\"allow\")\n\n{BEGIN_MARKER}\nprefix_rule(pattern=[\"cat\", \"~/.old-stale-path\"], decision=\"forbidden\")\n{END_MARKER}\n\nprefix_rule(pattern=[\"gh\"], decision=\"allow\")\n"
            ),
        )
        .unwrap();
        assert_eq!(ensure(&path).unwrap(), Status::Updated);
        let text = fs::read_to_string(&path).unwrap();
        assert!(!text.contains("old-stale-path"));
        assert!(text.contains(r#"prefix_rule(pattern=["cat", "~/.secrets"], decision="forbidden")"#));
        assert!(text.contains(r#"prefix_rule(pattern=["rg"], decision="allow")"#));
        assert!(text.contains(r#"prefix_rule(pattern=["gh"], decision="allow")"#));
        assert_eq!(text.matches(BEGIN_MARKER).count(), 1);
    }
}
