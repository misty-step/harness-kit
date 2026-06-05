use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;
use serde::Deserialize;

const BENCH_MAP: &str = "skills/code-review/references/bench-map.yaml";
const LENSES: &str = "harnesses/shared/references/lenses.md";
const AGENTS: &str = "agents";

#[derive(Debug, Clone, Deserialize)]
pub struct BenchMap {
    #[serde(default)]
    pub default: Vec<String>,
    #[serde(default)]
    pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Rule {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub paths: Vec<String>,
    #[serde(default)]
    pub add: Vec<String>,
    #[serde(default)]
    pub replace: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchMapReport {
    pub lines: Vec<String>,
}

pub fn run(repo: &Path) -> Result<BenchMapReport> {
    let config = load_config(repo)?;
    let mut errors = validate_ids(repo, &config)?;

    let security = selected_reviewers(&["src/auth/login.ts"], &config)?;
    if !security.iter().any(|reviewer| reviewer == "security") {
        errors.push("security fixture did not select security".to_string());
    }
    if security.iter().any(|reviewer| reviewer == "grug") {
        errors.push("security fixture did not replace grug".to_string());
    }
    if security.len() > 5 {
        errors.push("security fixture exceeded bench cap".to_string());
    }

    let tests = selected_reviewers(&["tests/auth.spec.ts"], &config)?;
    if !tests.iter().any(|reviewer| reviewer == "cooper") {
        errors.push("tests fixture did not select cooper".to_string());
    }
    if tests.len() > 5 {
        errors.push("tests fixture exceeded bench cap".to_string());
    }

    if !errors.is_empty() {
        anyhow::bail!(
            "{}",
            errors
                .into_iter()
                .map(|error| format!("FAIL: {error}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    Ok(BenchMapReport {
        lines: vec!["OK: bench-map reviewer ids and replacement fixtures valid".to_string()],
    })
}

pub fn load_config(repo: &Path) -> Result<BenchMap> {
    let path = repo.join(BENCH_MAP);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", display_path(&path)))?;
    serde_yaml::from_str(&text).with_context(|| format!("failed to parse {}", display_path(&path)))
}

pub fn selected_reviewers(changed_files: &[&str], config: &BenchMap) -> Result<Vec<String>> {
    let mut selected = config.default.clone();

    for rule in &config.rules {
        if !changed_files.iter().any(|path| {
            rule.paths
                .iter()
                .any(|pattern| matches_pattern(path, pattern).unwrap_or(false))
        }) {
            continue;
        }

        let replace: HashSet<&str> = rule.replace.iter().map(String::as_str).collect();
        selected.retain(|reviewer| !replace.contains(reviewer.as_str()));
        selected.extend(rule.add.iter().cloned());
    }

    selected = unique(selected);
    if !selected.iter().any(|reviewer| reviewer == "critic") {
        selected.insert(0, "critic".to_string());
    }

    if selected.len() > 5 {
        let mut capped = Vec::new();
        if selected.iter().any(|reviewer| reviewer == "critic") {
            capped.push("critic".to_string());
        }
        for reviewer in selected.into_iter().filter(|reviewer| reviewer != "critic") {
            if capped.len() >= 5 {
                break;
            }
            capped.push(reviewer);
        }
        selected = capped;
    }

    Ok(selected)
}

pub fn validate_ids(repo: &Path, config: &BenchMap) -> Result<Vec<String>> {
    let allowed = known_reviewers(repo)?;
    let mut errors = Vec::new();

    let unknown = unknown_ids(&config.default, &allowed);
    if !unknown.is_empty() {
        errors.push(format!(
            "default: unknown reviewer id(s): {}",
            unknown.join(", ")
        ));
    }

    for rule in &config.rules {
        for (field, reviewers) in [("add", &rule.add), ("replace", &rule.replace)] {
            let unknown = unknown_ids(reviewers, &allowed);
            if unknown.is_empty() {
                continue;
            }
            errors.push(format!(
                "rule {} {field}: unknown reviewer id(s): {}",
                rule.name.as_deref().unwrap_or("<unnamed>"),
                unknown.join(", ")
            ));
        }
    }

    Ok(errors)
}

fn known_reviewers(repo: &Path) -> Result<HashSet<String>> {
    let mut reviewers = lens_names(repo)?;
    let agents_dir = repo.join(AGENTS);
    if agents_dir.exists() {
        for entry in fs::read_dir(&agents_dir)
            .with_context(|| format!("failed to read {}", display_path(&agents_dir)))?
        {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("md")
                && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            {
                reviewers.insert(stem.to_string());
            }
        }
    }
    Ok(reviewers)
}

fn lens_names(repo: &Path) -> Result<HashSet<String>> {
    let path = repo.join(LENSES);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", display_path(&path)))?;
    let heading = Regex::new(r"(?m)^## ([a-z][a-z0-9-]*)\s*$").expect("valid lens regex");
    Ok(heading
        .captures_iter(&text)
        .filter_map(|captures| captures.get(1).map(|name| name.as_str().to_string()))
        .filter(|name| name != "adding-a-lens")
        .collect())
}

fn unknown_ids(reviewers: &[String], allowed: &HashSet<String>) -> Vec<String> {
    let mut unknown: Vec<String> = reviewers
        .iter()
        .filter(|reviewer| !allowed.contains(reviewer.as_str()))
        .cloned()
        .collect::<HashSet<_>>()
        .into_iter()
        .collect();
    unknown.sort();
    unknown
}

fn unique(items: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut result = Vec::new();
    for item in items {
        if seen.insert(item.clone()) {
            result.push(item);
        }
    }
    result
}

fn matches_pattern(path: &str, pattern: &str) -> Result<bool> {
    let regex = Regex::new(&format!("^{}$", fnmatch_regex(pattern)))
        .with_context(|| format!("invalid path pattern {pattern:?}"))?;
    Ok(regex.is_match(path))
}

fn fnmatch_regex(pattern: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = pattern.chars().collect();
    let mut index = 0;
    while index < chars.len() {
        match chars[index] {
            '*' => result.push_str(".*"),
            '?' => result.push('.'),
            '[' => {
                if let Some(end) = chars[index + 1..].iter().position(|ch| *ch == ']') {
                    let class: String = chars[index + 1..index + 1 + end].iter().collect();
                    if !class.is_empty() {
                        result.push('[');
                        result.push_str(&class);
                        result.push(']');
                        index += end + 1;
                    } else {
                        result.push_str(r"\[");
                    }
                } else {
                    result.push_str(r"\[");
                }
            }
            ch => result.push_str(&regex::escape(&ch.to_string())),
        }
        index += 1;
    }
    result
}

fn display_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn fixture_config() -> BenchMap {
        BenchMap {
            default: vec!["critic".into(), "ousterhout".into(), "grug".into()],
            rules: vec![
                Rule {
                    name: Some("auth-and-secrets".into()),
                    paths: vec![
                        "**/auth/**".into(),
                        "**/*auth*.{ts,tsx,js,jsx,py,rs,go,java}".into(),
                    ],
                    replace: vec!["grug".into()],
                    add: vec!["security".into()],
                },
                Rule {
                    name: Some("tests".into()),
                    paths: vec!["**/*.spec.*".into(), "**/tests/**".into()],
                    replace: Vec::new(),
                    add: vec!["cooper".into()],
                },
                Rule {
                    name: Some("typescript".into()),
                    paths: vec!["**/*.ts".into()],
                    replace: Vec::new(),
                    add: vec!["ousterhout".into()],
                },
            ],
        }
    }

    #[test]
    fn security_paths_replace_grug_and_keep_critic() {
        let selected = selected_reviewers(&["src/auth/login.ts"], &fixture_config()).unwrap();
        assert_eq!(selected, vec!["critic", "ousterhout", "security"]);
    }

    #[test]
    fn tests_paths_add_cooper() {
        let selected = selected_reviewers(&["tests/auth.spec.ts"], &fixture_config()).unwrap();
        assert!(selected.contains(&"cooper".to_string()));
        assert!(selected.len() <= 5);
    }

    #[test]
    fn selection_deduplicates_and_caps_at_five_without_dropping_critic() {
        let mut config = fixture_config();
        config.rules.push(Rule {
            name: Some("wide".into()),
            paths: vec!["**/*.ts".into()],
            replace: Vec::new(),
            add: vec![
                "beck".into(),
                "carmack".into(),
                "security".into(),
                "cooper".into(),
            ],
        });

        let selected = selected_reviewers(&["src/index.ts"], &config).unwrap();
        assert_eq!(selected[0], "critic");
        assert_eq!(selected.len(), 5);
        assert_eq!(
            selected,
            vec!["critic", "ousterhout", "grug", "beck", "carmack"]
        );
    }

    #[test]
    fn brace_patterns_are_literal_like_python_fnmatch() {
        assert!(!matches_pattern("src/auth_token.ts", "**/*auth*.{ts,tsx}").unwrap());
        assert!(matches_pattern("src/auth/login.ts", "**/auth/**").unwrap());
    }

    #[test]
    fn validate_ids_reports_unknown_defaults_and_rules() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path();
        fs::create_dir_all(repo.join("harnesses/shared/references")).unwrap();
        fs::create_dir_all(repo.join("agents")).unwrap();
        fs::write(
            repo.join("harnesses/shared/references/lenses.md"),
            "## critic\n## ousterhout\n## grug\n## adding-a-lens\n",
        )
        .unwrap();
        fs::write(
            repo.join("agents/security.md"),
            "---\nname: security\n---\n",
        )
        .unwrap();

        let config = BenchMap {
            default: vec!["critic".into(), "ghost".into()],
            rules: vec![Rule {
                name: Some("bad".into()),
                paths: Vec::new(),
                add: vec!["phantom".into()],
                replace: vec!["grug".into()],
            }],
        };

        let errors = validate_ids(repo, &config).unwrap();
        assert_eq!(
            errors,
            vec![
                "default: unknown reviewer id(s): ghost",
                "rule bad add: unknown reviewer id(s): phantom"
            ]
        );
    }
}
