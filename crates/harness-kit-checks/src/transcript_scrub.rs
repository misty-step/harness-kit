//! One-time read-only scan of QMD-indexed transcript collections for secret
//! shapes already at rest (harness-kit-915, defense-in-depth net 2: a
//! historical leak that predates the PreToolUse redaction rewrite must not
//! sit retrievable indefinitely via QMD's semantic search).
//!
//! Deliberately SCAN-ONLY in this pass, not scan-and-rewrite: the two
//! collections named in the card (`claude-code-transcripts` at
//! `~/.claude/projects/**/*.jsonl`, `codex-sessions` at
//! `~/.codex/sessions/**/*.jsonl`) total ~3,000 files and ~3.4GB of live
//! session history on this machine -- files that are also read by every
//! other harness/session as working state, not just a data corpus. Silently
//! rewriting thousands of session transcripts in place is a materially
//! different risk class than reading them, and the card's own acceptance
//! bar only requires "a count of any redactions found reported" for this
//! pass, not that files be mutated. Report the count; a rewrite pass is a
//! separate, explicit follow-on once the count is known and reviewed.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use harness_kit_hooks::secret_redaction::redact;
use serde::Serialize;

#[derive(Debug, Default, Serialize)]
pub struct ScrubReport {
    pub collection: String,
    pub files_scanned: usize,
    pub files_with_findings: usize,
    pub total_findings: usize,
    pub findings: Vec<FileFinding>,
}

#[derive(Debug, Serialize)]
pub struct FileFinding {
    pub path: String,
    pub line_numbers: Vec<usize>,
}

/// Scan every `*.jsonl` file under `root` for text that `redact` would
/// change, without writing anything. Each JSONL line is scanned as its own
/// unit (matching how a transcript is actually line-delimited), so a
/// finding's line number is directly usable to locate it later.
pub fn scan_collection(collection: &str, root: &Path) -> Result<ScrubReport> {
    let mut report = ScrubReport {
        collection: collection.to_string(),
        ..Default::default()
    };
    let mut files = Vec::new();
    collect_jsonl_files(root, &mut files)?;
    files.sort();

    for path in files {
        report.files_scanned += 1;
        let Ok(contents) = fs::read_to_string(&path) else {
            continue; // unreadable/binary-garbled line — skip, don't fail the whole scan
        };
        let mut line_numbers = Vec::new();
        for (idx, line) in contents.lines().enumerate() {
            if line.is_empty() {
                continue;
            }
            if redact(line, &[]) != line {
                line_numbers.push(idx + 1);
            }
        }
        if !line_numbers.is_empty() {
            report.files_with_findings += 1;
            report.total_findings += line_numbers.len();
            report.findings.push(FileFinding {
                path: path.to_string_lossy().to_string(),
                line_numbers,
            });
        }
    }
    Ok(report)
}

fn collect_jsonl_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let entries = fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_jsonl_files(&path, out)?;
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("jsonl") {
            out.push(path);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn scan_finds_secret_shaped_lines_and_reports_line_numbers() {
        let temp = TempDir::new().unwrap();
        let file = temp.path().join("a.jsonl");
        fs::write(
            &file,
            "{\"clean\":\"line one\"}\n{\"leaked\":\"sk-or-v1-ABCDEF1234567890abcdef\"}\n{\"clean again\":true}\n",
        )
        .unwrap();

        let report = scan_collection("test-collection", temp.path()).unwrap();
        assert_eq!(report.files_scanned, 1);
        assert_eq!(report.files_with_findings, 1);
        assert_eq!(report.total_findings, 1);
        assert_eq!(report.findings[0].line_numbers, vec![2]);
    }

    #[test]
    fn scan_recurses_subdirectories_and_only_matches_jsonl() {
        let temp = TempDir::new().unwrap();
        fs::create_dir_all(temp.path().join("nested")).unwrap();
        fs::write(
            temp.path().join("nested/b.jsonl"),
            "{\"key\":\"AKIAZZZZZZZZZZZZZZZZ_pad\"}\n",
        )
        .unwrap();
        fs::write(temp.path().join("ignored.txt"), "sk-or-v1-shouldnotmatch12345").unwrap();

        let report = scan_collection("test", temp.path()).unwrap();
        assert_eq!(report.files_scanned, 1);
        assert_eq!(report.total_findings, 1);
    }

    #[test]
    fn scan_reports_zero_findings_for_clean_corpus() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("clean.jsonl"), "{\"hello\":\"world\"}\n").unwrap();
        let report = scan_collection("test", temp.path()).unwrap();
        assert_eq!(report.files_scanned, 1);
        assert_eq!(report.files_with_findings, 0);
        assert_eq!(report.total_findings, 0);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn scan_does_not_fail_on_unreadable_or_missing_root() {
        let report = scan_collection("test", Path::new("/does/not/exist")).unwrap();
        assert_eq!(report.files_scanned, 0);
    }
}
