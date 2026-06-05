use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::Path;

use anyhow::{Context, Result, bail};
use chrono::{SecondsFormat, Utc};
use serde_json::{Map, Value, json};

const SCHEMA_VERSION: u64 = 1;
const KNOWN_KINDS: &[&str] = &[
    "cycle.opened",
    "deliver.done",
    "deploy.done",
    "monitor.done",
    "monitor.alert",
    "triage.done",
    "reflect.done",
    "bucket.updated",
    "harness.suggested",
    "phase.failed",
    "budget.exhausted",
    "cycle.closed",
];

pub fn known_kinds() -> &'static [&'static str] {
    KNOWN_KINDS
}

pub fn emit_event(
    log: &Path,
    kind: &str,
    phase: &str,
    agent: &str,
    payload: &str,
) -> Result<Value> {
    if !KNOWN_KINDS.contains(&kind) {
        bail!("emit_event: unknown kind '{kind}'");
    }
    if kind.is_empty() {
        bail!("emit_event: kind is required");
    }
    if phase.is_empty() {
        bail!("emit_event: phase is required");
    }

    let payload: Value =
        serde_json::from_str(payload).context("emit_event: payload is not JSON")?;
    let Some(payload_object) = payload.as_object() else {
        bail!("emit_event: payload must be a JSON object");
    };

    let mut event: Map<String, Value> = payload_object.clone();
    event.insert("kind".to_string(), json!(kind));
    event.insert("phase".to_string(), json!(phase));
    event.insert("agent".to_string(), json!(agent));
    event.insert("schema_version".to_string(), json!(SCHEMA_VERSION));
    event.insert("cycle_id".to_string(), json!(cycle_id_from_log(log)));
    event.insert(
        "ts".to_string(),
        json!(Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)),
    );

    let value = Value::Object(event);
    append_jsonl(log, &value)?;
    Ok(value)
}

fn cycle_id_from_log(log: &Path) -> String {
    log.parent()
        .and_then(Path::file_name)
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_default()
}

fn append_jsonl(log: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = log.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        .with_context(|| format!("failed to open {}", log.display()))?;
    lock_file(&file)?;
    let write_result = writeln!(file, "{}", serde_json::to_string(value)?);
    let sync_result = file.sync_all();
    let unlock_result = unlock_file(&file);
    write_result.with_context(|| format!("failed to append {}", log.display()))?;
    sync_result.with_context(|| format!("failed to sync {}", log.display()))?;
    unlock_result?;
    Ok(())
}

#[cfg(unix)]
fn lock_file(file: &File) -> Result<()> {
    let result = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_EX) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error()).context("failed to lock event log")
    }
}

#[cfg(unix)]
fn unlock_file(file: &File) -> Result<()> {
    let result = unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_UN) };
    if result == 0 {
        Ok(())
    } else {
        Err(std::io::Error::last_os_error()).context("failed to unlock event log")
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
    use std::sync::Arc;
    use std::thread;

    use super::*;
    use tempfile::TempDir;

    fn log_path(temp: &TempDir) -> std::path::PathBuf {
        temp.path()
            .join("_cycles/01HTESTCYCLE00000000000000/cycle.jsonl")
    }

    fn read_rows(log: &Path) -> Vec<Value> {
        fs::read_to_string(log)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    #[test]
    fn emit_event_appends_jsonl_line() {
        let temp = TempDir::new().unwrap();
        let log = log_path(&temp);
        emit_event(
            &log,
            "cycle.opened",
            "shape",
            "planner",
            "{\"note\":\"hello\"}",
        )
        .unwrap();
        assert_eq!(fs::read_to_string(log).unwrap().lines().count(), 1);
    }

    #[test]
    fn emit_event_writes_required_fields() {
        let temp = TempDir::new().unwrap();
        let log = log_path(&temp);
        emit_event(
            &log,
            "cycle.opened",
            "shape",
            "planner",
            "{\"note\":\"hi\"}",
        )
        .unwrap();
        let row = read_rows(&log).remove(0);
        assert_eq!(row["kind"], "cycle.opened");
        assert_eq!(row["phase"], "shape");
        assert_eq!(row["agent"], "planner");
        assert_eq!(row["schema_version"], 1);
    }

    #[test]
    fn emit_event_sets_timestamp_and_cycle_id() {
        let temp = TempDir::new().unwrap();
        let log = log_path(&temp);
        emit_event(&log, "cycle.opened", "shape", "planner", "{}").unwrap();
        let row = read_rows(&log).remove(0);
        let ts = row["ts"].as_str().unwrap();
        assert!(ts.contains('T') && ts.ends_with('Z'));
        assert_eq!(row["cycle_id"], "01HTESTCYCLE00000000000000");
    }

    #[test]
    fn emit_event_rejects_unknown_kind_empty_phase_and_bad_payload() {
        let temp = TempDir::new().unwrap();
        let log = log_path(&temp);
        assert!(emit_event(&log, "bogus.kind", "shape", "planner", "{}").is_err());
        assert!(emit_event(&log, "cycle.opened", "", "planner", "{}").is_err());
        assert!(emit_event(&log, "cycle.opened", "shape", "planner", "not json").is_err());
    }

    #[test]
    fn emit_event_appends_multiple_lines() {
        let temp = TempDir::new().unwrap();
        let log = log_path(&temp);
        emit_event(&log, "cycle.opened", "pick", "orchestrator", "{}").unwrap();
        emit_event(&log, "deliver.done", "deliver", "builder", "{}").unwrap();
        emit_event(&log, "deploy.done", "deploy", "deployer", "{}").unwrap();
        emit_event(&log, "cycle.closed", "close", "orchestrator", "{}").unwrap();
        assert_eq!(read_rows(&log).len(), 4);
    }

    #[test]
    fn payload_cannot_override_core_envelope() {
        let temp = TempDir::new().unwrap();
        let log = log_path(&temp);
        emit_event(
            &log,
            "cycle.opened",
            "shape",
            "planner",
            "{\"kind\":\"attacker\",\"cycle_id\":\"fake\",\"ts\":\"1999-01-01T00:00:00Z\",\"agent\":\"evil\",\"schema_version\":999,\"phase\":\"hijacked\"}",
        )
        .unwrap();
        let row = read_rows(&log).remove(0);
        assert_eq!(row["kind"], "cycle.opened");
        assert_eq!(row["cycle_id"], "01HTESTCYCLE00000000000000");
        assert_eq!(row["agent"], "planner");
        assert_eq!(row["schema_version"], 1);
        assert_eq!(row["phase"], "shape");
        assert!(!row["ts"].as_str().unwrap().starts_with("1999-"));
    }

    #[test]
    fn payload_fields_are_merged() {
        let temp = TempDir::new().unwrap();
        let log = log_path(&temp);
        emit_event(
            &log,
            "deliver.done",
            "deliver",
            "builder",
            "{\"refs\":[\"a\",\"b\"],\"note\":\"ok\"}",
        )
        .unwrap();
        let row = read_rows(&log).remove(0);
        assert_eq!(row["refs"].as_array().unwrap().len(), 2);
        assert_eq!(row["note"], "ok");
    }

    #[test]
    fn all_known_kinds_are_accepted() {
        let temp = TempDir::new().unwrap();
        let log = log_path(&temp);
        for kind in known_kinds() {
            emit_event(&log, kind, "test", "test", "{}").unwrap();
        }
        assert_eq!(read_rows(&log).len(), 12);
    }

    #[test]
    fn concurrent_writes_do_not_corrupt_jsonl() {
        let temp = TempDir::new().unwrap();
        let log = Arc::new(log_path(&temp));
        let left_log = Arc::clone(&log);
        let left = thread::spawn(move || {
            for index in 1..=5 {
                emit_event(
                    &left_log,
                    "cycle.opened",
                    "pick",
                    "orchestrator",
                    &format!("{{\"note\":\"a{index}\"}}"),
                )
                .unwrap();
            }
        });
        let right_log = Arc::clone(&log);
        let right = thread::spawn(move || {
            for index in 1..=5 {
                emit_event(
                    &right_log,
                    "deliver.done",
                    "deliver",
                    "builder",
                    &format!("{{\"note\":\"b{index}\"}}"),
                )
                .unwrap();
            }
        });
        left.join().unwrap();
        right.join().unwrap();
        assert_eq!(read_rows(&log).len(), 10);
    }
}
