use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use chrono::{NaiveDate, Utc};
use regex::Regex;
use serde_json::Value;

use crate::{agent_roster, lane_harness, source_refs, summarize_delegations};

const CORE_WORKFLOW_SKILLS: &[&str] = &[
    "ci",
    "code-review",
    "create-repo-skill",
    "deliver",
    "dispatch",
    "demo",
    "design",
    "diagnose",
    "flywheel",
    "groom",
    "hardening",
    "harness-engineering",
    "implement",
    "monitor",
    "qa",
    "refactor",
    "reflect",
    "research",
    "shape",
    "ship",
    "yeet",
];

const ROSTER_PROVIDER_IDS: &[&str] = &[
    "codex",
    "claude",
    "pi",
    "agy",
    "cursor-agent",
    "grok-build",
    "manual",
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

const WORK_RECORD_FIELDS: &[&str] = &[
    "schema_version",
    "record_type",
    "trace_id",
    "created_at",
    "backlog_ref",
    "spec_ref",
    "branch",
    "commits",
    "reviewer_verdict_refs",
    "qa_refs",
    "demo_refs",
    "transcript_refs",
    "shipped_ref",
    "waiver_reason",
    "metadata",
];
const WORK_LEDGER_FIELDS: &[&str] = &[
    "schema_version",
    "record_type",
    "event_id",
    "created_at",
    "event_type",
    "work_id",
    "parent_work_id",
    "backlog_ref",
    "branch",
    "owning_skill",
    "phase",
    "evidence_refs",
    "blockers",
    "spawned_agents",
    "trace_refs",
    "next_action",
    "status",
];
const SKILL_INVOCATION_FIELDS: &[&str] = &[
    "schema_version",
    "event_type",
    "ts",
    "harness",
    "source_protocol",
    "skill",
    "args",
    "session_id",
    "cwd",
    "project",
];
const OPTIONAL_SKILL_INVOCATION_FIELDS: &[&str] = &[
    "model_id",
    "outcome",
    "duration_ms",
    "usage",
    "backlog_ref",
    "work_id",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckReport {
    pub lines: Vec<String>,
}

pub fn run(repo: &Path) -> Result<CheckReport> {
    validate_roster(repo)?;
    validate_delegation_floor(repo)?;
    validate_runtime_delegation_references(repo)?;
    validate_shared_roster_doctrine(repo)?;
    validate_completion_evidence(repo)?;
    validate_groom_completeness_contract(repo)?;
    validate_clean_closeout_pointers(repo)?;
    validate_adversarial_done_review(repo)?;
    validate_no_source_skill_bridges(repo)?;
    validate_no_retired_provider_references(repo)?;
    validate_open_model_roster_review_due(repo)?;
    validate_model_provider_harness_index(repo)?;
    validate_source_agent_catalog(repo)?;
    validate_agents_placement_doctrine(repo)?;
    validate_code_review_pattern_references(repo)?;

    let receipts =
        validate_receipts_fixture(&repo.join(".harness-kit/examples/delegation-receipt.jsonl"))?;
    let work_records =
        validate_work_records(&repo.join(".harness-kit/examples/work-record.jsonl"))?;
    let work_ledger_records =
        validate_work_ledger(&repo.join(".harness-kit/examples/work-ledger.jsonl"))?;
    let skill_invocation_records =
        validate_skill_invocations(&repo.join(".harness-kit/examples/skill-invocations.jsonl"))?;
    validate_lane_harness_fixture(repo, &repo.join(".harness-kit/examples/lane-harness.yaml"))?;

    if receipts == 0 {
        bail!(
            ".harness-kit/examples/delegation-receipt.jsonl: must contain at least one receipt fixture"
        );
    }
    if work_records == 0 {
        bail!(
            ".harness-kit/examples/work-record.jsonl: must contain at least one work record fixture"
        );
    }
    if work_ledger_records == 0 {
        bail!(".harness-kit/examples/work-ledger.jsonl: must contain at least one ledger event");
    }
    if skill_invocation_records == 0 {
        bail!(
            ".harness-kit/examples/skill-invocations.jsonl: must contain at least one skill invocation"
        );
    }

    validate_rust_summary_helper(&repo.join(".harness-kit/examples/delegation-receipt.jsonl"))?;
    validate_rust_probe_helper(&repo.join(".harness-kit/agents.yaml"))?;
    validate_rust_dispatch_helper(&repo.join(".harness-kit/agents.yaml"))?;
    validate_helper_output(
        repo,
        &[
            "cargo",
            "run",
            "--locked",
            "-p",
            "harness-kit-checks",
            "--",
            "skill-invocation-analytics",
            "--self-test",
        ],
        "analyze-skill-invocations self-test ok",
        "harness-kit-checks skill-invocation-analytics: skill analytics helper failed",
    )?;

    validate_runtime_ignores_and_trace_paths(repo)?;

    Ok(CheckReport {
        lines: vec![
            ".harness-kit/agents.yaml: valid".to_string(),
            format!(
                ".harness-kit/examples/delegation-receipt.jsonl: {receipts} receipt fixture(s) valid"
            ),
            format!(
                ".harness-kit/examples/work-record.jsonl: {work_records} work record fixture(s) valid"
            ),
            format!(
                ".harness-kit/examples/work-ledger.jsonl: {work_ledger_records} ledger fixture(s) valid"
            ),
            ".harness-kit/examples/lane-harness.yaml: lane harness fixture valid".to_string(),
            format!(
                "skills/: {} delegation judgment section(s) valid",
                CORE_WORKFLOW_SKILLS.len()
            ),
            "skills/: 4 completion evidence pointer(s) valid".to_string(),
            "skills/: 4 local completion gate(s) valid".to_string(),
            "skills/groom/SKILL.md: completeness contract valid".to_string(),
            "closeout: 4 shared pointer(s) valid".to_string(),
            "skills/: 4 adversarial review stance(s) valid".to_string(),
            "harnesses/: 4 runtime delegation reference(s) valid".to_string(),
            "source repo: no repo-local skill bridges".to_string(),
            "active roster/docs: no retired provider references".to_string(),
            "skills/harness-engineering/references/open-model-roster.md: review due date valid"
                .to_string(),
            "skills/harness-engineering/references/model-provider-harness-index.md: factual model reference valid"
                .to_string(),
            "agents/: 3 source agent file(s) valid".to_string(),
            "AGENTS doctrine placement: valid".to_string(),
            "harness-kit-checks summarize-delegations: report helper valid".to_string(),
            "harness-kit-checks probe-agent-roster: roster helper valid".to_string(),
            "harness-kit-checks dispatch-agent: dispatch helper valid".to_string(),
        ],
    })
}

pub fn markdown_section<'a>(text: &'a str, heading: &str) -> &'a str {
    let Some(start) = text.find(heading) else {
        return "";
    };
    let search_start = start.saturating_add(1);
    match text[search_start..].find("\n## ") {
        Some(relative_end) => &text[start..search_start + relative_end],
        None => &text[start..],
    }
}

fn delegation_floor_section(text: &str) -> &str {
    markdown_section(text, "## Delegation Judgment")
}

fn has_delegation_floor_pointer(section: &str) -> bool {
    let low = section.to_lowercase();
    (low.contains("delegate on judgment") || low.contains("native subagents by default"))
        && low.contains("harnesses/shared/agents.md")
        && low.contains("roster")
}

fn has_local_lane_guidance(section: &str) -> bool {
    Regex::new(r"(?im)^local lane guidance:\s*(.+)$")
        .expect("static regex compiles")
        .captures(section)
        .and_then(|capture| capture.get(1))
        .is_some_and(|matched| !matched.as_str().trim().is_empty())
}

pub fn delegation_contract_gaps(section: &str) -> Vec<String> {
    let lowered = section.to_lowercase();
    let mut missing = Vec::new();
    for (name, phrases) in [
        (
            "native-first default",
            vec!["native subagents", "native first"],
        ),
        (
            "cross-model criticism",
            vec!["cross-model", "different model family"],
        ),
        ("sprite routing", vec!["sprite"]),
        ("lane responsibilities", vec!["lane"]),
        ("context boundary", vec!["context", "give", "scope"]),
        ("output evidence", vec!["receipt", "evidence"]),
        ("lead verification", vec!["lead"]),
    ] {
        if !phrases.iter().any(|phrase| lowered.contains(phrase)) {
            missing.push(name.to_string());
        }
    }
    if !lowered.contains("provider roster is available")
        && !lowered.contains(".harness-kit/agents.yaml")
    {
        missing.push("roster availability".to_string());
    }

    let flattened = Regex::new(r"\s+")
        .expect("static regex compiles")
        .replace_all(&lowered, " ")
        .into_owned();
    for (name, patterns) in delegation_commitments() {
        if !has_unhedged_match(&patterns, &flattened) {
            missing.push(name.to_string());
        }
    }
    missing
}

fn delegation_commitments() -> Vec<(String, Vec<Regex>)> {
    vec![
        (
            "native-first commitment".to_string(),
            vec![
                Regex::new(r"\bnative (subagents?|first)\b[^.]{0,160}\b(default|first)\b").expect("static regex compiles"),
                Regex::new(r"\b(default|first)\b[^.]{0,160}\bnative (subagents?|delegation)\b").expect("static regex compiles"),
                Regex::new(r"\bsubagents\b[^.]{0,80}\bdefault delegation\b").expect("static regex compiles"),
                Regex::new(r"\bsubagents\b[^.]{0,80}\bare the default\b").expect("static regex compiles"),
            ],
        ),
        (
            "cross-model critic commitment".to_string(),
            vec![
                Regex::new(r"\b(cross-model|different model family)\b[^.]{0,160}\b(critic\w*|criticism|review\w*|judgment)\b").expect("static regex compiles"),
                Regex::new(r"\b(critic\w*|criticism|review\w*)\b[^.]{0,160}\b(cross-model|different model family)\b").expect("static regex compiles"),
            ],
        ),
        (
            "scoped lane handoff".to_string(),
            vec![
                Regex::new(r"\b(give|gives)\b[^.]{0,80}\b(lane|lanes|each lane|providers?|members?|reviewers?|critics?|them)\b[^.]{0,160}\b(scoped|scope|context|files|questions|commands|output|evidence|receipt|sources|methods|risk|artifact|boundar\w*|oracle)\b").expect("static regex compiles"),
                Regex::new(r"\bscoped\b[^.]{0,80}\b(lane|lanes|each lane|providers?|members?|reviewers?|critics?)\b").expect("static regex compiles"),
                Regex::new(r"\buse\b[^.]{0,80}\blanes?\b[^.]{0,160}\b(scoped|scope|context|files|questions|commands|output|evidence|receipt|sources|methods|risk|boundar\w*)\b").expect("static regex compiles"),
            ],
        ),
        (
            "lead-owned synthesis".to_string(),
            vec![
                Regex::new(r"\bthe lead(?: agent| model)?\b[^.]{0,160}\b(owns|verif\w*|records?|accepts?|keeps|synthesis|final)\b").expect("static regex compiles"),
                Regex::new(r"\blead agent\b[^.]{0,160}\b(owns|verif\w*|records?|accepts?|keeps|synthesis|final)\b").expect("static regex compiles"),
                Regex::new(r"\blead synthesis\b").expect("static regex compiles"),
            ],
        ),
    ]
}

fn has_unhedged_match(patterns: &[Regex], flattened: &str) -> bool {
    let hedged = Regex::new(
        r"\b(may|might|optional|whether|if available|at [^.]{0,40} discretion|decide later|reminders only|only matters)\b",
    )
    .expect("static regex compiles");
    for pattern in patterns {
        for matched in pattern.find_iter(flattened) {
            let sentence_end = flattened[matched.start()..]
                .find('.')
                .map(|end| matched.start() + end)
                .unwrap_or_else(|| (matched.end() + 160).min(flattened.len()));
            let window_start = matched.start().saturating_sub(40);
            let window = &flattened[window_start..sentence_end];
            if !hedged.is_match(window) {
                return true;
            }
        }
    }
    false
}

pub fn phrase_group_gaps(text: &str, requirements: &[(&str, &[&str])]) -> Vec<String> {
    let lowered = text.to_lowercase();
    requirements
        .iter()
        .filter(|(_, phrases)| !phrases.iter().any(|phrase| lowered.contains(phrase)))
        .map(|(name, _)| (*name).to_string())
        .collect()
}

fn validate_delegation_floor(repo: &Path) -> Result<()> {
    let mut missing = Vec::new();
    let mut weak = Vec::new();
    for skill in CORE_WORKFLOW_SKILLS {
        let path = PathBuf::from("skills").join(skill).join("SKILL.md");
        let full_path = repo.join(&path);
        if !full_path.exists() {
            continue;
        }
        let text = read_to_string(&full_path)?;
        let section = delegation_floor_section(&text);
        if section.is_empty() {
            missing.push(path.display().to_string());
            continue;
        }
        if has_delegation_floor_pointer(section) {
            if !has_local_lane_guidance(section) {
                weak.push(format!("{} (missing local lane guidance)", path.display()));
            }
            continue;
        }
        let gaps = delegation_contract_gaps(section);
        if !gaps.is_empty() {
            weak.push(format!("{} ({})", path.display(), gaps.join(", ")));
            continue;
        }
        if section.to_lowercase().contains("explicit user waivers") {
            weak.push(path.display().to_string());
        }
    }
    let mut errors = Vec::new();
    if !missing.is_empty() {
        errors.push(format!(
            "missing delegation judgment: {}",
            missing.join(", ")
        ));
    }
    if !weak.is_empty() {
        errors.push(format!("weak delegation judgment: {}", weak.join(", ")));
    }
    if !errors.is_empty() {
        bail!("{}", errors.join("; "));
    }
    Ok(())
}

fn validate_runtime_delegation_references(repo: &Path) -> Result<()> {
    let refs = [
        ("Claude Code", "harnesses/claude/README.md"),
        ("Codex", "harnesses/codex/README.md"),
        ("Antigravity", "harnesses/antigravity-cli/README.md"),
        ("Pi", "harnesses/pi/README.md"),
    ];
    let mut issues = Vec::new();
    for (runtime, relative) in refs {
        let path = repo.join(relative);
        if !path.exists() {
            issues.push(format!(
                "{relative}: missing {runtime} dynamic delegation reference"
            ));
            continue;
        }
        let text = read_to_string(&path)?.to_lowercase();
        let missing: Vec<_> = [
            "dynamic delegation",
            "roster",
            "receipt",
            "evidence",
            "lead",
        ]
        .into_iter()
        .filter(|phrase| !text.contains(phrase))
        .collect();
        if !missing.is_empty() {
            issues.push(format!(
                "{relative}: missing phrase(s): {}",
                missing.join(", ")
            ));
        }
    }
    if !issues.is_empty() {
        bail!("{}", issues.join("; "));
    }
    for path in [
        "skills/deliver/SKILL.md",
        "skills/flywheel/SKILL.md",
        "skills/ship/SKILL.md",
        "skills/yeet/SKILL.md",
    ] {
        let text = read_to_string(&repo.join(path))?.to_lowercase();
        if !text.contains("git rev-list --left-right --count") || !text.contains("unpushed") {
            issues.push(format!("{path}: missing remote-sync closeout language"));
        }
    }
    if !issues.is_empty() {
        bail!("{}", issues.join("; "));
    }
    Ok(())
}

fn validate_shared_roster_doctrine(repo: &Path) -> Result<()> {
    let path = repo.join("harnesses/shared/AGENTS.md");
    let text = read_to_string(&path)?;
    let lowered = text.to_lowercase();
    let missing: Vec<_> = [
        "native first",
        "cross-model criticism",
        "sprites are substrate, not providers",
        "a probe is not a provider attempt",
    ]
    .into_iter()
    .filter(|phrase| !lowered.contains(phrase))
    .collect();
    if !missing.is_empty() {
        bail!(
            "harnesses/shared/AGENTS.md: missing roster doctrine phrase(s): {}",
            missing.join(", ")
        );
    }
    let roster_section = markdown_section(&text, "## Roster");
    if roster_section.is_empty() {
        bail!("harnesses/shared/AGENTS.md: missing '## Roster' single-source section");
    }
    let gaps = delegation_contract_gaps(roster_section);
    if !gaps.is_empty() {
        bail!(
            "harnesses/shared/AGENTS.md (## Roster): missing delegation-contract requirement(s): {}",
            gaps.join(", ")
        );
    }
    Ok(())
}

fn validate_completion_evidence(repo: &Path) -> Result<()> {
    let shared = read_to_string(&repo.join("harnesses/shared/AGENTS.md"))?;
    let section = markdown_section(&shared, "## Completion Evidence");
    if section.is_empty() {
        bail!("harnesses/shared/AGENTS.md: missing '## Completion Evidence' section");
    }
    let gaps = phrase_group_gaps(
        section,
        &[
            (
                "behavior",
                &["behavior", "end-user", "developer", "operator"],
            ),
            ("live evidence", &["live evidence"]),
            (
                "exercised surface",
                &["command", "path", "route", "artifact", "surface"],
            ),
            ("repo fit", &["repo-fit"]),
            ("residual risk", &["residual", "waiver", "follow-up"]),
        ],
    );
    if !gaps.is_empty() {
        bail!(
            "harnesses/shared/AGENTS.md (## Completion Evidence): missing requirement(s): {}",
            gaps.join(", ")
        );
    }

    let mut issues = Vec::new();
    for skill in ["code-review", "deliver", "implement", "refactor"] {
        let path = format!("skills/{skill}/SKILL.md");
        let text = read_to_string(&repo.join(&path))?.to_lowercase();
        let missing: Vec<_> = [
            "completion evidence core applies",
            "harnesses/shared/agents.md",
            "completion evidence",
            "local fields",
        ]
        .into_iter()
        .filter(|phrase| !text.contains(phrase))
        .collect();
        if !missing.is_empty() {
            issues.push(format!(
                "{path}: missing completion evidence pointer ({})",
                missing.join(", ")
            ));
        }
    }
    for skill in ["demo", "design", "hardening", "qa"] {
        let path = format!("skills/{skill}/SKILL.md");
        let text = read_to_string(&repo.join(&path))?.to_lowercase();
        if !text.contains("## completion gate") && !text.contains("### completion gate") {
            issues.push(format!("{path}: missing local completion gate"));
        }
        if !text.contains("harnesses/shared/agents.md") || !text.contains("completion evidence") {
            issues.push(format!(
                "{path}: missing shared Completion Evidence pointer"
            ));
        }
    }
    if !issues.is_empty() {
        bail!("{}", issues.join("; "));
    }
    Ok(())
}

fn validate_groom_completeness_contract(repo: &Path) -> Result<()> {
    let text = read_to_string(&repo.join("skills/groom/SKILL.md"))?.to_lowercase();
    let mut issues = Vec::new();
    for (label, phrases) in [
        ("groom completeness gate", vec!["groom completeness gate"]),
        (
            "minimum strategic fanout",
            vec!["minimum strategic fanout", "at least seven"],
        ),
        (
            "mandatory research",
            vec![
                "research is mandatory",
                "exa",
                "xai/grok",
                "thinktank",
                "codebase",
            ],
        ),
        (
            "product aperture",
            vec!["ideal-form", "product should become"],
        ),
        ("security privacy", vec!["security/privacy"]),
        ("agent readiness", vec!["agent-readiness"]),
        ("simplification deletion", vec!["simplification/deletion"]),
        (
            "operator artifact",
            vec![
                "providers used",
                "accepted/rejected findings",
                "residual risks",
            ],
        ),
    ] {
        let missing: Vec<_> = phrases
            .into_iter()
            .filter(|phrase| !text.contains(phrase))
            .collect();
        if !missing.is_empty() {
            issues.push(format!("{label} ({})", missing.join(", ")));
        }
    }
    if !issues.is_empty() {
        bail!(
            "skills/groom/SKILL.md: incomplete groom completeness contract: {}",
            issues.join("; ")
        );
    }
    Ok(())
}

fn validate_clean_closeout_pointers(repo: &Path) -> Result<()> {
    let shared = read_to_string(&repo.join("harnesses/shared/AGENTS.md"))?;
    let section = markdown_section(&shared, "## Closeout");
    let missing: Vec<_> = [
        "single source for clean-tree closeout",
        "git status --short --untracked-files=all",
        "committing it, deleting",
        "git rev-list --left-right --count",
        "visible path is an action item",
    ]
    .into_iter()
    .filter(|phrase| !section.to_lowercase().contains(phrase))
    .collect();
    if !missing.is_empty() {
        bail!(
            "harnesses/shared/AGENTS.md (## Closeout): missing phrase(s): {}",
            missing.join(", ")
        );
    }
    let mut issues = Vec::new();
    for path in [
        "AGENTS.md",
        "skills/deliver/SKILL.md",
        "skills/flywheel/SKILL.md",
        "skills/ship/SKILL.md",
        "skills/yeet/SKILL.md",
    ] {
        let text = read_to_string(&repo.join(path))?.to_lowercase();
        if !text.contains("harnesses/shared/agents.md") || !text.contains("closeout") {
            issues.push(format!("{path}: missing shared Closeout pointer"));
        }
    }
    if !issues.is_empty() {
        bail!("{}", issues.join("; "));
    }
    Ok(())
}

fn validate_adversarial_done_review(repo: &Path) -> Result<()> {
    let shared = read_to_string(&repo.join("harnesses/shared/AGENTS.md"))?.to_lowercase();
    let missing: Vec<_> = [
        "adversarial",
        "embarrass us in production",
        "automatic veto",
        "lead accepts or",
    ]
    .into_iter()
    .filter(|phrase| !shared.contains(phrase))
    .collect();
    if !missing.is_empty() {
        bail!(
            "harnesses/shared/AGENTS.md: missing adversarial review phrase(s): {}",
            missing.join(", ")
        );
    }
    let mut issues = Vec::new();
    for skill in ["code-review", "implement", "qa", "shape"] {
        let path = format!("skills/{skill}/SKILL.md");
        let text = read_to_string(&repo.join(&path))?.to_lowercase();
        if !text.contains("adversarial") {
            issues.push(format!("{path}: missing adversarial review stance"));
            continue;
        }
        if !text.contains("embarrass us") && !text.contains("production embarrassment") {
            issues.push(format!(
                "{path}: missing production-embarrassment calibration"
            ));
        }
    }
    if !issues.is_empty() {
        bail!("{}", issues.join("; "));
    }
    Ok(())
}

fn validate_no_source_skill_bridges(repo: &Path) -> Result<()> {
    let present: Vec<_> = [
        ".agents/skills",
        ".codex/skills",
        ".claude/skills",
        ".pi/skills",
    ]
    .into_iter()
    .filter(|path| repo.join(path).exists())
    .collect();
    if !present.is_empty() {
        bail!(
            "source repo must not commit repo-local skill bridges: {}",
            present.join(", ")
        );
    }
    Ok(())
}

fn validate_no_retired_provider_references(repo: &Path) -> Result<()> {
    let pattern = Regex::new(r"(?i)\bopen[- ]?code\b|\bopencode\b").expect("static regex compiles");
    let mut hits = Vec::new();
    for relative in [
        ".harness-kit/agents.yaml",
        "harnesses/shared/AGENTS.md",
        "docs/copy/site.json",
        "harnesses/pi/README.md",
        "harnesses/pi/settings.json",
        "skills/harness-engineering/references/open-model-roster.md",
    ] {
        let path = repo.join(relative);
        if !path.exists() {
            continue;
        }
        for (index, line) in read_to_string(&path)?.lines().enumerate() {
            if line.contains("RETIRED_RECEIPT_PROVIDER_IDS") {
                continue;
            }
            if pattern.is_match(line) {
                hits.push(format!("{relative}:{}", index + 1));
            }
        }
    }
    if !hits.is_empty() {
        bail!(
            "retired provider reference(s) in active roster/docs: {}",
            hits.join(", ")
        );
    }
    Ok(())
}

fn validate_open_model_roster_review_due(repo: &Path) -> Result<()> {
    let relative = "skills/harness-engineering/references/open-model-roster.md";
    let text = read_to_string(&repo.join(relative))?;
    let pattern = Regex::new(r"(?m)^roster_review_due:\s*(\d{4}-\d{2}-\d{2})$")
        .expect("static regex compiles");
    let Some(captures) = pattern.captures(&text) else {
        bail!("{relative}: missing roster_review_due");
    };
    let review_due = NaiveDate::parse_from_str(&captures[1], "%Y-%m-%d")
        .with_context(|| format!("{relative}: invalid roster_review_due"))?;
    let today = Utc::now().date_naive();
    if today > review_due {
        bail!("{relative}: roster review overdue since {review_due}");
    }
    Ok(())
}

fn validate_model_provider_harness_index(repo: &Path) -> Result<()> {
    let relative = "skills/harness-engineering/references/model-provider-harness-index.md";
    let text = read_to_string(&repo.join(relative))?;
    let due = extract_review_due(&text, "model_reference_review_due", relative)?;
    let today = Utc::now().date_naive();
    if today > due {
        bail!("{relative}: model reference review overdue since {due}");
    }
    for required in [
        "Factual context for composition design",
        "not a routing policy",
        "must not prescribe role fit",
        "Do not add subjective labels",
        "probe-agent-roster",
        "OpenRouter",
        "Anthropic",
    ] {
        if !text.contains(required) {
            bail!("{relative}: missing required factual-reference phrase {required:?}");
        }
    }
    if text.contains("best for planning") || text.contains("preferred for review") {
        bail!("{relative}: contains role-fit policy language");
    }
    let active_models = active_roster_models(repo)?;
    let missing: Vec<_> = active_models
        .iter()
        .filter(|model| !model_mentioned(&text, model))
        .cloned()
        .collect();
    if !missing.is_empty() {
        bail!(
            "{relative}: missing active roster model(s): {}",
            missing.join(", ")
        );
    }
    Ok(())
}

fn extract_review_due(text: &str, key: &str, relative: &str) -> Result<NaiveDate> {
    let pattern = Regex::new(&format!(r"(?m)^{key}:\s*(\d{{4}}-\d{{2}}-\d{{2}})$"))
        .expect("static regex compiles");
    let Some(captures) = pattern.captures(text) else {
        bail!("{relative}: missing {key}");
    };
    NaiveDate::parse_from_str(&captures[1], "%Y-%m-%d")
        .with_context(|| format!("{relative}: invalid {key}"))
}

fn active_roster_models(repo: &Path) -> Result<BTreeSet<String>> {
    let path = repo.join(".harness-kit/agents.yaml");
    let roster: serde_yaml::Value = serde_yaml::from_str(&read_to_string(&path)?)
        .with_context(|| ".harness-kit/agents.yaml: invalid YAML")?;
    let providers = roster
        .as_mapping()
        .and_then(|mapping| mapping.get("providers"))
        .and_then(|value| value.as_mapping())
        .ok_or_else(|| anyhow!("roster must define providers mapping."))?;
    let mut models = BTreeSet::new();
    for provider in providers.values().filter_map(|value| value.as_mapping()) {
        if let Some(model) = provider.get("model").and_then(|value| value.as_str()) {
            models.insert(model.to_string());
        }
        if let Some(variants) = provider
            .get("model_variants")
            .and_then(|value| value.as_mapping())
        {
            for model in variants.values().filter_map(|value| value.as_str()) {
                models.insert(model.to_string());
            }
        }
    }
    Ok(models)
}

fn model_mentioned(text: &str, model: &str) -> bool {
    if text.contains(model) {
        return true;
    }
    if let Some(stripped) = model.strip_prefix("openrouter/") {
        return text.contains(stripped);
    }
    false
}

fn validate_source_agent_catalog(repo: &Path) -> Result<()> {
    let allowed: BTreeSet<_> = ["a11y-auditor.md", "a11y-critic.md", "a11y-fixer.md"]
        .into_iter()
        .collect();
    let actual: BTreeSet<String> = fs::read_dir(repo.join("agents"))?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".md"))
        .collect();
    let extra: Vec<_> = actual
        .iter()
        .filter(|name| !allowed.contains(name.as_str()))
        .cloned()
        .collect();
    let missing: Vec<_> = allowed
        .iter()
        .filter(|name| !actual.contains(**name))
        .copied()
        .collect();
    if !extra.is_empty() || !missing.is_empty() {
        let mut detail = Vec::new();
        if !extra.is_empty() {
            detail.push(format!("retired static agent file(s): {extra:?}"));
        }
        if !missing.is_empty() {
            detail.push(format!("missing allowed a11y agent file(s): {missing:?}"));
        }
        bail!("agents/: {}", detail.join("; "));
    }
    Ok(())
}

fn validate_agents_placement_doctrine(repo: &Path) -> Result<()> {
    let root = read_to_string(&repo.join("AGENTS.md"))?;
    let shared = read_to_string(&repo.join("harnesses/shared/AGENTS.md"))?;
    let mut issues = Vec::new();
    for phrase in [
        "all 27 Harness Kit CI lanes",
        "## Root Skills",
        "Use these for harness work:",
        "docs/copy/site.json",
        "scripts/build-docs-site.sh",
        "cargo run --locked -p harness-kit-checks -- check-docs-site",
        "skills/harness-engineering/SKILL.md",
    ] {
        if root.contains(phrase) {
            issues.push(format!(
                "AGENTS.md: remove drift-prone catalog/prose phrase: {phrase:?}"
            ));
        }
    }
    if !root.contains("generated skill catalog for skill discovery") {
        issues.push(
            "AGENTS.md: must point to generated skill catalog instead of mirroring skills"
                .to_string(),
        );
    }
    if !root.contains("Harness Kit architecture constraints:") {
        issues
            .push("AGENTS.md: root Red Lines must name Harness Kit architecture scope".to_string());
    }
    for phrase in [
        "Harness Kit source checkout",
        "not for the Harness Kit source",
    ] {
        if shared.contains(phrase) {
            issues.push(format!(
                "harnesses/shared/AGENTS.md: shared doctrine contains source-repo-specific phrase: {phrase:?}"
            ));
        }
    }
    if !shared.contains("Universal agent safety rules:") {
        issues.push(
            "harnesses/shared/AGENTS.md: shared Red Lines must name universal safety scope"
                .to_string(),
        );
    }
    if !shared.contains("consumer-repo artifacts") {
        issues.push("harnesses/shared/AGENTS.md: shared Harness section must frame vendored bridges as consumer-repo artifacts".to_string());
    }
    if !issues.is_empty() {
        bail!("{}", issues.join("; "));
    }
    Ok(())
}

fn validate_code_review_pattern_references(repo: &Path) -> Result<()> {
    let skill = read_to_string(&repo.join("skills/code-review/SKILL.md"))?;
    let template =
        read_to_string(&repo.join("skills/code-review/references/review-patterns-template.md"))?;
    let bounded_relative = "skills/code-review/references/bounded-payload-discipline.md";
    let bounded_path = repo.join(bounded_relative);
    if !bounded_path.exists() {
        bail!("{bounded_relative}: missing bounded-payload reference");
    }
    let bounded = read_to_string(&bounded_path)?;
    for phrase in [
        "Any API response that advertises a cap",
        "Shape A: Bounded Fetch",
        "Shape B: Count Plus Bounded Fetch",
        "Ecto's",
        "Prisma",
        "Assertion Pattern",
    ] {
        if !bounded.contains(phrase) {
            bail!("{bounded_relative}: missing required phrase {phrase:?}");
        }
    }
    if !skill.contains("review-patterns.md") || !skill.contains("bounded-payload-discipline.md") {
        bail!("skills/code-review/SKILL.md: missing review pattern context loading");
    }
    if !template.contains("bounded-payload-discipline.md") || !template.contains("**Reference.**") {
        bail!(
            "skills/code-review/references/review-patterns-template.md: missing shared reference wiring"
        );
    }
    Ok(())
}

fn validate_roster(repo: &Path) -> Result<()> {
    let path = repo.join(".harness-kit/agents.yaml");
    let roster: serde_yaml::Value = serde_yaml::from_str(&read_to_string(&path)?)
        .with_context(|| ".harness-kit/agents.yaml: invalid YAML")?;
    let mapping = roster
        .as_mapping()
        .ok_or_else(|| anyhow!(".harness-kit/agents.yaml must contain a YAML mapping."))?;
    if mapping.get("version").and_then(|value| value.as_i64()) != Some(1) {
        bail!("roster version must be 1.");
    }
    let providers = mapping
        .get("providers")
        .and_then(|value| value.as_mapping())
        .ok_or_else(|| anyhow!("roster must define providers mapping."))?;
    let actual: BTreeSet<String> = providers
        .keys()
        .filter_map(|key| key.as_str().map(ToOwned::to_owned))
        .collect();
    let expected: BTreeSet<String> = ROSTER_PROVIDER_IDS
        .iter()
        .map(|value| (*value).to_string())
        .collect();
    let missing: Vec<_> = expected.difference(&actual).cloned().collect();
    let extra: Vec<_> = actual.difference(&expected).cloned().collect();
    if !missing.is_empty() {
        bail!("roster missing providers: {}", missing.join(", "));
    }
    if !extra.is_empty() {
        bail!("roster contains unknown providers: {}", extra.join(", "));
    }
    for provider_id in ROSTER_PROVIDER_IDS {
        let provider = providers
            .get(*provider_id)
            .and_then(|value| value.as_mapping())
            .ok_or_else(|| anyhow!("{provider_id}: provider entry must be a mapping."))?;
        for field in [
            "tier",
            "kind",
            "probe",
            "dispatch",
            "output",
            "permissions",
            "worktree",
            "notes",
        ] {
            if !provider.contains_key(field) {
                bail!("{provider_id}: missing fields: {field}");
            }
        }
        validate_enum(
            provider_id,
            provider,
            "tier",
            &["primary", "conditional", "manual", "disabled"],
        )?;
        validate_enum(provider_id, provider, "kind", &["cli", "bench", "manual"])?;
        validate_enum(
            provider_id,
            provider,
            "output",
            &["json", "stream-json", "text", "patch-ref", "manual-summary"],
        )?;
        validate_enum(
            provider_id,
            provider,
            "worktree",
            &["required", "recommended", "not_applicable"],
        )?;
        if *provider_id == "manual"
            && (scalar(provider, "kind") != Some("manual")
                || scalar(provider, "tier") != Some("manual"))
        {
            bail!("manual provider must use tier=manual and kind=manual.");
        }
        for field in ["probe", "dispatch", "permissions", "notes"] {
            let Some(value) = scalar(provider, field) else {
                bail!("{provider_id}: {field} must be a non-empty string.");
            };
            if value.trim().is_empty() {
                bail!("{provider_id}: {field} must be a non-empty string.");
            }
            validate_no_secret_like_text(value, &format!("{provider_id}: {field}"))?;
            if matches!(field, "probe" | "dispatch") && shell_meta_regex().is_match(value) {
                bail!("{provider_id}: {field} contains shell metacharacters.");
            }
        }
        if let Some(variants) = provider.get("model_variants") {
            let Some(variants) = variants.as_mapping() else {
                bail!("{provider_id}: model_variants must be a mapping.");
            };
            for (name, model) in variants {
                let Some(name) = name.as_str() else {
                    bail!("{provider_id}: model_variants keys must be non-empty strings.");
                };
                let Some(model) = model.as_str() else {
                    bail!("{provider_id}: model_variants values must be non-empty strings.");
                };
                if name.trim().is_empty() || model.trim().is_empty() {
                    bail!(
                        "{provider_id}: model_variants keys and values must be non-empty strings."
                    );
                }
                validate_no_secret_like_text(model, &format!("{provider_id}: model_variants"))?;
            }
        }
    }
    Ok(())
}

fn validate_enum(
    provider_id: &str,
    provider: &serde_yaml::Mapping,
    field: &str,
    valid: &[&str],
) -> Result<()> {
    let value = scalar(provider, field);
    if value.is_none_or(|value| !valid.contains(&value)) {
        bail!("{provider_id}: {field} must be one of: {}.", {
            let mut sorted = valid.to_vec();
            sorted.sort_unstable();
            sorted.join(", ")
        });
    }
    Ok(())
}

fn scalar<'a>(mapping: &'a serde_yaml::Mapping, key: &str) -> Option<&'a str> {
    mapping.get(key).and_then(|value| value.as_str())
}

fn validate_receipts_fixture(path: &Path) -> Result<usize> {
    let mut count = 0;
    for (line_number, record) in read_jsonl(path)? {
        validate_receipt_record(&record)
            .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        count += 1;
    }
    Ok(count)
}

fn validate_receipt_record(record: &Value) -> Result<()> {
    let object = object(record, "receipt must be a JSON object")?;
    validate_field_set(
        object,
        required_receipt_fields(),
        optional_receipt_fields(),
        "receipt",
    )?;
    expect_i64(
        object,
        "schema_version",
        1,
        "receipt schema_version must be 1.",
    )?;
    expect_uuid_like(
        object,
        "delegation_id",
        "receipt delegation_id must be a UUID.",
    )?;
    expect_string_list(
        object,
        "redactions_applied",
        "receipt redactions_applied must be a list.",
    )?;
    expect_enum(
        object,
        "provider_target",
        RECEIPT_PROVIDER_IDS,
        "receipt provider_target is not a known provider id.",
    )?;
    expect_enum(
        object,
        "provider_status",
        VALID_PROVIDER_STATUS,
        "receipt provider_status is invalid.",
    )?;
    expect_enum(
        object,
        "attempt_status",
        VALID_ATTEMPT_STATUS,
        "receipt attempt_status is invalid.",
    )?;
    expect_enum(
        object,
        "lead_verdict",
        VALID_LEAD_VERDICTS,
        "receipt lead_verdict is invalid.",
    )?;
    validate_optional_text(object, "model_id")?;
    validate_optional_nonnegative_int(object, "duration_ms")?;
    validate_optional_nonnegative_int(object, "transcript_bytes")?;
    validate_optional_text(object, "lane_harness_ref")?;
    validate_optional_sha256(object, "lane_harness_sha256")?;
    if let Some(value) = object.get("projection_status").and_then(Value::as_str)
        && !lane_harness::validate_projection_status(value)
    {
        bail!("receipt projection_status is invalid.");
    }
    if let Some(value) = object.get("failure_kind").and_then(Value::as_str)
        && !lane_harness::validate_failure_kind(value)
    {
        bail!("receipt failure_kind is invalid.");
    }
    if let Some(usage) = object.get("usage") {
        validate_usage(usage)?;
    }
    if let Some(output_check) = object.get("output_check") {
        validate_output_check(output_check)?;
    }
    validate_optional_work_source_refs(object, object.get("backlog_ref").and_then(Value::as_str))?;
    let refs = object
        .get("evidence_refs")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("receipt evidence_refs must be a list."))?;
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
        validate_no_secret_like_text(reference, "receipt evidence_refs")?;
    }
    for field in ["objective", "summary", "input_ref", "backlog_ref"] {
        if let Some(value) = object.get(field).and_then(Value::as_str) {
            validate_no_secret_like_text(value, &format!("receipt {field}"))?;
        }
    }
    Ok(())
}

fn validate_output_check(value: &Value) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    let object = object(value, "output_check must be an object or null.")?;
    let valid_fields: BTreeSet<_> = ["expected", "matched", "observed_ref"]
        .into_iter()
        .collect();
    let extra: Vec<_> = object
        .keys()
        .filter(|field| !valid_fields.contains(field.as_str()))
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

fn validate_optional_work_source_refs(
    object: &serde_json::Map<String, Value>,
    backlog_ref: Option<&str>,
) -> Result<()> {
    let Some(value) = object.get(source_refs::FIELD) else {
        return Ok(());
    };
    let Some(refs) = value.as_array() else {
        bail!("work_source_refs must be a list.");
    };
    source_refs::validate_refs(refs, backlog_ref).context("invalid work_source_refs")
}

fn validate_usage(value: &Value) -> Result<()> {
    if value.is_null() {
        return Ok(());
    }
    let object = object(value, "usage must be an object or null.")?;
    let valid_fields: BTreeSet<_> = [
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
        .filter(|field| !valid_fields.contains(field.as_str()))
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

fn validate_work_records(path: &Path) -> Result<usize> {
    let mut count = 0;
    for (line_number, record) in read_jsonl(path)? {
        let object = object(&record, "work record must be a JSON object")
            .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        validate_field_set(
            object,
            set(WORK_RECORD_FIELDS),
            set(&["work_source_refs"]),
            "work-record",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_i64(object, "schema_version", 1, "schema_version must be 1")
            .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_string_value(
            object,
            "record_type",
            "agent-session-trace",
            "record_type must be agent-session-trace",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_prefix(
            object,
            "trace_id",
            "trace-",
            "trace_id must start with trace-",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        if array_len(object, "transcript_refs").unwrap_or(0) == 0
            && object
                .get("waiver_reason")
                .and_then(Value::as_str)
                .unwrap_or("")
                .is_empty()
        {
            bail!(
                "{}:{line_number}: transcript_refs or waiver_reason required",
                display_path(path)
            );
        }
        for field in [
            "commits",
            "reviewer_verdict_refs",
            "qa_refs",
            "demo_refs",
            "transcript_refs",
        ] {
            expect_array(object, field, &format!("{field} must be a list"))
                .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        }
        if !object.get("metadata").is_some_and(Value::is_object) {
            bail!(
                "{}:{line_number}: metadata must be an object",
                display_path(path)
            );
        }
        validate_optional_work_source_refs(
            object,
            object.get("backlog_ref").and_then(Value::as_str),
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        validate_no_secret_like_values(&record, &format!("{}:{line_number}", display_path(path)))?;
        count += 1;
    }
    Ok(count)
}

fn validate_work_ledger(path: &Path) -> Result<usize> {
    let mut count = 0;
    for (line_number, record) in read_jsonl(path)? {
        let object = object(&record, "work ledger event must be a JSON object")
            .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        validate_field_set(
            object,
            set(WORK_LEDGER_FIELDS),
            set(&["usage", "work_source_refs"]),
            "work-ledger",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_i64(object, "schema_version", 1, "schema_version must be 1")
            .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_string_value(
            object,
            "record_type",
            "work-ledger-event",
            "record_type must be work-ledger-event",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_prefix(
            object,
            "event_id",
            "work-",
            "event_id must start with work-",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        for field in ["evidence_refs", "blockers", "spawned_agents", "trace_refs"] {
            expect_array(object, field, &format!("{field} must be a list"))
                .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        }
        expect_enum(
            object,
            "event_type",
            &[
                "phase_started",
                "phase_completed",
                "blocker_added",
                "next_action_changed",
            ],
            "invalid work-ledger event_type",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_enum(
            object,
            "status",
            &["active", "blocked", "completed", "failed", "superseded"],
            "invalid work-ledger status",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        if let Some(usage) = object.get("usage") {
            validate_usage(usage)
                .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        }
        validate_optional_work_source_refs(
            object,
            object.get("backlog_ref").and_then(Value::as_str),
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        count += 1;
    }
    Ok(count)
}

fn validate_skill_invocations(path: &Path) -> Result<usize> {
    let mut count = 0;
    for (line_number, record) in read_jsonl(path)? {
        let object = object(&record, "skill invocation must be a JSON object")
            .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        validate_field_set(
            object,
            set(SKILL_INVOCATION_FIELDS),
            set(OPTIONAL_SKILL_INVOCATION_FIELDS),
            "skill-invocation",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_i64(
            object,
            "schema_version",
            2,
            "skill invocation schema_version must be 2",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        expect_string_value(
            object,
            "event_type",
            "skill_invocation",
            "invalid skill invocation event_type",
        )
        .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        if object
            .get("source_protocol")
            .and_then(Value::as_str)
            .is_none_or(|value| value.is_empty())
        {
            bail!(
                "{}:{line_number}: source_protocol must be a non-empty string",
                display_path(path)
            );
        }
        if let Some(usage) = object.get("usage") {
            validate_usage(usage)
                .with_context(|| format!("{}:{line_number}", display_path(path)))?;
        }
        count += 1;
    }
    Ok(count)
}

fn validate_runtime_ignores_and_trace_paths(repo: &Path) -> Result<()> {
    let gitignore = read_to_string(&repo.join(".gitignore"))?;
    if !gitignore.contains(".harness-kit/traces/*.jsonl") {
        bail!(".gitignore must ignore runtime JSONL traces");
    }
    for relative in [
        "skills/trace/SKILL.md",
        "crates/harness-kit-checks/src/trace_record.rs",
    ] {
        if !read_to_string(&repo.join(relative))?.contains(".harness-kit/traces/work-records.jsonl")
        {
            bail!("{relative}: must name the work-record JSONL store");
        }
    }
    if !gitignore.contains(".harness-kit/work/*.jsonl") {
        bail!(".gitignore must ignore runtime work-ledger JSONL");
    }
    if !gitignore.contains(".harness-kit/tmp/lane-harness/") {
        bail!(".gitignore must ignore lane-harness runtime projections");
    }
    if !read_to_string(&repo.join("crates/harness-kit-checks/src/work_ledger.rs"))?
        .contains(".harness-kit/work/ledger.jsonl")
    {
        bail!("crates/harness-kit-checks/src/work_ledger.rs must name the work-ledger JSONL store");
    }
    let present: Vec<_> = [
        ".harness-kit/auth",
        ".harness-kit/sessions",
        ".harness-kit/provider-sessions",
        ".harness-kit/raw-transcripts",
    ]
    .into_iter()
    .filter(|relative| repo.join(relative).exists())
    .collect();
    if !present.is_empty() {
        bail!(
            "forbidden provider runtime directories: {}",
            present.join(", ")
        );
    }
    Ok(())
}

fn validate_helper_output(
    repo: &Path,
    command: &[&str],
    expected: &str,
    failure_prefix: &str,
) -> Result<()> {
    let Some((program, args)) = command.split_first() else {
        bail!("empty helper command");
    };
    let output = Command::new(program)
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("{failure_prefix}: command failed to start"))?;
    if !output.status.success() || !String::from_utf8_lossy(&output.stdout).contains(expected) {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let detail = stderr
            .lines()
            .chain(stdout.lines())
            .rfind(|line| !line.trim().is_empty())
            .unwrap_or("");
        if detail.is_empty() {
            bail!("{failure_prefix}");
        }
        bail!("{failure_prefix}: {detail}");
    }
    Ok(())
}

fn validate_rust_summary_helper(path: &Path) -> Result<()> {
    let summary = summarize_delegations::summarize_receipts(path, "")
        .context("harness-kit-checks summarize-delegations: report helper failed")?;
    let report = summarize_delegations::format_text(&summary);
    if !report.contains("Roster delegation report") {
        bail!("harness-kit-checks summarize-delegations: report helper failed");
    }
    Ok(())
}

fn validate_rust_probe_helper(path: &Path) -> Result<()> {
    let roster = agent_roster::load_roster(path)
        .context("harness-kit-checks probe-agent-roster: roster helper failed")?;
    agent_roster::validate_roster(&roster)
        .context("harness-kit-checks probe-agent-roster: roster helper failed")?;
    let receipts = agent_roster::build_probe_receipts(
        &roster,
        Some(""),
        "codex",
        "harness-kit-checks",
        ".harness-kit/agents.yaml",
        "probe helper fixture",
        "",
    )
    .context("harness-kit-checks probe-agent-roster: roster helper failed")?;
    if receipts.len() != agent_roster::ROSTER_PROVIDER_IDS.len() {
        bail!("harness-kit-checks probe-agent-roster: roster helper failed");
    }
    Ok(())
}

fn validate_rust_dispatch_helper(path: &Path) -> Result<()> {
    let roster = agent_roster::load_roster(path)
        .context("harness-kit-checks dispatch-agent: dispatch helper failed")?;
    let temp_dir = std::env::temp_dir().join(format!(
        "harness-kit-dispatch-check-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&temp_dir)
        .with_context(|| format!("failed to create {}", temp_dir.display()))?;
    let receipt_output = temp_dir.join("delegations.jsonl");
    let transcript_dir = temp_dir.join("traces");
    let receipt = agent_roster::dispatch_provider_lane(
        &roster,
        "codex",
        "dispatch helper fixture",
        agent_roster::DispatchRequest {
            objective: "dispatch helper fixture",
            input_ref: ".harness-kit/agents.yaml",
            transcript_dir: &transcript_dir,
            receipt_output: &receipt_output,
            timeout_s: 1.0,
            grace_s: 0.1,
            lead_harness: "codex",
            lead_provider: "harness-kit-checks",
            backlog_ref: "",
            path_env: Some(""),
            model_override: None,
            lane_harness: None,
            expect_output: None,
            repo_root: &temp_dir,
        },
    )
    .context("harness-kit-checks dispatch-agent: dispatch helper failed")?;
    let _ = fs::remove_dir_all(&temp_dir);
    if receipt.get("provider_status").and_then(Value::as_str) != Some("unavailable")
        || receipt.get("attempt_status").and_then(Value::as_str) != Some("failed")
    {
        bail!("harness-kit-checks dispatch-agent: dispatch helper failed");
    }
    Ok(())
}

fn validate_lane_harness_fixture(repo: &Path, path: &Path) -> Result<()> {
    let roster = agent_roster::load_roster(&repo.join(".harness-kit/agents.yaml"))
        .context(".harness-kit/examples/lane-harness.yaml: roster load failed")?;
    lane_harness::validate_manifest_path(repo, &roster, path)
        .context(".harness-kit/examples/lane-harness.yaml: manifest validation failed")?;
    let temp_dir = repo
        .join(".harness-kit/tmp/lane-harness")
        .join(format!("check-{}", uuid::Uuid::new_v4().simple()));
    let report = lane_harness::materialize_manifest(repo, &roster, path, Some(&temp_dir))
        .context(".harness-kit/examples/lane-harness.yaml: materialization failed")?;
    let codex_skill_root = report.root.join(".codex/skills");
    if !codex_skill_root.join("ci/SKILL.md").exists() {
        let _ = fs::remove_dir_all(&temp_dir);
        bail!(".harness-kit/examples/lane-harness.yaml: projected ci skill missing");
    }
    if codex_skill_root.join("shape").exists() || codex_skill_root.join("groom").exists() {
        let _ = fs::remove_dir_all(&temp_dir);
        bail!(".harness-kit/examples/lane-harness.yaml: projected root leaked excluded skills");
    }
    let _ = fs::remove_dir_all(&temp_dir);
    Ok(())
}

fn read_jsonl(path: &Path) -> Result<Vec<(usize, Value)>> {
    let text = read_to_string(path)?;
    let mut rows = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)
            .with_context(|| format!("{}:{line_number}: invalid JSON", display_path(path)))?;
        rows.push((line_number, value));
    }
    Ok(rows)
}

fn object<'a>(value: &'a Value, error: &str) -> Result<&'a serde_json::Map<String, Value>> {
    value.as_object().ok_or_else(|| anyhow!(error.to_string()))
}

fn validate_field_set(
    object: &serde_json::Map<String, Value>,
    required: BTreeSet<&str>,
    optional: BTreeSet<&str>,
    label: &str,
) -> Result<()> {
    let actual: BTreeSet<&str> = object.keys().map(String::as_str).collect();
    let missing: Vec<_> = required.difference(&actual).copied().collect();
    let allowed: BTreeSet<_> = required.union(&optional).copied().collect();
    let extra: Vec<_> = actual.difference(&allowed).copied().collect();
    if !missing.is_empty() {
        bail!("missing {label} fields: {missing:?}");
    }
    if !extra.is_empty() {
        bail!("unknown {label} fields: {extra:?}");
    }
    Ok(())
}

fn required_receipt_fields() -> BTreeSet<&'static str> {
    set(&[
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
    ])
}

fn optional_receipt_fields() -> BTreeSet<&'static str> {
    set(&[
        "model_id",
        "duration_ms",
        "usage",
        "transcript_bytes",
        "output_check",
        "lane_harness_ref",
        "lane_harness_sha256",
        "projection_status",
        "failure_kind",
        "work_source_refs",
    ])
}

fn set<'a>(items: &'a [&'a str]) -> BTreeSet<&'a str> {
    items.iter().copied().collect()
}

fn expect_i64(
    object: &serde_json::Map<String, Value>,
    field: &str,
    expected: i64,
    error: &str,
) -> Result<()> {
    if object.get(field).and_then(Value::as_i64) != Some(expected) {
        bail!("{error}");
    }
    Ok(())
}

fn expect_uuid_like(
    object: &serde_json::Map<String, Value>,
    field: &str,
    error: &str,
) -> Result<()> {
    let pattern = Regex::new(
        r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$",
    )
    .expect("static regex compiles");
    if object
        .get(field)
        .and_then(Value::as_str)
        .is_none_or(|value| !pattern.is_match(value))
    {
        bail!("{error}");
    }
    Ok(())
}

fn expect_string_list(
    object: &serde_json::Map<String, Value>,
    field: &str,
    error: &str,
) -> Result<()> {
    let Some(values) = object.get(field).and_then(Value::as_array) else {
        bail!("{error}");
    };
    if values.iter().any(|value| !value.is_string()) {
        bail!("{error}");
    }
    Ok(())
}

fn expect_enum(
    object: &serde_json::Map<String, Value>,
    field: &str,
    valid: &[&str],
    error: &str,
) -> Result<()> {
    if object
        .get(field)
        .and_then(Value::as_str)
        .is_none_or(|value| !valid.contains(&value))
    {
        bail!("{error}");
    }
    Ok(())
}

fn expect_string_value(
    object: &serde_json::Map<String, Value>,
    field: &str,
    expected: &str,
    error: &str,
) -> Result<()> {
    if object.get(field).and_then(Value::as_str) != Some(expected) {
        bail!("{error}");
    }
    Ok(())
}

fn expect_prefix(
    object: &serde_json::Map<String, Value>,
    field: &str,
    prefix: &str,
    error: &str,
) -> Result<()> {
    if object
        .get(field)
        .and_then(Value::as_str)
        .is_none_or(|value| !value.starts_with(prefix))
    {
        bail!("{error}");
    }
    Ok(())
}

fn expect_array(object: &serde_json::Map<String, Value>, field: &str, error: &str) -> Result<()> {
    if !object.get(field).is_some_and(Value::is_array) {
        bail!("{error}");
    }
    Ok(())
}

fn array_len(object: &serde_json::Map<String, Value>, field: &str) -> Option<usize> {
    object.get(field).and_then(Value::as_array).map(Vec::len)
}

fn validate_optional_text(object: &serde_json::Map<String, Value>, field: &str) -> Result<()> {
    let Some(value) = object.get(field) else {
        return Ok(());
    };
    if value.is_null() {
        return Ok(());
    }
    let Some(text) = value.as_str() else {
        bail!("receipt {field} must be a non-empty string or null.");
    };
    if text.trim().is_empty() {
        bail!("receipt {field} must be a non-empty string or null.");
    }
    validate_no_secret_like_text(text, &format!("receipt {field}"))?;
    Ok(())
}

fn validate_optional_sha256(object: &serde_json::Map<String, Value>, field: &str) -> Result<()> {
    let Some(value) = object.get(field) else {
        return Ok(());
    };
    let Some(text) = value.as_str() else {
        bail!("receipt {field} must be a sha256 hex string.");
    };
    let re = Regex::new(r"^[0-9a-f]{64}$").expect("static regex compiles");
    if !re.is_match(text) {
        bail!("receipt {field} must be a sha256 hex string.");
    }
    Ok(())
}

fn validate_optional_nonnegative_int(
    object: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<()> {
    let Some(value) = object.get(field) else {
        return Ok(());
    };
    if value.is_null() {
        return Ok(());
    }
    if value.as_u64().is_none() {
        bail!("receipt {field} must be a non-negative integer or null.");
    }
    Ok(())
}

fn validate_no_secret_like_values(value: &Value, path: &str) -> Result<()> {
    match value {
        Value::Object(object) => {
            for (key, child) in object {
                validate_no_secret_like_text(key, &format!("{path}.{key}"))?;
                validate_no_secret_like_values(child, &format!("{path}.{key}"))?;
            }
        }
        Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                validate_no_secret_like_values(child, &format!("{path}[{index}]"))?;
            }
        }
        Value::String(text) => validate_no_secret_like_text(text, path)?,
        _ => {}
    }
    Ok(())
}

fn validate_no_secret_like_text(text: &str, path: &str) -> Result<()> {
    if secret_regex().is_match(text) {
        bail!("secret-like value in work-record fixture at {path}");
    }
    Ok(())
}

fn secret_regex() -> Regex {
    Regex::new(
        r"(?i)(api[_-]?key|access[_-]?token|refresh[_-]?token|auth[_-]?token|secret|password|bearer|xai_api_key|exa_api_key|anthropic_api_key)",
    )
    .expect("static regex compiles")
}

fn shell_meta_regex() -> Regex {
    Regex::new(r"[;&|`$<>]").expect("static regex compiles")
}

fn read_to_string(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("failed to read {}", display_path(path)))
}

fn display_path(path: &Path) -> String {
    path.strip_prefix(std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .unwrap_or(path)
        .display()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn delegation_judgment_rejects_keyword_stuffing_without_commitment() {
        let weak_section = r#"## Delegation Judgment

When a provider roster is available, this section mentions native subagents
and cross-model and sprite. It also contains the words lane and receipt. A
separate sentence mentions context, give, and scope, plus the word lead.
These are reminders only; the primary may decide later whether native
delegation is the default or whether cross-model review is useful.
"#;

        let gaps = delegation_contract_gaps(weak_section);

        assert!(gaps.contains(&"native-first commitment".to_string()));
        assert!(gaps.contains(&"cross-model critic commitment".to_string()));
        assert!(gaps.contains(&"scoped lane handoff".to_string()));
        assert!(gaps.contains(&"lead-owned synthesis".to_string()));
    }

    #[test]
    fn delegation_judgment_rejects_hedged_commitments() {
        let hedged_section = r#"## Delegation Judgment

When a provider roster is available, the lead may treat native subagents as
the default if available. Cross-model critics might review the work at the
lead's discretion. Use lanes for review. Give them scoped context and
evidence. The lead agent owns synthesis. Sprite lanes exist.
"#;

        let gaps = delegation_contract_gaps(hedged_section);

        assert!(gaps.contains(&"native-first commitment".to_string()));
        assert!(gaps.contains(&"cross-model critic commitment".to_string()));
    }

    #[test]
    fn delegation_judgment_accepts_shared_roster_contract() {
        let section = r#"## Roster

This section is the single source for delegation judgment. There is no
provider quota (repo `.harness-kit/agents.yaml` or system roster).

- Native first. The harness's own subagents are the default delegation path
  for exploration, scoped builds, and review fan-out.
- Cross-model criticism is the strongest multi-provider case. A
  fresh-context critic on a different model family has decorrelated failure
  modes. Give critics ONLY the artifact (diff + oracle).
- Roster providers earn a lane when the card is bounded. Probe before
  dispatching; a probe is not a provider attempt.
- Sprites are substrate, not providers. Route heavy lanes to /sprites.
- Record meaningful roster and sprite lanes via receipts.

Provider output is evidence, not authority. The lead owns the result.
"#;

        assert!(delegation_contract_gaps(section).is_empty());
    }

    #[test]
    fn markdown_section_stops_at_next_h2() {
        let text = "# T\n\n## One\nbody\n### Nested\nx\n\n## Two\ny";

        assert_eq!(
            markdown_section(text, "## One"),
            "## One\nbody\n### Nested\nx\n"
        );
    }

    #[test]
    fn usage_requires_cost_source_when_cost_is_known() {
        let usage = serde_json::json!({"cost_usd": 0.1});

        let error = validate_usage(&usage).unwrap_err().to_string();

        assert!(error.contains("usage cost_source is required"));
    }

    #[test]
    fn receipt_accepts_work_source_refs_with_path_shaped_backlog_ref() {
        let receipt = serde_json::json!({
            "schema_version": 1,
            "delegation_id": "11111111-1111-4111-8111-111111111111",
            "created_at": "2026-06-09T00:00:00Z",
            "repo_root": "/tmp/harness-kit",
            "worktree_id": "wt",
            "lead_harness": "codex",
            "lead_provider": "codex",
            "backlog_ref": "backlog.d/062-agent-provider-roster.md",
            "objective": "fixture",
            "input_ref": "backlog.d/062-agent-provider-roster.md",
            "provider_target": "codex",
            "provider_status": "available",
            "attempt_status": "succeeded",
            "evidence_refs": ["receipt-062"],
            "summary": "done",
            "lead_verdict": "accepted",
            "redactions_applied": [],
            "work_source_refs": [{
                "role": "backlog",
                "kind": "local_backlog",
                "id": "062",
                "uri": "backlog.d/062-agent-provider-roster.md"
            }]
        });

        assert!(validate_receipt_record(&receipt).is_ok());
    }

    #[test]
    fn receipt_rejects_contradictory_local_backlog_work_source_ref() {
        let receipt = serde_json::json!({
            "schema_version": 1,
            "delegation_id": "11111111-1111-4111-8111-111111111111",
            "created_at": "2026-06-09T00:00:00Z",
            "repo_root": "/tmp/harness-kit",
            "worktree_id": "wt",
            "lead_harness": "codex",
            "lead_provider": "codex",
            "backlog_ref": "backlog.d/062-agent-provider-roster.md",
            "objective": "fixture",
            "input_ref": "backlog.d/062-agent-provider-roster.md",
            "provider_target": "codex",
            "provider_status": "available",
            "attempt_status": "succeeded",
            "evidence_refs": ["receipt-062"],
            "summary": "done",
            "lead_verdict": "accepted",
            "redactions_applied": [],
            "work_source_refs": [{
                "role": "backlog",
                "kind": "local_backlog",
                "id": "063"
            }]
        });

        let error = validate_receipt_record(&receipt).unwrap_err().to_string();

        assert!(error.contains("work_source_refs"));
    }

    #[test]
    fn work_record_gate_accepts_optional_work_source_refs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("work-records.jsonl");
        fs::write(
            &path,
            serde_json::json!({
                "schema_version": 1,
                "record_type": "agent-session-trace",
                "trace_id": "trace-11111111-1111-4111-8111-111111111111",
                "created_at": "2026-06-09T00:00:00Z",
                "backlog_ref": "backlog.d/056-agent-session-trace-lifecycle.md",
                "spec_ref": "backlog.d/056-agent-session-trace-lifecycle.md",
                "branch": "deliver/056-agent-session-trace-lifecycle",
                "commits": [],
                "reviewer_verdict_refs": [],
                "qa_refs": [],
                "demo_refs": [],
                "transcript_refs": [],
                "shipped_ref": "",
                "waiver_reason": "fixture waiver",
                "metadata": {},
                "work_source_refs": [{
                    "role": "backlog",
                    "kind": "local_backlog",
                    "id": "056",
                    "uri": "backlog.d/056-agent-session-trace-lifecycle.md"
                }]
            })
            .to_string(),
        )
        .expect("write");

        assert_eq!(validate_work_records(&path).expect("valid records"), 1);
    }

    #[test]
    fn work_ledger_gate_accepts_optional_work_source_refs() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("work-ledger.jsonl");
        fs::write(
            &path,
            serde_json::json!({
                "schema_version": 1,
                "record_type": "work-ledger-event",
                "event_id": "work-11111111-1111-4111-8111-111111111111",
                "created_at": "2026-06-09T00:00:00Z",
                "event_type": "phase_started",
                "work_id": "work-056",
                "parent_work_id": "",
                "backlog_ref": "backlog.d/056-agent-session-trace-lifecycle.md",
                "branch": "deliver/056-agent-session-trace-lifecycle",
                "owning_skill": "deliver",
                "phase": "implement",
                "evidence_refs": [],
                "blockers": [],
                "spawned_agents": [],
                "trace_refs": [],
                "next_action": "continue",
                "status": "active",
                "work_source_refs": [{
                    "role": "backlog",
                    "kind": "local_backlog",
                    "id": "056",
                    "uri": "backlog.d/056-agent-session-trace-lifecycle.md"
                }]
            })
            .to_string(),
        )
        .expect("write");

        assert_eq!(validate_work_ledger(&path).expect("valid ledger"), 1);
    }

    #[test]
    fn skill_invocation_requires_v2_protocol_fields() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("skill-invocations.jsonl");
        fs::write(
            &path,
            serde_json::json!({
                "schema_version": 1,
                "event_type": "skill_invocation",
                "ts": "2026-06-04T00:00:00Z",
                "harness": "claude",
                "source_protocol": "post_tool_use",
                "skill": "shape",
                "args": "",
                "session_id": "s",
                "cwd": "/tmp",
                "project": "p"
            })
            .to_string(),
        )
        .expect("write");

        let error = format!("{:#}", validate_skill_invocations(&path).unwrap_err());

        assert!(
            error.contains("schema_version must be 2"),
            "unexpected error: {error}"
        );
    }
}
