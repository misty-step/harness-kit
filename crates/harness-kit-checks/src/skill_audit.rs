use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Check {
    pub passed: bool,
    pub detail: String,
}

impl Check {
    fn pass(detail: impl Into<String>) -> Self {
        Self {
            passed: true,
            detail: detail.into(),
        }
    }

    fn fail(detail: impl Into<String>) -> Self {
        Self {
            passed: false,
            detail: detail.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillAudit {
    pub name: String,
    pub frontmatter: Check,
    pub trigger: Check,
    pub tests: Check,
    pub routed: Check,
}

impl SkillAudit {
    pub fn failures(&self) -> usize {
        [&self.frontmatter, &self.trigger, &self.tests, &self.routed]
            .iter()
            .filter(|check| !check.passed)
            .count()
    }
}

pub fn repo_root(start: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .arg("-C")
        .arg(start)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .with_context(|| "failed to run git rev-parse --show-toplevel")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        bail!(
            "{}",
            if stderr.is_empty() {
                "not inside a git repository".to_string()
            } else {
                stderr
            }
        );
    }
    Ok(PathBuf::from(String::from_utf8(output.stdout)?.trim()).canonicalize()?)
}

pub fn audit_repo(root: &Path) -> Result<(Vec<SkillAudit>, Vec<PathBuf>)> {
    let skills_root = root.join("skills");
    let routing_paths: Vec<PathBuf> = [root.join("harnesses/shared/AGENTS.md")]
        .into_iter()
        .filter(|path| path.is_file())
        .collect();
    let routing_docs: Vec<(PathBuf, String)> = routing_paths
        .iter()
        .map(|path| {
            Ok((
                path.strip_prefix(root).unwrap_or(path).to_path_buf(),
                fs::read_to_string(path)
                    .with_context(|| format!("failed to read {}", path.display()))?,
            ))
        })
        .collect::<Result<_>>()?;

    let mut skill_dirs = Vec::new();
    for entry in fs::read_dir(&skills_root)
        .with_context(|| format!("failed to read {}", skills_root.display()))?
    {
        let path = entry?.path();
        if path.join("SKILL.md").is_file() {
            skill_dirs.push(path);
        }
    }
    skill_dirs.sort();

    let audits = skill_dirs
        .iter()
        .map(|skill_dir| audit_skill(skill_dir, &routing_docs))
        .collect::<Result<Vec<_>>>()?;
    Ok((
        audits,
        routing_paths
            .iter()
            .map(|path| path.strip_prefix(root).unwrap_or(path).to_path_buf())
            .collect(),
    ))
}

pub fn frontmatter(text: &str) -> BTreeMap<String, String> {
    if !text.starts_with("---\n") {
        return BTreeMap::new();
    }
    let Some(raw) = text.split("---").nth(1) else {
        return BTreeMap::new();
    };

    let mut data = BTreeMap::new();
    let mut current: Option<String> = None;
    for line in raw.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if line.starts_with([' ', '\t']) {
            if let Some(current_key) = &current {
                let existing = data.get(current_key).cloned().unwrap_or_default();
                data.insert(
                    current_key.clone(),
                    format!("{} {}", existing, line.trim()).trim().to_string(),
                );
            }
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            current = None;
            continue;
        };
        let key = key.trim().to_string();
        let mut value = value.trim().to_string();
        if value == "|" || value == ">" {
            value.clear();
        }
        data.insert(key.clone(), value.trim_matches(['"', '\'']).to_string());
        current = Some(key);
    }
    data
}

pub fn render_report(audits: &[SkillAudit], routing_paths: &[PathBuf]) -> String {
    let mut ordered = audits.to_vec();
    ordered.sort_by(|left, right| {
        right
            .failures()
            .cmp(&left.failures())
            .then_with(|| left.name.cmp(&right.name))
    });
    let counts = (0..=4)
        .map(|failures| {
            (
                failures,
                audits
                    .iter()
                    .filter(|audit| audit.failures() == failures)
                    .count(),
            )
        })
        .collect::<BTreeMap<_, _>>();

    let mut lines = vec![
        "# Skill Quality Audit".to_string(),
        String::new(),
        format!("Skills audited: {}", audits.len()),
        format!(
            "Routing docs: {}",
            if routing_paths.is_empty() {
                "none".to_string()
            } else {
                routing_paths
                    .iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            }
        ),
        String::new(),
        "## Summary".to_string(),
        String::new(),
        "| Failed dimensions | Skills |".to_string(),
        "|---:|---:|".to_string(),
    ];
    for failures in (0..=4).rev() {
        lines.push(format!(
            "| {failures} | {} |",
            counts.get(&failures).copied().unwrap_or(0)
        ));
    }

    lines.extend([String::new(), "## Findings".to_string(), String::new()]);
    for audit in ordered {
        let verdict = if audit.failures() == 0 {
            "PASS".to_string()
        } else {
            format!("FAIL {}/4", audit.failures())
        };
        lines.push(format!("### {} - {verdict}", audit.name));
        for (label, check) in [
            ("frontmatter", audit.frontmatter),
            ("trigger", audit.trigger),
            ("tests", audit.tests),
            ("routing", audit.routed),
        ] {
            let mark = if check.passed { "PASS" } else { "FAIL" };
            lines.push(format!("- {label}: {mark} - {}", check.detail));
        }
        lines.push(String::new());
    }
    lines.join("\n").trim_end().to_string() + "\n"
}

fn audit_skill(skill_dir: &Path, routing_docs: &[(PathBuf, String)]) -> Result<SkillAudit> {
    let skill_md = skill_dir.join("SKILL.md");
    let text = fs::read_to_string(&skill_md)
        .with_context(|| format!("failed to read {}", skill_md.display()))?;
    let meta = frontmatter(&text);
    let name = skill_dir
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| anyhow!("invalid skill directory {}", skill_dir.display()))?
        .to_string();
    let desc = meta
        .get("description")
        .map(|value| value.split_whitespace().collect::<Vec<_>>().join(" "))
        .unwrap_or_default();
    let missing: Vec<_> = ["name", "description"]
        .into_iter()
        .filter(|field| meta.get(*field).is_none_or(String::is_empty))
        .collect();
    let frontmatter = if missing.is_empty() {
        match meta.get("name") {
            Some(value) if value != &name => Check::fail(format!(
                "frontmatter name {} differs from directory {}",
                py_repr(value),
                py_repr(&name)
            )),
            _ => Check::pass("name and description present"),
        }
    } else {
        Check::fail(format!(
            "missing frontmatter field(s): {}",
            missing.join(", ")
        ))
    };

    let trigger = trigger_check(&desc)?;
    Ok(SkillAudit {
        name: name.clone(),
        frontmatter,
        trigger,
        tests: has_testing_evidence(skill_dir, &text)?,
        routed: is_routed(&name, routing_docs),
    })
}

fn trigger_check(desc: &str) -> Result<Check> {
    let generic = Regex::new(r"(?i)^\s*(use this skill to|this skill|helps?|guides?)\b")?;
    let trigger = Regex::new(
        r"(?i)\b(use when|use for|triggered by|when the user|applies when|invoke when|for tasks involving)\b|\btriggers?:",
    )?;
    if desc.is_empty() {
        Ok(Check::fail("description missing"))
    } else if generic.is_match(desc) && !trigger.is_match(desc) {
        Ok(Check::fail(
            "description is generic and lacks trigger language",
        ))
    } else if !trigger.is_match(desc) {
        Ok(Check::fail("description lacks concrete trigger phrase"))
    } else {
        Ok(Check::pass("description has trigger language"))
    }
}

fn has_testing_evidence(skill_dir: &Path, body: &str) -> Result<Check> {
    let mut evidence = Vec::new();
    for dirname in ["tests", "test", "evals"] {
        let path = skill_dir.join(dirname);
        if path.is_dir() && has_any_file_recursive(&path)? {
            evidence.push(format!("{dirname}/"));
        }
    }
    let scripts_dir = skill_dir.join("scripts");
    if scripts_dir.is_dir()
        && let Some(pattern) = test_script_pattern(&recursive_files(&scripts_dir)?)
    {
        evidence.push(format!("scripts/{pattern}"));
    }
    if Regex::new(r"(?im)^#{1,3}\s*(testing|verification|evals?)\b")?.is_match(body) {
        evidence.push("SKILL.md testing section".to_string());
    }

    if evidence.is_empty() {
        Ok(Check::fail(
            "no tests/, evals/, test script, or Testing/Verification section",
        ))
    } else {
        evidence.sort();
        evidence.dedup();
        Ok(Check::pass(evidence.join(", ")))
    }
}

fn is_routed(name: &str, docs: &[(PathBuf, String)]) -> Check {
    let needles = [
        format!("/{name}"),
        format!("skills/{name}"),
        format!("`{name}`"),
        format!("| `{name}`"),
        format!("| /{name}"),
    ];
    let hits: Vec<_> = docs
        .iter()
        .filter(|(_, text)| needles.iter().any(|needle| text.contains(needle)))
        .map(|(path, _)| path.display().to_string())
        .collect();
    if hits.is_empty() {
        Check::fail("not referenced from harnesses/shared/AGENTS.md routing")
    } else {
        Check::pass(hits.join(", "))
    }
}

fn has_any_file_recursive(path: &Path) -> Result<bool> {
    Ok(recursive_files(path)?.iter().any(|path| path.is_file()))
}

fn recursive_files(path: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    collect_files(path, &mut paths)?;
    Ok(paths)
}

fn collect_files(path: &Path, paths: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, paths)?;
        } else if path.is_file() {
            paths.push(path);
        }
    }
    Ok(())
}

fn test_script_pattern(paths: &[PathBuf]) -> Option<&'static str> {
    for (pattern, predicate) in [
        ("test_*.sh", is_test_sh as fn(&str) -> bool),
        ("*_test.sh", is_sh_test as fn(&str) -> bool),
        ("test_*.py", is_test_py as fn(&str) -> bool),
        ("*_test.py", is_py_test as fn(&str) -> bool),
    ] {
        if paths.iter().any(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(predicate)
        }) {
            return Some(pattern);
        }
    }
    None
}

fn is_test_sh(name: &str) -> bool {
    name.starts_with("test_") && name.ends_with(".sh")
}

fn is_sh_test(name: &str) -> bool {
    name.ends_with("_test.sh")
}

fn is_test_py(name: &str) -> bool {
    name.starts_with("test_") && name.ends_with(".py")
}

fn is_py_test(name: &str) -> bool {
    name.ends_with("_test.py")
}

fn py_repr(value: &str) -> String {
    if value.contains('\'') && !value.contains('"') {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_orders_by_severity_and_uses_shared_routing() {
        let temp = tempfile::tempdir().unwrap();
        setup_repo(temp.path());

        let (audits, routing_paths) = audit_repo(temp.path()).unwrap();
        let report = render_report(&audits, &routing_paths);
        let weak_line = report.find("### weak").unwrap();
        let good_line = report.find("### good").unwrap();

        assert!(weak_line < good_line);
        assert!(report.contains("description is generic"));
        assert!(report.contains("no tests/, evals/"));
        assert!(report.contains("harnesses/shared/AGENTS.md"));
        assert!(!report.contains("root project guidance"));
    }

    #[test]
    fn report_is_reproducible() {
        let temp = tempfile::tempdir().unwrap();
        setup_repo(temp.path());

        let (audits, routing_paths) = audit_repo(temp.path()).unwrap();
        let first = render_report(&audits, &routing_paths);
        let (audits, routing_paths) = audit_repo(temp.path()).unwrap();
        let second = render_report(&audits, &routing_paths);

        assert_eq!(first, second);
    }

    #[test]
    fn output_report_shape_matches_contract() {
        let temp = tempfile::tempdir().unwrap();
        setup_repo(temp.path());

        let (audits, routing_paths) = audit_repo(temp.path()).unwrap();
        let report = render_report(&audits, &routing_paths);

        assert!(report.starts_with("# Skill Quality Audit\n\nSkills audited: 2\n"));
        assert!(report.contains("| Failed dimensions | Skills |"));
        assert!(report.ends_with('\n'));
    }

    #[test]
    fn name_mismatch_uses_python_repr_style() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("skills/wrong")).unwrap();
        fs::create_dir_all(temp.path().join("harnesses/shared")).unwrap();
        fs::write(temp.path().join("harnesses/shared/AGENTS.md"), "").unwrap();
        fs::write(
            temp.path().join("skills/wrong/SKILL.md"),
            "---\nname: other\ndescription: Use when: testing mismatch.\n---\n",
        )
        .unwrap();

        let (audits, _) = audit_repo(temp.path()).unwrap();

        assert_eq!(
            audits[0].frontmatter.detail,
            "frontmatter name 'other' differs from directory 'wrong'"
        );
    }

    fn setup_repo(root: &Path) {
        fs::create_dir_all(root.join("skills/good/scripts")).unwrap();
        fs::create_dir_all(root.join("skills/weak")).unwrap();
        fs::create_dir_all(root.join("harnesses/shared")).unwrap();
        fs::write(
            root.join("AGENTS.md"),
            "# root project guidance should not satisfy shared routing\n",
        )
        .unwrap();
        fs::write(
            root.join("harnesses/shared/AGENTS.md"),
            "| Skill | Role |\n|---|---|\n| `/good` | Routed skill. |\n",
        )
        .unwrap();
        fs::write(
            root.join("skills/good/SKILL.md"),
            "---\nname: good\ndescription: |\n  Use when: testing the audit helper on a well-shaped skill.\n---\n\n# good\n",
        )
        .unwrap();
        fs::write(
            root.join("skills/good/scripts/test_good.sh"),
            "#!/usr/bin/env bash\nexit 0\n",
        )
        .unwrap();
        fs::write(
            root.join("skills/weak/SKILL.md"),
            "---\nname: weak\ndescription: This skill helps.\n---\n\n# weak\n",
        )
        .unwrap();
    }
}
