use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Days, NaiveDate, SecondsFormat, Utc};
use serde_yaml::{Mapping, Value};

const PLACEHOLDERS: &[&str] = &[
    "",
    "tbd",
    "todo",
    "n/a",
    "na",
    "none",
    "placeholder",
    "unknown",
];
const REQUIRED_TOP_LEVEL: &[&str] = &[
    "version",
    "generated_at",
    "profile",
    "gates",
    "adr_policy",
    "infrastructure",
    "module_boundaries",
    "mock_policy",
    "observability",
    "readiness_state",
    "waivers",
];
const READINESS_STATES: &[&str] = &["unknown", "improved", "preserved", "regressed"];
const FEEDBACK_STRENGTH: &[&str] = &["unknown", "weak", "moderate", "strong", "strict"];
const INFRA_MANAGEABILITY: &[&str] = &["unknown", "human_only", "cli_api_sdk", "mixed"];
const OBSERVABILITY_ACCESS: &[&str] = &["unknown", "none", "logs", "metrics", "traces", "full"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileError {
    message: String,
}

impl ProfileError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for ProfileError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.message.fmt(formatter)
    }
}

impl std::error::Error for ProfileError {}

pub type ProfileResult<T> = std::result::Result<T, ProfileError>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateOptions {
    pub profile: PathBuf,
    pub repo_root: PathBuf,
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateOptions {
    pub profile: PathBuf,
    pub waiver_id: String,
    pub scope: String,
    pub reason: String,
    pub expires_on: String,
    pub adr: String,
    pub readiness_state: Option<String>,
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

pub fn default_profile_path() -> PathBuf {
    PathBuf::from(".harness-kit/agent-readiness.yaml")
}

pub fn create(options: &CreateOptions) -> ProfileResult<String> {
    if options.profile.exists() && !options.force {
        return Err(ProfileError::new(format!(
            "{}: already exists; use --force to overwrite",
            options.profile.display()
        )));
    }
    let repo_root = options
        .repo_root
        .canonicalize()
        .map_err(|error| ProfileError::new(error.to_string()))?;
    let data = default_profile(&repo_root);
    validate_profile(&data, None)?;
    write_profile(&options.profile, &data)?;
    Ok(format!("created {}", options.profile.display()))
}

pub fn read(path: &Path) -> ProfileResult<String> {
    let data = load_profile(path)?;
    validate_profile(&data, None)?;
    profile_summary(&data)
}

pub fn validate(path: &Path) -> ProfileResult<String> {
    let data = load_profile(path)?;
    validate_profile(&data, None)?;
    Ok(format!("{}: valid", path.display()))
}

pub fn update(options: &UpdateOptions) -> ProfileResult<String> {
    let mut data = load_profile(&options.profile)?;
    validate_profile(&data, None)?;
    let waiver = waiver_mapping(
        &options.waiver_id,
        &options.scope,
        &options.reason,
        &options.expires_on,
        &options.adr,
    );
    validate_waiver(&Value::Mapping(waiver.clone()), Utc::now().date_naive())?;
    replace_waiver(&mut data, &options.waiver_id, waiver)?;
    if let Some(readiness_state) = &options.readiness_state {
        mapping_mut(&mut data, "")?.insert(
            Value::String("readiness_state".to_string()),
            Value::String(readiness_state.clone()),
        );
    }
    validate_profile(&data, None)?;
    write_profile(&options.profile, &data)?;
    Ok(format!(
        "updated {}: waiver {}",
        options.profile.display(),
        options.waiver_id
    ))
}

pub fn delete(path: &Path, waiver_id: &str) -> ProfileResult<String> {
    let mut data = load_profile(path)?;
    validate_profile(&data, None)?;
    let waivers = require_sequence_mut(&mut data, "waivers")?;
    let before = waivers.len();
    waivers.retain(|entry| mapping_get(entry, "id").and_then(Value::as_str) != Some(waiver_id));
    if waivers.len() == before {
        return Err(ProfileError::new(format!("waiver not found: {waiver_id}")));
    }
    validate_profile(&data, None)?;
    write_profile(path, &data)?;
    Ok(format!(
        "deleted waiver {waiver_id} from {}",
        path.display()
    ))
}

pub fn default_profile(repo_root: &Path) -> Value {
    map([
        ("version", Value::Number(1.into())),
        ("generated_at", Value::String(now_iso())),
        (
            "profile",
            map([
                (
                    "repo_root",
                    Value::String(repo_root.display().to_string()),
                ),
                ("detected_stack", string_sequence(detect_stack(repo_root))),
                (
                    "stack_feedback_strength",
                    Value::String(infer_feedback_strength(repo_root).to_string()),
                ),
            ]),
        ),
        (
            "gates",
            map([
                ("local", string_sequence(detect_local_gates(repo_root))),
                (
                    "ci",
                    string_sequence(if repo_root.join("ci/src").exists() {
                        vec!["dagger call check --source=.".to_string()]
                    } else {
                        Vec::new()
                    }),
                ),
                (
                    "coverage",
                    map([
                        ("command", Value::String(String::new())),
                        ("threshold", Value::String(String::new())),
                    ]),
                ),
            ]),
        ),
        (
            "adr_policy",
            map([
                (
                    "required_when",
                    Value::String(
                        "Decision is hard to reverse, surprising without context, and the result of a real tradeoff."
                            .to_string(),
                    ),
                ),
                ("paths", string_sequence(vec!["docs/adr/".to_string()])),
            ]),
        ),
        (
            "infrastructure",
            map([
                ("manageability", Value::String("unknown".to_string())),
                ("surfaces", Value::Sequence(Vec::new())),
            ]),
        ),
        ("module_boundaries", Value::Sequence(Vec::new())),
        (
            "mock_policy",
            Value::String(
                "Mock only external boundaries; internal mocks are readiness regressions.".to_string(),
            ),
        ),
        (
            "observability",
            map([
                ("access", Value::String("unknown".to_string())),
                ("signals", Value::Sequence(Vec::new())),
            ]),
        ),
        ("readiness_state", Value::String("unknown".to_string())),
        ("waivers", Value::Sequence(Vec::new())),
    ])
}

pub fn load_profile(path: &Path) -> ProfileResult<Value> {
    if !path.exists() {
        return Err(ProfileError::new(format!(
            "{}: missing readiness profile",
            path.display()
        )));
    }
    let text = fs::read_to_string(path).map_err(|error| ProfileError::new(error.to_string()))?;
    let data: Value =
        serde_yaml::from_str(&text).map_err(|error| ProfileError::new(error.to_string()))?;
    if !data.is_mapping() {
        return Err(ProfileError::new(format!(
            "{}: profile must be a YAML mapping",
            path.display()
        )));
    }
    Ok(data)
}

pub fn write_profile(path: &Path, data: &Value) -> ProfileResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| ProfileError::new(error.to_string()))?;
    }
    let text = serde_yaml::to_string(data).map_err(|error| ProfileError::new(error.to_string()))?;
    fs::write(path, text).map_err(|error| ProfileError::new(error.to_string()))
}

pub fn validate_profile(data: &Value, today: Option<NaiveDate>) -> ProfileResult<()> {
    let root = data
        .as_mapping()
        .ok_or_else(|| ProfileError::new("profile must be a YAML mapping"))?;
    let actual: BTreeSet<_> = root.keys().filter_map(Value::as_str).collect();
    let required: BTreeSet<_> = REQUIRED_TOP_LEVEL.iter().copied().collect();
    let missing: Vec<_> = required.difference(&actual).copied().collect();
    let extra: Vec<_> = actual.difference(&required).copied().collect();
    if !missing.is_empty() {
        return Err(ProfileError::new(format!(
            "missing field(s): {}",
            py_list_repr(&missing)
        )));
    }
    if !extra.is_empty() {
        return Err(ProfileError::new(format!(
            "unknown field(s): {}",
            py_list_repr(&extra)
        )));
    }
    if mapping_get(data, "version").and_then(Value::as_i64) != Some(1) {
        return Err(ProfileError::new("version must be 1"));
    }

    let profile = require_mapping(data, "profile")?;
    if is_placeholder(profile.get(Value::String("repo_root".to_string()))) {
        return Err(ProfileError::new("profile.repo_root must be set"));
    }
    match profile.get(Value::String("detected_stack".to_string())) {
        Some(Value::Sequence(values)) if !values.is_empty() => {}
        _ => {
            return Err(ProfileError::new(
                "profile.detected_stack must be a non-empty list",
            ));
        }
    }
    if !string_in(
        profile.get(Value::String("stack_feedback_strength".to_string())),
        FEEDBACK_STRENGTH,
    ) {
        return Err(ProfileError::new(
            "profile.stack_feedback_strength is invalid",
        ));
    }

    let gates = require_mapping(data, "gates")?;
    require_list_in_mapping(gates, "local")?;
    require_list_in_mapping(gates, "ci")?;
    let coverage = gates
        .get(Value::String("coverage".to_string()))
        .and_then(Value::as_mapping)
        .ok_or_else(|| ProfileError::new("gates.coverage must include command and threshold"))?;
    if !coverage.contains_key(Value::String("command".to_string()))
        || !coverage.contains_key(Value::String("threshold".to_string()))
    {
        return Err(ProfileError::new(
            "gates.coverage must include command and threshold",
        ));
    }

    let adr_policy = require_mapping(data, "adr_policy")?;
    if is_placeholder(adr_policy.get(Value::String("required_when".to_string()))) {
        return Err(ProfileError::new("adr_policy.required_when must be set"));
    }
    require_list_in_mapping(adr_policy, "paths")?;

    let infrastructure = require_mapping(data, "infrastructure")?;
    if !string_in(
        infrastructure.get(Value::String("manageability".to_string())),
        INFRA_MANAGEABILITY,
    ) {
        return Err(ProfileError::new("infrastructure.manageability is invalid"));
    }
    require_list_in_mapping(infrastructure, "surfaces")?;

    require_list(data, "module_boundaries")?;
    if is_placeholder(mapping_get(data, "mock_policy")) {
        return Err(ProfileError::new("mock_policy must be set"));
    }

    let observability = require_mapping(data, "observability")?;
    if !string_in(
        observability.get(Value::String("access".to_string())),
        OBSERVABILITY_ACCESS,
    ) {
        return Err(ProfileError::new("observability.access is invalid"));
    }
    require_list_in_mapping(observability, "signals")?;

    if !string_in(mapping_get(data, "readiness_state"), READINESS_STATES) {
        return Err(ProfileError::new("readiness_state is invalid"));
    }

    for waiver in require_list(data, "waivers")? {
        validate_waiver(waiver, today.unwrap_or_else(|| Utc::now().date_naive()))?;
    }
    Ok(())
}

pub fn profile_summary(data: &Value) -> ProfileResult<String> {
    let profile = require_mapping(data, "profile")?;
    let gates = require_mapping(data, "gates")?;
    let detected_stack = sequence_strings(
        profile
            .get(Value::String("detected_stack".to_string()))
            .ok_or_else(|| ProfileError::new("profile.detected_stack must be a non-empty list"))?,
    );
    let local_gates = sequence_strings(
        gates
            .get(Value::String("local".to_string()))
            .ok_or_else(|| ProfileError::new("gates.local: must be a list"))?,
    );
    let waivers = require_list(data, "waivers")?;
    Ok([
        "Agent readiness profile".to_string(),
        format!(
            "- repo_root: {}",
            value_string(profile.get(Value::String("repo_root".to_string()))).unwrap_or_default()
        ),
        format!("- detected_stack: {}", detected_stack.join(", ")),
        format!(
            "- stack_feedback_strength: {}",
            value_string(profile.get(Value::String("stack_feedback_strength".to_string())))
                .unwrap_or_default()
        ),
        format!(
            "- readiness_state: {}",
            value_string(mapping_get(data, "readiness_state")).unwrap_or_default()
        ),
        format!(
            "- local_gates: {}",
            if local_gates.is_empty() {
                "none".to_string()
            } else {
                local_gates.join(", ")
            }
        ),
        format!("- waivers: {}", waivers.len()),
    ]
    .join("\n"))
}

pub fn detect_stack(repo_root: &Path) -> Vec<String> {
    let checks = [
        ("node", "package.json"),
        ("python", "pyproject.toml"),
        ("python", "requirements.txt"),
        ("rust", "Cargo.toml"),
        ("go", "go.mod"),
        ("dagger", "ci/src"),
    ];
    let mut found = Vec::new();
    for (label, relative) in checks {
        if repo_root.join(relative).exists() && !found.iter().any(|item| item == label) {
            found.push(label.to_string());
        }
    }
    if found.is_empty() {
        vec!["unknown".to_string()]
    } else {
        found
    }
}

pub fn infer_feedback_strength(repo_root: &Path) -> &'static str {
    let strong_signals = [
        repo_root.join("ci/src"),
        repo_root.join(".githooks"),
        repo_root.join("crates/harness-kit-checks/src/check_agent_roster.rs"),
    ];
    let strict_signals = [
        repo_root.join("dagger.json"),
        repo_root.join("pyproject.toml"),
        repo_root.join("package-lock.json"),
        repo_root.join("bun.lock"),
        repo_root.join("Cargo.lock"),
    ];
    let score = strong_signals.iter().filter(|path| path.exists()).count()
        + strict_signals.iter().filter(|path| path.exists()).count();
    match score {
        0 => "unknown",
        1 => "moderate",
        2 | 3 => "strong",
        _ => "strict",
    }
}

pub fn detect_local_gates(repo_root: &Path) -> Vec<String> {
    let mut gates = Vec::new();
    if repo_root.join("ci/src").exists() {
        gates.push("dagger call check --source=.".to_string());
    }
    if repo_root.join("Makefile").exists() {
        gates.push("make test".to_string());
    }
    if repo_root.join("package.json").exists() {
        gates.push("npm test".to_string());
    }
    gates
}

fn validate_waiver(waiver: &Value, today: NaiveDate) -> ProfileResult<()> {
    if !waiver.is_mapping() {
        return Err(ProfileError::new("waiver must be a mapping"));
    }
    for field in ["id", "scope", "reason", "adr"] {
        if is_placeholder(mapping_get(waiver, field)) {
            return Err(ProfileError::new(format!(
                "waiver {field} must be a non-placeholder string"
            )));
        }
    }
    let expires_on = parse_expiry(mapping_get(waiver, "expires_on"))?;
    if expires_on <= today {
        return Err(ProfileError::new(format!(
            "waiver {}: expires_on must be in the future",
            value_string(mapping_get(waiver, "id")).unwrap_or_else(|| "<unknown>".to_string())
        )));
    }
    Ok(())
}

fn parse_expiry(value: Option<&Value>) -> ProfileResult<NaiveDate> {
    let Some(Value::String(value)) = value else {
        return Err(ProfileError::new(
            "waiver expires_on must be a YYYY-MM-DD string",
        ));
    };
    if value.trim().is_empty() {
        return Err(ProfileError::new(
            "waiver expires_on must be a YYYY-MM-DD string",
        ));
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map_err(|_| ProfileError::new(format!("waiver expires_on is not YYYY-MM-DD: {value}")))
}

fn replace_waiver(data: &mut Value, waiver_id: &str, waiver: Mapping) -> ProfileResult<()> {
    let waivers = require_sequence_mut(data, "waivers")?;
    waivers.retain(|entry| mapping_get(entry, "id").and_then(Value::as_str) != Some(waiver_id));
    waivers.push(Value::Mapping(waiver));
    waivers.sort_by(|left, right| {
        value_string(mapping_get(left, "id"))
            .unwrap_or_default()
            .cmp(&value_string(mapping_get(right, "id")).unwrap_or_default())
    });
    Ok(())
}

fn waiver_mapping(id: &str, scope: &str, reason: &str, expires_on: &str, adr: &str) -> Mapping {
    let mut mapping = Mapping::new();
    for (key, value) in [
        ("id", id),
        ("scope", scope),
        ("reason", reason),
        ("expires_on", expires_on),
        ("adr", adr),
    ] {
        mapping.insert(
            Value::String(key.to_string()),
            Value::String(value.to_string()),
        );
    }
    mapping
}

fn map<const N: usize>(entries: [(&str, Value); N]) -> Value {
    let mut mapping = Mapping::new();
    for (key, value) in entries {
        mapping.insert(Value::String(key.to_string()), value);
    }
    Value::Mapping(mapping)
}

fn string_sequence(values: Vec<String>) -> Value {
    Value::Sequence(values.into_iter().map(Value::String).collect())
}

fn mapping_get<'a>(data: &'a Value, key: &str) -> Option<&'a Value> {
    data.as_mapping()?.get(Value::String(key.to_string()))
}

fn mapping_mut<'a>(data: &'a mut Value, label: &str) -> ProfileResult<&'a mut Mapping> {
    data.as_mapping_mut()
        .ok_or_else(|| ProfileError::new(format!("{label}: must be a mapping")))
}

fn require_mapping<'a>(data: &'a Value, key: &str) -> ProfileResult<&'a Mapping> {
    mapping_get(data, key)
        .and_then(Value::as_mapping)
        .ok_or_else(|| ProfileError::new(format!("{key}: must be a mapping")))
}

fn require_list<'a>(data: &'a Value, key: &str) -> ProfileResult<&'a Vec<Value>> {
    mapping_get(data, key)
        .and_then(Value::as_sequence)
        .ok_or_else(|| ProfileError::new(format!("{key}: must be a list")))
}

fn require_sequence_mut<'a>(data: &'a mut Value, key: &str) -> ProfileResult<&'a mut Vec<Value>> {
    data.as_mapping_mut()
        .and_then(|mapping| mapping.get_mut(Value::String(key.to_string())))
        .and_then(Value::as_sequence_mut)
        .ok_or_else(|| ProfileError::new(format!("{key}: must be a list")))
}

fn require_list_in_mapping<'a>(data: &'a Mapping, key: &str) -> ProfileResult<&'a Vec<Value>> {
    data.get(Value::String(key.to_string()))
        .and_then(Value::as_sequence)
        .ok_or_else(|| ProfileError::new(format!("{key}: must be a list")))
}

fn is_placeholder(value: Option<&Value>) -> bool {
    let Some(Value::String(value)) = value else {
        return true;
    };
    PLACEHOLDERS.contains(&value.trim().to_lowercase().as_str())
}

fn string_in(value: Option<&Value>, allowed: &[&str]) -> bool {
    value
        .and_then(Value::as_str)
        .is_some_and(|value| allowed.contains(&value))
}

fn value_string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(ToString::to_string)
}

fn sequence_strings(value: &Value) -> Vec<String> {
    value
        .as_sequence()
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn py_list_repr(values: &[&str]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("'{}'", value.replace('\\', "\\\\").replace('\'', "\\'")))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

pub fn future_date(days: u64) -> String {
    Utc::now()
        .date_naive()
        .checked_add_days(Days::new(days))
        .unwrap_or_else(|| Utc::now().date_naive())
        .to_string()
}

pub fn parse_generated_at(value: &str) -> Result<DateTime<Utc>> {
    Ok(DateTime::parse_from_rfc3339(value)
        .with_context(|| format!("invalid generated_at: {value}"))?
        .with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crud_lifecycle_matches_shell_smoke() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(repo.join("ci/src")).unwrap();
        fs::write(repo.join("Cargo.toml"), "[workspace]\n").unwrap();
        let profile = temp.path().join("agent-readiness.yaml");

        assert_eq!(
            create(&CreateOptions {
                profile: profile.clone(),
                repo_root: repo.clone(),
                force: false,
            })
            .unwrap(),
            format!("created {}", profile.display())
        );
        assert_eq!(
            validate(&profile).unwrap(),
            format!("{}: valid", profile.display())
        );
        assert!(read(&profile).unwrap().contains("Agent readiness profile"));

        let expires_on = future_date(30);
        assert_eq!(
            update(&UpdateOptions {
                profile: profile.clone(),
                waiver_id: "waiver-1".to_string(),
                scope: "coverage".to_string(),
                reason: "coverage command is tracked in a follow-up ticket".to_string(),
                expires_on,
                adr: "not-required:temporary remediation waiver".to_string(),
                readiness_state: Some("preserved".to_string()),
            })
            .unwrap(),
            format!("updated {}: waiver waiver-1", profile.display())
        );
        let data = load_profile(&profile).unwrap();
        assert_eq!(
            mapping_get(&data, "readiness_state").and_then(Value::as_str),
            Some("preserved")
        );
        assert_eq!(
            delete(&profile, "waiver-1").unwrap(),
            format!("deleted waiver waiver-1 from {}", profile.display())
        );
        validate(&profile).unwrap();
    }

    #[test]
    fn expired_placeholder_waiver_fails() {
        let profile = expired_profile();
        let error = validate_profile(&profile, Some(NaiveDate::from_ymd_opt(2026, 6, 4).unwrap()))
            .unwrap_err();

        assert!(error.message().contains("non-placeholder") || error.message().contains("future"));
    }

    #[test]
    fn waiver_ids_are_replaced_and_sorted() {
        let temp = tempfile::tempdir().unwrap();
        let repo = temp.path().join("repo");
        fs::create_dir_all(&repo).unwrap();
        let profile = temp.path().join("agent-readiness.yaml");
        create(&CreateOptions {
            profile: profile.clone(),
            repo_root: repo,
            force: false,
        })
        .unwrap();
        for waiver_id in ["z", "a", "z"] {
            update(&UpdateOptions {
                profile: profile.clone(),
                waiver_id: waiver_id.to_string(),
                scope: "scope".to_string(),
                reason: format!("reason {waiver_id}"),
                expires_on: future_date(10),
                adr: "adr-present".to_string(),
                readiness_state: None,
            })
            .unwrap();
        }

        let data = load_profile(&profile).unwrap();
        let ids: Vec<_> = require_list(&data, "waivers")
            .unwrap()
            .iter()
            .filter_map(|waiver| mapping_get(waiver, "id").and_then(Value::as_str))
            .collect();

        assert_eq!(ids, vec!["a", "z"]);
    }

    #[test]
    fn missing_and_extra_top_level_messages_use_python_list_repr() {
        let mut data = expired_profile();
        mapping_mut(&mut data, "")
            .unwrap()
            .remove(Value::String("version".to_string()));
        mapping_mut(&mut data, "")
            .unwrap()
            .insert(Value::String("surprise".to_string()), Value::Null);

        let error = validate_profile(&data, Some(NaiveDate::from_ymd_opt(2026, 6, 4).unwrap()))
            .unwrap_err();

        assert_eq!(error.message(), "missing field(s): ['version']");
    }

    #[test]
    fn stack_and_gate_detection_are_ordered() {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("package.json"), "{}").unwrap();
        fs::write(temp.path().join("requirements.txt"), "").unwrap();
        fs::write(temp.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        fs::create_dir_all(temp.path().join("ci/src")).unwrap();

        assert_eq!(
            detect_stack(temp.path()),
            vec!["node", "python", "rust", "dagger"]
        );
        assert_eq!(
            detect_local_gates(temp.path()),
            vec!["dagger call check --source=.", "npm test"]
        );
    }

    #[test]
    fn feedback_strength_uses_rust_check_agent_roster_signal() {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir_all(temp.path().join("crates/harness-kit-checks/src")).unwrap();
        fs::write(
            temp.path()
                .join("crates/harness-kit-checks/src/check_agent_roster.rs"),
            "",
        )
        .unwrap();

        assert_eq!(infer_feedback_strength(temp.path()), "moderate");
    }

    fn expired_profile() -> Value {
        map([
            ("version", Value::Number(1.into())),
            (
                "generated_at",
                Value::String("2026-06-02T00:00:00Z".to_string()),
            ),
            (
                "profile",
                map([
                    ("repo_root", Value::String("/repo".to_string())),
                    (
                        "detected_stack",
                        string_sequence(vec!["python".to_string()]),
                    ),
                    (
                        "stack_feedback_strength",
                        Value::String("strict".to_string()),
                    ),
                ]),
            ),
            (
                "gates",
                map([
                    (
                        "local",
                        string_sequence(vec!["dagger call check --source=.".to_string()]),
                    ),
                    (
                        "ci",
                        string_sequence(vec!["dagger call check --source=.".to_string()]),
                    ),
                    (
                        "coverage",
                        map([
                            ("command", Value::String(String::new())),
                            ("threshold", Value::String(String::new())),
                        ]),
                    ),
                ]),
            ),
            (
                "adr_policy",
                map([
                    (
                        "required_when",
                        Value::String("hard to reverse".to_string()),
                    ),
                    ("paths", string_sequence(vec!["docs/adr/".to_string()])),
                ]),
            ),
            (
                "infrastructure",
                map([
                    ("manageability", Value::String("unknown".to_string())),
                    ("surfaces", Value::Sequence(Vec::new())),
                ]),
            ),
            ("module_boundaries", Value::Sequence(Vec::new())),
            (
                "mock_policy",
                Value::String("Mock only external boundaries.".to_string()),
            ),
            (
                "observability",
                map([
                    ("access", Value::String("unknown".to_string())),
                    ("signals", Value::Sequence(Vec::new())),
                ]),
            ),
            ("readiness_state", Value::String("unknown".to_string())),
            (
                "waivers",
                Value::Sequence(vec![Value::Mapping(waiver_mapping(
                    "waiver-expired",
                    "tests",
                    "TBD",
                    "2020-01-01",
                    "n/a",
                ))]),
            ),
        ])
    }
}
