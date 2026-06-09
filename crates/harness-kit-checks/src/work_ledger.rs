use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use serde_json::{Map as JsonMap, Value as JsonValue, json};
use uuid::Uuid;

use crate::source_refs;

pub const DEFAULT_STORE: &str = ".harness-kit/work/ledger.jsonl";
const RECORD_TYPE: &str = "work-ledger-event";

const ACTIVE_STATUSES: &[&str] = &["active", "blocked"];
const VALID_STATUSES: &[&str] = &["active", "blocked", "completed", "failed", "superseded"];
const VALID_COST_SOURCES: &[&str] = &["provider_reported", "estimated", "manual", "unknown"];
const VALID_EVENT_TYPES: &[&str] = &[
    "phase_started",
    "phase_completed",
    "blocker_added",
    "next_action_changed",
];

#[derive(Debug, Clone, PartialEq)]
pub struct AppendOptions {
    pub store: PathBuf,
    pub event_type: String,
    pub work_id: String,
    pub parent_work_id: String,
    pub backlog: String,
    pub branch: String,
    pub owning_skill: String,
    pub phase: String,
    pub evidence_refs: Vec<String>,
    pub blockers: Vec<String>,
    pub spawned_agents: Vec<String>,
    pub trace_refs: Vec<String>,
    pub next_action: String,
    pub status: String,
    pub usage: Option<JsonValue>,
    pub work_source_refs: Vec<JsonValue>,
}

pub fn default_store() -> PathBuf {
    PathBuf::from(DEFAULT_STORE)
}

pub fn build_event(options: &AppendOptions) -> Result<JsonValue> {
    validate_enum("event_type", &options.event_type, VALID_EVENT_TYPES)?;
    validate_enum("status", &options.status, VALID_STATUSES)?;
    if let Some(usage) = &options.usage {
        validate_usage(usage).context("invalid --usage-json")?;
    }
    source_refs::validate_refs(&options.work_source_refs, Some(&options.backlog))
        .context("invalid work source refs")?;

    let mut event = JsonMap::new();
    event.insert("backlog_ref".to_string(), json!(options.backlog));
    event.insert("blockers".to_string(), json!(options.blockers));
    event.insert("branch".to_string(), json!(options.branch));
    event.insert(
        "created_at".to_string(),
        json!(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
    );
    event.insert(
        "event_id".to_string(),
        json!(format!("work-{}", Uuid::new_v4())),
    );
    event.insert("event_type".to_string(), json!(options.event_type));
    event.insert("evidence_refs".to_string(), json!(options.evidence_refs));
    event.insert("next_action".to_string(), json!(options.next_action));
    event.insert("owning_skill".to_string(), json!(options.owning_skill));
    event.insert("parent_work_id".to_string(), json!(options.parent_work_id));
    event.insert("phase".to_string(), json!(options.phase));
    event.insert("record_type".to_string(), json!(RECORD_TYPE));
    event.insert("schema_version".to_string(), json!(1));
    event.insert("spawned_agents".to_string(), json!(options.spawned_agents));
    event.insert("status".to_string(), json!(options.status));
    event.insert("trace_refs".to_string(), json!(options.trace_refs));
    if !options.work_source_refs.is_empty() {
        event.insert(
            source_refs::FIELD.to_string(),
            json!(options.work_source_refs),
        );
    }
    if let Some(usage) = &options.usage {
        event.insert("usage".to_string(), usage.clone());
    }
    event.insert("work_id".to_string(), json!(options.work_id));
    Ok(JsonValue::Object(event))
}

pub fn append_event(store: &Path, event: &JsonValue) -> Result<()> {
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
        serde_json::to_string(event).context("failed to serialize work-ledger event")?
    );
    let unlock_result = unlock_file(&file);
    write_result.with_context(|| format!("failed to append {}", store.display()))?;
    unlock_result?;
    Ok(())
}

pub fn append(options: &AppendOptions) -> Result<JsonValue> {
    let event = build_event(options)?;
    append_event(&options.store, &event)?;
    Ok(json!({
        "event_id": event.get("event_id").and_then(JsonValue::as_str).unwrap_or_default(),
        "store": options.store.to_string_lossy(),
    }))
}

pub fn read_events(store: &Path) -> Result<Vec<JsonMap<String, JsonValue>>> {
    if !store.exists() {
        return Ok(Vec::new());
    }
    let file = File::open(store).with_context(|| format!("failed to read {}", store.display()))?;
    let mut events = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line_number = index + 1;
        let line = line.with_context(|| format!("{}:{line_number}", store.display()))?;
        if line.trim().is_empty() {
            continue;
        }
        let event: JsonValue = serde_json::from_str(&line)
            .with_context(|| format!("{}:{line_number}", store.display()))?;
        let JsonValue::Object(object) = event else {
            bail!(
                "{}:{line_number}: event must be a JSON object",
                store.display()
            );
        };
        events.push(object);
    }
    Ok(events)
}

pub fn summary(store: &Path) -> Result<String> {
    summary_text(&read_events(store)?)
}

pub fn summary_text(events: &[JsonMap<String, JsonValue>]) -> Result<String> {
    let mut latest: JsonMap<String, JsonValue> = JsonMap::new();
    for event in events {
        let work_id = event
            .get("work_id")
            .ok_or_else(|| anyhow::anyhow!("event missing work_id"))?
            .to_string();
        latest.insert(work_id, JsonValue::Object(event.clone()));
    }

    let mut active: Vec<JsonMap<String, JsonValue>> = latest
        .values()
        .filter_map(JsonValue::as_object)
        .filter(|event| {
            event
                .get("status")
                .and_then(JsonValue::as_str)
                .is_some_and(|status| ACTIVE_STATUSES.contains(&status))
        })
        .cloned()
        .collect();
    if active.is_empty() {
        return Ok("No active work ledger entries.".to_string());
    }
    active.sort_by_key(|event| string_field(event, "created_at"));

    let mut lines = vec!["Work ledger".to_string()];
    for event in active {
        let latest_evidence = event
            .get("evidence_refs")
            .and_then(JsonValue::as_array)
            .and_then(|values| values.last())
            .map(format_scalar)
            .unwrap_or_else(|| "none".to_string());
        lines.extend([
            format!("- work_id: {}", string_field(&event, "work_id")),
            format!("  branch: {}", string_field(&event, "branch")),
            format!("  backlog: {}", string_field(&event, "backlog_ref")),
            format!("  event_type: {}", string_field(&event, "event_type")),
            format!("  owning_skill: {}", string_field(&event, "owning_skill")),
            format!("  phase: {}", string_field(&event, "phase")),
            format!("  status: {}", string_field(&event, "status")),
            format!("  latest_evidence: {latest_evidence}"),
            format!("  blockers: {}", format_list(event.get("blockers"))),
            format!(
                "  spawned_agents: {}",
                format_list(event.get("spawned_agents"))
            ),
            format!("  trace_refs: {}", format_list(event.get("trace_refs"))),
            format!("  next_action: {}", string_field(&event, "next_action")),
        ]);
    }
    Ok(lines.join("\n"))
}

pub fn parse_usage_json(value: Option<&str>) -> Result<Option<JsonValue>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let usage: JsonValue =
        serde_json::from_str(value).with_context(|| format!("invalid --usage-json: {value}"))?;
    if usage.is_null() {
        return Ok(None);
    }
    validate_usage(&usage).context("invalid --usage-json")?;
    Ok(Some(usage))
}

pub fn validate_usage(usage: &JsonValue) -> Result<()> {
    let Some(object) = usage.as_object() else {
        bail!("usage must be an object or null.");
    };
    for key in object.keys() {
        if ![
            "input_tokens",
            "output_tokens",
            "total_tokens",
            "cost_usd",
            "cost_source",
        ]
        .contains(&key.as_str())
        {
            bail!("usage has unknown fields: {key}");
        }
    }
    for token_field in ["input_tokens", "output_tokens", "total_tokens"] {
        if let Some(value) = object.get(token_field)
            && !(value.is_null() || value.as_u64().is_some())
        {
            bail!("usage {token_field} must be a non-negative integer or null.");
        }
    }
    if let Some(value) = object.get("cost_usd")
        && !(value.is_null() || value.as_f64().is_some_and(|number| number >= 0.0))
    {
        bail!("usage cost_usd must be a non-negative number or null.");
    }
    if let Some(value) = object.get("cost_source")
        && !(value.is_null()
            || value
                .as_str()
                .is_some_and(|source| VALID_COST_SOURCES.contains(&source)))
    {
        bail!("usage cost_source is invalid.");
    }
    if object.get("cost_usd").is_some_and(|value| !value.is_null())
        && object
            .get("cost_source")
            .is_none_or(|value| value.is_null())
    {
        bail!("usage cost_source is required when cost_usd is known.");
    }
    Ok(())
}

pub fn self_test() -> Result<()> {
    let temp = tempfile::tempdir().context("failed to create temporary directory")?;
    let store = temp.path().join("ledger.jsonl");
    let mut options = AppendOptions {
        store: store.clone(),
        event_type: "phase_started".to_string(),
        work_id: "058".to_string(),
        parent_work_id: String::new(),
        backlog: "058".to_string(),
        branch: "deliver/058-work-ledger-mission-control".to_string(),
        owning_skill: "deliver".to_string(),
        phase: "review".to_string(),
        evidence_refs: vec![".harness-kit/traces/delegations.jsonl#abc".to_string()],
        blockers: vec!["waiting for critic".to_string()],
        spawned_agents: vec!["grok-build:critic".to_string()],
        trace_refs: vec![".harness-kit/traces/work-records.jsonl#trace-abc".to_string()],
        next_action: "address critic output".to_string(),
        status: "active".to_string(),
        usage: Some(json!({
            "input_tokens": 100,
            "output_tokens": 25,
            "total_tokens": 125,
            "cost_usd": 0.01,
            "cost_source": "manual",
        })),
        work_source_refs: Vec::new(),
    };
    append_event(&store, &build_event(&options)?)?;
    let text = summary(&store)?;
    assert!(text.contains("backlog: 058"));
    assert!(text.contains("phase: review"));
    assert!(text.contains("trace_refs: .harness-kit/traces/work-records.jsonl#trace-abc"));

    options.phase = "done".to_string();
    options.event_type = "phase_completed".to_string();
    options.status = "completed".to_string();
    options.blockers.clear();
    options.spawned_agents.clear();
    options.trace_refs.clear();
    options.evidence_refs.clear();
    options.next_action = "none".to_string();
    options.usage = None;
    append_event(&store, &build_event(&options)?)?;
    assert_eq!(summary(&store)?, "No active work ledger entries.");
    Ok(())
}

fn validate_enum(name: &str, value: &str, valid: &[&str]) -> Result<()> {
    if valid.contains(&value) {
        Ok(())
    } else {
        bail!("{name} must be one of: {}", valid.join(", "))
    }
}

fn string_field(event: &JsonMap<String, JsonValue>, key: &str) -> String {
    event.get(key).map(format_scalar).unwrap_or_default()
}

fn format_list(value: Option<&JsonValue>) -> String {
    let Some(value) = value else {
        return "none".to_string();
    };
    if let Some(values) = value.as_array() {
        if values.is_empty() {
            "none".to_string()
        } else {
            values
                .iter()
                .map(format_scalar)
                .collect::<Vec<_>>()
                .join(", ")
        }
    } else {
        format_scalar(value)
    }
}

fn format_scalar(value: &JsonValue) -> String {
    if let Some(value) = value.as_str() {
        value.to_string()
    } else {
        value.to_string()
    }
}

#[cfg(unix)]
fn lock_file(file: &File) -> Result<()> {
    let result = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_EX) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error()).context("failed to lock work-ledger store")
    }
}

#[cfg(unix)]
fn unlock_file(file: &File) -> Result<()> {
    let result = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_UN) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error()).context("failed to unlock work-ledger store")
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

    fn sample_options(store: PathBuf) -> AppendOptions {
        AppendOptions {
            store,
            event_type: "phase_started".to_string(),
            work_id: "058".to_string(),
            parent_work_id: "flywheel-058".to_string(),
            backlog: "058".to_string(),
            branch: "deliver/058-work-ledger-mission-control".to_string(),
            owning_skill: "deliver".to_string(),
            phase: "review".to_string(),
            evidence_refs: vec![".harness-kit/traces/delegations.jsonl#abc".to_string()],
            blockers: vec!["waiting for critic".to_string()],
            spawned_agents: vec!["grok-build:critic".to_string()],
            trace_refs: vec![".harness-kit/traces/work-records.jsonl#trace-abc".to_string()],
            next_action: "address critic output".to_string(),
            status: "active".to_string(),
            usage: None,
            work_source_refs: Vec::new(),
        }
    }

    #[test]
    fn work_ledger_appends_compact_event_and_summary_matches_shell_contract() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("nested/ledger.jsonl");
        let options = sample_options(store.clone());

        let receipt = append(&options)?;
        assert_eq!(receipt["store"].as_str().unwrap(), store.to_string_lossy());
        assert!(receipt["event_id"].as_str().unwrap().starts_with("work-"));

        let rows: Vec<JsonValue> = fs::read_to_string(&store)?
            .lines()
            .map(serde_json::from_str)
            .collect::<Result<_, _>>()?;
        let raw = fs::read_to_string(&store)?;
        assert!(raw.starts_with("{\"backlog_ref\":"));
        assert_eq!(rows.len(), 1);
        let row = rows[0].as_object().unwrap();
        assert_eq!(row["schema_version"], 1);
        assert_eq!(row["record_type"], RECORD_TYPE);
        assert_eq!(row["event_type"], "phase_started");
        assert_eq!(row["work_id"], "058");
        assert_eq!(row["parent_work_id"], "flywheel-058");
        assert_eq!(row["backlog_ref"], "058");
        assert_eq!(row["branch"], "deliver/058-work-ledger-mission-control");
        assert_eq!(row["owning_skill"], "deliver");
        assert_eq!(row["phase"], "review");
        assert_eq!(
            row["evidence_refs"],
            json!([".harness-kit/traces/delegations.jsonl#abc"])
        );
        assert_eq!(row["blockers"], json!(["waiting for critic"]));
        assert_eq!(row["spawned_agents"], json!(["grok-build:critic"]));
        assert_eq!(
            row["trace_refs"],
            json!([".harness-kit/traces/work-records.jsonl#trace-abc"])
        );
        assert_eq!(row["next_action"], "address critic output");
        assert_eq!(row["status"], "active");
        assert!(row["created_at"].as_str().unwrap().ends_with('Z'));

        let text = summary(&store)?;
        assert!(text.contains("branch: deliver/058-work-ledger-mission-control"));
        assert!(text.contains("backlog: 058"));
        assert!(text.contains("event_type: phase_started"));
        assert!(text.contains("phase: review"));
        assert!(text.contains("latest_evidence: .harness-kit/traces/delegations.jsonl#abc"));
        assert!(text.contains("blockers: waiting for critic"));
        assert!(text.contains("spawned_agents: grok-build:critic"));
        assert!(text.contains("trace_refs: .harness-kit/traces/work-records.jsonl#trace-abc"));
        assert!(text.contains("next_action: address critic output"));
        Ok(())
    }

    #[test]
    fn work_ledger_records_optional_work_source_refs() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let mut options = sample_options(temp.path().join("ledger.jsonl"));
        options.work_source_refs = vec![json!({
            "role": "backlog",
            "kind": "local_backlog",
            "id": "058",
            "uri": "backlog.d/058-work-ledger-mission-control.md",
            "closure": {"mode": "local_archive"}
        })];

        let row = build_event(&options)?;

        assert_eq!(
            row["work_source_refs"][0]["uri"],
            "backlog.d/058-work-ledger-mission-control.md"
        );
        Ok(())
    }

    #[test]
    fn work_ledger_rejects_local_backlog_ref_mismatch() {
        let temp = tempfile::tempdir().unwrap();
        let mut options = sample_options(temp.path().join("ledger.jsonl"));
        options.work_source_refs = vec![json!({
            "role": "backlog",
            "kind": "local_backlog",
            "id": "059"
        })];

        let error = build_event(&options).unwrap_err().to_string();
        assert!(error.contains("work source refs"));
    }

    #[test]
    fn work_ledger_latest_terminal_status_hides_active_work() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("ledger.jsonl");
        let mut options = sample_options(store.clone());
        append(&options)?;

        options.event_type = "phase_completed".to_string();
        options.phase = "done".to_string();
        options.status = "completed".to_string();
        options.evidence_refs.clear();
        options.blockers.clear();
        options.spawned_agents.clear();
        options.trace_refs.clear();
        options.next_action = "none".to_string();
        append(&options)?;

        assert_eq!(summary(&store)?, "No active work ledger entries.");
        Ok(())
    }

    #[test]
    fn work_ledger_missing_store_summarizes_empty() -> Result<()> {
        let temp = tempfile::tempdir()?;
        assert_eq!(
            summary(&temp.path().join("missing.jsonl"))?,
            "No active work ledger entries."
        );
        Ok(())
    }

    #[test]
    fn work_ledger_rejects_bad_jsonl_and_non_object_rows() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("ledger.jsonl");
        fs::write(&store, "[]\n")?;
        let error = read_events(&store).unwrap_err().to_string();
        assert!(error.contains("event must be a JSON object"));

        fs::write(&store, "{bad\n")?;
        let error = read_events(&store).unwrap_err().to_string();
        assert!(error.contains("ledger.jsonl:1"));
        Ok(())
    }

    #[test]
    fn work_ledger_usage_validation_matches_python_contract() {
        assert!(validate_usage(&json!({"cost_usd": 0.01, "cost_source": "manual"})).is_ok());
        assert!(validate_usage(&json!({"input_tokens": true})).is_err());
        assert!(validate_usage(&json!({"input_tokens": -1})).is_err());
        assert!(validate_usage(&json!({"cost_usd": 0.01})).is_err());
        assert!(validate_usage(&json!({"cost_source": "bogus"})).is_err());
        assert!(validate_usage(&json!({"extra": 1})).is_err());
        assert!(validate_usage(&json!(null)).is_err());
        assert_eq!(parse_usage_json(Some("null")).unwrap(), None);
    }

    #[test]
    fn work_ledger_self_test_covers_embedded_contract() -> Result<()> {
        self_test()
    }
}
