use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde_json::{Value as JsonValue, json};

const DIMENSIONS: &[&str] = &["correctness", "depth", "simplicity", "craft"];
const REQUIRED_SCORE_FIELDS: &[&str] = &[
    "date",
    "branch",
    "sha",
    "correctness",
    "depth",
    "simplicity",
    "craft",
    "verdict",
    "providers",
    "findings_total",
    "findings_accepted",
    "findings_false_positive",
    "post_merge_bugs_found",
];

pub fn default_path() -> PathBuf {
    PathBuf::from(".groom/review-scores.ndjson")
}

pub fn load_rows(path: &Path) -> Result<Vec<JsonValue>> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    let mut rows = Vec::new();
    for (index, line) in content.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let row: JsonValue = serde_json::from_str(line)
            .with_context(|| format!("{}:{line_number}: invalid JSON", path.display()))?;
        if !row.is_object() {
            bail!(
                "{}:{line_number}: row must be a JSON object",
                path.display()
            );
        }
        rows.push(row);
    }
    Ok(rows)
}

pub fn report(rows: &[JsonValue], path: &Path) -> String {
    let mut lines = vec![
        "Review Score Trend".to_string(),
        format!("- Source: {}", path.display()),
        format!("- Entries: {}", rows.len()),
    ];
    if rows.is_empty() {
        lines.push("- Status: no review scores recorded.".to_string());
        return lines.join("\n");
    }

    let missing = missing_schema_fields(rows);
    if missing.is_empty() {
        lines
            .push("- Schema coverage: all rows include required feedback-loop fields.".to_string());
    } else {
        lines.push(format!(
            "- Schema coverage: legacy or incomplete rows ({}).",
            missing
                .iter()
                .map(|(field, count)| format!("{field} missing in {count}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    let (false_positive_line, false_positive_needs_tuning) = false_positive_summary(rows);
    lines.push(format!("- {false_positive_line}"));

    if rows.len() < 5 {
        lines.push(format!(
            "- Status: insufficient trend data ({}/5 entries).",
            rows.len()
        ));
        return lines.join("\n");
    }

    lines.push("- Rolling window: last 5 entries.".to_string());
    for dimension in DIMENSIONS {
        let present: Vec<f64> = rows[rows.len() - 5..]
            .iter()
            .filter_map(|row| numeric(row, dimension))
            .collect();
        if !present.is_empty() {
            lines.push(format!("- {dimension} avg: {:.1}", mean(&present)));
        }
    }

    let regressions = regression_lines(rows);
    if !regressions.is_empty() {
        lines.push("Skill tuning suggestions:".to_string());
        lines.extend(regressions);
    } else if false_positive_needs_tuning {
        lines.push("Skill tuning suggestions:".to_string());
        lines.push(
            "- false-positive rate high: tighten finding acceptance and rejection-after-steelman guidance in skills/code-review/SKILL.md.".to_string(),
        );
    } else {
        lines.push("Skill tuning suggestions: none from current score window.".to_string());
    }
    lines.join("\n")
}

pub fn self_test() -> Result<()> {
    let rows = self_test_rows();
    let temp = tempfile::tempdir().context("failed to create temporary directory")?;
    let path = temp.path().join("review-scores.ndjson");
    let payload = rows
        .iter()
        .map(serde_json::to_string)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .join("\n")
        + "\n";
    fs::write(&path, payload)?;
    let output = report(&load_rows(&path)?, &path);
    let missing: Vec<_> = [
        "correctness regression",
        "False-positive rate: 25.0%",
        "Skill tuning suggestions",
    ]
    .into_iter()
    .filter(|token| !output.contains(token))
    .collect();
    if !missing.is_empty() {
        bail!("self-test missing token(s): {}", missing.join(", "));
    }
    Ok(())
}

fn missing_schema_fields(rows: &[JsonValue]) -> Vec<(&'static str, usize)> {
    let mut missing = Vec::new();
    for field in REQUIRED_SCORE_FIELDS {
        let count = rows.iter().filter(|row| row.get(*field).is_none()).count();
        if count > 0 {
            missing.push((*field, count));
        }
    }
    missing.sort_by_key(|(field, _)| *field);
    missing
}

fn regression_lines(rows: &[JsonValue]) -> Vec<String> {
    if rows.len() < 5 {
        return Vec::new();
    }
    let window = &rows[rows.len() - 5..];
    let mut lines = Vec::new();
    for dimension in DIMENSIONS {
        let values: Option<Vec<f64>> = window.iter().map(|row| numeric(row, dimension)).collect();
        let Some(values) = values else {
            continue;
        };
        let first = mean(&values[..2]);
        let last = mean(&values[2..]);
        let delta = last - first;
        if delta <= -2.0 {
            lines.push(format!(
                "- {dimension} regression: last-3 avg {last:.1} vs first-2 avg {first:.1}. Proposed skill target: {}.",
                tuning_target(dimension)
            ));
        }
    }
    lines
}

fn false_positive_summary(rows: &[JsonValue]) -> (String, bool) {
    let usable: Vec<(f64, f64)> = rows[rows.len().saturating_sub(20)..]
        .iter()
        .filter_map(|row| {
            let total = numeric(row, "findings_total")?;
            let false_positive = numeric(row, "findings_false_positive")?;
            (total > 0.0).then_some((total, false_positive))
        })
        .collect();
    if usable.is_empty() {
        return (
            "False-positive rate: unavailable (no calibrated finding counts).".to_string(),
            false,
        );
    }
    let total_findings: f64 = usable.iter().map(|(total, _)| total).sum();
    let total_false: f64 = usable
        .iter()
        .map(|(_, false_positive)| false_positive)
        .sum();
    let rate = if total_findings == 0.0 {
        0.0
    } else {
        total_false / total_findings
    };
    let needs_tuning = rate >= 0.25 && total_findings >= 4.0;
    (
        format!(
            "False-positive rate: {:.1}% ({}/{} calibrated findings).",
            rate * 100.0,
            total_false as i64,
            total_findings as i64
        ),
        needs_tuning,
    )
}

fn numeric(row: &JsonValue, field: &str) -> Option<f64> {
    let value = row.get(field)?;
    if value.is_boolean() {
        return None;
    }
    value.as_f64()
}

fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

fn tuning_target(dimension: &str) -> &'static str {
    match dimension {
        "correctness" => {
            "skills/code-review/SKILL.md executable-path and blocking-finding instructions"
        }
        "depth" => "skills/code-review/references/deep-review-lens.md reviewer prompts",
        "simplicity" => "skills/code-review/SKILL.md Thermo / Deslop Lens",
        "craft" => "skills/code-review/SKILL.md Completion Gate and evidence formatting",
        _ => "",
    }
}

fn self_test_rows() -> Vec<JsonValue> {
    vec![
        json!({"date":"2026-06-01","branch":"a","sha":"1","correctness":9,"depth":8,"simplicity":8,"craft":8,"verdict":"ship","providers":["codex"],"findings_total":4,"findings_accepted":4,"findings_false_positive":0,"post_merge_bugs_found":0}),
        json!({"date":"2026-06-01","branch":"b","sha":"2","correctness":9,"depth":8,"simplicity":8,"craft":8,"verdict":"ship","providers":["codex"],"findings_total":4,"findings_accepted":3,"findings_false_positive":1,"post_merge_bugs_found":0}),
        json!({"date":"2026-06-01","branch":"c","sha":"3","correctness":6,"depth":8,"simplicity":8,"craft":8,"verdict":"conditional","providers":["codex"],"findings_total":4,"findings_accepted":2,"findings_false_positive":2,"post_merge_bugs_found":1}),
        json!({"date":"2026-06-01","branch":"d","sha":"4","correctness":6,"depth":8,"simplicity":8,"craft":8,"verdict":"conditional","providers":["codex"],"findings_total":4,"findings_accepted":3,"findings_false_positive":1,"post_merge_bugs_found":0}),
        json!({"date":"2026-06-01","branch":"e","sha":"5","correctness":6,"depth":8,"simplicity":8,"craft":8,"verdict":"dont-ship","providers":["codex"],"findings_total":4,"findings_accepted":3,"findings_false_positive":1,"post_merge_bugs_found":1}),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn review_score_self_test_tokens_are_present() -> Result<()> {
        self_test()
    }

    #[test]
    fn review_score_missing_file_reports_empty_status() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("missing.ndjson");
        let output = report(&load_rows(&path)?, &path);
        assert!(output.contains("- Entries: 0"));
        assert!(output.contains("- Status: no review scores recorded."));
        Ok(())
    }

    #[test]
    fn review_score_rejects_invalid_json_and_non_object_rows() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("scores.ndjson");
        fs::write(&path, "{bad\n")?;
        assert!(
            load_rows(&path)
                .unwrap_err()
                .to_string()
                .contains("invalid JSON")
        );
        fs::write(&path, "[]\n")?;
        assert!(
            load_rows(&path)
                .unwrap_err()
                .to_string()
                .contains("row must be a JSON object")
        );
        Ok(())
    }

    #[test]
    fn review_score_reports_insufficient_data_and_schema_gaps() {
        let rows = vec![json!({"date":"2026-06-01","branch":"a"})];
        let output = report(&rows, Path::new(".groom/review-scores.ndjson"));
        assert!(output.contains("legacy or incomplete rows"));
        assert!(
            output.contains("correctness missing in 1, craft missing in 1, depth missing in 1")
        );
        assert!(output.contains("insufficient trend data (1/5 entries)"));
    }

    #[test]
    fn review_score_high_false_positive_without_regression_suggests_tuning() {
        let rows = (0..5)
            .map(|index| {
                json!({"date":"2026-06-01","branch":format!("b{index}"),"sha":format!("{index}"),"correctness":8,"depth":8,"simplicity":8,"craft":8,"verdict":"ship","providers":["codex"],"findings_total":4,"findings_accepted":2,"findings_false_positive":2,"post_merge_bugs_found":0})
            })
            .collect::<Vec<_>>();
        let output = report(&rows, Path::new(".groom/review-scores.ndjson"));
        assert!(output.contains("False-positive rate: 50.0%"));
        assert!(output.contains("false-positive rate high"));
    }
}
