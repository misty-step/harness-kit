use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use serde_yaml::Value as YamlValue;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontmatterReport {
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

impl FrontmatterReport {
    pub fn ensure_success(&self) -> Result<()> {
        if self.errors.is_empty() {
            return Ok(());
        }
        bail!(self.errors.join("\n"));
    }
}

pub fn check_repo(repo: &Path) -> Result<FrontmatterReport> {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();
    let mut skill_frontmatters = Vec::new();

    for path in markdown_files(&repo.join("skills"), "SKILL.md")? {
        errors.extend(check_frontmatter(
            &path,
            &["name", "description"],
            Some(500),
        )?);
        if let Some(frontmatter) = load_frontmatter(&path)? {
            skill_frontmatters.push((display_path(repo, &path), frontmatter));
        }
    }

    for path in markdown_files(&repo.join("agents"), ".md")? {
        errors.extend(check_frontmatter(&path, &["name", "description"], None)?);
    }

    let (trigger_errors, trigger_warnings) = check_trigger_contracts(&skill_frontmatters);
    errors.extend(trigger_errors);
    warnings.extend(trigger_warnings);
    Ok(FrontmatterReport { errors, warnings })
}

pub fn check_frontmatter(
    path: &Path,
    required_fields: &[&str],
    max_lines: Option<usize>,
) -> Result<Vec<String>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut errors = Vec::new();
    if let Some(max_lines) = max_lines {
        let line_count = content.lines().count();
        if line_count > max_lines {
            errors.push(format!(
                "{}: {line_count} lines (max {max_lines})",
                path.display()
            ));
        }
    }
    if !content.starts_with("---") {
        return Ok(vec![format!("{}: missing frontmatter", path.display())]);
    }
    let frontmatter = match parse_frontmatter(&content) {
        Ok(frontmatter) => frontmatter,
        Err(error) => return Ok(vec![format!("{}: {error}", path.display())]),
    };
    for field in required_fields {
        if !frontmatter.contains_key(*field) {
            errors.push(format!(
                "{}: missing '{field}' in frontmatter",
                path.display()
            ));
        }
    }
    Ok(errors)
}

pub fn load_frontmatter(path: &Path) -> Result<Option<BTreeMap<String, YamlValue>>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    if !content.starts_with("---") {
        return Ok(None);
    }
    Ok(parse_frontmatter(&content).ok())
}

pub fn parse_frontmatter(content: &str) -> Result<BTreeMap<String, YamlValue>> {
    let Some(rest) = content.strip_prefix("---") else {
        bail!("malformed frontmatter");
    };
    let rest = rest
        .strip_prefix("\r\n")
        .or_else(|| rest.strip_prefix('\n'))
        .ok_or_else(|| anyhow!("malformed frontmatter"))?;
    let Some(end) = rest.find("\n---") else {
        bail!("malformed frontmatter");
    };
    let yaml = &rest[..end];
    let parsed: YamlValue =
        serde_yaml::from_str(yaml).map_err(|error| anyhow!("invalid YAML frontmatter: {error}"))?;
    let mapping = parsed
        .as_mapping()
        .ok_or_else(|| anyhow!("empty frontmatter"))?;
    if mapping.is_empty() {
        bail!("empty frontmatter");
    }
    let mut result = BTreeMap::new();
    for (key, value) in mapping {
        let Some(key) = key.as_str() else {
            continue;
        };
        result.insert(key.to_string(), value.clone());
    }
    if result.is_empty() {
        bail!("empty frontmatter");
    }
    Ok(result)
}

pub fn normalize_trigger_claim(value: &str) -> String {
    let mut value = value.trim().trim_matches('.').trim_matches('"').to_string();
    value = Regex::new(r"\s*\([^)]*\)\s*$")
        .expect("static regex compiles")
        .replace(&value, "")
        .to_string();
    value = Regex::new(r"\s+")
        .expect("static regex compiles")
        .replace_all(&value, " ")
        .to_string();
    value.to_lowercase()
}

pub fn collision_key(value: &str) -> String {
    let normalized = normalize_trigger_claim(value);
    let without_slash = normalized.strip_prefix('/').unwrap_or(&normalized);
    Regex::new(r"[^a-z0-9]+")
        .expect("static regex compiles")
        .replace_all(without_slash, " ")
        .trim()
        .to_string()
}

pub fn explicit_triggers(description: &str) -> Vec<String> {
    let trigger_re = Regex::new(r"(?i)\bTriggers?:\s*(.*)").expect("static regex compiles");
    let use_when_re = Regex::new(r"(?i)\bUse (?:when|for):").expect("static regex compiles");
    let lines: Vec<_> = description.lines().collect();
    let mut triggers = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let Some(captures) = trigger_re.captures(line) else {
            continue;
        };
        let mut trigger_text = captures
            .get(1)
            .map(|matched| matched.as_str().to_string())
            .unwrap_or_default();
        for continuation in lines.iter().skip(index + 1) {
            if use_when_re.is_match(continuation) || trigger_re.is_match(continuation) {
                break;
            }
            trigger_text.push(' ');
            trigger_text.push_str(continuation.trim());
        }
        triggers.extend(split_trigger_values(&trigger_text));
    }
    triggers
}

pub fn use_when_phrases(description: &str) -> Vec<String> {
    let trigger_re = Regex::new(r"(?i)\bTriggers?:").expect("static regex compiles");
    let use_when_re = Regex::new(r"(?i)\bUse (?:when|for):").expect("static regex compiles");
    let quoted_re = Regex::new("\"([^\"]+)\"").expect("static regex compiles");
    let mut phrases = Vec::new();
    let mut in_block = false;
    for line in description.lines() {
        if use_when_re.is_match(line) {
            in_block = true;
        } else if trigger_re.is_match(line) || line.trim().is_empty() {
            in_block = false;
        }
        if in_block {
            phrases.extend(
                quoted_re
                    .captures_iter(line)
                    .filter_map(|captures| captures.get(1))
                    .map(|matched| normalize_trigger_claim(matched.as_str())),
            );
        }
    }
    phrases
}

pub fn trigger_claims(description: &str) -> Vec<String> {
    let mut claims = explicit_triggers(description);
    claims.extend(use_when_phrases(description));
    claims
}

pub fn check_trigger_contracts(
    skill_frontmatters: &[(String, BTreeMap<String, YamlValue>)],
) -> (Vec<String>, Vec<String>) {
    let mut warnings = Vec::new();
    let mut claims: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, frontmatter) in skill_frontmatters {
        let description = frontmatter
            .get("description")
            .and_then(YamlValue::as_str)
            .unwrap_or_default();
        if explicit_triggers(description).is_empty() {
            warnings.push(format!("{path}: missing Trigger definition in description"));
        }
        for claim in trigger_claims(description) {
            let key = collision_key(&claim);
            if !key.is_empty() {
                claims.entry(key).or_default().push(path.clone());
            }
        }
    }
    let mut errors = Vec::new();
    for (claim, owners) in claims {
        let mut unique = owners;
        unique.sort();
        unique.dedup();
        if unique.len() > 1 {
            errors.push(format!(
                "trigger claim collision '{claim}': {}",
                unique.join(", ")
            ));
        }
    }
    (errors, warnings)
}

fn split_trigger_values(text: &str) -> Vec<String> {
    text.split(',')
        .map(normalize_trigger_claim)
        .filter(|value| !value.is_empty())
        .collect()
}

fn markdown_files(root: &Path, suffix: &str) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut paths = Vec::new();
    for entry in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let entry = entry?;
        let path = entry.path();
        if suffix == "SKILL.md" {
            let skill = path.join("SKILL.md");
            if skill.is_file() {
                paths.push(skill);
            }
        } else if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(suffix))
            && path.is_file()
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn display_path(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_triggers_strip_alias_notes() {
        let description = "Use when: \"commit and push\".\nTrigger: /yeet, /ship-local (alias).\n";

        assert_eq!(explicit_triggers(description), vec!["/yeet", "/ship-local"]);
    }

    #[test]
    fn use_when_phrases_are_trigger_claims() {
        let description = "Use when: \"ship it\", \"finish this ticket\".\nTrigger: /ship.\n";

        assert_eq!(
            trigger_claims(description),
            vec!["/ship", "ship it", "finish this ticket"]
        );
    }

    #[test]
    fn use_when_phrases_ignore_nonrouting_quotes() {
        let description = "End-to-end \"ship it to the remote\" is descriptive prose.\nUse when: \"yeet\", \"commit and push\".\nTrigger: /yeet.\n";

        assert_eq!(
            trigger_claims(description),
            vec!["/yeet", "yeet", "commit and push"]
        );
    }

    #[test]
    fn collision_key_unifies_slash_hyphen_and_phrase_forms() {
        assert_eq!(collision_key("/ship-it"), "ship it");
        assert_eq!(collision_key("ship it"), "ship it");
    }

    #[test]
    fn trigger_contract_warns_on_missing_trigger() {
        let (errors, warnings) = check_trigger_contracts(&[(
            "skills/example/SKILL.md".to_string(),
            BTreeMap::from([(
                "description".to_string(),
                YamlValue::String("Use when: \"example\".".to_string()),
            )]),
        )]);

        assert!(errors.is_empty());
        assert_eq!(
            warnings,
            vec!["skills/example/SKILL.md: missing Trigger definition in description"]
        );
    }

    #[test]
    fn trigger_contract_rejects_duplicate_claims() {
        let (errors, warnings) = check_trigger_contracts(&[
            (
                "skills/ship/SKILL.md".to_string(),
                BTreeMap::from([(
                    "description".to_string(),
                    YamlValue::String("Use when: \"ship it\".\nTrigger: /ship.".to_string()),
                )]),
            ),
            (
                "skills/yeet/SKILL.md".to_string(),
                BTreeMap::from([(
                    "description".to_string(),
                    YamlValue::String("Use when: \"ship it\".\nTrigger: /yeet.".to_string()),
                )]),
            ),
        ]);

        assert!(warnings.is_empty());
        assert_eq!(
            errors,
            vec!["trigger claim collision 'ship it': skills/ship/SKILL.md, skills/yeet/SKILL.md"]
        );
    }

    #[test]
    fn trigger_contract_rejects_slash_phrase_collisions() {
        let (errors, warnings) = check_trigger_contracts(&[
            (
                "skills/deploy/SKILL.md".to_string(),
                BTreeMap::from([(
                    "description".to_string(),
                    YamlValue::String("Use when: \"deploy\".\nTrigger: /ship-it.".to_string()),
                )]),
            ),
            (
                "skills/ship/SKILL.md".to_string(),
                BTreeMap::from([(
                    "description".to_string(),
                    YamlValue::String("Use when: \"ship it\".\nTrigger: /ship.".to_string()),
                )]),
            ),
        ]);

        assert!(warnings.is_empty());
        assert_eq!(
            errors,
            vec!["trigger claim collision 'ship it': skills/deploy/SKILL.md, skills/ship/SKILL.md"]
        );
    }

    #[test]
    fn frontmatter_parser_ignores_body_horizontal_rules() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("SKILL.md");
        fs::write(
            &path,
            "---\nname: demo\ndescription: |\n  Use when: \"demo\".\n  Trigger: /demo.\n---\n\n# Body\n\n---\n\nBody horizontal rule.\n",
        )?;

        assert!(check_frontmatter(&path, &["name", "description"], None)?.is_empty());
        assert_eq!(
            load_frontmatter(&path)?
                .unwrap()
                .get("name")
                .and_then(YamlValue::as_str),
            Some("demo")
        );
        Ok(())
    }
}
