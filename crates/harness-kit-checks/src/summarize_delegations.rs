use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{SecondsFormat, Utc};
use serde_json::{Map, Value, json};

const REQUIRED_RECEIPT_FIELDS: &[&str] = &[
    "schema_version",
    "delegation_id",
    "created_at",
    "repo_root",
    "worktree_id",
    "lead_harness",
    "lead_provider",
    "backlog_ref",
    "objective",
    "input_ref",
    "provider_target",
    "provider_status",
    "attempt_status",
    "evidence_refs",
    "summary",
    "lead_verdict",
    "redactions_applied",
];
const OPTIONAL_RECEIPT_FIELDS: &[&str] = &[
    "model_id",
    "duration_ms",
    "usage",
    "transcript_bytes",
    "output_check",
];
const RECEIPT_PROVIDER_IDS: &[&str] = &[
    "codex",
    "claude",
    "pi",
    "agy",
    "cursor-agent",
    "grok-build",
    "manual",
    "opencode",
];
const VALID_PROVIDER_STATUS: &[&str] = &["available", "unavailable", "error", "partial", "manual"];
const VALID_ATTEMPT_STATUS: &[&str] = &[
    "not_started",
    "running",
    "succeeded",
    "failed",
    "rejected",
    "superseded",
    "manual",
];
const VALID_LEAD_VERDICTS: &[&str] = &[
    "accepted",
    "partially_accepted",
    "rejected",
    "reference_only",
    "pending",
];
const VALID_COST_SOURCES: &[&str] = &["provider_reported", "estimated", "manual", "unknown"];

#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSummary {
    pub total: usize,
    pub backlog_ref: String,
    pub providers: BTreeMap<String, BTreeMap<String, u64>>,
    pub provider_statuses: BTreeMap<String, BTreeMap<String, u64>>,
    pub usage_by_provider: BTreeMap<String, FinalUsageSummary>,
    pub lead_verdicts: BTreeMap<String, u64>,
    pub worktrees: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FinalUsageSummary {
    pub known_count: u64,
    pub unknown_count: u64,
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
    pub cost_sources: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReceiptInput {
    pub provider_target: String,
    pub provider_status: String,
    pub attempt_status: String,
    pub objective: String,
    pub input_ref: String,
    pub evidence_refs: Vec<String>,
    pub lead_verdict: String,
    pub worktree_id: String,
    pub backlog_ref: String,
    pub lead_harness: String,
    pub lead_provider: String,
    pub summary: String,
    pub model_id: Option<String>,
    pub duration_ms: Option<u64>,
    pub usage: Option<UsageInput>,
    pub transcript_bytes: Option<u64>,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct UsageInput {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub total_tokens: Option<u64>,
    pub cost_usd: Option<f64>,
    pub cost_source: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq)]
struct UsageAccumulator {
    known_count: u64,
    unknown_count: u64,
    input_tokens: u64,
    input_tokens_known_count: u64,
    output_tokens: u64,
    output_tokens_known_count: u64,
    total_tokens: u64,
    total_tokens_known_count: u64,
    cost_usd: f64,
    cost_usd_known_count: u64,
    cost_sources: BTreeMap<String, u64>,
}

pub fn summarize_receipts(path: &Path, backlog_ref: &str) -> Result<DelegationSummary> {
    let mut receipts = read_receipts(path)?;
    if !backlog_ref.is_empty() {
        receipts.retain(|receipt| {
            receipt
                .get("backlog_ref")
                .and_then(Value::as_str)
                .is_some_and(|value| value == backlog_ref)
        });
    }

    let mut providers = BTreeMap::new();
    let mut provider_statuses = BTreeMap::new();
    let mut usage_by_provider: BTreeMap<String, UsageAccumulator> = BTreeMap::new();
    let mut lead_verdicts = BTreeMap::new();
    let mut worktrees = BTreeMap::new();

    for receipt in &receipts {
        let provider = string_field(receipt, "provider_target")?.to_string();
        increment_nested(
            &mut providers,
            &provider,
            string_field(receipt, "attempt_status")?,
        );
        increment_nested(
            &mut provider_statuses,
            &provider,
            string_field(receipt, "provider_status")?,
        );
        add_usage(
            usage_by_provider.entry(provider).or_default(),
            receipt.get("usage"),
        );
        increment(&mut lead_verdicts, string_field(receipt, "lead_verdict")?);
        increment(&mut worktrees, string_field(receipt, "worktree_id")?);
    }

    Ok(DelegationSummary {
        total: receipts.len(),
        backlog_ref: backlog_ref.to_string(),
        providers,
        provider_statuses,
        usage_by_provider: usage_by_provider
            .into_iter()
            .map(|(provider, summary)| (provider, summary.finalize()))
            .collect(),
        lead_verdicts,
        worktrees,
    })
}

pub fn format_json(summary: &DelegationSummary) -> Result<String> {
    serde_json::to_string_pretty(&summary_to_value(summary)?).map_err(Into::into)
}

pub fn format_text(summary: &DelegationSummary) -> String {
    let mut lines = vec![
        "Roster delegation report".to_string(),
        format!(
            "backlog_ref: {}",
            if summary.backlog_ref.is_empty() {
                "all receipts"
            } else {
                &summary.backlog_ref
            }
        ),
        format!("total_receipts: {}", summary.total),
        "providers:".to_string(),
    ];
    if summary.providers.is_empty() {
        lines.push("  - none".to_string());
    } else {
        for (provider, attempts) in &summary.providers {
            let statuses = summary
                .provider_statuses
                .get(provider)
                .cloned()
                .unwrap_or_default();
            lines.push(format!(
                "  - {provider}: attempts[{}]; status[{}]",
                format_counts(attempts),
                format_counts(&statuses)
            ));
        }
    }

    lines.push("usage_by_provider:".to_string());
    if summary.usage_by_provider.is_empty() {
        lines.push("  - none".to_string());
    } else {
        for (provider, usage) in &summary.usage_by_provider {
            lines.push(format!(
                "  - {provider}: known={}; unknown={}; input_tokens={}; output_tokens={}; total_tokens={}; cost_usd={}; cost_sources[{}]",
                usage.known_count,
                usage.unknown_count,
                format_unknown(usage.input_tokens),
                format_unknown(usage.output_tokens),
                format_unknown(usage.total_tokens),
                usage
                    .cost_usd
                    .map(format_float)
                    .unwrap_or_else(|| "unknown".to_string()),
                format_counts(&usage.cost_sources)
            ));
        }
    }

    lines.push(format!(
        "lead_verdicts: {}",
        format_counts(&summary.lead_verdicts)
    ));
    lines.push(format!("worktrees: {}", format_counts(&summary.worktrees)));
    lines.join("\n")
}

pub fn build_attempt_receipt(input: ReceiptInput) -> Result<Map<String, Value>> {
    let mut receipt = Map::new();
    receipt.insert("schema_version".to_string(), json!(1));
    receipt.insert(
        "delegation_id".to_string(),
        json!(uuid::Uuid::new_v4().to_string()),
    );
    receipt.insert(
        "created_at".to_string(),
        json!(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
    );
    receipt.insert(
        "repo_root".to_string(),
        json!(repo_root().display().to_string()),
    );
    receipt.insert("worktree_id".to_string(), json!(input.worktree_id));
    receipt.insert("lead_harness".to_string(), json!(input.lead_harness));
    receipt.insert("lead_provider".to_string(), json!(input.lead_provider));
    receipt.insert("backlog_ref".to_string(), json!(input.backlog_ref));
    receipt.insert("objective".to_string(), json!(input.objective));
    receipt.insert("input_ref".to_string(), json!(input.input_ref));
    receipt.insert("provider_target".to_string(), json!(input.provider_target));
    receipt.insert("provider_status".to_string(), json!(input.provider_status));
    receipt.insert("attempt_status".to_string(), json!(input.attempt_status));
    receipt.insert("evidence_refs".to_string(), json!(input.evidence_refs));
    receipt.insert("summary".to_string(), json!(input.summary));
    receipt.insert("lead_verdict".to_string(), json!(input.lead_verdict));
    receipt.insert("redactions_applied".to_string(), json!([]));
    if let Some(model_id) = input.model_id {
        receipt.insert("model_id".to_string(), json!(model_id));
    }
    if let Some(duration_ms) = input.duration_ms {
        receipt.insert("duration_ms".to_string(), json!(duration_ms));
    }
    if let Some(usage) = input.usage {
        receipt.insert("usage".to_string(), usage_to_value(usage)?);
    }
    if let Some(transcript_bytes) = input.transcript_bytes {
        receipt.insert("transcript_bytes".to_string(), json!(transcript_bytes));
    }
    validate_receipt(&receipt)?;
    Ok(receipt)
}

pub fn append_receipt(path: &Path, receipt: &Map<String, Value>) -> Result<()> {
    validate_receipt(receipt)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    serde_json::to_writer(&mut file, &Value::Object(receipt.clone()))?;
    file.write_all(b"\n")?;
    Ok(())
}

pub fn validate_roster_path(path: &Path) -> Result<()> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read roster {}", path.display()))?;
    let value: serde_yaml::Value = serde_yaml::from_str(&text)
        .with_context(|| format!("failed to parse roster {}", path.display()))?;
    let object = value
        .as_mapping()
        .ok_or_else(|| anyhow!("{} must contain a YAML mapping.", path.display()))?;
    let version = object
        .get(serde_yaml::Value::String("version".to_string()))
        .and_then(serde_yaml::Value::as_i64);
    if version != Some(1) {
        bail!("roster version must be 1.");
    }
    if !object
        .get(serde_yaml::Value::String("providers".to_string()))
        .is_some_and(serde_yaml::Value::is_mapping)
    {
        bail!("roster must define providers mapping.");
    }
    Ok(())
}

fn read_receipts(path: &Path) -> Result<Vec<Map<String, Value>>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut receipts = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("{}:{line_number}: invalid JSON", path.display()))?;
        let object = value.as_object().ok_or_else(|| {
            anyhow!(
                "{}:{line_number}: receipt must be a JSON object",
                path.display()
            )
        })?;
        validate_receipt(object).with_context(|| format!("{}:{line_number}", path.display()))?;
        receipts.push(object.clone());
    }
    Ok(receipts)
}

fn usage_to_value(mut usage: UsageInput) -> Result<Value> {
    if usage.cost_source.is_none() {
        usage.cost_source = Some(if usage.cost_usd.is_some() {
            "manual".to_string()
        } else {
            "unknown".to_string()
        });
    }
    let mut object = Map::new();
    object.insert("input_tokens".to_string(), json!(usage.input_tokens));
    object.insert("output_tokens".to_string(), json!(usage.output_tokens));
    object.insert("total_tokens".to_string(), json!(usage.total_tokens));
    object.insert("cost_usd".to_string(), optional_float(usage.cost_usd)?);
    object.insert("cost_source".to_string(), json!(usage.cost_source));
    let value = Value::Object(object);
    validate_usage(&value)?;
    Ok(value)
}

fn validate_receipt(receipt: &Map<String, Value>) -> Result<()> {
    let actual: std::collections::BTreeSet<&str> = receipt.keys().map(String::as_str).collect();
    let required: std::collections::BTreeSet<&str> =
        REQUIRED_RECEIPT_FIELDS.iter().copied().collect();
    let optional: std::collections::BTreeSet<&str> =
        OPTIONAL_RECEIPT_FIELDS.iter().copied().collect();
    let missing: Vec<_> = required.difference(&actual).copied().collect();
    let allowed: std::collections::BTreeSet<_> = required.union(&optional).copied().collect();
    let extra: Vec<_> = actual.difference(&allowed).copied().collect();
    if !missing.is_empty() {
        bail!("receipt missing fields: {}", missing.join(", "));
    }
    if !extra.is_empty() {
        bail!("receipt has unknown fields: {}", extra.join(", "));
    }
    expect_i64(
        receipt,
        "schema_version",
        1,
        "receipt schema_version must be 1.",
    )?;
    uuid::Uuid::parse_str(string_field(receipt, "delegation_id")?)
        .context("receipt delegation_id must be a UUID.")?;
    expect_array(
        receipt,
        "redactions_applied",
        "receipt redactions_applied must be a list.",
    )?;
    expect_one_of(
        receipt,
        "provider_target",
        RECEIPT_PROVIDER_IDS,
        "receipt provider_target is not a known provider id.",
    )?;
    expect_one_of(
        receipt,
        "provider_status",
        VALID_PROVIDER_STATUS,
        "receipt provider_status is invalid.",
    )?;
    expect_one_of(
        receipt,
        "attempt_status",
        VALID_ATTEMPT_STATUS,
        "receipt attempt_status is invalid.",
    )?;
    expect_one_of(
        receipt,
        "lead_verdict",
        VALID_LEAD_VERDICTS,
        "receipt lead_verdict is invalid.",
    )?;
    validate_optional_nonnegative_int(receipt, "duration_ms")?;
    validate_optional_nonnegative_int(receipt, "transcript_bytes")?;
    if let Some(usage) = receipt.get("usage") {
        validate_usage(usage)?;
    }
    if let Some(output_check) = receipt.get("output_check") {
        validate_output_check(output_check)?;
    }
    let refs = expect_array(
        receipt,
        "evidence_refs",
        "receipt evidence_refs must be a list.",
    )?;
    for reference in refs {
        let Some(reference) = reference.as_str() else {
            bail!("receipt evidence_refs must contain strings.");
        };
        if reference.is_empty() {
            bail!("receipt evidence_refs must contain strings.");
        }
        if reference.chars().any(char::is_whitespace) {
            bail!("receipt evidence_refs must be paths or ids only.");
        }
    }
    Ok(())
}

fn validate_output_check(value: &Value) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    let Some(object) = value.as_object() else {
        bail!("output_check must be an object or null.");
    };
    let valid_fields = ["expected", "matched", "observed_ref"];
    let extra: Vec<_> = object
        .keys()
        .filter(|field| !valid_fields.contains(&field.as_str()))
        .cloned()
        .collect();
    if !extra.is_empty() {
        bail!("output_check has unknown fields: {}", extra.join(", "));
    }
    for field in ["expected", "observed_ref"] {
        if let Some(value) = object.get(field)
            && !value.is_null()
            && value.as_str().is_none_or(str::is_empty)
        {
            bail!("output_check {field} must be a non-empty string or null.");
        }
    }
    if let Some(value) = object.get("matched")
        && !value.is_null()
        && value.as_bool().is_none()
    {
        bail!("output_check matched must be a boolean or null.");
    }
    Ok(())
}

fn validate_usage(value: &Value) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    let Some(object) = value.as_object() else {
        bail!("usage must be an object or null.");
    };
    let valid_fields = [
        "input_tokens",
        "output_tokens",
        "total_tokens",
        "cost_usd",
        "cost_source",
    ];
    let extra: Vec<_> = object
        .keys()
        .filter(|field| !valid_fields.contains(&field.as_str()))
        .cloned()
        .collect();
    if !extra.is_empty() {
        bail!("usage has unknown fields: {}", extra.join(", "));
    }
    for field in ["input_tokens", "output_tokens", "total_tokens"] {
        if let Some(value) = object.get(field)
            && !value.is_null()
            && value.as_u64().is_none()
        {
            bail!("usage {field} must be a non-negative integer or null.");
        }
    }
    if let Some(cost) = object.get("cost_usd")
        && !cost.is_null()
        && cost.as_f64().is_none_or(|value| value < 0.0)
    {
        bail!("usage cost_usd must be a non-negative number or null.");
    }
    if let Some(source) = object.get("cost_source")
        && !source.is_null()
        && source
            .as_str()
            .is_none_or(|source| !VALID_COST_SOURCES.contains(&source))
    {
        bail!("usage cost_source is invalid.");
    }
    if object.get("cost_usd").is_some_and(|value| !value.is_null())
        && !object.contains_key("cost_source")
    {
        bail!("usage cost_source is required when cost_usd is known.");
    }
    Ok(())
}

fn summary_to_value(summary: &DelegationSummary) -> Result<Value> {
    let mut object = Map::new();
    object.insert("backlog_ref".to_string(), json!(summary.backlog_ref));
    object.insert("lead_verdicts".to_string(), json!(summary.lead_verdicts));
    object.insert(
        "provider_statuses".to_string(),
        json!(summary.provider_statuses),
    );
    object.insert("providers".to_string(), json!(summary.providers));
    object.insert("total".to_string(), json!(summary.total));
    object.insert(
        "usage_by_provider".to_string(),
        usage_by_provider_to_value(&summary.usage_by_provider)?,
    );
    object.insert("worktrees".to_string(), json!(summary.worktrees));
    Ok(Value::Object(object))
}

fn usage_by_provider_to_value(
    usage_by_provider: &BTreeMap<String, FinalUsageSummary>,
) -> Result<Value> {
    let mut providers = Map::new();
    for (provider, summary) in usage_by_provider {
        let mut object = Map::new();
        object.insert("cost_sources".to_string(), json!(summary.cost_sources));
        object.insert("cost_usd".to_string(), optional_float(summary.cost_usd)?);
        object.insert("input_tokens".to_string(), json!(summary.input_tokens));
        object.insert("known_count".to_string(), json!(summary.known_count));
        object.insert("output_tokens".to_string(), json!(summary.output_tokens));
        object.insert("total_tokens".to_string(), json!(summary.total_tokens));
        object.insert("unknown_count".to_string(), json!(summary.unknown_count));
        providers.insert(provider.clone(), Value::Object(object));
    }
    Ok(Value::Object(providers))
}

fn optional_float(value: Option<f64>) -> Result<Value> {
    match value {
        Some(value) => Ok(Value::Number(
            serde_json::Number::from_f64(value)
                .ok_or_else(|| anyhow!("usage cost_usd cannot be represented as JSON"))?,
        )),
        None => Ok(Value::Null),
    }
}

fn add_usage(summary: &mut UsageAccumulator, usage: Option<&Value>) {
    let Some(usage) = usage.and_then(Value::as_object) else {
        summary.unknown_count += 1;
        return;
    };
    let has_known_usage = ["input_tokens", "output_tokens", "total_tokens", "cost_usd"]
        .iter()
        .any(|field| usage.get(*field).is_some_and(|value| !value.is_null()));
    if !has_known_usage {
        summary.unknown_count += 1;
        return;
    }
    summary.known_count += 1;
    for field in ["input_tokens", "output_tokens", "total_tokens"] {
        let Some(value) = usage.get(field).and_then(Value::as_u64) else {
            continue;
        };
        match field {
            "input_tokens" => {
                summary.input_tokens += value;
                summary.input_tokens_known_count += 1;
            }
            "output_tokens" => {
                summary.output_tokens += value;
                summary.output_tokens_known_count += 1;
            }
            "total_tokens" => {
                summary.total_tokens += value;
                summary.total_tokens_known_count += 1;
            }
            _ => {}
        }
    }
    if let Some(cost_usd) = usage.get("cost_usd").and_then(Value::as_f64) {
        summary.cost_usd += cost_usd;
        summary.cost_usd_known_count += 1;
    }
    if let Some(source) = usage.get("cost_source").and_then(Value::as_str)
        && !source.is_empty()
    {
        increment(&mut summary.cost_sources, source);
    }
}

impl UsageAccumulator {
    fn finalize(self) -> FinalUsageSummary {
        FinalUsageSummary {
            known_count: self.known_count,
            unknown_count: self.unknown_count,
            input_tokens: (self.input_tokens_known_count > 0).then_some(self.input_tokens),
            output_tokens: (self.output_tokens_known_count > 0).then_some(self.output_tokens),
            total_tokens: (self.total_tokens_known_count > 0).then_some(self.total_tokens),
            cost_usd: (self.cost_usd_known_count > 0).then_some(round_six(self.cost_usd)),
            cost_sources: self.cost_sources,
        }
    }
}

fn increment(counts: &mut BTreeMap<String, u64>, key: &str) {
    *counts.entry(key.to_string()).or_default() += 1;
}

fn increment_nested(
    counts: &mut BTreeMap<String, BTreeMap<String, u64>>,
    outer: &str,
    inner: &str,
) {
    increment(counts.entry(outer.to_string()).or_default(), inner);
}

fn string_field<'a>(object: &'a Map<String, Value>, field: &str) -> Result<&'a str> {
    object
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("receipt {field} must be a string."))
}

fn expect_i64(object: &Map<String, Value>, field: &str, expected: i64, error: &str) -> Result<()> {
    if object.get(field).and_then(Value::as_i64) != Some(expected) {
        bail!("{error}");
    }
    Ok(())
}

fn expect_array<'a>(
    object: &'a Map<String, Value>,
    field: &str,
    error: &str,
) -> Result<&'a Vec<Value>> {
    object
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!(error.to_string()))
}

fn expect_one_of(
    object: &Map<String, Value>,
    field: &str,
    valid: &[&str],
    error: &str,
) -> Result<()> {
    if !object
        .get(field)
        .and_then(Value::as_str)
        .is_some_and(|value| valid.contains(&value))
    {
        bail!("{error}");
    }
    Ok(())
}

fn validate_optional_nonnegative_int(object: &Map<String, Value>, field: &str) -> Result<()> {
    if let Some(value) = object.get(field)
        && !value.is_null()
        && value.as_u64().is_none()
    {
        bail!("receipt {field} must be a non-negative integer or null.");
    }
    Ok(())
}

fn format_counts(counts: &BTreeMap<String, u64>) -> String {
    if counts.is_empty() {
        return "none".to_string();
    }
    counts
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_unknown<T: ToString>(value: Option<T>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn format_float(value: f64) -> String {
    let text = format!("{value:.6}");
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

fn round_six(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

fn repo_root() -> std::path::PathBuf {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(output) = output
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            return trimmed.into();
        }
    }
    env::current_dir().unwrap_or_else(|_| ".".into())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn summarizes_receipts_as_text() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("delegations.jsonl");
        fs::write(&path, fixture_rows())?;

        let summary = summarize_receipts(&path, "")?;

        assert_eq!(
            format_text(&summary),
            "Roster delegation report\nbacklog_ref: all receipts\ntotal_receipts: 2\nproviders:\n  - codex: attempts[succeeded=2]; status[available=2]\nusage_by_provider:\n  - codex: known=1; unknown=1; input_tokens=1000; output_tokens=200; total_tokens=1200; cost_usd=0.0123; cost_sources[provider_reported=1]\nlead_verdicts: accepted=2\nworktrees: codex-062=1, codex-089=1"
        );
        Ok(())
    }

    #[test]
    fn summarizes_receipts_as_sorted_json() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("delegations.jsonl");
        fs::write(&path, fixture_rows())?;

        let summary = summarize_receipts(&path, "")?;
        let json: Value = serde_json::from_str(&format_json(&summary)?)?;

        assert_eq!(
            json,
            serde_json::json!({
              "backlog_ref": "",
              "lead_verdicts": {"accepted": 2},
              "provider_statuses": {"codex": {"available": 2}},
              "providers": {"codex": {"succeeded": 2}},
              "total": 2,
              "usage_by_provider": {
                "codex": {
                  "cost_sources": {"provider_reported": 1},
                  "cost_usd": 0.0123,
                  "input_tokens": 1000,
                  "known_count": 1,
                  "output_tokens": 200,
                  "total_tokens": 1200,
                  "unknown_count": 1
                }
              },
              "worktrees": {"codex-062": 1, "codex-089": 1}
            })
        );
        Ok(())
    }

    #[test]
    fn filters_by_backlog_ref() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("delegations.jsonl");
        fs::write(&path, fixture_rows())?;

        let summary = summarize_receipts(&path, "089")?;

        assert_eq!(summary.total, 1);
        assert_eq!(summary.backlog_ref, "089");
        assert_eq!(summary.worktrees.get("codex-089"), Some(&1));
        Ok(())
    }

    #[test]
    fn builds_and_appends_receipt_with_usage() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("delegations.jsonl");

        let receipt = build_attempt_receipt(ReceiptInput {
            provider_target: "codex".to_string(),
            provider_status: "available".to_string(),
            attempt_status: "succeeded".to_string(),
            objective: "record fixture".to_string(),
            input_ref: "backlog.d/089-token-cost-observability-schema.md".to_string(),
            evidence_refs: vec!["evidence-089".to_string()],
            lead_verdict: "accepted".to_string(),
            worktree_id: "wt".to_string(),
            backlog_ref: "089".to_string(),
            lead_harness: "codex".to_string(),
            lead_provider: "codex".to_string(),
            summary: "done".to_string(),
            model_id: Some("gpt-5.5".to_string()),
            duration_ms: Some(10),
            usage: Some(UsageInput {
                input_tokens: Some(1),
                output_tokens: Some(2),
                total_tokens: Some(3),
                cost_usd: Some(0.4),
                cost_source: None,
            }),
            transcript_bytes: Some(42),
        })?;
        append_receipt(&path, &receipt)?;

        let lines: Vec<_> = fs::read_to_string(&path)?
            .lines()
            .map(str::to_string)
            .collect();
        assert_eq!(lines.len(), 1);
        let row: Value = serde_json::from_str(&lines[0])?;
        assert_eq!(row["provider_target"], "codex");
        assert_eq!(row["usage"]["cost_source"], "manual");
        assert_eq!(summarize_receipts(&path, "")?.total, 1);
        Ok(())
    }

    fn fixture_rows() -> String {
        [
            serde_json::json!({
                "schema_version": 1,
                "delegation_id": "9a118cc3-d4e5-4a06-a85a-f0547a7ad0ba",
                "created_at": "2026-06-04T00:00:00Z",
                "repo_root": "/tmp/harness-kit",
                "worktree_id": "codex-062",
                "lead_harness": "codex",
                "lead_provider": "codex",
                "backlog_ref": "062",
                "objective": "one",
                "input_ref": "input",
                "provider_target": "codex",
                "provider_status": "available",
                "attempt_status": "succeeded",
                "evidence_refs": ["receipt:one"],
                "summary": "done",
                "lead_verdict": "accepted",
                "redactions_applied": [],
                "usage": {
                    "input_tokens": 1000,
                    "output_tokens": 200,
                    "total_tokens": 1200,
                    "cost_usd": 0.0123,
                    "cost_source": "provider_reported"
                },
                "output_check": {
                    "expected": "AGENT_OK",
                    "matched": true,
                    "observed_ref": "receipt:one"
                }
            })
            .to_string(),
            serde_json::json!({
                "schema_version": 1,
                "delegation_id": "bafdefe2-1531-4d8b-8b81-7cd1e372976d",
                "created_at": "2026-06-04T00:01:00Z",
                "repo_root": "/tmp/harness-kit",
                "worktree_id": "codex-089",
                "lead_harness": "codex",
                "lead_provider": "codex",
                "backlog_ref": "089",
                "objective": "two",
                "input_ref": "input",
                "provider_target": "codex",
                "provider_status": "available",
                "attempt_status": "succeeded",
                "evidence_refs": ["receipt:two"],
                "summary": "done",
                "lead_verdict": "accepted",
                "redactions_applied": []
            })
            .to_string(),
        ]
        .join("\n")
    }
}
