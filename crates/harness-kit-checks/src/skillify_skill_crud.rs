use std::fs;
use std::path::{Component, Path, PathBuf};

use anyhow::{Result, bail};
use regex::Regex;
use serde_json::{Value, json};
use tempfile::NamedTempFile;

pub fn normalize_name(name: &str) -> Result<String> {
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        bail!("invalid skill name: {name:?}");
    }
    let mut normalized = String::new();
    let mut previous_hyphen = false;
    for character in name.trim().to_lowercase().replace('_', "-").chars() {
        let next =
            if character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-' {
                character
            } else {
                '-'
            };
        if next == '-' {
            if !previous_hyphen {
                normalized.push(next);
            }
            previous_hyphen = true;
        } else {
            normalized.push(next);
            previous_hyphen = false;
        }
    }
    let normalized = normalized.trim_matches('-').to_string();
    let valid = Regex::new(r"^[a-z0-9][a-z0-9-]*[a-z0-9]$").unwrap();
    if normalized.is_empty() || !valid.is_match(&normalized) {
        bail!("invalid skill name: {name:?}");
    }
    Ok(normalized)
}

pub fn skill_dir(skills_root: &Path, name: &str) -> Result<PathBuf> {
    let root = resolve_lexical(skills_root)?;
    let target = resolve_lexical(&root.join(normalize_name(name)?))?;
    if target != root && !target.starts_with(&root) {
        bail!("skill path escapes skills root");
    }
    Ok(target)
}

pub fn build_skill_md(name: &str, description: &str, body: &str) -> Result<String> {
    let clean_name = normalize_name(name)?;
    let description = description.split_whitespace().collect::<Vec<_>>().join(" ");
    if description.is_empty() {
        bail!("description is required");
    }
    let body = format!("{}\n", body.trim());
    Ok(format!(
        "---\nname: {clean_name}\ndescription: |\n  {description}\nargument-hint: \"[{clean_name}-args]\"\n---\n\n# /{clean_name}\n\n{body}"
    ))
}

pub fn create_skill(
    skills_root: &Path,
    name: &str,
    description: &str,
    body: &str,
) -> Result<Value> {
    let target = skill_dir(skills_root, name)?;
    if target.exists() {
        bail!("skill already exists: {}", target.display());
    }
    fs::create_dir_all(&target)?;
    atomic_write(
        &target.join("SKILL.md"),
        &build_skill_md(name, description, body)?,
    )?;
    Ok(
        json!({"status": "created", "name": normalize_name(name)?, "path": target.display().to_string()}),
    )
}

pub fn read_skill(skills_root: &Path, name: &str) -> Result<Value> {
    let target = skill_dir(skills_root, name)?;
    let skill_md = target.join("SKILL.md");
    if !skill_md.exists() {
        bail!("missing SKILL.md for {}", normalize_name(name)?);
    }
    let mut files = Vec::new();
    collect_files(&target, &target, &mut files)?;
    files.sort();
    Ok(json!({
        "status": "read",
        "name": normalize_name(name)?,
        "path": target.display().to_string(),
        "files": files,
        "skill_md": fs::read_to_string(skill_md)?,
    }))
}

pub fn update_skill(
    skills_root: &Path,
    name: &str,
    description: Option<&str>,
    body: Option<&str>,
) -> Result<Value> {
    let target = skill_dir(skills_root, name)?;
    let current = read_skill(skills_root, name)?;
    let current_text = current
        .get("skill_md")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let frontmatter = parse_frontmatter(current_text)?;
    let current_body = current_text
        .splitn(3, "---")
        .nth(2)
        .unwrap_or_default()
        .trim_start();
    let description = description.unwrap_or_else(|| {
        frontmatter
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or_default()
    });
    let owned_body;
    let body = match body {
        Some(body) => body,
        None => {
            owned_body = remove_first_heading(current_body);
            &owned_body
        }
    };
    atomic_write(
        &target.join("SKILL.md"),
        &build_skill_md(name, description, body)?,
    )?;
    Ok(
        json!({"status": "updated", "name": normalize_name(name)?, "path": target.display().to_string()}),
    )
}

pub fn delete_skill(skills_root: &Path, name: &str) -> Result<Value> {
    let target = skill_dir(skills_root, name)?;
    if !target.exists() {
        bail!("skill does not exist: {}", normalize_name(name)?);
    }
    fs::remove_dir_all(&target)?;
    Ok(
        json!({"status": "deleted", "name": normalize_name(name)?, "path": target.display().to_string()}),
    )
}

pub fn parse_frontmatter(text: &str) -> Result<Value> {
    let Some(rest) = text.strip_prefix("---") else {
        bail!("missing or malformed frontmatter");
    };
    let Some((frontmatter, _body)) = rest.trim_start_matches('\n').split_once("\n---") else {
        bail!("missing or malformed frontmatter");
    };
    let mut values = serde_json::Map::new();
    let lines: Vec<&str> = frontmatter.lines().collect();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        let Some((key, raw)) = line.split_once(':') else {
            index += 1;
            continue;
        };
        let key = key.trim();
        let raw = raw.trim();
        if raw == "|" {
            let mut parts = Vec::new();
            index += 1;
            while index < lines.len() && lines[index].starts_with("  ") {
                parts.push(&lines[index][2..]);
                index += 1;
            }
            values.insert(key.to_string(), json!(parts.join("\n").trim()));
            continue;
        }
        values.insert(key.to_string(), json!(raw.trim_matches('"')));
        index += 1;
    }
    Ok(Value::Object(values))
}

pub fn validate_portability(text: &str) -> Result<()> {
    let forbidden = Regex::new(r"\b(SendUserMessage|Edit|Skill|bash)\b").unwrap();
    if forbidden.is_match(text) && !text.to_lowercase().contains("fallback") {
        bail!("harness-specific operation appears without fallback");
    }
    Ok(())
}

pub fn validate_skill(skills_root: &Path, name: &str) -> Result<Value> {
    let target = skill_dir(skills_root, name)?;
    let skill_md = target.join("SKILL.md");
    if !skill_md.exists() {
        bail!("missing SKILL.md");
    }
    let text = fs::read_to_string(&skill_md)?;
    let frontmatter = parse_frontmatter(&text)?;
    let expected = normalize_name(name)?;
    if frontmatter.get("name").and_then(Value::as_str) != Some(expected.as_str()) {
        bail!("frontmatter name does not match skill directory");
    }
    let description = frontmatter
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !description.contains("Trigger:") {
        bail!("description must include Trigger:");
    }
    let use_phrase = Regex::new(r#"(?is)Use (?:when|for):.*""#).unwrap();
    if !use_phrase.is_match(description) {
        bail!("description must include quoted Use when/for phrases");
    }
    validate_portability(&text)?;
    Ok(json!({"status": "valid", "name": expected, "path": target.display().to_string()}))
}

fn atomic_write(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
        let mut temp = NamedTempFile::new_in(parent)?;
        std::io::Write::write_all(&mut temp, text.as_bytes())?;
        temp.persist(path)?;
    }
    Ok(())
}

fn collect_files(root: &Path, dir: &Path, files: &mut Vec<String>) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if entry.metadata()?.is_dir() {
            collect_files(root, &path, files)?;
        } else if entry.metadata()?.is_file() {
            files.push(path.strip_prefix(root)?.display().to_string());
        }
    }
    Ok(())
}

fn remove_first_heading(text: &str) -> String {
    if !text.starts_with("# ") {
        return text.to_string();
    }
    if let Some(index) = text.find("\n\n") {
        text[index + 2..].to_string()
    } else {
        String::new()
    }
}

fn resolve_lexical(path: &Path) -> Result<PathBuf> {
    let base = if path.is_absolute() {
        PathBuf::new()
    } else {
        std::env::current_dir()?
    };
    let mut resolved = base;
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => resolved.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                resolved.pop();
            }
            Component::Normal(part) => resolved.push(part),
        }
    }
    Ok(resolved)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn create_read_update_delete_validate_round_trip() {
        let tmp = TempDir::new().unwrap();
        let skills_root = tmp.path().join("skills");
        let created = create_skill(
            &skills_root,
            "demo-skill",
            "Demo skill for repeatable transcript extraction. Use when: \"demo skillify\". Trigger: /demo-skill.",
            "## Contract\n\nUse portable filesystem instructions with fallback commands.\n",
        )
        .unwrap();

        assert_eq!(created["name"], "demo-skill");
        assert_eq!(
            created["path"],
            skills_root.join("demo-skill").display().to_string()
        );
        assert!(skills_root.join("demo-skill/SKILL.md").exists());

        let validation = validate_skill(&skills_root, "demo-skill").unwrap();
        assert_eq!(validation["status"], "valid");

        let read = read_skill(&skills_root, "demo-skill").unwrap();
        assert!(
            read["files"]
                .as_array()
                .unwrap()
                .contains(&json!("SKILL.md"))
        );
        assert!(read["skill_md"].as_str().unwrap().contains("/demo-skill"));

        let updated = update_skill(
            &skills_root,
            "demo-skill",
            None,
            Some("## Contract\n\nUpdated portable instructions with fallback commands.\n"),
        )
        .unwrap();
        assert_eq!(updated["status"], "updated");
        assert!(
            read_skill(&skills_root, "demo-skill").unwrap()["skill_md"]
                .as_str()
                .unwrap()
                .contains("Updated portable")
        );

        let deleted = delete_skill(&skills_root, "demo-skill").unwrap();
        assert_eq!(deleted["status"], "deleted");
        assert!(!skills_root.join("demo-skill").exists());
    }

    #[test]
    fn rejects_path_traversal_and_harness_specific_ops_without_fallback() {
        let tmp = TempDir::new().unwrap();
        let skills_root = tmp.path().join("skills");
        assert!(
            create_skill(&skills_root, "../escape", "bad", "bad")
                .unwrap_err()
                .to_string()
                .contains("invalid skill name")
        );
        create_skill(
            &skills_root,
            "bad-skill",
            "Bad skill. Use when: \"bad skill\". Trigger: /bad-skill.",
            "Call SendUserMessage directly.",
        )
        .unwrap();

        assert_eq!(
            validate_skill(&skills_root, "bad-skill")
                .unwrap_err()
                .to_string(),
            "harness-specific operation appears without fallback"
        );
    }
}
