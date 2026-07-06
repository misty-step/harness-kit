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
//! 1. **Verbs whose canonical shape puts the path at a variable position**
//!    (`grep`/`egrep`/`fgrep` take a pattern before the file; `awk` a
//!    program) can't be covered by a `[verb, path]` rule at all — a
//!    `[verb, path]`-shaped rule never matches real usage, so that shape is
//!    not shipped for these verbs. **Live-verified the hard way**: this
//!    exact gap let a real POWDER_API_KEY leak into a transcript during
//!    this card's own testing — the model ran
//!    `grep POWDER_API_KEY ~/.secrets --`. Root-cause fix in this revision:
//!    `SECRET_KEY_NAMES` below enumerates the actual (small, finite) set of
//!    secret variable names this fleet has today, and ships an
//!    argv-position-aware `[verb, KEYNAME, path]` (or, for `awk`,
//!    `[verb, "/KEYNAME/", path]`) exact-match forbidden rule per
//!    verb/name/path-form combination. This closes the *specific,
//!    repeatable* incident shape (grep-for-a-known-secret-name) without a
//!    blanket ban on grep/awk — live-verified both that the fixture command
//!    (including its trailing `--`) is now blocked AND that ordinary
//!    `grep TODO README.md` still works. It does NOT catch an arbitrary,
//!    not-yet-named pattern string (`grep S[EK].*KEY ~/.secrets` or any
//!    novel/obfuscated search term) — a blanket `forbidden` on the bare verb
//!    was tested and does block that, but also blocks all ordinary grep
//!    usage fleet-wide (verified: `grep TODO README.md` gets rejected too),
//!    which is a worse tradeoff than accepting this narrower gap. Extend
//!    `SECRET_KEY_NAMES` whenever a new secret variable name is designated.
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
//! the literal bare-invocation misfire shape, plus the specific known-key-name
//! shape for grep/egrep/fgrep/awk), not a general guarantee the way Claude
//! Code's full command-string regex hook
//! (`harness_kit_hooks::claude_hooks::secrets_read_guard`) is. Codex's actual
//! backstop for the leading-flag, shell-wrapper, and novel-pattern gaps is
//! harness-kit-915 (transcript-level redaction) — do not represent this
//! module as closing the class, only the slices named above.
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

/// Verbs whose canonical shape puts a pattern/program *before* the file
/// (`grep PATTERN FILE`), covered instead via `SECRET_KEY_NAMES` exact-match
/// rules — see module docs, limitation 1.
const PATTERN_FIRST_VERBS: &[&str] = &["grep", "egrep", "fgrep"];

/// The actual, current, finite set of secret variable names this fleet
/// designates (kept manually in sync with `~/.secrets`'s real key names —
/// not derived at bootstrap time, since that would mean reading the secret
/// file's key names into this generator, which is a smaller but real
/// version of the same "don't touch it more than necessary" concern the
/// card raises about the file itself; the names are not secret, only the
/// values are). Extend this list whenever a new secret variable name is
/// designated. `POWDER_API_BASE_URL` is deliberately excluded: it's a URL,
/// not a secret.
const SECRET_KEY_NAMES: &[&str] = &["ARTIFACTS_API_TOKEN", "POWDER_API_KEY", "CANARY_API_KEY"];

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
        "# Blocks (1) the BARE invocation (verb + path, no flags/pattern) of direct-read"
            .to_string(),
    );
    lines.push(
        "# commands, and (2) grep/egrep/fgrep/awk searching for a KNOWN secret variable name,"
            .to_string(),
    );
    lines.push(
        "# against designated secret files (harness-kit-913). NOT a general guarantee -- a"
            .to_string(),
    );
    lines.push(
        "# leading flag (`cat -n ~/.secrets`), a `bash -c` wrapper, or a grep/awk search for a"
            .to_string(),
    );
    lines.push(
        "# NOT-yet-named pattern all bypass this. See crate::codex_execpolicy module docs for"
            .to_string(),
    );
    lines.push(
        "# the live-verified limitations and why harness-kit-915 (transcript redaction) is the"
            .to_string(),
    );
    lines.push("# real backstop on Codex.".to_string());
    let path_forms = |suffix: &str| [format!("~/{suffix}"), format!("$HOME/{suffix}")];
    for suffix in SECRET_FILE_HOME_SUFFIXES {
        for verb in SECRET_READ_VERBS {
            for form in path_forms(suffix) {
                lines.push(format!(
                    "prefix_rule(pattern=[{verb:?}, {form:?}], decision=\"forbidden\")"
                ));
            }
        }
        for verb in PATTERN_FIRST_VERBS {
            for key in SECRET_KEY_NAMES {
                for form in path_forms(suffix) {
                    lines.push(format!(
                        "prefix_rule(pattern=[{verb:?}, {key:?}, {form:?}], decision=\"forbidden\")"
                    ));
                }
            }
        }
        // awk takes a PROGRAM, not a bare pattern -- the idiomatic
        // equivalent of `grep KEY FILE` is `awk '/KEY/' FILE`. Live-verified
        // this exact-match rule blocks that shape without a blanket ban on
        // awk (see module docs, limitation 1).
        for key in SECRET_KEY_NAMES {
            for form in path_forms(suffix) {
                let program = format!("/{key}/");
                lines.push(format!(
                    "prefix_rule(pattern=[\"awk\", {program:?}, {form:?}], decision=\"forbidden\")"
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
        // A bare [verb, path] rule for grep/awk would be dead code (their
        // real shape puts a pattern/program before the file) -- confirm
        // that specific shape is absent even though grep/awk rules exist
        // for the argv-position-aware key-name form (below).
        assert!(!text.contains(r#"prefix_rule(pattern=["grep", "~/.secrets"]"#));
        assert!(!text.contains(r#"prefix_rule(pattern=["awk", "~/.secrets"]"#));
    }

    #[test]
    fn covers_the_exact_grep_fixture_that_caused_a_live_leak() {
        // Regression pin, operator-requested 2026-07-07: harness-kit-913's
        // own Codex-side live testing leaked a real POWDER_API_KEY because
        // `grep POWDER_API_KEY ~/.secrets --` (pattern before path) could not
        // match a [verb, path]-shaped rule. This rule shape must cover it.
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("default.rules");
        ensure(&path).unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains(
            r#"prefix_rule(pattern=["grep", "POWDER_API_KEY", "~/.secrets"], decision="forbidden")"#
        ));
        // The trailing `--` in the real incident command doesn't change
        // this: `prefix_rule` matches a leading argv sequence, and codex's
        // own policy engine confirmed live that a 3-token forbidden rule
        // still matches `grep POWDER_API_KEY ~/.secrets --` (extra trailing
        // args don't defeat a prefix match).
        assert!(text.contains(
            r#"prefix_rule(pattern=["grep", "ARTIFACTS_API_TOKEN", "$HOME/.secrets"], decision="forbidden")"#
        ));
        // awk's idiomatic equivalent (`awk '/KEY/' FILE`) is covered too.
        assert!(text.contains(
            r#"prefix_rule(pattern=["awk", "/POWDER_API_KEY/", "~/.secrets"], decision="forbidden")"#
        ));
        // Ordinary grep usage must remain untouched by this generator --
        // no rule exists that would match a bare `grep PATTERN FILE` for an
        // arbitrary (non-designated) pattern.
        assert!(!text.contains(r#"pattern=["grep"], decision="forbidden""#));
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
