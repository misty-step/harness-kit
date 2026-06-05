use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};

const VALID_COST_SOURCES: &[&str] = &["provider_reported", "estimated", "manual", "unknown"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Text,
    Markdown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnalyzeOptions {
    pub skill_log: PathBuf,
    pub work_ledger: PathBuf,
    pub delegations: PathBuf,
    pub since: String,
    pub repo: String,
    pub project: String,
    pub skill: String,
}

pub fn default_skill_log() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/skill-invocations.jsonl")
}

pub fn default_work_ledger() -> PathBuf {
    PathBuf::from(".harness-kit/work/ledger.jsonl")
}

pub fn default_delegations() -> PathBuf {
    PathBuf::from(".harness-kit/traces/delegations.jsonl")
}

pub fn analyze(options: &AnalyzeOptions) -> Result<Value> {
    let since = parse_since(&options.since)?;
    let (mut skill_rows, skill_coverage, mut warnings) =
        read_jsonl(&options.skill_log, "skill invocation")?;
    let (mut work_rows, work_coverage, work_warnings) =
        read_jsonl(&options.work_ledger, "work ledger")?;
    let (mut delegation_rows, delegation_coverage, delegation_warnings) =
        read_jsonl(&options.delegations, "delegation")?;
    warnings.extend(work_warnings);
    warnings.extend(delegation_warnings);

    skill_rows.retain(|row| passes_filters(row, options, since.as_ref()));
    work_rows.retain(|row| passes_filters(row, options, since.as_ref()));
    delegation_rows.retain(|row| passes_filters(row, options, since.as_ref()));

    let mut by_skill: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    let mut sessions: BTreeMap<String, Vec<Value>> = BTreeMap::new();
    for row in &skill_rows {
        let skill = value_str(row, "skill").unwrap_or("unknown").to_string();
        by_skill.entry(skill).or_default().push(row.clone());
        let session_id = value_str(row, "session_id")
            .unwrap_or("unknown")
            .to_string();
        sessions.entry(session_id).or_default().push(row.clone());
    }

    let mut skills = Vec::new();
    for (skill, rows) in by_skill {
        let mut timestamps: Vec<_> = rows
            .iter()
            .filter_map(|row| parse_ts(row.get("ts")))
            .collect();
        timestamps.sort();
        let projects: BTreeSet<_> = rows.iter().map(repo_id).collect();
        skills.push(json!({
            "skill": skill,
            "count": rows.len(),
            "health": classify(rows.len()),
            "last_used": timestamps.last().map(DateTime::<Utc>::to_rfc3339).unwrap_or_else(|| "unknown".to_string()),
            "projects": projects.into_iter().collect::<Vec<_>>(),
            "usage": usage_summary(&rows)?,
        }));
    }
    skills.sort_by(|left, right| {
        let left_count = left["count"].as_u64().unwrap_or(0);
        let right_count = right["count"].as_u64().unwrap_or(0);
        right_count
            .cmp(&left_count)
            .then_with(|| left["skill"].as_str().cmp(&right["skill"].as_str()))
    });

    let mut transition_counts: BTreeMap<(String, String), usize> = BTreeMap::new();
    for rows in sessions.values_mut() {
        rows.sort_by_key(|row| value_str(row, "ts").unwrap_or("").to_string());
        let names: Vec<_> = rows
            .iter()
            .map(|row| value_str(row, "skill").unwrap_or("unknown").to_string())
            .collect();
        for window in names.windows(2) {
            *transition_counts
                .entry((window[0].clone(), window[1].clone()))
                .or_default() += 1;
        }
    }
    let mut transitions: Vec<_> = transition_counts
        .into_iter()
        .map(|((before, after), count)| json!({"from": before, "to": after, "count": count}))
        .collect();
    transitions.sort_by(|left, right| {
        let left_count = left["count"].as_u64().unwrap_or(0);
        let right_count = right["count"].as_u64().unwrap_or(0);
        right_count
            .cmp(&left_count)
            .then_with(|| left["from"].as_str().cmp(&right["from"].as_str()))
            .then_with(|| left["to"].as_str().cmp(&right["to"].as_str()))
    });

    let mut sequences: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in &skill_rows {
        let reference = value_str(row, "backlog_ref")
            .or_else(|| value_str(row, "work_id"))
            .unwrap_or("");
        if !reference.is_empty() {
            sequences
                .entry(reference.to_string())
                .or_default()
                .push(value_str(row, "skill").unwrap_or("unknown").to_string());
        }
    }
    for row in &work_rows {
        let reference = value_str(row, "backlog_ref")
            .or_else(|| value_str(row, "work_id"))
            .unwrap_or("");
        let skill = value_str(row, "owning_skill").unwrap_or("");
        if !reference.is_empty() && !skill.is_empty() {
            sequences
                .entry(reference.to_string())
                .or_default()
                .push(skill.to_string());
        }
    }

    let unmatched_skill_rows = skill_rows
        .iter()
        .filter(|row| {
            value_str(row, "backlog_ref").is_none() && value_str(row, "work_id").is_none()
        })
        .count();
    if unmatched_skill_rows > 0 {
        warnings.push(format!(
            "{unmatched_skill_rows} skill invocation row(s) lack backlog_ref/work_id"
        ));
    }
    if !delegation_rows.is_empty() && work_rows.is_empty() {
        warnings.push("delegation rows are present but work ledger rows are absent".to_string());
    }

    let harness_coverage = harness_coverage(&skill_rows);
    for row in &harness_coverage {
        if row["adapter"] == "unsupported" {
            warnings.push(format!(
                "{} skill telemetry adapter unsupported: {}",
                row["harness"].as_str().unwrap_or("unknown"),
                row["evidence"].as_str().unwrap_or("")
            ));
        }
        if row["rows"].as_u64().unwrap_or(0) > 0
            && row["observed_source_protocols"]
                .as_array()
                .is_some_and(|values| values.iter().any(|value| value == "missing"))
        {
            warnings.push(format!(
                "{} row(s) missing source_protocol",
                row["harness"].as_str().unwrap_or("unknown")
            ));
        }
    }

    Ok(json!({
        "skills": skills,
        "transitions": transitions,
        "work_sequences": sequences.into_iter().map(|(reference, skills)| json!({"ref": reference, "skills": skills})).collect::<Vec<_>>(),
        "delegation_usage": usage_summary(&delegation_rows)?,
        "coverage": {
            "skill_log": skill_coverage,
            "work_ledger": work_coverage,
            "delegations": delegation_coverage,
        },
        "harness_coverage": harness_coverage,
        "warnings": warnings,
    }))
}

pub fn render(report: &Value, format: &OutputFormat) -> Result<String> {
    match format {
        OutputFormat::Json => Ok(serde_json::to_string_pretty(report)?),
        OutputFormat::Text => render_text(report),
        OutputFormat::Markdown => render_markdown(report),
    }
}

pub fn self_test() -> Result<()> {
    let temp = tempfile::tempdir().context("failed to create temporary directory")?;
    let root = temp.path();
    let skill_log = root.join("skill-invocations.jsonl");
    let work_ledger = root.join("work-ledger.jsonl");
    let delegations = root.join("delegations.jsonl");
    fs::write(
        &skill_log,
        [
            json!({
                "schema_version": 2,
                "event_type": "skill_invocation",
                "ts": "2026-06-04T00:00:00Z",
                "harness": "claude",
                "source_protocol": "post_tool_use",
                "skill": "shape",
                "args": "088",
                "session_id": "s1",
                "cwd": "/tmp/harness-kit",
                "project": "harness-kit",
                "backlog_ref": "088",
                "work_id": "work-088",
                "usage": {
                    "input_tokens": 10,
                    "output_tokens": 5,
                    "total_tokens": 15,
                    "cost_usd": 0.001,
                    "cost_source": "provider_reported",
                },
            }),
            json!({
                "schema_version": 2,
                "event_type": "skill_invocation",
                "ts": "2026-06-04T00:01:00Z",
                "harness": "claude",
                "source_protocol": "post_tool_use",
                "skill": "implement",
                "args": "088",
                "session_id": "s1",
                "cwd": "/tmp/harness-kit",
                "project": "harness-kit",
                "backlog_ref": "088",
            }),
            json!({
                "schema_version": 2,
                "event_type": "skill_invocation",
                "ts": "2026-06-04T00:02:00Z",
                "harness": "codex",
                "source_protocol": "manual_fixture",
                "skill": "shape",
                "args": "090",
                "session_id": "s2",
                "cwd": "/tmp/harness-kit",
                "project": "harness-kit",
            }),
        ]
        .into_iter()
        .map(|row| serde_json::to_string(&row))
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n")
            + "\n",
    )?;
    fs::write(
        &work_ledger,
        serde_json::to_string(&json!({
            "created_at": "2026-06-04T00:00:00Z",
            "owning_skill": "deliver",
            "backlog_ref": "088",
            "work_id": "work-088",
            "usage": {
                "input_tokens": null,
                "output_tokens": null,
                "total_tokens": null,
                "cost_usd": null,
                "cost_source": "unknown",
            },
        }))? + "\n",
    )?;
    fs::write(
        &delegations,
        serde_json::to_string(&json!({
            "created_at": "2026-06-04T00:00:00Z",
            "provider_target": "codex",
            "backlog_ref": "088",
        }))? + "\n",
    )?;
    let report = analyze(&AnalyzeOptions {
        skill_log,
        work_ledger,
        delegations,
        since: String::new(),
        repo: String::new(),
        project: String::new(),
        skill: String::new(),
    })?;
    ensure(
        report["skills"][0]["skill"] == "shape",
        "shape should be top skill",
    )?;
    ensure(
        report["transitions"].as_array().is_some_and(|rows| {
            rows.iter()
                .any(|row| row == &json!({"from": "shape", "to": "implement", "count": 1}))
        }),
        "shape -> implement transition missing",
    )?;
    ensure(
        report["delegation_usage"]["total_tokens"].is_null(),
        "delegation total_tokens should be null",
    )?;
    ensure(
        report["harness_coverage"].as_array().is_some_and(|rows| {
            rows.iter().any(|row| {
                row["harness"] == "claude"
                    && row["adapter"] == "implemented"
                    && row["observed_source_protocols"] == json!(["post_tool_use"])
            })
        }),
        "claude harness coverage missing",
    )?;
    ensure(
        report["harness_coverage"].as_array().is_some_and(|rows| {
            rows.iter()
                .any(|row| row["harness"] == "pi" && row["adapter"] == "unsupported")
        }),
        "pi unsupported coverage missing",
    )?;
    ensure(
        render_markdown(&report)?.contains("unknown"),
        "markdown should include unknown",
    )?;

    let missing_report = analyze(&AnalyzeOptions {
        skill_log: root.join("missing-skill.jsonl"),
        work_ledger: root.join("missing-work.jsonl"),
        delegations: root.join("missing-delegations.jsonl"),
        since: String::new(),
        repo: String::new(),
        project: String::new(),
        skill: String::new(),
    })?;
    ensure(
        missing_report["coverage"]["skill_log"]["present"] == false,
        "missing skill log coverage should be false",
    )?;
    ensure(
        missing_report["warnings"]
            .as_array()
            .is_some_and(|warnings| !warnings.is_empty()),
        "missing report should include warnings",
    )?;
    Ok(())
}

fn read_jsonl(path: &Path, label: &str) -> Result<(Vec<Value>, Value, Vec<String>)> {
    let coverage = json!({"path": path.display().to_string(), "present": path.exists(), "rows": 0});
    let mut warnings = Vec::new();
    if !path.exists() {
        warnings.push(format!("{label} store missing: {}", path.display()));
        return Ok((Vec::new(), coverage, warnings));
    }
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let row: Value = serde_json::from_str(line)
            .with_context(|| format!("{}:{}: invalid JSON", path.display(), index + 1))?;
        if !row.is_object() {
            bail!(
                "{}:{}: row must be a JSON object",
                path.display(),
                index + 1
            );
        }
        rows.push(row);
    }
    let row_count = rows.len();
    Ok((
        rows,
        json!({"path": path.display().to_string(), "present": true, "rows": row_count}),
        warnings,
    ))
}

fn parse_ts(value: Option<&Value>) -> Option<DateTime<Utc>> {
    let text = value?.as_str()?;
    DateTime::parse_from_rfc3339(&text.replace('Z', "+00:00"))
        .ok()
        .map(|value| value.with_timezone(&Utc))
}

fn parse_since(value: &str) -> Result<Option<DateTime<Utc>>> {
    if value.is_empty() {
        return Ok(None);
    }
    let (amount, unit) = value.split_at(value.len().saturating_sub(1));
    if !matches!(unit, "d" | "h")
        || amount.is_empty()
        || !amount.chars().all(|c| c.is_ascii_digit())
    {
        bail!("--since must look like 7d, 30d, or 12h");
    }
    let amount: i64 = amount.parse()?;
    let delta = if unit == "d" {
        Duration::days(amount)
    } else {
        Duration::hours(amount)
    };
    Ok(Some(Utc::now() - delta))
}

fn passes_filters(row: &Value, options: &AnalyzeOptions, since: Option<&DateTime<Utc>>) -> bool {
    if let Some(since) = since {
        let Some(ts) = parse_ts(row.get("ts").or_else(|| row.get("created_at"))) else {
            return false;
        };
        if &ts < since {
            return false;
        }
    }
    if !options.project.is_empty() && value_str(row, "project") != Some(options.project.as_str()) {
        return false;
    }
    if !options.repo.is_empty() {
        let cwd_name = value_str(row, "cwd")
            .and_then(|cwd| Path::new(cwd).file_name())
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if options.repo != repo_id(row) && options.repo != cwd_name {
            return false;
        }
    }
    if !options.skill.is_empty()
        && value_str(row, "skill").or_else(|| value_str(row, "owning_skill"))
            != Some(options.skill.as_str())
    {
        return false;
    }
    true
}

fn usage_summary(rows: &[Value]) -> Result<Value> {
    let mut known = 0usize;
    let mut unknown = 0usize;
    let mut total_tokens = 0i64;
    let mut cost_usd = 0.0f64;
    let mut cost_sources: BTreeMap<String, usize> = BTreeMap::new();
    for row in rows {
        let Some(usage) = row.get("usage") else {
            unknown += 1;
            continue;
        };
        if !usage.is_object() {
            unknown += 1;
            continue;
        }
        validate_usage(usage).map_err(|error| anyhow::anyhow!("invalid usage payload: {error}"))?;
        known += 1;
        if let Some(value) = usage.get("total_tokens").and_then(Value::as_i64) {
            total_tokens += value;
        }
        if let Some(value) = usage.get("cost_usd").and_then(Value::as_f64) {
            cost_usd += value;
        }
        if let Some(source) = usage.get("cost_source").and_then(Value::as_str) {
            *cost_sources.entry(source.to_string()).or_default() += 1;
        }
    }
    Ok(json!({
        "known_count": known,
        "unknown_count": unknown,
        "total_tokens": if known > 0 { json!(total_tokens) } else { Value::Null },
        "cost_usd": if known > 0 { json!((cost_usd * 1_000_000.0).round() / 1_000_000.0) } else { Value::Null },
        "cost_sources": cost_sources,
    }))
}

fn validate_usage(usage: &Value) -> std::result::Result<(), String> {
    if usage.is_null() {
        return Ok(());
    }
    let Some(object) = usage.as_object() else {
        return Err("usage must be an object or null.".to_string());
    };
    let valid: BTreeSet<_> = [
        "input_tokens",
        "output_tokens",
        "total_tokens",
        "cost_usd",
        "cost_source",
    ]
    .into_iter()
    .collect();
    let extra: Vec<_> = object
        .keys()
        .filter(|key| !valid.contains(key.as_str()))
        .cloned()
        .collect();
    if !extra.is_empty() {
        return Err(format!("usage has unknown fields: {}", extra.join(", ")));
    }
    for field in ["input_tokens", "output_tokens", "total_tokens"] {
        if let Some(value) = object.get(field)
            && !value.is_null()
            && (!value.is_i64() || value.as_i64().unwrap_or(-1) < 0)
        {
            return Err(format!(
                "usage {field} must be a non-negative integer or null."
            ));
        }
    }
    if let Some(value) = object.get("cost_usd")
        && !value.is_null()
        && (!value.is_number() || value.as_f64().unwrap_or(-1.0) < 0.0)
    {
        return Err("usage cost_usd must be a non-negative number or null.".to_string());
    }
    if let Some(value) = object.get("cost_source")
        && !value.is_null()
        && !value
            .as_str()
            .is_some_and(|source| VALID_COST_SOURCES.contains(&source))
    {
        return Err("usage cost_source is invalid.".to_string());
    }
    if object.get("cost_usd").is_some_and(|value| !value.is_null())
        && !object.contains_key("cost_source")
    {
        return Err("usage cost_source is required when cost_usd is known.".to_string());
    }
    Ok(())
}

fn harness_coverage(rows: &[Value]) -> Vec<Value> {
    let expected = expected_harnesses();
    let mut by_harness: BTreeMap<String, Vec<&Value>> = BTreeMap::new();
    for row in rows {
        by_harness
            .entry(value_str(row, "harness").unwrap_or("unknown").to_string())
            .or_default()
            .push(row);
    }
    let names: BTreeSet<_> = expected
        .keys()
        .cloned()
        .chain(by_harness.keys().cloned())
        .collect();
    names
        .into_iter()
        .map(|name| {
            let support = expected.get(&name).cloned().unwrap_or(HarnessSupport {
                adapter: "unknown",
                source_protocol: "unknown",
                evidence: "observed row only",
            });
            let rows_for_harness = by_harness.get(&name).cloned().unwrap_or_default();
            let protocols: BTreeSet<_> = rows_for_harness
                .iter()
                .map(|row| {
                    value_str(row, "source_protocol")
                        .unwrap_or("missing")
                        .to_string()
                })
                .collect();
            json!({
                "harness": name,
                "adapter": support.adapter,
                "expected_source_protocol": support.source_protocol,
                "rows": rows_for_harness.len(),
                "observed_source_protocols": protocols.into_iter().collect::<Vec<_>>(),
                "evidence": support.evidence,
            })
        })
        .collect()
}

#[derive(Clone)]
struct HarnessSupport {
    adapter: &'static str,
    source_protocol: &'static str,
    evidence: &'static str,
}

fn expected_harnesses() -> BTreeMap<String, HarnessSupport> {
    [
        (
            "claude",
            HarnessSupport {
                adapter: "implemented",
                source_protocol: "post_tool_use",
                evidence: "harness-kit-checks claude-hook skill-invocation-tracker",
            },
        ),
        (
            "codex",
            HarnessSupport {
                adapter: "unsupported",
                source_protocol: "unavailable",
                evidence: "harnesses/codex/README.md",
            },
        ),
        (
            "pi",
            HarnessSupport {
                adapter: "unsupported",
                source_protocol: "unavailable",
                evidence: "harnesses/pi/README.md",
            },
        ),
        (
            "antigravity-cli",
            HarnessSupport {
                adapter: "unsupported",
                source_protocol: "unavailable",
                evidence: "harnesses/antigravity-cli/README.md",
            },
        ),
    ]
    .into_iter()
    .map(|(name, support)| (name.to_string(), support))
    .collect()
}

fn render_markdown(report: &Value) -> Result<String> {
    let mut lines = vec![
        "# Skill Invocation Analytics".to_string(),
        String::new(),
        "## Skill Frequency".to_string(),
        String::new(),
    ];
    lines.push("| Skill | Count | Health | Last Used | Projects | Tokens | Cost |".to_string());
    lines.push("|---|---:|---|---|---|---:|---:|".to_string());
    let skills = array(report, "skills");
    for row in skills {
        let usage = &row["usage"];
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            string_value(&row["skill"]),
            number_or_zero(&row["count"]),
            string_value(&row["health"]),
            string_value(&row["last_used"]),
            row["projects"]
                .as_array()
                .map(|values| values
                    .iter()
                    .map(string_value)
                    .collect::<Vec<_>>()
                    .join(", "))
                .unwrap_or_default(),
            unknown(&usage["total_tokens"]),
            unknown(&usage["cost_usd"]),
        ));
    }
    if skills.is_empty() {
        lines.push("| none | 0 | dead | unknown | unknown | unknown | unknown |".to_string());
    }
    lines.extend(
        [
            "",
            "## Skill Transitions",
            "",
            "| From | To | Count |",
            "|---|---|---:|",
        ]
        .into_iter()
        .map(String::from),
    );
    let transitions = array(report, "transitions");
    for row in transitions {
        lines.push(format!(
            "| {} | {} | {} |",
            string_value(&row["from"]),
            string_value(&row["to"]),
            number_or_zero(&row["count"])
        ));
    }
    if transitions.is_empty() {
        lines.push("| none | none | 0 |".to_string());
    }
    lines.extend(
        ["", "## Work Sequences", "", "| Ref | Skills |", "|---|---|"]
            .into_iter()
            .map(String::from),
    );
    let work_sequences = array(report, "work_sequences");
    for row in work_sequences {
        let skills = row["skills"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .map(string_value)
                    .collect::<Vec<_>>()
                    .join(" -> ")
            })
            .unwrap_or_default();
        lines.push(format!("| {} | {} |", string_value(&row["ref"]), skills));
    }
    if work_sequences.is_empty() {
        lines.push("| none | none |".to_string());
    }
    lines.extend(
        [
            "",
            "## Source Coverage",
            "",
            "| Store | Present | Rows | Path |",
            "|---|---|---:|---|",
        ]
        .into_iter()
        .map(String::from),
    );
    for name in ["skill_log", "work_ledger", "delegations"] {
        let coverage = &report["coverage"][name];
        lines.push(format!(
            "| {name} | {} | {} | {} |",
            coverage["present"].as_bool().unwrap_or(false),
            number_or_zero(&coverage["rows"]),
            string_value(&coverage["path"])
        ));
    }
    lines.extend(
        [
            "",
            "## Harness Coverage",
            "",
            "| Harness | Adapter | Expected Protocol | Rows | Observed Protocols | Evidence |",
            "|---|---|---|---:|---|---|",
        ]
        .into_iter()
        .map(String::from),
    );
    for row in array(report, "harness_coverage") {
        let observed = row["observed_source_protocols"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .map(string_value)
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "none".to_string());
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} |",
            string_value(&row["harness"]),
            string_value(&row["adapter"]),
            string_value(&row["expected_source_protocol"]),
            number_or_zero(&row["rows"]),
            observed,
            string_value(&row["evidence"])
        ));
    }
    let usage = &report["delegation_usage"];
    lines.extend(
        [
            "",
            "## Delegation Usage",
            "",
            &format!("- known: {}", number_or_zero(&usage["known_count"])),
            &format!("- unknown: {}", number_or_zero(&usage["unknown_count"])),
            &format!("- total_tokens: {}", unknown(&usage["total_tokens"])),
            &format!("- cost_usd: {}", unknown(&usage["cost_usd"])),
            "",
            "## Warnings",
        ]
        .into_iter()
        .map(String::from),
    );
    let warnings = report["warnings"].as_array().cloned().unwrap_or_default();
    if warnings.is_empty() {
        lines.push("- none".to_string());
    } else {
        lines.extend(
            warnings
                .iter()
                .map(|warning| format!("- {}", string_value(warning))),
        );
    }
    Ok(lines.join("\n"))
}

fn render_text(report: &Value) -> Result<String> {
    let mut lines = vec!["Skill invocation analytics".to_string()];
    for row in array(report, "skills") {
        let usage = &row["usage"];
        let projects = row["projects"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .map(string_value)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .unwrap_or_default();
        lines.push(format!(
            "- {}: count={} health={} projects={} total_tokens={} cost_usd={}",
            string_value(&row["skill"]),
            number_or_zero(&row["count"]),
            string_value(&row["health"]),
            projects,
            unknown(&usage["total_tokens"]),
            unknown(&usage["cost_usd"])
        ));
    }
    lines.push("transitions:".to_string());
    lines.extend(array(report, "transitions").iter().map(|row| {
        format!(
            "- {} -> {}: {}",
            string_value(&row["from"]),
            string_value(&row["to"]),
            number_or_zero(&row["count"])
        )
    }));
    lines.push("coverage:".to_string());
    for name in ["skill_log", "work_ledger", "delegations"] {
        let coverage = &report["coverage"][name];
        lines.push(format!(
            "- {name}: present={} rows={} path={}",
            coverage["present"].as_bool().unwrap_or(false),
            number_or_zero(&coverage["rows"]),
            string_value(&coverage["path"])
        ));
    }
    lines.push("harness coverage:".to_string());
    for row in array(report, "harness_coverage") {
        let observed = row["observed_source_protocols"]
            .as_array()
            .map(|values| {
                values
                    .iter()
                    .map(string_value)
                    .collect::<Vec<_>>()
                    .join(",")
            })
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| "none".to_string());
        lines.push(format!(
            "- {}: adapter={} expected_protocol={} rows={} observed={}",
            string_value(&row["harness"]),
            string_value(&row["adapter"]),
            string_value(&row["expected_source_protocol"]),
            number_or_zero(&row["rows"]),
            observed
        ));
    }
    lines.push("warnings:".to_string());
    let warnings = report["warnings"].as_array().cloned().unwrap_or_default();
    if warnings.is_empty() {
        lines.push("- none".to_string());
    } else {
        lines.extend(
            warnings
                .iter()
                .map(|warning| format!("- {}", string_value(warning))),
        );
    }
    Ok(lines.join("\n"))
}

fn repo_id(row: &Value) -> String {
    if let Some(project) = value_str(row, "project").filter(|value| !value.trim().is_empty()) {
        return project.to_string();
    }
    value_str(row, "cwd")
        .and_then(|cwd| Path::new(cwd).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn classify(count: usize) -> &'static str {
    if count > 10 {
        "hot"
    } else if count >= 3 {
        "warm"
    } else if count >= 1 {
        "cold"
    } else {
        "dead"
    }
}

fn value_str<'a>(row: &'a Value, key: &str) -> Option<&'a str> {
    row.get(key).and_then(Value::as_str)
}

fn string_value(value: &Value) -> String {
    if value.is_null() {
        "unknown".to_string()
    } else if let Some(text) = value.as_str() {
        text.to_string()
    } else {
        value.to_string()
    }
}

fn unknown(value: &Value) -> String {
    if value.is_null() {
        "unknown".to_string()
    } else {
        string_value(value)
    }
}

fn number_or_zero(value: &Value) -> String {
    value
        .as_u64()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "0".to_string())
}

fn array<'a>(report: &'a Value, key: &str) -> &'a Vec<Value> {
    static EMPTY: std::sync::OnceLock<Vec<Value>> = std::sync::OnceLock::new();
    report
        .get(key)
        .and_then(Value::as_array)
        .unwrap_or_else(|| EMPTY.get_or_init(Vec::new))
}

fn ensure(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        bail!("{message}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_contract_passes() {
        self_test().unwrap();
    }

    #[test]
    fn invalid_usage_requires_cost_source_when_cost_known() {
        let error = validate_usage(&json!({"cost_usd": 0.1})).unwrap_err();

        assert_eq!(
            error,
            "usage cost_source is required when cost_usd is known."
        );
    }

    #[test]
    fn since_parser_rejects_bad_duration() {
        let error = parse_since("30x").unwrap_err().to_string();

        assert_eq!(error, "--since must look like 7d, 30d, or 12h");
    }

    #[test]
    fn usage_summary_counts_null_usage_as_unknown() {
        let summary = usage_summary(&[json!({"usage": null}), json!({})]).unwrap();

        assert_eq!(summary["known_count"], json!(0));
        assert_eq!(summary["unknown_count"], json!(2));
        assert!(summary["total_tokens"].is_null());
    }
}
