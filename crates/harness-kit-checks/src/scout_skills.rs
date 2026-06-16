use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use crate::external_sync;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScoutFormat {
    Markdown,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoutOptions {
    pub repo_root: PathBuf,
    pub input: PathBuf,
    pub output: Option<PathBuf>,
    pub format: ScoutFormat,
    pub live: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillScoutReport {
    pub input: String,
    pub candidates: Vec<SkillCandidateReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SkillCandidateReport {
    pub repo: String,
    pub default_branch_sha: Option<String>,
    pub license: String,
    pub layout: String,
    pub aliases: Vec<String>,
    pub duplicate: bool,
    pub verdict: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CandidateMetadata {
    pub default_branch_sha: Option<String>,
    pub license: Option<String>,
    pub description: Option<String>,
    pub skill_dirs: Vec<String>,
    pub root_skill: bool,
}

pub fn run(options: &ScoutOptions) -> Result<String> {
    let input_text = fs::read_to_string(&options.input)
        .with_context(|| format!("failed to read scout input {}", options.input.display()))?;
    let registry = RegistryIndex::from_repo(&options.repo_root)?;
    let repos = extract_github_repos(&input_text);
    let mut metadata = BTreeMap::new();
    for repo in &repos {
        metadata.insert(
            repo.clone(),
            if options.live {
                fetch_live_metadata(repo).unwrap_or_default()
            } else {
                CandidateMetadata::default()
            },
        );
    }
    let report = build_report(
        &repos,
        &registry,
        &metadata,
        &display_relative(&options.repo_root, &options.input),
    );
    let rendered = match options.format {
        ScoutFormat::Markdown => render_markdown(&report),
        ScoutFormat::Json => serde_json::to_string_pretty(&report)? + "\n",
    };
    if let Some(output) = &options.output {
        let output = if output.is_absolute() {
            output.clone()
        } else {
            options.repo_root.join(output)
        };
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, &rendered)?;
    }
    Ok(rendered)
}

pub fn extract_github_repos(text: &str) -> Vec<String> {
    let pattern = Regex::new(r"https://github\.com/([A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+)").unwrap();
    let mut seen = BTreeSet::new();
    let mut repos = Vec::new();
    for capture in pattern.captures_iter(text) {
        let Some(repo) = capture
            .get(1)
            .map(|item| item.as_str().trim_end_matches('.'))
        else {
            continue;
        };
        if seen.insert(repo.to_string()) {
            repos.push(repo.to_string());
        }
    }
    repos
}

fn build_report(
    repos: &[String],
    registry: &RegistryIndex,
    metadata: &BTreeMap<String, CandidateMetadata>,
    input: &str,
) -> SkillScoutReport {
    let candidates = repos
        .iter()
        .map(|repo| {
            let meta = metadata.get(repo).cloned().unwrap_or_default();
            let aliases = registry.aliases_for(repo);
            let duplicate = !aliases.is_empty();
            let layout = classify_layout(&meta);
            let license = classify_license(meta.license.as_deref());
            let (verdict, rationale) =
                verdict_for(&license, &layout, duplicate, meta.description.as_deref());
            SkillCandidateReport {
                repo: repo.clone(),
                default_branch_sha: meta.default_branch_sha,
                license,
                layout,
                aliases,
                duplicate,
                verdict,
                rationale,
            }
        })
        .collect();
    SkillScoutReport {
        input: input.to_string(),
        candidates,
    }
}

pub fn classify_license(license: Option<&str>) -> String {
    match license
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    {
        Some("MIT") | Some("Apache-2.0") | Some("BSD-2-Clause") | Some("BSD-3-Clause") => {
            "compatible".to_string()
        }
        Some("NOASSERTION") | None => "unknown".to_string(),
        Some(value) => format!("review:{value}"),
    }
}

pub fn classify_layout(metadata: &CandidateMetadata) -> String {
    if !metadata.skill_dirs.is_empty() {
        return "skills/*/SKILL.md".to_string();
    }
    if metadata.root_skill {
        return "root SKILL.md".to_string();
    }
    "no compatible skill layout detected".to_string()
}

fn verdict_for(
    license: &str,
    layout: &str,
    duplicate: bool,
    description: Option<&str>,
) -> (String, String) {
    if duplicate {
        return (
            "already-covered".to_string(),
            "repo is already represented in registry.yaml".to_string(),
        );
    }
    if license.starts_with("review:") {
        return (
            "reject-license".to_string(),
            "license needs explicit human review before any import".to_string(),
        );
    }
    if license == "unknown" {
        return (
            "needs-human-review".to_string(),
            "license was not detected; do not import automatically".to_string(),
        );
    }
    if layout == "skills/*/SKILL.md" || layout == "root SKILL.md" {
        return (
            "recommend-import".to_string(),
            "compatible license and skill-shaped layout detected".to_string(),
        );
    }
    if description
        .unwrap_or_default()
        .to_ascii_lowercase()
        .contains("skill")
    {
        return (
            "defer-exemplar".to_string(),
            "skill-related repo without a directly syncable layout".to_string(),
        );
    }
    (
        "reject-not-skill".to_string(),
        "no compatible Harness Kit skill layout detected".to_string(),
    )
}

pub fn render_markdown(report: &SkillScoutReport) -> String {
    let mut out = String::new();
    out.push_str("# Agent Skill Scout Report\n\n");
    out.push_str(&format!("Input: `{}`\n\n", report.input));
    out.push_str("| Repo | SHA | License | Layout | Aliases | Verdict | Rationale |\n");
    out.push_str("|---|---|---|---|---|---|---|\n");
    for candidate in &report.candidates {
        out.push_str(&format!(
            "| `{}` | `{}` | {} | {} | {} | `{}` | {} |\n",
            candidate.repo,
            candidate
                .default_branch_sha
                .as_deref()
                .map(short_sha)
                .unwrap_or("-"),
            candidate.license,
            candidate.layout,
            if candidate.aliases.is_empty() {
                "-".to_string()
            } else {
                candidate.aliases.join(", ")
            },
            candidate.verdict,
            candidate.rationale
        ));
    }
    out
}

#[derive(Debug, Default)]
struct RegistryIndex {
    repo_to_aliases: BTreeMap<String, Vec<String>>,
}

impl RegistryIndex {
    fn from_repo(repo_root: &Path) -> Result<Self> {
        let entries = external_sync::parse_registry_file(&repo_root.join("registry.yaml"))?;
        let mut repo_to_aliases: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for entry in entries {
            let aliases = repo_to_aliases.entry(entry.repo.clone()).or_default();
            let prefix = entry.alias_prefix.unwrap_or_default();
            for include in entry.include {
                aliases.push(format!("{prefix}{include}"));
            }
        }
        Ok(Self { repo_to_aliases })
    }

    fn aliases_for(&self, repo: &str) -> Vec<String> {
        self.repo_to_aliases.get(repo).cloned().unwrap_or_default()
    }
}

fn fetch_live_metadata(repo: &str) -> Result<CandidateMetadata> {
    let mut metadata = CandidateMetadata {
        default_branch_sha: resolve_head_sha(repo),
        ..CandidateMetadata::default()
    };
    let api = curl_json(&format!("https://api.github.com/repos/{repo}")).unwrap_or(Value::Null);
    metadata.license = api
        .pointer("/license/spdx_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    metadata.description = api
        .get("description")
        .and_then(Value::as_str)
        .map(ToString::to_string);
    let ref_name = metadata
        .default_branch_sha
        .as_deref()
        .or_else(|| api.get("default_branch").and_then(Value::as_str))
        .unwrap_or("HEAD");
    metadata.skill_dirs = fetch_skill_dirs(repo, ref_name).unwrap_or_default();
    metadata.root_skill = content_exists(repo, "SKILL.md", ref_name).unwrap_or(false);
    Ok(metadata)
}

fn resolve_head_sha(repo: &str) -> Option<String> {
    let output = Command::new("git")
        .args([
            "ls-remote",
            &format!("https://github.com/{repo}.git"),
            "HEAD",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .filter(|sha| sha.len() == 40)
        .map(ToString::to_string)
}

fn fetch_skill_dirs(repo: &str, ref_name: &str) -> Result<Vec<String>> {
    let value = curl_json(&format!(
        "https://api.github.com/repos/{repo}/contents/skills?ref={ref_name}"
    ))?;
    let mut dirs = Vec::new();
    if let Value::Array(items) = value {
        for item in items {
            if item.get("type").and_then(Value::as_str) == Some("dir")
                && let Some(name) = item.get("name").and_then(Value::as_str)
                && content_exists(repo, &format!("skills/{name}/SKILL.md"), ref_name)?
            {
                dirs.push(name.to_string());
            }
        }
    }
    dirs.sort();
    Ok(dirs)
}

fn content_exists(repo: &str, path: &str, ref_name: &str) -> Result<bool> {
    let status = Command::new("curl")
        .args([
            "-fsSLo",
            "/dev/null",
            "-w",
            "%{http_code}",
            &format!("https://api.github.com/repos/{repo}/contents/{path}?ref={ref_name}"),
        ])
        .output()?;
    let code = String::from_utf8_lossy(&status.stdout);
    Ok(code.trim() == "200")
}

fn curl_json(url: &str) -> Result<Value> {
    let output = Command::new("curl").args(["-fsSL", url]).output()?;
    if !output.status.success() {
        anyhow::bail!("curl failed for {url}");
    }
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn display_relative(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn short_sha(sha: &str) -> &str {
    sha.get(..12).unwrap_or(sha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_github_urls_in_order_without_duplicates() {
        let repos = extract_github_repos(
            "https://github.com/one/skill\nhttps://github.com/two/tool.\nhttps://github.com/one/skill",
        );
        assert_eq!(repos, vec!["one/skill", "two/tool"]);
    }

    #[test]
    fn classifies_license_and_layout() {
        assert_eq!(classify_license(Some("MIT")), "compatible");
        assert_eq!(classify_license(None), "unknown");
        assert_eq!(classify_license(Some("GPL-3.0")), "review:GPL-3.0");
        assert_eq!(
            classify_layout(&CandidateMetadata {
                skill_dirs: vec!["demo".to_string()],
                ..CandidateMetadata::default()
            }),
            "skills/*/SKILL.md"
        );
        assert_eq!(
            classify_layout(&CandidateMetadata {
                root_skill: true,
                ..CandidateMetadata::default()
            }),
            "root SKILL.md"
        );
    }

    #[test]
    fn detects_registry_duplicates_and_renders_verdicts() {
        let repos = vec![
            "already/skills".to_string(),
            "fresh/skills".to_string(),
            "plain/tool".to_string(),
        ];
        let registry = RegistryIndex {
            repo_to_aliases: BTreeMap::from([(
                "already/skills".to_string(),
                vec!["already-demo".to_string()],
            )]),
        };
        let metadata = BTreeMap::from([
            (
                "fresh/skills".to_string(),
                CandidateMetadata {
                    default_branch_sha: Some(
                        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                    ),
                    license: Some("MIT".to_string()),
                    skill_dirs: vec!["demo".to_string()],
                    ..CandidateMetadata::default()
                },
            ),
            (
                "plain/tool".to_string(),
                CandidateMetadata {
                    license: Some("MIT".to_string()),
                    ..CandidateMetadata::default()
                },
            ),
        ]);
        let report = build_report(&repos, &registry, &metadata, "fixture.md");
        assert_eq!(report.candidates[0].verdict, "already-covered");
        assert_eq!(report.candidates[1].verdict, "recommend-import");
        assert_eq!(report.candidates[2].verdict, "reject-not-skill");
        let rendered = render_markdown(&report);
        assert!(
            rendered.contains("| `fresh/skills` | `aaaaaaaaaaaa` | compatible | skills/*/SKILL.md")
        );
    }
}
