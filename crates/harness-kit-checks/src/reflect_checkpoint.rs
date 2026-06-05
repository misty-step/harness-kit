use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde_json::Value;

const REQUIRED_FIELDS: &[&str] = &[
    "topic",
    "source_refs",
    "question",
    "operator_restatement",
    "lead_verdict",
    "gaps",
    "next_action",
    "timestamp",
];

pub fn validate_path(path: &Path) -> Result<()> {
    validate_checkpoint(&read_json(path)?)
}

pub fn gate(checkpoint: Option<&Path>, topic: &str, packet: Option<&Path>) -> Result<()> {
    let topics = required_topics(packet)?;
    if packet.is_none() || !topics.contains(topic) {
        return Ok(());
    }
    let Some(checkpoint) = checkpoint else {
        bail!("checkpoint required for topic {topic:?}");
    };
    let data = read_json(checkpoint)?;
    validate_checkpoint(&data)?;
    if text_field(&data, "topic")? != topic {
        bail!("checkpoint topic does not match gate topic");
    }
    if text_field(&data, "lead_verdict")? != "pass" {
        bail!("checkpoint gate requires lead_verdict pass");
    }
    if !array_of_strings(&data, "gaps")?.is_empty() {
        bail!("checkpoint gate requires empty gaps");
    }
    Ok(())
}

pub fn self_test() -> Result<&'static str> {
    let passing = serde_json::json!({
        "topic": "load-bearing-decision",
        "source_refs": ["backlog.d/096-reflect-teach-back-checkpoints.md"],
        "question": "What decision did we make, what can fail, and what happens next?",
        "operator_restatement": "We keep this opt-in, record refs only, and continue after a pass.",
        "lead_verdict": "pass",
        "gaps": [],
        "next_action": "Continue the session.",
        "timestamp": "2026-01-01T00:00:00Z"
    });
    validate_checkpoint(&passing)?;
    assert!(validate_checkpoint(&serde_json::json!({ "topic": "" })).is_err());
    let mut partial = passing.clone();
    partial["lead_verdict"] = Value::String("partial".to_string());
    partial["gaps"] = serde_json::json!(["Next action was unclear."]);
    validate_checkpoint(&partial)?;

    let temp = tempfile::TempDir::new()?;
    let pass_path = temp.path().join("pass.json");
    fs::write(&pass_path, serde_json::to_string(&passing)?)?;
    let partial_path = temp.path().join("partial.json");
    fs::write(&partial_path, serde_json::to_string(&partial)?)?;
    let packet = temp.path().join("packet.md");
    fs::write(&packet, "Comprehension-required: load-bearing-decision\n")?;
    let unrelated = temp.path().join("unrelated.md");
    fs::write(&unrelated, "Comprehension-required: other-topic\n")?;
    gate(Some(&pass_path), "load-bearing-decision", Some(&packet))?;
    gate(None, "load-bearing-decision", None)?;
    gate(
        Some(&partial_path),
        "load-bearing-decision",
        Some(&unrelated),
    )?;
    assert!(gate(Some(&partial_path), "load-bearing-decision", Some(&packet)).is_err());
    assert!(gate(None, "load-bearing-decision", Some(&packet)).is_err());
    Ok("reflect checkpoint self-test ok")
}

fn read_json(path: &Path) -> Result<Value> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("cannot read checkpoint {}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .with_context(|| format!("checkpoint {} is not valid JSON", path.display()))?;
    if !value.is_object() {
        bail!("checkpoint must be a JSON object");
    }
    Ok(value)
}

fn required_topics(packet: Option<&Path>) -> Result<BTreeSet<String>> {
    let Some(packet) = packet else {
        return Ok(BTreeSet::new());
    };
    let text = fs::read_to_string(packet)
        .with_context(|| format!("cannot read packet {}", packet.display()))?;
    let re = Regex::new(r"(?im)^Comprehension-required:\s*(.+?)\s*$").unwrap();
    Ok(re
        .captures_iter(&text)
        .map(|capture| capture[1].trim().to_string())
        .collect())
}

fn validate_checkpoint(data: &Value) -> Result<()> {
    let object = data
        .as_object()
        .context("checkpoint must be a JSON object")?;
    let fields: BTreeSet<_> = object.keys().map(String::as_str).collect();
    let required: BTreeSet<_> = REQUIRED_FIELDS.iter().copied().collect();
    let missing: Vec<_> = required.difference(&fields).copied().collect();
    if !missing.is_empty() {
        bail!("checkpoint missing field(s): {}", missing.join(", "));
    }
    let extra: Vec<_> = fields.difference(&required).copied().collect();
    if !extra.is_empty() {
        bail!("checkpoint has unknown field(s): {}", extra.join(", "));
    }

    let topic = text_field(data, "topic")?;
    let question = text_field(data, "question")?;
    let restatement = text_field(data, "operator_restatement")?;
    let next_action = text_field(data, "next_action")?;
    let verdict = text_field(data, "lead_verdict")?;
    let gaps = array_of_strings(data, "gaps")?;
    array_of_strings(data, "source_refs").and_then(validate_refs)?;
    validate_timestamp(text_field(data, "timestamp")?)?;

    if !["pass", "partial", "fail"].contains(&verdict) {
        bail!("lead_verdict must be pass, partial, or fail");
    }
    if verdict == "pass" && !gaps.is_empty() {
        bail!("lead_verdict pass requires empty gaps");
    }
    if ["partial", "fail"].contains(&verdict) && gaps.is_empty() {
        bail!("lead_verdict partial/fail requires at least one gap");
    }
    let joined_gaps = gaps.join("\n");
    for (name, value) in [
        ("topic", topic),
        ("question", question),
        ("operator_restatement", restatement),
        ("next_action", next_action),
        ("gaps", joined_gaps.as_str()),
    ] {
        if secret_or_raw(value) {
            bail!("checkpoint {name} contains raw/private content");
        }
    }
    if restatement.len() > 1000 {
        bail!("operator_restatement must be short; store refs, not transcripts");
    }
    Ok(())
}

fn text_field<'a>(data: &'a Value, field: &str) -> Result<&'a str> {
    let value = data
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .with_context(|| format!("{field} must be a non-empty string"))?;
    Ok(value)
}

fn array_of_strings(data: &Value, field: &str) -> Result<Vec<String>> {
    let value = data
        .get(field)
        .and_then(Value::as_array)
        .with_context(|| format!("{field} must be a list"))?;
    let mut strings = Vec::new();
    for item in value {
        let Some(text) = item.as_str().map(str::trim).filter(|text| !text.is_empty()) else {
            bail!("{field} must contain non-empty strings");
        };
        strings.push(text.to_string());
    }
    Ok(strings)
}

fn validate_refs(refs: Vec<String>) -> Result<()> {
    if refs.is_empty() {
        bail!("source_refs must be a non-empty list");
    }
    for reference in refs {
        if secret_or_raw(&reference) {
            bail!("source_refs contain raw/private content");
        }
    }
    Ok(())
}

fn validate_timestamp(value: &str) -> Result<()> {
    let parsed = DateTime::parse_from_rfc3339(&value.replace('Z', "+00:00"))
        .context("timestamp must be ISO-8601")?;
    if parsed.with_timezone(&Utc) > Utc::now() {
        bail!("timestamp must not be in the future");
    }
    Ok(())
}

fn secret_or_raw(value: &str) -> bool {
    Regex::new(r"(?i)(raw transcript|system prompt|developer prompt|raw tool output|private_customer_data|api[_-]?key\s*[:=]|authorization\s*[:=]|bearer\s+[a-z0-9._-]+|-----BEGIN [A-Z ]*PRIVATE KEY-----)")
        .unwrap()
        .is_match(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_contract_passes() {
        assert_eq!(self_test().unwrap(), "reflect checkpoint self-test ok");
    }
}
