use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use regex::Regex;
use sha2::{Digest, Sha256};

const REQUIRED_VOICE_METADATA: &[&str] = &[
    "source_kind",
    "source_hash",
    "transcript_model",
    "transcript_confidence",
    "audio_duration_seconds",
    "redaction_status",
    "redaction_tool",
    "created_at",
    "residual_risk",
];

pub fn validate_packet(repo: &Path, path: &Path) -> Result<()> {
    let text =
        fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?;
    let Some(block) = section(&text, "Premise Source") else {
        if !requires_premise(&text) {
            return Ok(());
        }
        bail!("missing ## Premise Source section");
    };
    if let Some(reason) = capture_line(&block, r"(?im)^Premise Source Waiver:\s*(.+)$") {
        let residual = capture_line(&block, r"(?im)^Residual risk:\s*(.+)$");
        if reason.trim().len() < 12 {
            bail!("premise source waiver has no reason");
        }
        if residual.as_deref().unwrap_or("").trim().len() < 12 {
            bail!("premise source waiver missing residual risk");
        }
        return Ok(());
    }
    if Regex::new(r"(?i)(raw transcript|system prompt|developer prompt|tool output|bearer\s+[a-z0-9._-]+|api[_-]?key\s*[:=])")
        .unwrap()
        .is_match(&block)
    {
        bail!("premise source block appears to include raw transcript or secret-like text");
    }
    let source = Regex::new(r"(?im)^Premise Source:\s+sha256:([0-9a-fA-F]{64})\s+(\S+)\s*$")
        .unwrap()
        .captures(&block)
        .context("missing Premise Source: sha256:<digest> <path-or-url>")?;
    let expected = source[1].to_lowercase();
    let source_ref = &source[2];
    if is_url(source_ref) {
        validate_voice_metadata(&block, &expected)?;
        return Ok(());
    }
    let source_path = resolve_source(repo, source_ref, path);
    if !source_path.exists() {
        bail!("premise source path does not exist");
    }
    if !source_path.is_file() {
        bail!("premise source path is not a file");
    }
    let actual = sha256(&source_path)?;
    if actual != expected {
        bail!("premise source hash mismatch: expected {expected}, got {actual}");
    }
    validate_voice_metadata(&block, &expected)
}

pub fn self_test(repo: &Path) -> Result<&'static str> {
    let cases = repo.join("skills/shape/evals/cases");
    for name in [
        "premise-source-valid.md",
        "premise-source-voice-valid.md",
        "premise-source-voice-unknowns.md",
        "premise-source-waiver.md",
        "premise-source-small-skip.md",
    ] {
        validate_packet(repo, &cases.join(name))
            .with_context(|| format!("valid fixture failed: {name}"))?;
    }
    let invalid = [
        "premise-source-missing.md",
        "premise-source-missing-path.md",
        "premise-source-bad-hash.md",
        "premise-source-raw-transcript.md",
        "premise-source-voice-missing-hash.md",
        "premise-source-voice-missing-unknowns.md",
        "premise-source-voice-raw-audio-path.md",
    ];
    let rejected = invalid
        .iter()
        .filter(|name| validate_packet(repo, &cases.join(name)).is_err())
        .count();
    if rejected != invalid.len() {
        bail!("self-test failed to reject all invalid premise-source fixtures");
    }
    Ok("premise-source checker self-test ok")
}

fn section(text: &str, heading: &str) -> Option<String> {
    let wanted = format!("## {heading}");
    let mut in_section = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim() == wanted {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if in_section {
            lines.push(line);
        }
    }
    in_section.then(|| lines.join("\n").trim().to_string())
}

fn requires_premise(text: &str) -> bool {
    let Some(estimate) = capture_line(text, r"(?im)^Estimate:\s*(\S+)\s*$") else {
        return true;
    };
    !matches!(
        estimate.trim().to_lowercase().as_str(),
        "xs" | "s" | "small" | "trivial"
    )
}

fn capture_line(text: &str, pattern: &str) -> Option<String> {
    Regex::new(pattern)
        .unwrap()
        .captures(text)
        .map(|capture| capture[1].to_string())
}

fn is_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn resolve_source(repo: &Path, value: &str, packet_path: &Path) -> PathBuf {
    let source = PathBuf::from(value);
    if source.is_absolute() {
        return source;
    }
    let repo_candidate = repo.join(&source);
    if repo_candidate.exists() {
        repo_candidate
    } else {
        packet_path.parent().unwrap_or(repo).join(source)
    }
}

fn sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

fn validate_voice_metadata(block: &str, expected_hash: &str) -> Result<()> {
    if Regex::new(r"(?i)(?:raw_audio_path\s*:|(?:^|\s)\S+\.(?:wav|mp3|m4a|flac|aac|aiff|ogg)\b)")
        .unwrap()
        .is_match(block)
    {
        bail!("voice transcript metadata must not retain raw audio paths");
    }
    let Some(metadata) = parse_metadata(block) else {
        return Ok(());
    };
    let missing: Vec<_> = REQUIRED_VOICE_METADATA
        .iter()
        .filter(|key| !metadata.contains_key(**key))
        .copied()
        .collect();
    if !missing.is_empty() {
        bail!(
            "voice transcript metadata missing field(s): {}",
            missing.join(", ")
        );
    }
    if !["voice", "raw_transcript"].contains(&metadata["source_kind"].as_str()) {
        bail!("voice transcript metadata source_kind is invalid");
    }
    let source_hash = &metadata["source_hash"];
    if !Regex::new(r"^sha256:[0-9a-fA-F]{64}$")
        .unwrap()
        .is_match(source_hash)
    {
        bail!("voice transcript metadata source_hash must be sha256:<64 hex>");
    }
    if source_hash.split_once(':').unwrap().1.to_lowercase() != expected_hash {
        bail!("voice transcript metadata source_hash must match Premise Source digest");
    }
    validate_unknown_or_float(
        &metadata["transcript_confidence"],
        0.0,
        1.0,
        "transcript_confidence",
    )?;
    validate_unknown_or_float(
        &metadata["audio_duration_seconds"],
        0.0,
        f64::MAX,
        "audio_duration_seconds",
    )?;
    if !["redacted", "sanitized"].contains(&metadata["redaction_status"].as_str()) {
        bail!("voice transcript metadata redaction_status is invalid");
    }
    if metadata["redaction_tool"].len() < 3 {
        bail!("voice transcript metadata redaction_tool must be set");
    }
    if metadata["residual_risk"].len() < 12 {
        bail!("voice transcript metadata residual_risk must be substantive");
    }
    let parsed = DateTime::parse_from_rfc3339(&metadata["created_at"].replace('Z', "+00:00"))
        .context("voice transcript metadata created_at must be ISO-8601")?;
    if parsed.with_timezone(&Utc) > Utc::now() {
        bail!("voice transcript metadata created_at must not be in the future");
    }
    Ok(())
}

fn parse_metadata(block: &str) -> Option<std::collections::BTreeMap<String, String>> {
    if !Regex::new(
        r"(?im)^Voice Transcript Metadata:\s*$|^-\s+source_kind:\s*(voice|raw_transcript)\s*$",
    )
    .unwrap()
    .is_match(block)
    {
        return None;
    }
    let line_re = Regex::new(r"^-\s+([a-z_]+):\s*(.*)\s*$").unwrap();
    let mut metadata = std::collections::BTreeMap::new();
    for line in block.lines() {
        if let Some(capture) = line_re.captures(line.trim()) {
            metadata.insert(capture[1].to_string(), capture[2].trim().to_string());
        }
    }
    Some(metadata)
}

fn validate_unknown_or_float(value: &str, min: f64, max: f64, field: &str) -> Result<()> {
    if value == "unknown" {
        return Ok(());
    }
    let parsed: f64 = value
        .parse()
        .with_context(|| format!("voice transcript metadata {field} must be unknown or numeric"))?;
    if parsed < min || parsed > max {
        bail!("voice transcript metadata {field} must be between {min} and {max}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_contract_passes() {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap();
        assert_eq!(
            self_test(repo).unwrap(),
            "premise-source checker self-test ok"
        );
    }
}
