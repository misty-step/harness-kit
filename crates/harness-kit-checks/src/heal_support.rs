use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use chrono::Utc;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateFailure {
    pub name: String,
    pub detail: String,
}

pub fn healable_gates() -> &'static [&'static str] {
    &[
        "check-frontmatter",
        "lint-python",
        "lint-shell",
        "lint-yaml",
    ]
}

pub fn targeted_validation_commands() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "lint-yaml",
            "find . \\( -name '*.yaml' -o -name '*.yml' \\) -not -path './ci/*' | xargs python3 -c 'import sys,yaml; [yaml.safe_load(open(f)) for f in sys.argv[1:]]'",
        ),
        (
            "lint-shell",
            "find . -name '*.sh' -not -path './ci/*' | xargs shellcheck --severity=error",
        ),
        (
            "lint-python",
            "find . -name '*.py' -not -path './ci/*' | xargs -I{} python3 -m py_compile {}",
        ),
        (
            "check-frontmatter",
            "cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .",
        ),
    ]
}

pub fn repair_prompt(failure: &GateFailure, attempt: usize, attempts: usize) -> Result<String> {
    if attempts == 0 {
        bail!("attempts must be at least 1.");
    }
    if attempt == 0 || attempt > attempts {
        bail!("attempt must be between 1 and attempts.");
    }

    let commands = targeted_validation_commands()
        .iter()
        .map(|(name, command)| format!("- {name}: {command}"))
        .collect::<Vec<_>>()
        .join("\n");

    Ok(format!(
        r#"You are repairing a failing CI gate in the Harness Kit repository.

Gate: {gate}
Attempt: {attempt} of {attempts}
Failure details:
{detail}

Rules:
- Work only in /src.
- Fix the root cause for {gate}. Do not broaden scope.
- Keep edits minimal and ASCII unless the file already requires otherwise.
- Re-run the targeted gate after each meaningful edit.
- Before finishing, ensure the targeted gate passes and leave the updated repo in $repaired.
- Do not use git. Branching and committing happen after verification.

Available tool:
- $builder is a writable repo container rooted at /src with the linting tools installed.

Targeted validation commands:
{commands}

When the target gate passes, bind the updated container to $repaired."#,
        gate = failure.name,
        attempt = attempt,
        attempts = attempts,
        detail = failure.detail,
        commands = commands,
    ))
}

pub fn parse_check_failures(summary: &str) -> Vec<GateFailure> {
    let gate_line = Regex::new(r"^\s{2}(PASS|FAIL)\s{2}(.+)$").expect("valid regex");
    let mut failures = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_ok = true;
    let mut details = Vec::new();

    for line in summary.lines() {
        if let Some(captures) = gate_line.captures(line) {
            if let Some(name) = current_name.take()
                && !current_ok
            {
                failures.push(GateFailure {
                    name,
                    detail: detail_or_default(&details),
                });
            }
            current_ok = captures.get(1).map(|m| m.as_str()) == Some("PASS");
            current_name = captures.get(2).map(|m| m.as_str().trim().to_string());
            details.clear();
            continue;
        }
        if current_name.is_some() && !current_ok && line.starts_with("         ") {
            details.push(line.trim().to_string());
        }
    }
    if let Some(name) = current_name
        && !current_ok
    {
        failures.push(GateFailure {
            name,
            detail: detail_or_default(&details),
        });
    }
    failures
}

pub fn first_failed_gate(summary: &str) -> Option<String> {
    parse_check_failures(summary)
        .into_iter()
        .next()
        .map(|failure| failure.name)
}

pub fn select_healable_failure(failures: &[GateFailure]) -> Result<GateFailure> {
    if failures.is_empty() {
        bail!("heal requires at least one failing gate.");
    }
    let healable = healable_gates().iter().copied().collect::<BTreeSet<_>>();
    let unsupported = failures
        .iter()
        .filter(|failure| !healable.contains(failure.name.as_str()))
        .map(|failure| failure.name.clone())
        .collect::<Vec<_>>();
    if !unsupported.is_empty() {
        let supported = healable_gates().join(", ");
        let mut got = failures
            .iter()
            .map(|failure| failure.name.clone())
            .collect::<Vec<_>>();
        got.sort();
        bail!(
            "heal currently supports one lint-style failure at a time. Supported gates: {supported}. Got: {}.",
            got.join(", ")
        );
    }
    if failures.len() != 1 {
        let mut got = failures
            .iter()
            .map(|failure| failure.name.clone())
            .collect::<Vec<_>>();
        got.sort();
        bail!(
            "heal currently supports one failing gate at a time. Resolve the other failures first: {}.",
            got.join(", ")
        );
    }
    Ok(failures[0].clone())
}

pub fn repair_branch_name(gate_name: &str) -> String {
    let slug = Regex::new(r"[^a-z0-9]+")
        .expect("valid regex")
        .replace_all(&gate_name.to_ascii_lowercase(), "-")
        .trim_matches('-')
        .to_string();
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    format!("heal/{slug}-{timestamp}")
}

pub fn repair_commit_message(gate_name: &str) -> String {
    format!("ci: heal {gate_name}")
}

pub fn snapshot_delta(before_root: &Path, after_root: &Path) -> Result<(Vec<String>, Vec<String>)> {
    let excluded = BTreeSet::from([".git", ".env", "__pycache__"]);
    let before = collect_digests(before_root, &excluded)?;
    let after = collect_digests(after_root, &excluded)?;
    let paths = before
        .keys()
        .chain(after.keys())
        .cloned()
        .collect::<BTreeSet<_>>();
    let mut stage = Vec::new();
    let mut remove = Vec::new();
    for path in paths {
        match (before.get(&path), after.get(&path)) {
            (None, Some(_)) => stage.push(path),
            (Some(_), None) => remove.push(path),
            (Some(before), Some(after)) if before != after => stage.push(path),
            _ => {}
        }
    }
    Ok((stage, remove))
}

fn detail_or_default(details: &[String]) -> String {
    let detail = details.join("\n").trim().to_string();
    if detail.is_empty() {
        "No stderr captured.".to_string()
    } else {
        detail
    }
}

fn collect_digests(
    root: &Path,
    excluded: &BTreeSet<&str>,
) -> Result<std::collections::BTreeMap<String, Vec<u8>>> {
    let mut files = std::collections::BTreeMap::new();
    collect_digests_inner(root, root, excluded, &mut files)?;
    Ok(files)
}

fn collect_digests_inner(
    root: &Path,
    current: &Path,
    excluded: &BTreeSet<&str>,
    files: &mut std::collections::BTreeMap<String, Vec<u8>>,
) -> Result<()> {
    if !current.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(current)? {
        let path = entry?.path();
        let rel = path.strip_prefix(root)?.to_path_buf();
        if rel
            .components()
            .any(|part| excluded.contains(part.as_os_str().to_string_lossy().as_ref()))
        {
            continue;
        }
        if path.is_dir() {
            collect_digests_inner(root, &path, excluded, files)?;
        } else if path.is_file() {
            files.insert(
                rel.to_string_lossy().replace('\\', "/"),
                Sha256::digest(fs::read(&path)?).to_vec(),
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_failed_gate_details() {
        let summary = "Harness Kit CI Results\n========================================\n  PASS  lint-yaml\n  FAIL  lint-shell\n         first detail\n         second detail\n  FAIL  check-frontmatter\n         missing field\n========================================\n1 passed, 2 failed\n";
        assert_eq!(
            parse_check_failures(summary),
            vec![
                GateFailure {
                    name: "lint-shell".to_string(),
                    detail: "first detail\nsecond detail".to_string(),
                },
                GateFailure {
                    name: "check-frontmatter".to_string(),
                    detail: "missing field".to_string(),
                },
            ]
        );
        assert_eq!(first_failed_gate(summary).as_deref(), Some("lint-shell"));
    }

    #[test]
    fn selects_healable_failure_and_rejects_bad_shapes() {
        let failure = GateFailure {
            name: "lint-yaml".to_string(),
            detail: "bad yaml".to_string(),
        };
        assert_eq!(
            select_healable_failure(std::slice::from_ref(&failure)).unwrap(),
            failure
        );
        assert!(
            select_healable_failure(&[])
                .unwrap_err()
                .to_string()
                .contains("at least one")
        );
        assert!(
            select_healable_failure(&[GateFailure {
                name: "test-bun".to_string(),
                detail: "boom".to_string(),
            }])
            .unwrap_err()
            .to_string()
            .contains("Supported gates")
        );
    }

    #[test]
    fn repair_metadata_matches_shell_contract() {
        assert_eq!(repair_commit_message("lint-shell"), "ci: heal lint-shell");
        assert!(
            Regex::new(r"^heal/check-frontmatter-\d{14}$")
                .unwrap()
                .is_match(&repair_branch_name("Check Frontmatter"))
        );
        assert_eq!(
            healable_gates(),
            &[
                "check-frontmatter",
                "lint-python",
                "lint-shell",
                "lint-yaml"
            ]
        );
    }

    #[test]
    fn repair_prompt_is_generated_from_rust_gate_contract() {
        let prompt = repair_prompt(
            &GateFailure {
                name: "lint-shell".to_string(),
                detail: "bad shell".to_string(),
            },
            1,
            2,
        )
        .unwrap();

        assert!(prompt.contains("Gate: lint-shell"));
        assert!(prompt.contains("Attempt: 1 of 2"));
        assert!(prompt.contains("bad shell"));
        assert!(prompt.contains("Targeted validation commands:"));
        assert!(prompt.contains("- lint-shell: find . -name '*.sh'"));
        assert!(prompt.contains(
            "- check-frontmatter: cargo run --locked -p harness-kit-checks -- check-frontmatter --repo ."
        ));
        assert_eq!(
            targeted_validation_commands()
                .iter()
                .map(|(name, _)| *name)
                .collect::<BTreeSet<_>>(),
            healable_gates().iter().copied().collect::<BTreeSet<_>>()
        );
        assert!(
            repair_prompt(
                &GateFailure {
                    name: "lint-shell".to_string(),
                    detail: String::new(),
                },
                0,
                1,
            )
            .unwrap_err()
            .to_string()
            .contains("attempt must be between")
        );
    }

    #[test]
    fn snapshot_delta_reports_changed_paths_and_ignores_metadata() {
        let before = tempfile::tempdir().unwrap();
        let after = tempfile::tempdir().unwrap();
        fs::write(before.path().join("same.txt"), "same\n").unwrap();
        fs::write(after.path().join("same.txt"), "same\n").unwrap();
        fs::write(before.path().join("changed.txt"), "before\n").unwrap();
        fs::write(after.path().join("changed.txt"), "after\n").unwrap();
        fs::write(before.path().join("removed.txt"), "gone\n").unwrap();
        fs::write(after.path().join("added.txt"), "new\n").unwrap();
        fs::write(before.path().join(".env"), "before\n").unwrap();
        fs::write(after.path().join(".env"), "after\n").unwrap();

        let (stage, remove) = snapshot_delta(before.path(), after.path()).unwrap();
        assert_eq!(stage, ["added.txt", "changed.txt"]);
        assert_eq!(remove, ["removed.txt"]);
    }
}
