use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result, anyhow, bail};
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Turn {
    pub role: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Evidence {
    pub row_count: usize,
    pub turn_count: usize,
    pub malformed_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TranscriptPacket {
    pub source: String,
    pub turns: Vec<Turn>,
    pub candidate_instructions: Vec<String>,
    pub evidence: Evidence,
}

pub fn iter_jsonl(path: &Path) -> Result<(Vec<Value>, usize)> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut rows = Vec::new();
    let mut malformed = 0;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(line) {
            Ok(value @ Value::Object(_)) => rows.push(value),
            Ok(_) => {}
            Err(_) => malformed += 1,
        }
    }
    Ok((rows, malformed))
}

pub fn extract_text(content: Option<&Value>) -> String {
    let Some(content) = content else {
        return String::new();
    };
    match content {
        Value::String(text) => text.trim().to_string(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| match item {
                Value::String(text) => Some(text.as_str()),
                Value::Object(map) => map
                    .get("text")
                    .and_then(Value::as_str)
                    .or_else(|| map.get("content").and_then(Value::as_str)),
                _ => None,
            })
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string(),
        Value::Object(map) => map
            .get("text")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default()
            .to_string(),
        _ => String::new(),
    }
}

pub fn extract_turns(rows: &[Value]) -> Vec<Turn> {
    let mut turns = Vec::new();
    for row in rows {
        let Value::Object(row_map) = row else {
            continue;
        };
        let message = match row_map.get("message") {
            Some(Value::Object(message)) => message,
            _ => row_map,
        };
        let role = message
            .get("role")
            .and_then(Value::as_str)
            .or_else(|| row_map.get("type").and_then(Value::as_str));
        let Some(role @ ("user" | "assistant" | "system")) = role else {
            continue;
        };
        let text = extract_text(message.get("content"));
        if !text.is_empty() {
            turns.push(Turn {
                role: role.to_string(),
                text,
            });
        }
    }
    turns
}

pub fn candidate_instructions(turns: &[Turn]) -> Vec<String> {
    turns
        .iter()
        .filter(|turn| turn.role == "assistant")
        .filter(|turn| {
            let lowered = turn.text.to_lowercase();
            [
                "use ", "avoid ", "must ", "when ", "workflow", "skill", "contract",
            ]
            .iter()
            .any(|marker| lowered.contains(marker))
        })
        .map(|turn| turn.text.clone())
        .collect()
}

pub fn parse_transcript(path: &Path) -> Result<TranscriptPacket> {
    let (rows, malformed) = iter_jsonl(path)?;
    let turns = extract_turns(&rows);
    Ok(TranscriptPacket {
        source: path.display().to_string(),
        turns: turns.clone(),
        candidate_instructions: candidate_instructions(&turns),
        evidence: Evidence {
            row_count: rows.len(),
            turn_count: turns.len(),
            malformed_count: malformed,
        },
    })
}

pub fn claude_project_keys(cwd: &Path) -> Result<Vec<String>> {
    let resolved = resolve_lexical(cwd)?;
    let resolved = resolved.display().to_string();
    let stripped = resolved.trim_matches('/');
    Ok(vec![resolved.replace('/', "-"), stripped.replace('/', "-")])
}

pub fn find_current_transcript(projects_dir: &Path, cwd: &Path) -> Result<PathBuf> {
    let keys = claude_project_keys(cwd)?;
    let mut candidates: Vec<(SystemTime, PathBuf)> = Vec::new();
    collect_project_jsonl(projects_dir, &keys, &mut candidates)?;
    if candidates.is_empty() {
        bail!(
            "no Claude JSONL transcripts found for {} under {}",
            cwd.display(),
            projects_dir.display()
        );
    }
    candidates.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| right.1.cmp(&left.1)));
    Ok(candidates.remove(0).1)
}

pub fn resolve_transcript(
    transcript: Option<PathBuf>,
    from_current: bool,
    projects_dir: &Path,
    cwd: &Path,
) -> Result<PathBuf> {
    if transcript.is_some() && from_current {
        bail!("provide either a transcript path or --from-current, not both");
    }
    if let Some(transcript) = transcript {
        return Ok(transcript);
    }
    if from_current {
        return find_current_transcript(projects_dir, cwd);
    }
    bail!("provide a transcript path or --from-current");
}

fn collect_project_jsonl(
    dir: &Path,
    project_keys: &[String],
    candidates: &mut Vec<(SystemTime, PathBuf)>,
) -> Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", dir.display()));
        }
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            collect_project_jsonl(&path, project_keys, candidates)?;
        } else if metadata.is_file()
            && path.extension().and_then(|value| value.to_str()) == Some("jsonl")
            && path
                .parent()
                .and_then(Path::file_name)
                .and_then(|value| value.to_str())
                .is_some_and(|name| project_keys.iter().any(|key| key == name))
        {
            candidates.push((metadata.modified()?, path));
        }
    }
    Ok(())
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
    if resolved.as_os_str().is_empty() {
        Err(anyhow!("failed to resolve {}", path.display()))
    } else {
        Ok(resolved)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::thread;
    use std::time::Duration;

    use serde_json::json;
    use tempfile::TempDir;

    use super::*;

    #[test]
    fn parses_claude_jsonl_into_instruction_packet() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("claude.jsonl");
        let rows = [
            json!({"type": "user", "message": {"role": "user", "content": "Make this workflow reusable."}}).to_string(),
            json!({"type": "assistant", "message": {"role": "assistant", "content": [{"type": "text", "text": "Use a local SKILL.md and avoid harness-only tools."}]}}).to_string(),
            json!({"type": "tool_result", "content": "ignored"}).to_string(),
            "not json".to_string(),
        ];
        fs::write(&path, rows.join("\n")).unwrap();

        let packet = parse_transcript(&path).unwrap();

        assert_eq!(packet.source, path.display().to_string());
        assert_eq!(packet.evidence.row_count, 3);
        assert_eq!(packet.evidence.turn_count, 2);
        assert_eq!(packet.evidence.malformed_count, 1);
        assert_eq!(packet.turns[0].role, "user");
        assert!(packet.candidate_instructions[0].contains("local SKILL.md"));
    }

    #[test]
    fn from_current_selects_latest_transcript_for_project() {
        let tmp = TempDir::new().unwrap();
        let projects_dir = tmp.path().join("projects");
        let cwd = tmp.path().join("Users/phaedrus/Development/harness-kit");
        fs::create_dir_all(&cwd).unwrap();
        let keys = claude_project_keys(&cwd).unwrap();
        let current_project = projects_dir.join(&keys[0]);
        let other_project = projects_dir.join("-Users-phaedrus-Development-other");
        fs::create_dir_all(&current_project).unwrap();
        fs::create_dir_all(&other_project).unwrap();
        let older = current_project.join("older.jsonl");
        let newer = current_project.join("newer.jsonl");
        let unrelated = other_project.join("unrelated.jsonl");
        fs::write(&older, "{}").unwrap();
        thread::sleep(Duration::from_millis(5));
        fs::write(&newer, "{}").unwrap();
        fs::write(&unrelated, "{}").unwrap();

        let resolved = resolve_transcript(None, true, &projects_dir, &cwd).unwrap();

        assert_eq!(
            resolved.file_name().and_then(|name| name.to_str()),
            Some("newer.jsonl")
        );
    }

    #[test]
    fn rejects_unrelated_project_transcript() {
        let tmp = TempDir::new().unwrap();
        let projects_dir = tmp.path().join("projects");
        let cwd = tmp.path().join("Users/phaedrus/Development/harness-kit");
        let other_project = projects_dir.join("-Users-phaedrus-Development-other");
        fs::create_dir_all(&cwd).unwrap();
        fs::create_dir_all(&other_project).unwrap();
        fs::write(other_project.join("unrelated.jsonl"), "{}").unwrap();

        let error = resolve_transcript(None, true, &projects_dir, &cwd).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("no Claude JSONL transcripts found")
        );
    }

    #[test]
    fn rejects_missing_transcript_source() {
        let tmp = TempDir::new().unwrap();
        let error = resolve_transcript(None, false, tmp.path(), tmp.path()).unwrap_err();

        assert_eq!(
            error.to_string(),
            "provide a transcript path or --from-current"
        );
    }
}
