use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalSkillLintReport {
    pub external_root: PathBuf,
    pub total: usize,
    pub dirty: Vec<DirtyAlias>,
    pub missing_external_root: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyAlias {
    pub alias: String,
    pub violations: Vec<String>,
}

impl ExternalSkillLintReport {
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    pub fn clean_count(&self) -> usize {
        self.total.saturating_sub(self.dirty_count())
    }
}

pub fn lint_repo(repo_root: &Path) -> Result<ExternalSkillLintReport> {
    lint_external_root(&repo_root.join("skills/.external"))
}

pub fn lint_external_root(external_root: &Path) -> Result<ExternalSkillLintReport> {
    if !external_root.is_dir() {
        return Ok(ExternalSkillLintReport {
            external_root: external_root.to_path_buf(),
            total: 0,
            dirty: Vec::new(),
            missing_external_root: true,
        });
    }
    let mut total = 0;
    let mut dirty = Vec::new();
    for entry in sorted_dirs(external_root)? {
        let alias = entry
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        if alias == "_checkouts" {
            continue;
        }
        total += 1;
        let violations = violations_for_alias(external_root, &entry)?;
        if !violations.is_empty() {
            dirty.push(DirtyAlias { alias, violations });
        }
    }
    Ok(ExternalSkillLintReport {
        external_root: external_root.to_path_buf(),
        total,
        dirty,
        missing_external_root: false,
    })
}

pub fn render(report: &ExternalSkillLintReport) -> String {
    if report.missing_external_root {
        return "no .external/ -- run harness-kit-checks sync-external first\n".to_string();
    }
    let mut output = String::new();
    for alias in &report.dirty {
        output.push_str(&format!(
            "{} -- {} violation(s)\n",
            alias.alias,
            alias.violations.len()
        ));
        for violation in alias.violations.iter().take(3) {
            output.push_str(&format!("  {violation}\n"));
        }
        if alias.violations.len() > 3 {
            output.push_str(&format!("  ... +{} more\n", alias.violations.len() - 3));
        }
        output.push('\n');
    }
    output.push_str(&format!(
        "{} / {} aliases self-contained\n",
        report.clean_count(),
        report.total
    ));
    if !report.dirty.is_empty() {
        let aliases = report
            .dirty
            .iter()
            .map(|dirty| dirty.alias.as_str())
            .collect::<Vec<_>>()
            .join(" ");
        output.push_str(&format!(
            "{} alias(es) with self-containment violations: {}\n",
            report.dirty_count(),
            aliases
        ));
        output
            .push_str("These skills hardcode user-specific paths and will break when symlinked\n");
        output.push_str("into a foreign project. See registry.yaml notes per source.\n");
    }
    output
}

fn violations_for_alias(external_root: &Path, alias_dir: &Path) -> Result<Vec<String>> {
    let path_patterns = [
        Regex::new(r"/Users/").unwrap(),
        Regex::new(r"/home/[a-z]").unwrap(),
        Regex::new(r"\$HOME/\.claude").unwrap(),
        Regex::new(r"\$HOME/\.agents").unwrap(),
        Regex::new(r"~/\.claude").unwrap(),
        Regex::new(r"~/\.agents").unwrap(),
    ];
    let escape_pattern = Regex::new(r"\.\./\.\./\.\./").unwrap();
    let mut violations = Vec::new();
    for path in sorted_files(alias_dir)? {
        if path.file_name().and_then(|name| name.to_str()) == Some(".sync-meta.json") {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        for (line_number, line) in text.lines().enumerate() {
            if path_patterns.iter().any(|pattern| pattern.is_match(line)) {
                violations.push(format_violation(
                    external_root,
                    &path,
                    line_number + 1,
                    line,
                ));
            }
            if is_script_file(&path) && escape_pattern.is_match(line) {
                violations.push(format_violation(
                    external_root,
                    &path,
                    line_number + 1,
                    line,
                ));
            }
        }
    }
    Ok(violations)
}

fn sorted_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if entry.metadata()?.is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn sorted_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files(root, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_files(&path, files)?;
        } else if metadata.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn is_script_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("sh" | "py" | "ts" | "js")
    )
}

fn format_violation(external_root: &Path, path: &Path, line_number: usize, line: &str) -> String {
    let rel = path.strip_prefix(external_root).unwrap_or(path);
    format!("{}:{line_number}:{}", rel.display(), line)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn reports_missing_external_root_as_advisory() {
        let temp = TempDir::new().unwrap();

        let report = lint_external_root(&temp.path().join("skills/.external")).unwrap();

        assert!(report.missing_external_root);
        assert_eq!(
            render(&report),
            "no .external/ -- run harness-kit-checks sync-external first\n"
        );
    }

    #[test]
    fn flags_hardcoded_paths_and_deep_script_escapes() {
        let temp = TempDir::new().unwrap();
        let external = temp.path().join("skills/.external");
        fs::create_dir_all(external.join("clean")).unwrap();
        fs::write(
            external.join("clean/SKILL.md"),
            "Use relative local files.\n",
        )
        .unwrap();
        fs::create_dir_all(external.join("dirty/scripts")).unwrap();
        fs::write(
            external.join("dirty/SKILL.md"),
            "Read /Users/example/private.txt\nAllowed Codex log: ~/.codex/history.jsonl\n",
        )
        .unwrap();
        fs::write(
            external.join("dirty/.sync-meta.json"),
            "{\"path\":\"/Users/ok\"}\n",
        )
        .unwrap();
        fs::write(
            external.join("dirty/scripts/run.sh"),
            "source ../../../shared.sh\n",
        )
        .unwrap();
        fs::create_dir_all(external.join("_checkouts/ignored")).unwrap();
        fs::write(
            external.join("_checkouts/ignored/SKILL.md"),
            "/Users/ignored\n",
        )
        .unwrap();

        let report = lint_external_root(&external).unwrap();
        let rendered = render(&report);

        assert_eq!(report.total, 2);
        assert_eq!(report.dirty_count(), 1);
        assert_eq!(report.dirty[0].alias, "dirty");
        assert_eq!(report.dirty[0].violations.len(), 2);
        assert!(rendered.contains("1 / 2 aliases self-contained"));
        assert!(rendered.contains("dirty -- 2 violation(s)"));
        assert!(!rendered.contains("~/.codex/history.jsonl"));
        assert!(!rendered.contains(".sync-meta.json"));
        assert!(!rendered.contains("_checkouts"));
    }
}
