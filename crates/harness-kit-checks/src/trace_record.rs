use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use regex::Regex;
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use uuid::Uuid;

use crate::source_refs;

pub const DEFAULT_STORE: &str = ".harness-kit/traces/work-records.jsonl";
const RECORD_TYPE: &str = "agent-session-trace";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppendOptions {
    pub store: PathBuf,
    pub backlog: String,
    pub spec_ref: String,
    pub branch: String,
    pub commits: Vec<String>,
    pub reviewer_verdict_refs: Vec<String>,
    pub qa_refs: Vec<String>,
    pub demo_refs: Vec<String>,
    pub transcript_refs: Vec<String>,
    pub shipped_ref: String,
    pub waiver_reason: String,
    pub metadata: Vec<String>,
    pub work_source_refs: Vec<JsonValue>,
}

pub fn default_store() -> PathBuf {
    PathBuf::from(DEFAULT_STORE)
}

pub fn build_record(options: &AppendOptions) -> Result<JsonValue> {
    let metadata = parse_metadata(&options.metadata)?;
    if options.transcript_refs.is_empty() && options.waiver_reason.is_empty() {
        bail!("provide at least one --transcript-ref or --waiver-reason");
    }
    source_refs::validate_refs(&options.work_source_refs, Some(&options.backlog))
        .context("invalid work source refs")?;

    let mut record = JsonMap::new();
    record.insert("backlog_ref".to_string(), json!(options.backlog));
    record.insert("branch".to_string(), json!(options.branch));
    record.insert("commits".to_string(), json!(options.commits));
    record.insert(
        "created_at".to_string(),
        json!(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
    );
    record.insert("demo_refs".to_string(), json!(options.demo_refs));
    record.insert("metadata".to_string(), JsonValue::Object(metadata));
    record.insert("qa_refs".to_string(), json!(options.qa_refs));
    record.insert("record_type".to_string(), json!(RECORD_TYPE));
    record.insert(
        "reviewer_verdict_refs".to_string(),
        json!(options.reviewer_verdict_refs),
    );
    record.insert("schema_version".to_string(), json!(1));
    record.insert("shipped_ref".to_string(), json!(options.shipped_ref));
    record.insert("spec_ref".to_string(), json!(options.spec_ref));
    record.insert(
        "trace_id".to_string(),
        json!(format!("trace-{}", Uuid::new_v4())),
    );
    record.insert(
        "transcript_refs".to_string(),
        json!(options.transcript_refs),
    );
    record.insert("waiver_reason".to_string(), json!(options.waiver_reason));
    if !options.work_source_refs.is_empty() {
        record.insert(
            source_refs::FIELD.to_string(),
            json!(options.work_source_refs),
        );
    }

    reject_secret_like(record.values())?;
    Ok(JsonValue::Object(record))
}

pub fn append(options: &AppendOptions) -> Result<JsonValue> {
    let record = build_record(options)?;
    append_record(&options.store, &record)?;
    Ok(json!({
        "store": options.store.to_string_lossy(),
        "trace_id": record.get("trace_id").and_then(JsonValue::as_str).unwrap_or_default(),
    }))
}

pub fn append_record(store: &Path, record: &JsonValue) -> Result<()> {
    if let Some(parent) = store.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(store)
        .with_context(|| format!("failed to open {}", store.display()))?;
    lock_file(&file)?;
    let write_result = writeln!(
        file,
        "{}",
        serde_json::to_string(record).context("failed to serialize trace record")?
    );
    let unlock_result = unlock_file(&file);
    write_result.with_context(|| format!("failed to append {}", store.display()))?;
    unlock_result?;
    Ok(())
}

pub fn parse_metadata(entries: &[String]) -> Result<JsonMap<String, JsonValue>> {
    let mut metadata = JsonMap::new();
    for entry in entries {
        let Some((raw_key, value)) = entry.split_once('=') else {
            bail!("metadata must be key=value: {entry}");
        };
        let key = raw_key.trim();
        if key.is_empty() {
            bail!("metadata key must be non-empty: {entry}");
        }
        metadata.insert(key.to_string(), json!(value));
    }
    Ok(metadata)
}

pub fn reject_secret_like<'a>(values: impl IntoIterator<Item = &'a JsonValue>) -> Result<()> {
    let secret_re = secret_regex();
    for value in values {
        reject_secret_value(value, &secret_re)?;
    }
    Ok(())
}

pub fn self_test() -> Result<()> {
    let temp = tempfile::tempdir().context("failed to create temporary directory")?;
    let store = temp.path().join("records.jsonl");
    let mut options = sample_options(store.clone());
    let record = build_record(&options)?;
    append_record(&store, &record)?;
    let rows: Vec<JsonValue> = fs::read_to_string(&store)?
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["backlog_ref"], "056");
    assert_eq!(rows[0]["metadata"], json!({"source": "self-test"}));

    options.transcript_refs.clear();
    options.waiver_reason.clear();
    let error = build_record(&options).unwrap_err().to_string();
    assert!(error.contains("transcript-ref"));

    options.waiver_reason = "No safe transcript export available.".to_string();
    options.metadata = vec!["API_TOKEN=leak".to_string()];
    let error = build_record(&options).unwrap_err().to_string();
    assert!(error.contains("secret-like"));
    Ok(())
}

fn reject_secret_value(value: &JsonValue, secret_re: &Regex) -> Result<()> {
    match value {
        JsonValue::Null => Ok(()),
        JsonValue::Bool(value) => reject_secret_text(&value.to_string(), secret_re),
        JsonValue::Number(value) => reject_secret_text(&value.to_string(), secret_re),
        JsonValue::String(value) => reject_secret_text(value, secret_re),
        JsonValue::Array(values) => {
            for value in values {
                reject_secret_value(value, secret_re)?;
            }
            Ok(())
        }
        JsonValue::Object(object) => {
            for (key, value) in object {
                reject_secret_text(key, secret_re)?;
                reject_secret_value(value, secret_re)?;
            }
            Ok(())
        }
    }
}

fn reject_secret_text(text: &str, secret_re: &Regex) -> Result<()> {
    if secret_re.is_match(text) {
        let preview: String = text.chars().take(80).collect();
        bail!("secret-like value refused: {preview}");
    }
    Ok(())
}

fn secret_regex() -> Regex {
    Regex::new(concat!(
        r"(?i)(api[_-]?key|token|secret|password|credential|",
        r"xai[_-]?api[_-]?key|exa[_-]?api[_-]?key|anthropic[_-]?api[_-]?key|",
        r"bearer\s+[a-z0-9._~+/-]+|-----BEGIN [A-Z ]*PRIVATE KEY-----|",
        r"private[_ -]?customer[_ -]?data)"
    ))
    .expect("trace secret regex compiles")
}

fn sample_options(store: PathBuf) -> AppendOptions {
    AppendOptions {
        store,
        backlog: "056".to_string(),
        spec_ref: "backlog.d/056-agent-session-trace-lifecycle.md".to_string(),
        branch: "deliver/056-agent-session-trace-lifecycle".to_string(),
        commits: vec!["abc1234".to_string()],
        reviewer_verdict_refs: vec![".harness-kit/traces/delegations.jsonl#abc".to_string()],
        qa_refs: vec![".evidence/qa/056.md".to_string()],
        demo_refs: vec![".evidence/demo/056.gif".to_string()],
        transcript_refs: vec![".harness-kit/traces/transcripts/056.md".to_string()],
        shipped_ref: "master@deadbeef".to_string(),
        waiver_reason: String::new(),
        metadata: vec!["source=self-test".to_string()],
        work_source_refs: Vec::new(),
    }
}

#[cfg(unix)]
fn lock_file(file: &File) -> Result<()> {
    let result = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_EX) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error()).context("failed to lock trace store")
    }
}

#[cfg(unix)]
fn unlock_file(file: &File) -> Result<()> {
    let result = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_UN) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error()).context("failed to unlock trace store")
    }
}

#[cfg(not(unix))]
fn lock_file(_file: &File) -> Result<()> {
    Ok(())
}

#[cfg(not(unix))]
fn unlock_file(_file: &File) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_record_appends_shell_contract_row() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("work-records.jsonl");
        let options = sample_options(store.clone());

        let receipt = append(&options)?;
        assert_eq!(receipt["store"].as_str().unwrap(), store.to_string_lossy());
        assert!(receipt["trace_id"].as_str().unwrap().starts_with("trace-"));

        let raw = fs::read_to_string(&store)?;
        assert!(raw.starts_with("{\"backlog_ref\":"));
        let rows: Vec<JsonValue> = raw
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
        assert_eq!(rows.len(), 1);
        let row = rows[0].as_object().unwrap();
        assert_eq!(row["schema_version"], 1);
        assert_eq!(row["record_type"], RECORD_TYPE);
        assert_eq!(row["backlog_ref"], "056");
        assert_eq!(row["branch"], "deliver/056-agent-session-trace-lifecycle");
        assert_eq!(row["commits"], json!(["abc1234"]));
        assert_eq!(
            row["reviewer_verdict_refs"],
            json!([".harness-kit/traces/delegations.jsonl#abc"])
        );
        assert_eq!(row["qa_refs"], json!([".evidence/qa/056.md"]));
        assert_eq!(row["demo_refs"], json!([".evidence/demo/056.gif"]));
        assert_eq!(
            row["transcript_refs"],
            json!([".harness-kit/traces/transcripts/056.md"])
        );
        assert_eq!(row["shipped_ref"], "master@deadbeef");
        assert_eq!(row["metadata"], json!({"source": "self-test"}));
        assert!(row["trace_id"].as_str().unwrap().starts_with("trace-"));
        assert!(row["created_at"].as_str().unwrap().ends_with('Z'));
        Ok(())
    }

    #[test]
    fn trace_record_requires_transcript_or_waiver() {
        let temp = tempfile::tempdir().unwrap();
        let mut options = sample_options(temp.path().join("records.jsonl"));
        options.transcript_refs.clear();
        let error = build_record(&options).unwrap_err().to_string();
        assert!(error.contains("transcript-ref"));
    }

    #[test]
    fn trace_record_rejects_secret_like_metadata_keys_and_values() {
        let temp = tempfile::tempdir().unwrap();
        let mut options = sample_options(temp.path().join("records.jsonl"));
        options.metadata = vec!["API_TOKEN=leak".to_string()];
        let error = build_record(&options).unwrap_err().to_string();
        assert!(error.contains("secret-like"));

        options.metadata = vec!["source=bearer abc123".to_string()];
        let error = build_record(&options).unwrap_err().to_string();
        assert!(error.contains("secret-like"));
    }

    #[test]
    fn trace_record_metadata_requires_key_value_and_nonempty_key() {
        assert!(parse_metadata(&["source=self-test".to_string()]).is_ok());
        assert!(parse_metadata(&["source".to_string()]).is_err());
        assert!(parse_metadata(&[" =value".to_string()]).is_err());
        let metadata = parse_metadata(&["source=first".to_string(), "source=last".to_string()])
            .expect("duplicate metadata keys are accepted");
        assert_eq!(metadata["source"], "last");
    }

    #[test]
    fn trace_record_records_optional_work_source_refs() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let mut options = sample_options(temp.path().join("records.jsonl"));
        options.work_source_refs = vec![json!({
            "role": "backlog",
            "kind": "local_backlog",
            "id": "056",
            "uri": "backlog.d/056-agent-session-trace-lifecycle.md"
        })];

        let row = build_record(&options)?;

        assert_eq!(
            row["work_source_refs"][0]["uri"],
            "backlog.d/056-agent-session-trace-lifecycle.md"
        );
        Ok(())
    }

    #[test]
    fn trace_record_self_test_covers_embedded_contract() -> Result<()> {
        self_test()
    }
}
