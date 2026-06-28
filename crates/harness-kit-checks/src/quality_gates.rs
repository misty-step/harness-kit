//! Standing structural quality gates: a god-file ratchet, a debug/stub marker
//! scan, and a supply-chain policy check. See
//! `harnesses/shared/references/quality-gates.md` for the doctrine these
//! enforce — gate the diff, ratchet legacy debt, hard-block the behavioral and
//! supply-chain floor.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::{lint_gates::GateReport, process};

/// Extensions the structural gates treat as "code".
const SOURCE_EXTENSIONS: &[&str] = &["rs", "ts", "tsx", "js", "py"];

/// Line-count ceiling above which a file is a god-file. Files already over it
/// when the baseline was written are grandfathered and may only shrink; a new
/// file over it, or a grandfathered file that grows past its recorded size,
/// fails the gate.
const GODFILE_THRESHOLD: usize = 800;

/// Committed god-file ratchet baseline (repo-relative).
const GODFILE_BASELINE: &str = "crates/harness-kit-checks/baselines/godfiles.txt";

/// This module spells the forbidden markers as string literals, so it exempts
/// itself from the marker scan.
const MARKER_SELF_PATH: &str = "crates/harness-kit-checks/src/quality_gates.rs";

/// Ratchet gate: no new god-files, and grandfathered ones may only shrink.
pub fn check_godfiles(root: &Path) -> Result<GateReport> {
    let baseline = read_godfile_baseline(root)?;
    let violations = evaluate_godfiles(root, GODFILE_THRESHOLD, &baseline)?;
    if violations.is_empty() {
        Ok(GateReport::success(format!(
            "No new god-files (ceiling {GODFILE_THRESHOLD} lines; {} grandfathered).",
            baseline.len()
        )))
    } else {
        let mut out = vec![format!("Found {} god-file violation(s):", violations.len())];
        out.extend(violations);
        Ok(GateReport::failure(out))
    }
}

fn evaluate_godfiles(
    root: &Path,
    threshold: usize,
    baseline: &HashMap<String, usize>,
) -> Result<Vec<String>> {
    let mut violations = Vec::new();
    for path in source_files(root)? {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let loc = text.lines().count();
        if loc <= threshold {
            continue;
        }
        let rel = relative_slash(root, &path);
        match baseline.get(&rel) {
            Some(&ceiling) if loc <= ceiling => {}
            Some(&ceiling) => violations.push(format!(
                "  {rel}: {loc} lines, grew past grandfathered {ceiling}; split it"
            )),
            None => violations.push(format!(
                "  {rel}: {loc} lines exceeds god-file ceiling {threshold}; split into cohesive modules"
            )),
        }
    }
    violations.sort();
    Ok(violations)
}

/// Regenerate the committed baseline from the current tree. Operator-invoked
/// (`check-godfiles --write-baseline`) when a god-file is intentionally added or
/// grown, or to ratchet recorded ceilings down after a split.
pub fn write_godfile_baseline(root: &Path) -> Result<String> {
    let mut entries: Vec<(String, usize)> = Vec::new();
    for path in source_files(root)? {
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        let loc = text.lines().count();
        if loc > GODFILE_THRESHOLD {
            entries.push((relative_slash(root, &path), loc));
        }
    }
    entries.sort();
    let mut out = String::from(
        "# God-file ratchet baseline (harness-kit-checks check-godfiles).\n# Files over the line ceiling when baselined; they may only shrink.\n# Regenerate intentionally: harness-kit-checks check-godfiles --write-baseline\n",
    );
    for (rel, loc) in &entries {
        out.push_str(&format!("{loc} {rel}\n"));
    }
    let path = root.join(GODFILE_BASELINE);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&path, &out).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(format!(
        "Wrote {} god-file baseline entries to {GODFILE_BASELINE}",
        entries.len()
    ))
}

fn read_godfile_baseline(root: &Path) -> Result<HashMap<String, usize>> {
    let mut map = HashMap::new();
    let Ok(text) = fs::read_to_string(root.join(GODFILE_BASELINE)) else {
        return Ok(map);
    };
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((loc, rel)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        if let Ok(loc) = loc.trim().parse::<usize>() {
            map.insert(rel.trim().to_string(), loc);
        }
    }
    Ok(map)
}

/// Hard-block gate: debug/stub macros left in Rust source — `dbg!`, `todo!`,
/// `unimplemented!`. They compile and pass tests, so review misses them; they
/// are also common agent "tells" (stub-as-implementation, debug leftover).
pub fn check_source_markers(root: &Path) -> Result<GateReport> {
    let markers = ["dbg!(", "todo!(", "unimplemented!("];
    let mut findings = Vec::new();
    for path in source_files(root)? {
        let rel = relative_slash(root, &path);
        if rel == MARKER_SELF_PATH || !rel.ends_with(".rs") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (lineno, line) in text.lines().enumerate() {
            for marker in &markers {
                if line.contains(marker) {
                    let name = marker.trim_end_matches('(');
                    findings.push(format!("  {rel}:{}: {name}", lineno + 1));
                }
            }
        }
    }
    if findings.is_empty() {
        Ok(GateReport::success("No debug/stub markers in Rust source."))
    } else {
        let mut out = vec![format!("Found {} debug/stub marker(s):", findings.len())];
        out.extend(findings.into_iter().take(40));
        Ok(GateReport::failure(out))
    }
}

/// Hard-block gate: supply-chain policy via cargo-deny (offline checks: bans,
/// licenses, sources). Advisories (network) run in CI / the coverage+mutation
/// fast-follow. Gracefully skips when cargo-deny is absent (shellcheck
/// precedent), so the aggregate gate stays runnable without the extra tool.
pub fn check_supply_chain(root: &Path) -> Result<GateReport> {
    if !command_exists("cargo-deny") {
        if std::env::var_os("CI").is_some() {
            return Ok(GateReport::failure(vec![
                "cargo-deny is required in CI but was not found on PATH.".to_string(),
            ]));
        }
        return Ok(GateReport::success(
            "cargo-deny not installed; supply-chain gate skipped (cargo install cargo-deny to enforce).",
        ));
    }
    let output = process::command("cargo")
        .args(["deny", "check", "bans", "licenses", "sources"])
        .current_dir(root)
        .output()
        .context("failed to run cargo deny")?;
    if output.status.success() {
        return Ok(GateReport::success(
            "cargo-deny: bans, licenses, sources clean.",
        ));
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{stdout}{stderr}");
    let mut out = vec!["cargo deny check failed:".to_string()];
    out.extend(combined.lines().map(|line| format!("  {line}")).take(40));
    Ok(GateReport::failure(out))
}

fn source_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_source_files(root, root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_source_files(root: &Path, dir: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let rel = relative_slash(root, &path);
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default();
            if is_skippable_dir(name) || rel == "skills/.external" {
                continue;
            }
            collect_source_files(root, &path, paths)?;
        } else if path.is_file() && has_source_extension(&path) {
            paths.push(path);
        }
    }
    Ok(())
}

fn is_skippable_dir(name: &str) -> bool {
    matches!(
        name,
        ".git"
            | "target"
            | "node_modules"
            | ".venv"
            | "__pycache__"
            | "dist"
            | ".next"
            | "coverage"
            | ".harness-kit"
    )
}

fn has_source_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| SOURCE_EXTENSIONS.contains(&extension))
}

fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn command_exists(command: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|dir| dir.join(command).is_file())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_lines(path: &Path, count: usize) {
        fs::write(path, "x\n".repeat(count)).unwrap();
    }

    #[test]
    fn godfiles_flag_new_over_threshold_and_respect_baseline() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        write_lines(&root.join("src/new_big.rs"), 30);
        write_lines(&root.join("src/grandfathered.rs"), 25);
        write_lines(&root.join("src/small.rs"), 5);

        let mut baseline = HashMap::new();
        baseline.insert("src/grandfathered.rs".to_string(), 40usize);

        let violations = evaluate_godfiles(root, 20, &baseline).unwrap();
        assert_eq!(violations.len(), 1, "{violations:?}");
        assert!(violations[0].contains("src/new_big.rs"));
        assert!(violations[0].contains("ceiling"));
    }

    #[test]
    fn godfiles_flag_grandfathered_growth() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        write_lines(&root.join("grew.rs"), 50);
        let mut baseline = HashMap::new();
        baseline.insert("grew.rs".to_string(), 40usize);

        let violations = evaluate_godfiles(root, 20, &baseline).unwrap();
        assert_eq!(violations.len(), 1);
        assert!(violations[0].contains("grew past grandfathered 40"));
    }

    #[test]
    fn source_markers_flag_debug_and_stub_in_rust_only() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("src/a.rs"), "fn f() { dbg!(x); todo!() }\n").unwrap();
        fs::write(root.join("src/clean.rs"), "fn g() -> u8 { 1 }\n").unwrap();
        fs::write(root.join("note.md"), "dbg!( in prose\n").unwrap();

        let report = check_source_markers(root).unwrap();
        assert!(report.errors[0].contains("Found 2 debug/stub"));
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("src/a.rs") && error.contains("dbg!"))
        );
        assert!(report.errors.iter().any(|error| error.contains("todo!")));
        assert!(!report.errors.iter().any(|error| error.contains("note.md")));
    }

    #[test]
    fn source_markers_clean_repo_passes() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("ok.rs"), "fn ok() {}\n").unwrap();
        let report = check_source_markers(temp.path()).unwrap();
        assert!(report.errors.is_empty(), "{:?}", report.errors);
        assert!(report.ok_message.contains("No debug/stub"));
    }
}
