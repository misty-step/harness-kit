use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use regex::Regex;
use serde_json::{Value, json};

use crate::agent_transcript;

#[derive(Debug, Clone)]
pub struct Options {
    pub transcripts: Vec<PathBuf>,
    pub source_roots: Vec<PathBuf>,
    pub skill_log: PathBuf,
    pub work_ledger: PathBuf,
    pub delegations: PathBuf,
    pub review_scores: PathBuf,
    pub allow_redacted_excerpts: bool,
}

pub fn build_report(options: &Options) -> Result<Value> {
    let paths = transcript_paths(&options.transcripts, &options.source_roots)?;
    if paths.is_empty() {
        bail!("provide at least one --transcript path or explicit --source-root");
    }
    let transcripts = paths
        .iter()
        .map(|path| parse_transcript(path))
        .collect::<Result<Vec<_>>>()?;
    let turns = transcripts
        .iter()
        .flat_map(|transcript| transcript.turns.clone())
        .collect::<Vec<_>>();
    let categories = categorize(&turns, options.allow_redacted_excerpts);

    let (skill_rows, skill_coverage, skill_warnings) =
        read_jsonl(&options.skill_log, "skill invocation")?;
    let (work_rows, work_coverage, work_warnings) =
        read_jsonl(&options.work_ledger, "work ledger")?;
    let (delegation_rows, delegation_coverage, delegation_warnings) =
        read_jsonl(&options.delegations, "delegation")?;
    let (review_rows, review_coverage, review_warnings) =
        read_jsonl(&options.review_scores, "review scores")?;
    let joins = join_evidence(
        &transcripts,
        &skill_rows,
        &work_rows,
        &delegation_rows,
        &review_rows,
    );
    let redacted_segments: usize = transcripts
        .iter()
        .map(|transcript| transcript.redactions)
        .sum();
    let warnings = skill_warnings
        .into_iter()
        .chain(work_warnings)
        .chain(delegation_warnings)
        .chain(review_warnings)
        .collect::<Vec<_>>();

    Ok(json!({
        "schema_version": 1,
        "report_type": "transcript_effectiveness_mining",
        "transcripts": transcripts.iter().map(Transcript::summary).collect::<Vec<_>>(),
        "categories": categories,
        "joins": joins,
        "source_coverage": {
            "claude_transcripts": {"present": true, "rows": paths.len(), "path": paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(",")},
            "codex_sessions": {"present": false, "rows": 0, "path": "unsupported explicit export in this slice"},
            "skill_invocations": skill_coverage,
            "work_ledger": work_coverage,
            "delegations": delegation_coverage,
            "review_scores": review_coverage,
        },
        "redaction_summary": {
            "redacted_segments": redacted_segments,
            "excerpts_included": options.allow_redacted_excerpts,
        },
        "warnings": warnings,
        "proposed_actions": proposed_actions(&categories, &joins),
    }))
}

pub fn render_markdown(report: &Value) -> String {
    let mut lines = vec![
        "# Transcript Effectiveness Mining".to_string(),
        String::new(),
        "## Category Counts".to_string(),
        String::new(),
    ];
    lines.extend(["| Category | Count | Evidence Refs |", "|---|---:|---|"].map(str::to_string));
    if let Some(categories) = report.get("categories").and_then(Value::as_object) {
        for (name, data) in categories {
            let refs = data
                .get("refs")
                .and_then(Value::as_array)
                .map(|refs| {
                    refs.iter()
                        .filter_map(|reference| {
                            Some(format!(
                                "{}:{}",
                                reference.get("role")?.as_str()?,
                                reference.get("lineno")?.as_str()?
                            ))
                        })
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .filter(|text| !text.is_empty())
                .unwrap_or_else(|| "none".to_string());
            lines.push(format!("| {name} | {} | {refs} |", data["count"]));
        }
    }
    lines.extend([
        String::new(),
        "## Source Coverage".to_string(),
        String::new(),
        "| Source | Present | Rows | Path |".to_string(),
        "|---|---|---:|---|".to_string(),
    ]);
    if let Some(coverage) = report.get("source_coverage").and_then(Value::as_object) {
        for (name, data) in coverage {
            lines.push(format!(
                "| {name} | {} | {} | {} |",
                data["present"],
                data["rows"],
                data["path"].as_str().unwrap_or("")
            ));
        }
    }
    lines.extend([
        String::new(),
        "## Joins".to_string(),
        String::new(),
        "| Store | Matched | Missing | Refs |".to_string(),
        "|---|---:|---:|---|".to_string(),
    ]);
    if let Some(joins) = report.get("joins").and_then(Value::as_object) {
        for (name, data) in joins {
            let refs = data["refs"]
                .as_array()
                .map(|values| join_strings(values))
                .unwrap_or_else(|| "none".to_string());
            lines.push(format!(
                "| {name} | {} | {} | {refs} |",
                data["matched"], data["missing"]
            ));
        }
    }
    let redaction = &report["redaction_summary"];
    lines.extend([
        String::new(),
        "## Redaction Summary".to_string(),
        String::new(),
        format!("- redacted_segments: {}", redaction["redacted_segments"]),
        format!("- excerpts_included: {}", redaction["excerpts_included"]),
        String::new(),
        "## Proposed Actions".to_string(),
    ]);
    if let Some(actions) = report["proposed_actions"].as_array() {
        lines.extend(
            actions
                .iter()
                .filter_map(Value::as_str)
                .map(|action| format!("- {action}")),
        );
    }
    lines.extend([String::new(), "## Warnings".to_string()]);
    let warnings = report["warnings"].as_array().cloned().unwrap_or_default();
    if warnings.is_empty() {
        lines.push("- none".to_string());
    } else {
        lines.extend(
            warnings
                .iter()
                .filter_map(Value::as_str)
                .map(|warning| format!("- {warning}")),
        );
    }
    lines.join("\n")
}

pub fn self_test() -> Result<&'static str> {
    let temp = tempfile::TempDir::new()?;
    let root = temp.path();
    let transcript = root.join("session.jsonl");
    fs::write(
        &transcript,
        [
            json!({"type":"user","sessionId":"sess-1","gitBranch":"feat/test","cwd":"/tmp/harness-kit","message":{"role":"user","content":"This is wrong, use /reflect instead."}}).to_string(),
            json!({"type":"assistant","sessionId":"sess-1","backlog_ref":"091","work_id":"work-091","cwd":"/tmp/harness-kit","message":{"role":"assistant","content":"Tool failed, then Skill reflect succeeded. Authorization: Bearer sk-test_1234567890abcdef /Users/alice/project"}}).to_string(),
        ].join("\n") + "\n",
    )?;
    let skill_log = write_jsonl(
        root.join("skills.jsonl"),
        &[json!({"session_id":"sess-1","skill":"reflect","project":"harness-kit"})],
    )?;
    let work_ledger = write_jsonl(
        root.join("work.jsonl"),
        &[json!({"work_id":"work-091","owning_skill":"deliver"})],
    )?;
    let delegations = write_jsonl(
        root.join("delegations.jsonl"),
        &[json!({"backlog_ref":"091","delegation_id":"del-1"})],
    )?;
    let review_scores = write_jsonl(
        root.join("review.ndjson"),
        &[json!({"branch":"feat/test","correctness":8})],
    )?;
    let options = Options {
        transcripts: vec![transcript],
        source_roots: vec![],
        skill_log,
        work_ledger,
        delegations,
        review_scores,
        allow_redacted_excerpts: false,
    };
    let report = build_report(&options)?;
    let rendered = render_markdown(&report);
    assert_eq!(report["categories"]["user_corrections"]["count"], 1);
    assert_eq!(report["joins"]["skill_invocations"]["matched"], 1);
    assert_eq!(report["joins"]["review_scores"]["matched"], 1);
    assert!(
        report["redaction_summary"]["redacted_segments"]
            .as_u64()
            .unwrap_or(0)
            >= 1
    );
    assert!(!serde_json::to_string(&report)?.contains("sk-test"));
    assert!(!rendered.contains("Authorization: Bearer"));
    assert!(!rendered.contains("wrong, use"));

    let unsafe_transcript = root.join("unsafe.jsonl");
    fs::write(
        &unsafe_transcript,
        json!({"message":{"role":"user","content":"private_customer_data"}}).to_string() + "\n",
    )?;
    let mut unsafe_options = options;
    unsafe_options.transcripts = vec![unsafe_transcript];
    assert!(build_report(&unsafe_options).is_err());
    Ok("transcript effectiveness mining self-test ok")
}

#[derive(Clone, Debug)]
struct Turn {
    role: String,
    text: String,
    lineno: String,
}

#[derive(Clone, Debug)]
struct Transcript {
    path: String,
    turns: Vec<Turn>,
    malformed: usize,
    redactions: usize,
    session_ids: BTreeSet<String>,
    backlog_refs: BTreeSet<String>,
    work_ids: BTreeSet<String>,
    branches: BTreeSet<String>,
    projects: BTreeSet<String>,
}

impl Transcript {
    fn summary(&self) -> Value {
        json!({
            "path": self.path,
            "turn_count": self.turns.len(),
            "malformed_count": self.malformed,
            "session_ids": self.session_ids,
            "backlog_refs": self.backlog_refs,
            "work_ids": self.work_ids,
            "branches": self.branches,
            "projects": self.projects,
        })
    }
}

fn transcript_paths(paths: &[PathBuf], source_roots: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut resolved = Vec::new();
    for path in paths {
        if !path.exists() {
            bail!("transcript path missing: {}", path.display());
        }
        if path.is_dir() {
            bail!(
                "use --source-root for transcript directories: {}",
                path.display()
            );
        }
        resolved.push(path.clone());
    }
    for root in source_roots {
        if !root.is_dir() {
            bail!("source root missing or not a directory: {}", root.display());
        }
        collect_jsonl(root, &mut resolved)?;
    }
    let mut seen = BTreeSet::new();
    resolved.retain(|path| seen.insert(path.canonicalize().unwrap_or_else(|_| path.clone())));
    Ok(resolved)
}

fn collect_jsonl(root: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            if path
                .components()
                .any(|part| part.as_os_str() == "subagents")
            {
                continue;
            }
            collect_jsonl(&path, out)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
            out.push(path);
        }
    }
    Ok(())
}

fn parse_transcript(path: &Path) -> Result<Transcript> {
    let mut transcript = Transcript {
        path: path.display().to_string(),
        turns: Vec::new(),
        malformed: 0,
        redactions: 0,
        session_ids: BTreeSet::new(),
        backlog_refs: BTreeSet::new(),
        work_ids: BTreeSet::new(),
        branches: BTreeSet::new(),
        projects: BTreeSet::new(),
    };
    for (index, raw_line) in fs::read_to_string(path)?.lines().enumerate() {
        if raw_line.trim().is_empty() {
            continue;
        }
        let row: Value = match serde_json::from_str(raw_line) {
            Ok(row) => row,
            Err(_) => {
                transcript.malformed += 1;
                let text = safe_line(raw_line, path)?;
                if !text.is_empty() {
                    transcript.turns.push(Turn {
                        role: "unknown".to_string(),
                        text,
                        lineno: (index + 1).to_string(),
                    });
                }
                continue;
            }
        };
        let Some(object) = row.as_object() else {
            transcript.malformed += 1;
            continue;
        };
        let message = object
            .get("message")
            .filter(|value| value.is_object())
            .unwrap_or(&row);
        let role = message
            .get("role")
            .or_else(|| row.get("type"))
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let raw_text = extract_text(message.get("content"));
        if raw_text.is_empty() {
            continue;
        }
        let text = safe_line(&raw_text, path)?;
        transcript.redactions += text.matches("[REDACTED").count();
        transcript.turns.push(Turn {
            role,
            text,
            lineno: (index + 1).to_string(),
        });
        collect_key(&row, "sessionId", &mut transcript.session_ids);
        collect_key(&row, "session_id", &mut transcript.session_ids);
        collect_key(&row, "backlog_ref", &mut transcript.backlog_refs);
        collect_key(&row, "work_id", &mut transcript.work_ids);
        collect_key(&row, "gitBranch", &mut transcript.branches);
        if let Some(cwd) = row
            .get("cwd")
            .and_then(Value::as_str)
            .filter(|cwd| !cwd.trim().is_empty())
            && let Some(project) = Path::new(cwd).file_name().and_then(|name| name.to_str())
        {
            transcript.projects.insert(project.to_string());
        }
    }
    Ok(transcript)
}

fn extract_text(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| match item {
                Value::String(text) => Some(text.clone()),
                Value::Object(object) => object
                    .get("text")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .or_else(|| {
                        (object.get("type").and_then(Value::as_str) == Some("tool_use")).then(
                            || {
                                format!(
                                    "tool_use:{}",
                                    object
                                        .get("name")
                                        .and_then(Value::as_str)
                                        .unwrap_or("unknown")
                                )
                            },
                        )
                    }),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Some(Value::Object(object)) => object
            .get("text")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        _ => String::new(),
    }
}

fn safe_line(raw: &str, source: &Path) -> Result<String> {
    let redacted = agent_transcript::redact(raw);
    agent_transcript::assert_safe(&redacted)?;
    if Regex::new(r"(?i)(private[_ -]?customer[_ -]?data)")
        .unwrap()
        .is_match(&redacted)
    {
        bail!(
            "{}: unresolved secret-like transcript content refused",
            source.display()
        );
    }
    Ok(redacted)
}

fn collect_key(row: &Value, key: &str, target: &mut BTreeSet<String>) {
    if let Some(value) = row
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        target.insert(value.trim().to_string());
    }
}

fn categorize(turns: &[Turn], allow_excerpts: bool) -> BTreeMap<String, Value> {
    let patterns = [
        (
            "user_corrections",
            r"(?i)\b(wrong|not what i|actually|instead|still|again|revert|stop)\b",
        ),
        (
            "skill_missed_opportunities",
            r"(?i)\b(should have used|forgot to use|missed .*skill|use /[a-z-]+)\b",
        ),
        (
            "repeated_tool_failure",
            r"(?i)\b(error|failed|traceback|exception|command not found|timed out)\b",
        ),
        (
            "cost_token_concern",
            r"(?i)\b(cost|token|budget|too expensive|spend)\b",
        ),
        (
            "insufficient_evidence_claim",
            r"(?i)\b(no evidence|unverified|did not run|without checking|claimed|validated)\b",
        ),
        (
            "privacy_secret_risk",
            r"(?i)\b(secret|credential|private key|token leak|redact|privacy)\b",
        ),
        (
            "successful_skill_usage",
            r"(?i)(<command-name>[^<]+</command-name>|\bSkill\b|/[a-z][a-z-]+)",
        ),
    ];
    patterns
        .into_iter()
        .map(|(name, pattern)| {
            let regex = Regex::new(pattern).unwrap();
            let mut refs = Vec::new();
            let mut excerpts = Vec::new();
            for turn in turns {
                if regex.is_match(&turn.text) {
                    refs.push(json!({"role": turn.role, "lineno": turn.lineno}));
                    if allow_excerpts && excerpts.len() < 3 {
                        excerpts.push(Value::String(turn.text.chars().take(180).collect()));
                    }
                }
            }
            let mut value =
                json!({"count": refs.len(), "refs": refs.into_iter().take(10).collect::<Vec<_>>()});
            if allow_excerpts {
                value["redacted_excerpts"] = Value::Array(excerpts);
            }
            (name.to_string(), value)
        })
        .collect()
}

fn read_jsonl(path: &Path, label: &str) -> Result<(Vec<Value>, Value, Vec<String>)> {
    let mut warnings = Vec::new();
    if !path.exists() {
        warnings.push(format!("{label} store missing: {}", path.display()));
        return Ok((
            Vec::new(),
            json!({"path": path.display().to_string(), "present": false, "rows": 0}),
            warnings,
        ));
    }
    let mut rows = Vec::new();
    for (index, line) in fs::read_to_string(path)?.lines().enumerate() {
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
    Ok((
        rows.clone(),
        json!({"path": path.display().to_string(), "present": true, "rows": rows.len()}),
        warnings,
    ))
}

fn join_evidence(
    transcripts: &[Transcript],
    skill_rows: &[Value],
    work_rows: &[Value],
    delegation_rows: &[Value],
    review_rows: &[Value],
) -> Value {
    let keys = Keys::from_transcripts(transcripts);
    let skill_matches = skill_rows
        .iter()
        .filter(|row| row_matches(row, &keys))
        .collect::<Vec<_>>();
    let work_matches = work_rows
        .iter()
        .filter(|row| row_matches(row, &keys))
        .collect::<Vec<_>>();
    let delegation_matches = delegation_rows
        .iter()
        .filter(|row| row_matches(row, &keys))
        .collect::<Vec<_>>();
    let review_matches = review_rows
        .iter()
        .filter(|row| {
            row.get("branch")
                .and_then(Value::as_str)
                .is_some_and(|branch| keys.branches.contains(branch))
        })
        .collect::<Vec<_>>();
    json!({
        "skill_invocations": {"matched": skill_matches.len(), "missing": skill_rows.len().saturating_sub(skill_matches.len()), "refs": sorted_refs(&skill_matches, "skill")},
        "work_ledger": {"matched": work_matches.len(), "missing": work_rows.len().saturating_sub(work_matches.len()), "refs": sorted_refs_any(&work_matches, &["work_id", "backlog_ref"])},
        "delegations": {"matched": delegation_matches.len(), "missing": delegation_rows.len().saturating_sub(delegation_matches.len()), "refs": sorted_refs(&delegation_matches, "delegation_id")},
        "review_scores": {"matched": review_matches.len(), "missing": review_rows.len().saturating_sub(review_matches.len()), "refs": sorted_refs(&review_matches, "branch"), "trend_status": if review_rows.len() < 5 { "insufficient_data" } else { "available" }},
    })
}

struct Keys {
    session_id: BTreeSet<String>,
    backlog_ref: BTreeSet<String>,
    work_id: BTreeSet<String>,
    projects: BTreeSet<String>,
    branches: BTreeSet<String>,
}

impl Keys {
    fn from_transcripts(transcripts: &[Transcript]) -> Self {
        Self {
            session_id: transcripts
                .iter()
                .flat_map(|t| t.session_ids.clone())
                .collect(),
            backlog_ref: transcripts
                .iter()
                .flat_map(|t| t.backlog_refs.clone())
                .collect(),
            work_id: transcripts
                .iter()
                .flat_map(|t| t.work_ids.clone())
                .collect(),
            projects: transcripts
                .iter()
                .flat_map(|t| t.projects.clone())
                .collect(),
            branches: transcripts
                .iter()
                .flat_map(|t| t.branches.clone())
                .collect(),
        }
    }
}

fn row_matches(row: &Value, keys: &Keys) -> bool {
    string_in(row, "session_id", &keys.session_id)
        || string_in(row, "backlog_ref", &keys.backlog_ref)
        || string_in(row, "work_id", &keys.work_id)
        || string_in(row, "project", &keys.projects)
}

fn string_in(row: &Value, field: &str, values: &BTreeSet<String>) -> bool {
    row.get(field)
        .and_then(Value::as_str)
        .is_some_and(|value| values.contains(value))
}

fn sorted_refs(rows: &[&Value], field: &str) -> Vec<String> {
    let refs = rows
        .iter()
        .map(|row| {
            row.get(field)
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    refs.into_iter().collect()
}

fn sorted_refs_any(rows: &[&Value], fields: &[&str]) -> Vec<String> {
    let refs = rows
        .iter()
        .map(|row| {
            fields
                .iter()
                .find_map(|field| row.get(*field).and_then(Value::as_str))
                .unwrap_or("unknown")
                .to_string()
        })
        .collect::<BTreeSet<_>>();
    refs.into_iter().collect()
}

fn proposed_actions(categories: &BTreeMap<String, Value>, joins: &Value) -> Vec<String> {
    let mut actions = Vec::new();
    if categories["user_corrections"]["count"]
        .as_u64()
        .unwrap_or(0)
        > 0
    {
        actions.push(
            "Run /reflect prompt-debt on repeated correction categories before editing skills."
                .to_string(),
        );
    }
    if categories["skill_missed_opportunities"]["count"]
        .as_u64()
        .unwrap_or(0)
        > 0
    {
        actions.push(
            "Review skill trigger descriptions for missed or late invocation patterns.".to_string(),
        );
    }
    if joins["skill_invocations"]["matched"].as_u64().unwrap_or(0) == 0 {
        actions
            .push("Collect skill invocation rows before making effectiveness claims.".to_string());
    }
    if joins["review_scores"]["trend_status"].as_str() == Some("insufficient_data") {
        actions.push(
            "Treat review-score effectiveness as insufficient data until at least 5 entries exist."
                .to_string(),
        );
    }
    if actions.is_empty() {
        actions.push("No codification action proposed from this small sample.".to_string());
    }
    actions
}

fn join_strings(values: &[Value]) -> String {
    let joined = values
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>()
        .join(", ");
    if joined.is_empty() {
        "none".to_string()
    } else {
        joined
    }
}

fn write_jsonl(path: PathBuf, rows: &[Value]) -> Result<PathBuf> {
    fs::write(
        &path,
        rows.iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
            + "\n",
    )?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_contract_passes() {
        assert_eq!(
            self_test().unwrap(),
            "transcript effectiveness mining self-test ok"
        );
    }
}
