use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, SecondsFormat, Utc};

const STOPWORDS: &[&str] = &[
    "the", "and", "for", "use", "when", "with", "this", "that", "from", "into", "your", "each",
    "are", "not", "all", "can", "has", "will", "been", "have", "does", "its", "any", "our", "you",
    "was",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexReport {
    pub skill_count: usize,
    pub agent_count: usize,
}

pub fn write_index(repo: &Path, now: DateTime<Utc>) -> Result<IndexReport> {
    let (content, report) = render_index(repo, now)?;
    fs::write(repo.join("index.yaml"), content)
        .with_context(|| format!("failed to write {}", repo.join("index.yaml").display()))?;
    Ok(report)
}

pub fn check_drift(repo: &Path, now: DateTime<Utc>) -> Result<()> {
    let committed_path = repo.join("index.yaml");
    let committed = fs::read_to_string(&committed_path)
        .with_context(|| format!("failed to read {}", committed_path.display()))?;
    let (generated, _) = render_index(repo, now)?;
    if strip_generated_timestamp(&committed) == strip_generated_timestamp(&generated) {
        return Ok(());
    }
    bail!("index.yaml is stale; run `harness-kit-checks generate-index --repo .`");
}

pub fn render_index(repo: &Path, now: DateTime<Utc>) -> Result<(String, IndexReport)> {
    let mut output = String::new();
    output.push_str("# Harness Kit Index\n");
    output.push_str(&format!(
        "# Generated: {}\n",
        now.to_rfc3339_opts(SecondsFormat::Secs, true)
    ));
    output.push_str("# Do not edit manually. Run: harness-kit-checks generate-index --repo .\n\n");
    output.push_str("skills:\n");

    let skills_root = repo.join("skills");
    let mut skill_count = 0;
    for skill_dir in direct_child_dirs(&skills_root)? {
        let Some(name) = file_name(&skill_dir) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        skill_count += 1;
        let skill_md = skill_dir.join("SKILL.md");
        if !skill_md.is_file() {
            continue;
        }
        let desc = extract_description(&skill_md)?.unwrap_or_default();
        output.push_str(&format!("  - name: {name}\n"));
        output.push_str(&format!(
            "    description: \"{}\"\n",
            escape_double_quotes(&desc)
        ));
        output.push_str("    source: first-party\n");
        let tags = tags_for_description(&desc);
        if !tags.is_empty() {
            output.push_str(&format!("    tags: [{}]\n", tags.join(",")));
        }
    }

    output.push('\n');
    output.push_str("agents:\n");
    let mut agent_count = 0;
    for agent_file in direct_child_files_with_extension(&repo.join("agents"), "md")? {
        let Some(name) = agent_file.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        agent_count += 1;
        output.push_str(&format!("  - name: {name}\n"));
        let desc = extract_description(&agent_file)?.unwrap_or_default();
        if !desc.is_empty() {
            output.push_str(&format!(
                "    description: \"{}\"\n",
                escape_double_quotes(&desc)
            ));
        }
    }
    if agent_count == 0 {
        output.push_str("  []\n");
    }

    output.push('\n');
    output.push_str("# Collections are defined in collections.yaml\n");
    Ok((
        output,
        IndexReport {
            skill_count,
            agent_count,
        },
    ))
}

fn direct_child_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn direct_child_files_with_extension(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|value| value.to_str()) == Some(extension) {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn file_name(path: &Path) -> Option<String> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
}

fn extract_description(path: &Path) -> Result<Option<String>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut in_frontmatter = false;
    let mut delimiter_count = 0;
    let mut collecting = false;
    let mut description = String::new();

    for line in content.lines() {
        if line == "---" {
            delimiter_count += 1;
            in_frontmatter = delimiter_count == 1;
            if delimiter_count > 1 {
                break;
            }
            continue;
        }
        if !in_frontmatter {
            continue;
        }

        if let Some(rest) = line.strip_prefix("description:") {
            collecting = true;
            let value = rest
                .trim_start()
                .strip_prefix('|')
                .unwrap_or(rest.trim_start());
            let value = value.trim_start();
            if !value.is_empty() && value != "|" {
                description.push_str(value);
                break;
            }
            continue;
        }

        if collecting {
            if let Some(continuation) = line.strip_prefix("  ") {
                description.push_str(continuation);
                description.push(' ');
            } else {
                break;
            }
        }
    }

    let description = trim_trailing_spaces(&description);
    if description.is_empty() {
        return Ok(inline_description_fallback(&content).map(|value| truncate_bytes(&value, 200)));
    }
    Ok(Some(truncate_bytes(&description, 200)))
}

fn inline_description_fallback(content: &str) -> Option<String> {
    let mut in_frontmatter = false;
    let mut delimiter_count = 0;
    for line in content.lines() {
        if line == "---" {
            delimiter_count += 1;
            in_frontmatter = delimiter_count == 1;
            if delimiter_count > 1 {
                break;
            }
            continue;
        }
        if !in_frontmatter {
            continue;
        }
        let Some(rest) = line.strip_prefix("description:") else {
            continue;
        };
        let value = rest.trim_start().trim_start_matches('"').trim_end();
        return Some(value.trim_end_matches('"').trim_end().to_string());
    }
    None
}

fn trim_trailing_spaces(value: &str) -> String {
    value.trim_end_matches(' ').to_string()
}

fn truncate_bytes(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }
    let mut end = max;
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_string()
}

fn escape_double_quotes(value: &str) -> String {
    value.replace('"', "\\\"")
}

fn tags_for_description(description: &str) -> Vec<String> {
    let mut tags = BTreeSet::new();
    let mut current = String::new();
    for character in description.chars().flat_map(char::to_lowercase) {
        if character.is_ascii_alphanumeric() || character == '-' {
            current.push(character);
        } else if !current.is_empty() {
            insert_tag(&mut tags, &current);
            current.clear();
        }
    }
    if !current.is_empty() {
        insert_tag(&mut tags, &current);
    }
    tags.into_iter().take(10).collect()
}

fn insert_tag(tags: &mut BTreeSet<String>, tag: &str) {
    if tag.is_empty() || STOPWORDS.contains(&tag) {
        return;
    }
    tags.insert(tag.to_string());
}

fn strip_generated_timestamp(content: &str) -> String {
    content
        .lines()
        .filter(|line| !line.starts_with("# Generated:"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn renders_shell_compatible_index_shape() {
        let temp = TempDir::new().unwrap();
        write(
            &temp.path().join("skills/alpha/SKILL.md"),
            r#"---
name: alpha
description: |
  Use when: "ship it", and with durable checks.
  Second line has "quotes" and more.
---
"#,
        );
        write(
            &temp.path().join("skills/.external/hidden/SKILL.md"),
            r#"---
name: hidden
description: Should not emit
---
"#,
        );
        write(
            &temp.path().join("skills/no-skill/README.md"),
            "not emitted\n",
        );
        write(
            &temp.path().join("agents/reviewer.md"),
            r#"---
name: reviewer
description: Finds bugs
---
"#,
        );

        let now = DateTime::parse_from_rfc3339("2026-06-04T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let (content, report) = render_index(temp.path(), now).unwrap();
        assert_eq!(
            report,
            IndexReport {
                skill_count: 2,
                agent_count: 1
            }
        );
        assert!(content.contains("# Generated: 2026-06-04T12:00:00Z\n"));
        assert!(content.contains("  - name: alpha\n"));
        assert!(content.contains("    source: first-party\n"));
        assert!(content.contains("    description: \"Use when: \\\"ship it\\\", and with durable checks. Second line has \\\"quotes\\\" and more.\"\n"));
        assert!(content.contains("    tags: [checks,durable,it,line,more,quotes,second,ship]\n"));
        assert!(!content.contains("  - name: no-skill\n"));
        assert!(content.contains("  - name: reviewer\n"));
        assert!(content.contains("    description: \"Finds bugs\"\n"));
    }

    #[test]
    fn renders_empty_agents_marker() {
        let temp = TempDir::new().unwrap();
        write(
            &temp.path().join("skills/alpha/SKILL.md"),
            r#"---
name: alpha
description: Tiny
---
"#,
        );
        let now = Utc::now();
        let (content, report) = render_index(temp.path(), now).unwrap();
        assert_eq!(report.agent_count, 0);
        assert!(content.contains("\nagents:\n  []\n\n# Collections"));
    }

    #[test]
    fn drift_check_ignores_timestamp_only() {
        let temp = TempDir::new().unwrap();
        write(
            &temp.path().join("skills/alpha/SKILL.md"),
            r#"---
name: alpha
description: Tiny
---
"#,
        );
        let first = DateTime::parse_from_rfc3339("2026-06-04T12:00:00Z")
            .unwrap()
            .with_timezone(&Utc);
        let second = DateTime::parse_from_rfc3339("2026-06-04T12:01:00Z")
            .unwrap()
            .with_timezone(&Utc);
        write_index(temp.path(), first).unwrap();
        check_drift(temp.path(), second).unwrap();
    }
}
