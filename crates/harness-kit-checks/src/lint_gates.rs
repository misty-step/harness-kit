use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateReport {
    pub ok_message: String,
    pub errors: Vec<String>,
}

impl GateReport {
    pub fn success(message: impl Into<String>) -> Self {
        Self {
            ok_message: message.into(),
            errors: Vec::new(),
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        Self {
            ok_message: String::new(),
            errors,
        }
    }
}

pub fn check_exclusions(root: &Path) -> Result<GateReport> {
    let patterns = [
        (Regex::new(r"@ts-ignore")?, "TypeScript @ts-ignore"),
        (
            Regex::new(r"@ts-expect-error")?,
            "TypeScript @ts-expect-error",
        ),
        (Regex::new(r"\bas\s+any\b")?, "TypeScript as any"),
        (Regex::new(r":\s*any\b")?, "TypeScript : any"),
        (Regex::new(r"\.skip\s*\(")?, "Test .skip()"),
        (Regex::new(r"\bxit\s*\(")?, "xit()"),
        (Regex::new(r"\bxdescribe\s*\(")?, "xdescribe()"),
    ];
    let skip = [
        "hooks/",
        "coverage/",
        "dist/",
        ".next/",
        "node_modules/",
        "ci/",
    ];
    let mut findings = Vec::new();

    for path in files(root)? {
        let relative = relative_slash(root, &path);
        if !matches_extension(&relative, &["ts", "tsx", "js", "jsx", "py"])
            || contains_any(&relative, &skip)
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (lineno, line) in text.lines().enumerate() {
            for (regex, label) in &patterns {
                if regex.is_match(line) {
                    findings.push(format!("  {relative}:{}: {label}", lineno + 1));
                }
            }
            if line.contains("eslint-disable") && !line.contains("--") {
                findings.push(format!("  {relative}:{}: ESLint disable", lineno + 1));
            }
        }
    }

    if findings.is_empty() {
        Ok(GateReport::success("No exclusion patterns found."))
    } else {
        let mut errors = vec![format!("Found {} exclusion(s):", findings.len())];
        errors.extend(findings.into_iter().take(20));
        Ok(GateReport::failure(errors))
    }
}

pub fn check_conflict_markers(root: &Path) -> Result<GateReport> {
    let skip = [
        ".dagger",
        ".git",
        ".venv",
        "__pycache__",
        "node_modules",
        "skills/.external",
    ];
    let markers = ["<<<<<<<", "=======", ">>>>>>>"];
    let mut findings = Vec::new();

    for path in files(root)? {
        let relative = relative_slash(root, &path);
        if contains_any(&relative, &skip) {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (lineno, line) in text.lines().enumerate() {
            let stripped = line.trim();
            if markers.contains(&stripped)
                || line.starts_with("<<<<<<< ")
                || line.starts_with(">>>>>>> ")
            {
                findings.push(format!("  {relative}:{}: {stripped}", lineno + 1));
            }
        }
    }

    if findings.is_empty() {
        Ok(GateReport::success("No unresolved conflict markers found."))
    } else {
        let mut errors = vec!["Found unresolved conflict marker(s):".to_string()];
        errors.extend(findings.into_iter().take(40));
        Ok(GateReport::failure(errors))
    }
}

pub fn check_portable_paths(root: &Path) -> Result<GateReport> {
    let home = Regex::new(r"/Users/[a-zA-Z0-9_-]+/")?;
    let windows = Regex::new(r"C:\\Users\\[a-zA-Z0-9_-]+\\")?;
    let allow = [
        ".claude/hooks",
        "coverage/",
        ".next/",
        "dist/",
        "harnesses/claude/",
    ];
    let mut findings = Vec::new();

    for path in files(root)? {
        let relative = relative_slash(root, &path);
        if contains_any(&relative, &allow)
            || !(matches_extension(&relative, &["sh", "bash", "zsh"])
                || relative.ends_with("/Makefile")
                || relative == "Makefile"
                || relative
                    .rsplit('/')
                    .next()
                    .is_some_and(|name| name.starts_with(".env")))
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (lineno, line) in text.lines().enumerate() {
            if let Some(found) = home.find(line).or_else(|| windows.find(line)) {
                findings.push(format!("  {relative}:{}: {}", lineno + 1, found.as_str()));
            }
        }
    }

    if findings.is_empty() {
        Ok(GateReport::success("No hardcoded user paths found."))
    } else {
        let mut errors = vec![format!("Found {} hardcoded path(s):", findings.len())];
        errors.extend(findings.into_iter().take(20));
        Ok(GateReport::failure(errors))
    }
}

pub fn check_deliver_composition(root: &Path) -> Result<GateReport> {
    let target = root.join("skills/deliver/SKILL.md");
    let target_label = "skills/deliver/SKILL.md";
    if !target.exists() {
        return Ok(GateReport::success(format!(
            "{target_label} not present; skipping deliver-composition lint."
        )));
    }
    let denylist = [
        (
            Regex::new(r"\bsource\s+scripts/lib/claims\.sh\b")?,
            "claims.sh sourcing (dropped primitive)",
        ),
        (
            Regex::new(r"\bclaim_(acquire|release)\b")?,
            "claim_acquire/claim_release (dropped primitive)",
        ),
        (
            Regex::new(r"\bdagger\s+call\s+check\b")?,
            "raw `dagger call check` - use /ci instead",
        ),
        (
            Regex::new(r"\bbunx?\s+playwright\b")?,
            "raw playwright invocation - use /qa instead",
        ),
        (
            Regex::new(r"\bnpx\s+playwright\b")?,
            "raw playwright invocation - use /qa instead",
        ),
        (
            Regex::new(r#"Agent\s*\(\s*['"](?:critic|ousterhout|carmack|grug|beck)['"]"#)?,
            "direct bench-agent dispatch - use /code-review instead",
        ),
        (
            Regex::new(r#"subagent_type\s*=\s*['"](?:critic|ousterhout|carmack|grug|beck)['"]"#)?,
            "direct bench-agent dispatch - use /code-review instead",
        ),
    ];
    let text = fs::read_to_string(&target)
        .with_context(|| format!("failed to read {}", target.display()))?;
    let mut findings = Vec::new();

    for (lineno, line) in text.lines().enumerate() {
        let stripped = line.trim_start();
        if stripped.starts_with('#') || stripped.starts_with('>') || stripped.starts_with("<!--") {
            continue;
        }
        for (regex, label) in &denylist {
            if regex.is_match(line) {
                findings.push(format!(
                    "  {target_label}:{}: {label}\n    {}",
                    lineno + 1,
                    truncate_chars(stripped, 120)
                ));
            }
        }
    }

    if findings.is_empty() {
        Ok(GateReport::success(format!(
            "{target_label}: composition clean (no inlined-phase calls)."
        )))
    } else {
        let mut errors = vec![format!(
            "Found {} inlined-phase violation(s) in {target_label}:",
            findings.len()
        )];
        errors.extend(findings);
        errors.push(String::new());
        errors.push("/deliver must compose atomic phase skills via trigger syntax,".to_string());
        errors.push("not re-implement their internals. See backlog.d/032.".to_string());
        Ok(GateReport::failure(errors))
    }
}

pub fn check_no_claims(root: &Path) -> Result<GateReport> {
    let skills = root.join("skills");
    if !skills.exists() {
        return Ok(GateReport::success(
            "skills/ not present; skipping no-claims lint.",
        ));
    }
    let patterns = [
        (Regex::new(r"\bclaims\.sh\b")?, "claims.sh reference"),
        (Regex::new(r"\bclaim_acquire\b")?, "claim_acquire call"),
        (Regex::new(r"\bclaim_release\b")?, "claim_release call"),
    ];
    let mut findings = Vec::new();

    for path in files(&skills)? {
        let relative = relative_slash(root, &path);
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (lineno, line) in text.lines().enumerate() {
            for (regex, label) in &patterns {
                if regex.is_match(line) {
                    findings.push(format!("  {relative}:{}: {label}", lineno + 1));
                }
            }
        }
    }

    if findings.is_empty() {
        Ok(GateReport::success("skills/: no claims primitives found."))
    } else {
        let mut errors = vec![format!(
            "Found {} claims-primitive reference(s) under skills/:",
            findings.len()
        )];
        errors.extend(findings.into_iter().take(40));
        errors.push(String::new());
        errors.push("Claim coordination was dropped per backlog.d/032.".to_string());
        errors.push(
            "Do not reintroduce claims.sh / claim_acquire / claim_release in skills/.".to_string(),
        );
        Ok(GateReport::failure(errors))
    }
}

pub fn check_vendored_copies(root: &Path) -> Result<GateReport> {
    let legacy_python_copies = [
        "scripts/lib/search_core.py",
        "skills/focus/scripts/search_core.py",
        "scripts/gemini_embeddings.py",
        "skills/focus/scripts/gemini_embeddings.py",
    ];
    let mut findings = Vec::new();

    for path in legacy_python_copies {
        if root.join(path).exists() {
            findings.push(format!(
                "LEGACY: {path} should be Rust-backed, not vendored Python"
            ));
        }
    }

    if findings.is_empty() {
        Ok(GateReport::success(
            "No legacy Python vendored copies found.",
        ))
    } else {
        let count = findings.len();
        findings.push(format!(
            "{count} legacy Python vendored file(s) remain. Keep embedding/search helpers in Rust."
        ));
        Ok(GateReport::failure(findings))
    }
}

pub fn check_harness_install_paths(root: &Path) -> Result<GateReport> {
    let mut failures = Vec::new();

    if matches_any_file(
        root,
        r"GLOBAL_SKILLS=\(tailor seed\)|minimal global|/tailor or /seed|per-repo via /tailor",
        &["bootstrap.sh", "README.md", "AGENTS.md", "CODEBASE.md"],
    )? {
        failures.push(
            "global install docs/scripts must not describe the retired minimal tailor/seed model"
                .to_string(),
        );
    }

    if !matches_any_file(
        root,
        r"All first-party skills are installed system-wide",
        &["bootstrap.sh", "crates/harness-kit-checks/src/bootstrap.rs"],
    )? {
        failures.push(
            "bootstrap must report the all-first-party-skills system-wide install contract"
                .to_string(),
        );
    }

    if !matches_any_file(
        root,
        r"install_system_roster|~/.harness-kit/agents.yaml|\$HOME/.harness-kit|\.harness-kit/agents.yaml",
        &["bootstrap.sh", "crates/harness-kit-checks/src/bootstrap.rs"],
    )? {
        failures.push(
            "bootstrap must install the provider roster into a system-wide Harness Kit location"
                .to_string(),
        );
    }

    if !matches_any_file(
        root,
        r"cargo install --quiet --locked --path.*crates/harness-kit-checks|bin/harness-kit-checks",
        &["bootstrap.sh", "crates/harness-kit-checks/src/bootstrap.rs"],
    )? {
        failures.push(
            "bootstrap must install the Rust Harness Kit CLI into the system-wide Harness Kit location"
                .to_string(),
        );
    }

    if !matches_any_file(
        root,
        r"legacy_system_dir=.*\.spellbook|legacy agents.yaml",
        &["bootstrap.sh", "crates/harness-kit-checks/src/bootstrap.rs"],
    )? {
        failures.push(
            "bootstrap must keep a legacy Spellbook roster alias for long-running stale instruction contexts"
                .to_string(),
        );
    }

    if !matches_any_file(
        root,
        r"HARNESS_KIT_ROSTER|HARNESS_KIT_ROSTER_PATH|\.harness-kit.*agents.yaml",
        &["crates/harness-kit-checks/src/main.rs"],
    )? {
        failures.push(
            "roster helpers must fall back to a system-wide roster when repo-local roster is absent"
                .to_string(),
        );
    }

    if !matches_any_file(root, r"\+skills/\*\*", &["harnesses/pi/settings.json"])? {
        failures
            .push("Pi settings must allow all globally installed first-party skills".to_string());
    }

    if failures.is_empty() {
        Ok(GateReport::success(
            "harness install paths are cross-harness and global-skill first.",
        ))
    } else {
        let mut errors = vec!["harness install path check failed:".to_string()];
        errors.extend(failures.into_iter().map(|failure| format!("  - {failure}")));
        Ok(GateReport::failure(errors))
    }
}

fn files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_files(root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_files(root: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    if !root.exists() {
        return Ok(());
    }
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, paths)?;
        } else if path.is_file() {
            paths.push(path);
        }
    }
    Ok(())
}

fn contains_any(path: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| path.contains(needle))
}

fn matches_extension(path: &str, extensions: &[&str]) -> bool {
    extensions
        .iter()
        .any(|extension| path.ends_with(&format!(".{extension}")))
}

fn relative_slash(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
        .trim_start_matches("./")
        .to_string()
}

fn truncate_chars(value: &str, max: usize) -> String {
    value.chars().take(max).collect()
}

fn matches_any_file(root: &Path, pattern: &str, rel_paths: &[&str]) -> Result<bool> {
    let regex = Regex::new(pattern)?;
    for rel_path in rel_paths {
        let path = root.join(rel_path);
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        if regex.is_match(&text) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn exclusions_find_type_escape_and_skip_ci() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("src")).unwrap();
        fs::create_dir_all(temp.path().join("ci")).unwrap();
        fs::write(
            temp.path().join("src/app.ts"),
            "const value = item as any;\n",
        )
        .unwrap();
        fs::write(
            temp.path().join("ci/fixture.ts"),
            "const value = item as any;\n",
        )
        .unwrap();

        let report = check_exclusions(temp.path()).unwrap();
        assert!(report.errors[0].contains("Found 1 exclusion"));
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("src/app.ts:1"))
        );
        assert!(
            !report
                .errors
                .iter()
                .any(|error| error.contains("ci/fixture.ts"))
        );
    }

    #[test]
    fn conflict_markers_ignore_binary_and_report_marker() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("note.md"), "<<<<<<< HEAD\n").unwrap();
        fs::write(temp.path().join("image.bin"), [0xff, 0x00]).unwrap();

        let report = check_conflict_markers(temp.path()).unwrap();
        assert_eq!(report.errors[0], "Found unresolved conflict marker(s):");
        assert!(report.errors[1].contains("note.md:1"));
    }

    #[test]
    fn portable_paths_scan_shell_and_allow_claude_harness() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("scripts")).unwrap();
        fs::create_dir_all(temp.path().join("harnesses/claude")).unwrap();
        fs::write(
            temp.path().join("scripts/run.sh"),
            "cd /Users/alice/project\n",
        )
        .unwrap();
        fs::write(
            temp.path().join("harnesses/claude/run.sh"),
            "cd /Users/alice/project\n",
        )
        .unwrap();

        let report = check_portable_paths(temp.path()).unwrap();
        assert!(report.errors[0].contains("Found 1 hardcoded path"));
        assert!(report.errors[1].contains("scripts/run.sh:1: /Users/alice/"));
    }

    #[test]
    fn deliver_composition_flags_raw_ci_command() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("skills/deliver")).unwrap();
        fs::write(
            temp.path().join("skills/deliver/SKILL.md"),
            "Run dagger call check --source=. directly.\n",
        )
        .unwrap();

        let report = check_deliver_composition(temp.path()).unwrap();
        assert!(report.errors[0].contains("Found 1 inlined-phase violation"));
        assert!(report.errors[1].contains("raw `dagger call check`"));
    }

    #[test]
    fn no_claims_flags_claim_primitive_under_skills() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("skills/demo")).unwrap();
        fs::write(
            temp.path().join("skills/demo/SKILL.md"),
            "claim_acquire demo\n",
        )
        .unwrap();

        let report = check_no_claims(temp.path()).unwrap();
        assert!(report.errors[0].contains("Found 1 claims-primitive"));
        assert!(report.errors[1].contains("skills/demo/SKILL.md:1"));
    }

    #[test]
    fn vendored_copies_reject_legacy_python_embedding_helpers() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("scripts/lib")).unwrap();
        fs::write(temp.path().join("scripts/lib/search_core.py"), "legacy\n").unwrap();

        let report = check_vendored_copies(temp.path()).unwrap();
        assert!(report.errors[0].contains("LEGACY: scripts/lib/search_core.py"));
        assert!(report.errors[1].contains("1 legacy Python vendored file"));
    }

    #[test]
    fn harness_install_paths_reports_missing_contracts() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("crates/harness-kit-checks/src")).unwrap();
        fs::create_dir_all(temp.path().join("harnesses/pi")).unwrap();
        fs::write(temp.path().join("bootstrap.sh"), "minimal global\n").unwrap();
        fs::write(temp.path().join("README.md"), "").unwrap();
        fs::write(temp.path().join("AGENTS.md"), "").unwrap();
        fs::write(temp.path().join("CODEBASE.md"), "").unwrap();
        fs::write(
            temp.path().join("crates/harness-kit-checks/src/main.rs"),
            "",
        )
        .unwrap();
        fs::write(temp.path().join("harnesses/pi/settings.json"), "{}\n").unwrap();

        let report = check_harness_install_paths(temp.path()).unwrap();
        assert_eq!(report.errors[0], "harness install path check failed:");
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("retired minimal tailor/seed"))
        );
        assert!(
            report
                .errors
                .iter()
                .any(|error| error.contains("Pi settings must allow"))
        );
    }
}
