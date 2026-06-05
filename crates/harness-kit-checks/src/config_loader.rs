use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use regex::Regex;
use serde_json::{Map as JsonMap, Value as JsonValue};
use serde_yaml::{Mapping as YamlMapping, Value as YamlValue};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadOptions {
    pub name: String,
    pub repo: PathBuf,
    pub config: Option<PathBuf>,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    Found(JsonValue),
    OptionalMissing,
    RequiredMissing { path: PathBuf, create_path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigError {
    message: String,
}

impl ConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ConfigError {}

pub fn load(options: &LoadOptions) -> Result<LoadOutcome, ConfigError> {
    validate_name(&options.name)?;
    let root = repo_root(&options.repo)?;
    let path = config_path(&options.name, &root, options.config.as_deref());
    if !path.exists() {
        if options.optional {
            return Ok(LoadOutcome::OptionalMissing);
        }
        return Ok(LoadOutcome::RequiredMissing {
            path,
            create_path: root
                .join(".harness-kit")
                .join(format!("{}.yaml", options.name)),
        });
    }
    let payload = load_yaml(&path)?;
    Ok(LoadOutcome::Found(validate(&options.name, &path, payload)?))
}

pub fn format_json(value: &JsonValue) -> Result<String> {
    serde_json::to_string(value).context("failed to serialize normalized config")
}

fn validate_name(name: &str) -> Result<(), ConfigError> {
    match name {
        "deploy" | "monitor" | "flywheel" => Ok(()),
        _ => Err(ConfigError::new(format!(
            "name must be one of: deploy, monitor, flywheel (got {name})"
        ))),
    }
}

fn repo_root(repo: &Path) -> Result<PathBuf, ConfigError> {
    let candidate = repo
        .canonicalize()
        .map_err(|_| ConfigError::new(format!("--repo path does not exist: {}", repo.display())))?;
    let output = Command::new("git")
        .arg("-C")
        .arg(&candidate)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .map_err(|error| ConfigError::new(format!("unable to run git: {error}")))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let detail = if detail.is_empty() {
            "not a git repository".to_string()
        } else {
            detail
        };
        return Err(ConfigError::new(format!(
            "unable to resolve repo root from {}: {detail}",
            candidate.display()
        )));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(PathBuf::from(stdout.trim()))
}

fn config_path(name: &str, root: &Path, explicit: Option<&Path>) -> PathBuf {
    if let Some(explicit) = explicit {
        return explicit
            .canonicalize()
            .unwrap_or_else(|_| absolutize(explicit));
    }
    root.join(".harness-kit").join(format!("{name}.yaml"))
}

fn absolutize(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn load_yaml(path: &Path) -> Result<YamlMapping, ConfigError> {
    let content = fs::read_to_string(path).map_err(|error| {
        ConfigError::new(format!("{}: unable to read file: {error}", path.display()))
    })?;
    let payload: YamlValue = serde_yaml::from_str(&content).map_err(|error| {
        ConfigError::new(format!("{}: YAML parse error: {error}", path.display()))
    })?;
    match payload {
        YamlValue::Null => Ok(YamlMapping::new()),
        YamlValue::Mapping(mapping) => Ok(mapping),
        _ => Err(ConfigError::new(format!(
            "{}: expected top-level mapping",
            path.display()
        ))),
    }
}

fn validate(name: &str, path: &Path, data: YamlMapping) -> Result<JsonValue, ConfigError> {
    let mut normalized = yaml_mapping_to_json_map(path, data)?;
    match name {
        "deploy" => validate_deploy(path, &mut normalized)?,
        "monitor" => validate_monitor(path, &mut normalized)?,
        "flywheel" => validate_flywheel(path, &mut normalized)?,
        _ => unreachable!("validated name"),
    }
    Ok(JsonValue::Object(normalized))
}

fn yaml_mapping_to_json_map(
    path: &Path,
    data: YamlMapping,
) -> Result<JsonMap<String, JsonValue>, ConfigError> {
    let mut map = JsonMap::new();
    for (key, value) in data {
        let Some(key) = key.as_str() else {
            return Err(ConfigError::new(format!(
                "{}: expected string keys in top-level mapping",
                path.display()
            )));
        };
        map.insert(key.to_string(), yaml_to_json(value));
    }
    Ok(map)
}

fn yaml_to_json(value: YamlValue) -> JsonValue {
    match value {
        YamlValue::Null => JsonValue::Null,
        YamlValue::Bool(value) => JsonValue::Bool(value),
        YamlValue::Number(number) => {
            if let Some(value) = number.as_i64() {
                JsonValue::from(value)
            } else if let Some(value) = number.as_u64() {
                JsonValue::from(value)
            } else if let Some(value) = number.as_f64() {
                JsonValue::from(value)
            } else {
                JsonValue::Null
            }
        }
        YamlValue::String(value) => JsonValue::String(value),
        YamlValue::Sequence(values) => {
            JsonValue::Array(values.into_iter().map(yaml_to_json).collect())
        }
        YamlValue::Mapping(mapping) => {
            let mut result = JsonMap::new();
            for (key, value) in mapping {
                if let Some(key) = key.as_str() {
                    result.insert(key.to_string(), yaml_to_json(value));
                }
            }
            JsonValue::Object(result)
        }
        YamlValue::Tagged(tagged) => yaml_to_json(tagged.value),
    }
}

fn validate_deploy(path: &Path, data: &mut JsonMap<String, JsonValue>) -> Result<(), ConfigError> {
    let allowed = set([
        "schema_version",
        "target",
        "app",
        "envs",
        "healthcheck",
        "rollback_grace_seconds",
        "idempotent",
        "deploy_cmd",
        "current_sha_cmd",
        "rollback_handle_cmd",
        "rollback_cmd",
    ]);
    unknown(path, data, &allowed, "<root>")?;
    require_schema(path, data)?;
    require(path, data, "target", "<root>")?;
    let target = data
        .get("target")
        .and_then(JsonValue::as_str)
        .unwrap_or_default();
    if !set([
        "fly",
        "vercel",
        "cloudflare",
        "aws",
        "s3",
        "docker",
        "k8s",
        "custom",
    ])
    .contains(target)
    {
        return Err(ConfigError::new(format!(
            "{}: target is unsupported: {target}",
            path.display()
        )));
    }
    for key in allowed.difference(&set([
        "schema_version",
        "envs",
        "rollback_grace_seconds",
        "idempotent",
    ])) {
        require_string(path, data, key, "<root>")?;
    }
    require_url(path, data, "healthcheck", "<root>")?;
    require_int_min(path, data, "rollback_grace_seconds", 1, "<root>")?;
    require_bool(path, data, "idempotent", "<root>")?;
    if target == "custom" {
        for key in [
            "deploy_cmd",
            "current_sha_cmd",
            "rollback_handle_cmd",
            "rollback_cmd",
        ] {
            require(path, data, key, "<root>")?;
        }
    }
    if let Some(envs) = data.get("envs") {
        let JsonValue::Object(envs) = envs else {
            return Err(ConfigError::new(format!(
                "{}: envs must be a non-empty mapping",
                path.display()
            )));
        };
        if envs.is_empty() {
            return Err(ConfigError::new(format!(
                "{}: envs must be a non-empty mapping",
                path.display()
            )));
        }
        let env_allowed = set([
            "app",
            "healthcheck",
            "rollback_grace_seconds",
            "require_ci_green",
        ]);
        for (env_name, env_cfg) in envs {
            if env_name.is_empty() {
                return Err(ConfigError::new(format!(
                    "{}: env names must be non-empty strings",
                    path.display()
                )));
            }
            let JsonValue::Object(env_cfg) = env_cfg else {
                return Err(ConfigError::new(format!(
                    "{}: envs.{env_name} must be a mapping",
                    path.display()
                )));
            };
            let where_ = format!("envs.{env_name}");
            unknown(path, env_cfg, &env_allowed, &where_)?;
            require_string(path, env_cfg, "app", &where_)?;
            require_url(path, env_cfg, "healthcheck", &where_)?;
            require_int_min(path, env_cfg, "rollback_grace_seconds", 1, &where_)?;
            require_bool(path, env_cfg, "require_ci_green", &where_)?;
        }
    }
    Ok(())
}

fn validate_monitor(path: &Path, data: &mut JsonMap<String, JsonValue>) -> Result<(), ConfigError> {
    let allowed = set([
        "schema_version",
        "grace_window",
        "poll_interval",
        "observability",
        "healthcheck",
        "signals",
    ]);
    unknown(path, data, &allowed, "<root>")?;
    require_schema(path, data)?;
    if !["observability", "healthcheck", "signals"]
        .iter()
        .any(|key| data.contains_key(*key))
    {
        return Err(ConfigError::new(format!(
            "{}: monitor config needs observability, healthcheck, or signals",
            path.display()
        )));
    }
    for field in ["grace_window", "poll_interval"] {
        require_string(path, data, field, "<root>")?;
        normalize_duration(data, field)?;
    }
    if let Some(healthcheck) = data.get("healthcheck") {
        let JsonValue::Object(healthcheck) = healthcheck else {
            return Err(ConfigError::new(format!(
                "{}: healthcheck must be a mapping",
                path.display()
            )));
        };
        unknown(
            path,
            healthcheck,
            &set(["url", "expected_status", "hard_fail_on_5xx"]),
            "healthcheck",
        )?;
        require(path, healthcheck, "url", "healthcheck")?;
        require_url(path, healthcheck, "url", "healthcheck")?;
        require_int_min(path, healthcheck, "expected_status", 100, "healthcheck")?;
        if healthcheck
            .get("expected_status")
            .and_then(JsonValue::as_i64)
            .unwrap_or(100)
            > 599
        {
            return Err(ConfigError::new(format!(
                "{}: healthcheck.expected_status must be <= 599",
                path.display()
            )));
        }
        require_bool(path, healthcheck, "hard_fail_on_5xx", "healthcheck")?;
    }
    if let Some(observability) = data.get("observability") {
        let JsonValue::Object(observability) = observability else {
            return Err(ConfigError::new(format!(
                "{}: observability must be a mapping",
                path.display()
            )));
        };
        let list_keys = set([
            "evidence_dirs",
            "local_logs",
            "benchmark_outputs",
            "release_smoke",
        ]);
        let allowed_obs = set([
            "delegation_receipts",
            "workflow_events",
            "evidence_dirs",
            "local_logs",
            "benchmark_outputs",
            "release_smoke",
            "analytics_coverage",
        ]);
        unknown(path, observability, &allowed_obs, "observability")?;
        for key in &list_keys {
            if let Some(value) = observability.get(*key)
                && !is_non_empty_string_array(value)
            {
                return Err(ConfigError::new(format!(
                    "{}: observability.{key} must be a list of strings",
                    path.display()
                )));
            }
        }
        for key in allowed_obs.difference(&list_keys) {
            require_string(path, observability, key, "observability")?;
        }
    }
    if let Some(signals) = data.get("signals") {
        let JsonValue::Array(signals) = signals else {
            return Err(ConfigError::new(format!(
                "{}: signals must be a non-empty list",
                path.display()
            )));
        };
        if signals.is_empty() {
            return Err(ConfigError::new(format!(
                "{}: signals must be a non-empty list",
                path.display()
            )));
        }
        let allowed_signal = set([
            "name",
            "source",
            "query",
            "url",
            "threshold",
            "jq",
            "hard_fail",
        ]);
        let signal_sources = set(["datadog", "prometheus", "grafana", "loki", "logs", "custom"]);
        for (index, signal) in signals.iter().enumerate() {
            let where_ = format!("signals.{index}");
            let JsonValue::Object(signal) = signal else {
                return Err(ConfigError::new(format!(
                    "{}: {where_} must be a mapping",
                    path.display()
                )));
            };
            unknown(path, signal, &allowed_signal, &where_)?;
            for key in ["name", "source", "threshold"] {
                require(path, signal, key, &where_)?;
            }
            if !signal.contains_key("query") && !signal.contains_key("url") {
                return Err(ConfigError::new(format!(
                    "{}: {where_} needs query or url",
                    path.display()
                )));
            }
            for key in allowed_signal.difference(&set(["hard_fail"])) {
                require_string(path, signal, key, &where_)?;
            }
            require_url(path, signal, "url", &where_)?;
            require_bool(path, signal, "hard_fail", &where_)?;
            let source = signal
                .get("source")
                .and_then(JsonValue::as_str)
                .unwrap_or_default();
            if !signal_sources.contains(source) {
                return Err(ConfigError::new(format!(
                    "{}: {where_}.source is unsupported: {source}",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

fn validate_flywheel(
    path: &Path,
    data: &mut JsonMap<String, JsonValue>,
) -> Result<(), ConfigError> {
    let allowed = set([
        "schema_version",
        "cadence",
        "max_cycles",
        "budget_tokens",
        "backlog_includes",
        "stop_on_monitor_alert",
        "stop_on_phase_failed",
        "stop_on_budget_exhausted",
    ]);
    unknown(path, data, &allowed, "<root>")?;
    require_schema(path, data)?;
    normalize_duration(data, "cadence")?;
    for key in ["max_cycles", "budget_tokens"] {
        require_int_min(path, data, key, 1, "<root>")?;
    }
    for key in [
        "stop_on_monitor_alert",
        "stop_on_phase_failed",
        "stop_on_budget_exhausted",
    ] {
        require_bool(path, data, key, "<root>")?;
    }
    if let Some(includes) = data.get("backlog_includes")
        && !is_non_empty_string_array(includes)
    {
        let message = if matches!(includes, JsonValue::Array(_)) {
            "backlog_includes items must be non-empty strings"
        } else {
            "backlog_includes must be a non-empty list"
        };
        return Err(ConfigError::new(format!("{}: {message}", path.display())));
    }
    Ok(())
}

fn set<const N: usize>(values: [&'static str; N]) -> BTreeSet<&'static str> {
    values.into_iter().collect()
}

fn unknown(
    path: &Path,
    data: &JsonMap<String, JsonValue>,
    allowed: &BTreeSet<&str>,
    where_: &str,
) -> Result<(), ConfigError> {
    let extra: Vec<_> = data
        .keys()
        .filter(|key| !allowed.contains(key.as_str()))
        .cloned()
        .collect();
    if !extra.is_empty() {
        return Err(ConfigError::new(format!(
            "{}: unknown key at {where_}: {}",
            path.display(),
            extra.join(", ")
        )));
    }
    Ok(())
}

fn require(
    path: &Path,
    data: &JsonMap<String, JsonValue>,
    key: &str,
    where_: &str,
) -> Result<(), ConfigError> {
    if !data.contains_key(key) {
        return Err(ConfigError::new(format!(
            "{}: missing required key at {where_}: {key}",
            path.display()
        )));
    }
    Ok(())
}

fn require_schema(path: &Path, data: &JsonMap<String, JsonValue>) -> Result<(), ConfigError> {
    require(path, data, "schema_version", "<root>")?;
    if data.get("schema_version").and_then(JsonValue::as_i64) != Some(1) {
        return Err(ConfigError::new(format!(
            "{}: schema_version must be 1",
            path.display()
        )));
    }
    Ok(())
}

fn require_string(
    path: &Path,
    data: &JsonMap<String, JsonValue>,
    key: &str,
    where_: &str,
) -> Result<(), ConfigError> {
    if let Some(value) = data.get(key)
        && !matches!(value, JsonValue::String(value) if !value.is_empty())
    {
        return Err(ConfigError::new(format!(
            "{}: {where_}.{key} must be a non-empty string",
            path.display()
        )));
    }
    Ok(())
}

fn require_bool(
    path: &Path,
    data: &JsonMap<String, JsonValue>,
    key: &str,
    where_: &str,
) -> Result<(), ConfigError> {
    if let Some(value) = data.get(key)
        && !value.is_boolean()
    {
        return Err(ConfigError::new(format!(
            "{}: {where_}.{key} must be boolean",
            path.display()
        )));
    }
    Ok(())
}

fn require_int_min(
    path: &Path,
    data: &JsonMap<String, JsonValue>,
    key: &str,
    minimum: i64,
    where_: &str,
) -> Result<(), ConfigError> {
    if let Some(value) = data.get(key)
        && value.as_i64().is_none_or(|value| value < minimum)
    {
        return Err(ConfigError::new(format!(
            "{}: {where_}.{key} must be integer >= {minimum}",
            path.display()
        )));
    }
    Ok(())
}

fn require_url(
    path: &Path,
    data: &JsonMap<String, JsonValue>,
    key: &str,
    where_: &str,
) -> Result<(), ConfigError> {
    require_string(path, data, key, where_)?;
    if let Some(value) = data.get(key).and_then(JsonValue::as_str)
        && !Regex::new(r"^https?://")
            .expect("static regex compiles")
            .is_match(value)
    {
        return Err(ConfigError::new(format!(
            "{}: {where_}.{key} must be an http(s) URL",
            path.display()
        )));
    }
    Ok(())
}

fn normalize_duration(
    data: &mut JsonMap<String, JsonValue>,
    field: &str,
) -> Result<(), ConfigError> {
    if let Some(value) = data.get(field) {
        let seconds = duration_seconds(field, value)?;
        data.insert(format!("{field}_seconds"), JsonValue::from(seconds));
    }
    Ok(())
}

fn duration_seconds(field: &str, value: &JsonValue) -> Result<i64, ConfigError> {
    let Some(value) = value.as_str() else {
        return Err(ConfigError::new(format!(
            "{field}: expected duration string like 30s, 5m, 1h"
        )));
    };
    let captures = Regex::new(r"^\s*(\d+)\s*([smhd])\s*$")
        .expect("static regex compiles")
        .captures(value)
        .ok_or_else(|| {
            ConfigError::new(format!(
                "{field}: invalid duration '{value}' (expected <number><unit>, units: s, m, h, d)"
            ))
        })?;
    let amount: i64 = captures[1].parse().expect("regex captured digits");
    let multiplier = match &captures[2] {
        "s" => 1,
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        _ => unreachable!("regex captured known unit"),
    };
    Ok(amount * multiplier)
}

fn is_non_empty_string_array(value: &JsonValue) -> bool {
    matches!(value, JsonValue::Array(values) if !values.is_empty() && values.iter().all(|item| matches!(item, JsonValue::String(item) if !item.is_empty())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn write(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, content).unwrap();
    }

    fn git_repo() -> TempDir {
        let temp = TempDir::new().unwrap();
        Command::new("git")
            .arg("init")
            .arg("-q")
            .current_dir(temp.path())
            .status()
            .unwrap();
        fs::create_dir_all(temp.path().join(".harness-kit")).unwrap();
        temp
    }

    fn load_in(repo: &Path, name: &str, optional: bool) -> Result<LoadOutcome, ConfigError> {
        load(&LoadOptions {
            name: name.to_string(),
            repo: repo.to_path_buf(),
            config: None,
            optional,
        })
    }

    #[test]
    fn config_loader_valid_deploy_with_envs() {
        let temp = git_repo();
        write(
            &temp.path().join(".harness-kit/deploy.yaml"),
            r#"schema_version: 1
target: custom
app: api
healthcheck: https://example.com/health
rollback_grace_seconds: 300
deploy_cmd: ./scripts/deploy.sh
current_sha_cmd: ./scripts/current-sha.sh
rollback_handle_cmd: ./scripts/current-release.sh
rollback_cmd: "./scripts/rollback.sh {{handle}}"
envs:
  prod:
    app: api-prod
    healthcheck: https://example.com/health
    rollback_grace_seconds: 300
    require_ci_green: true
"#,
        );
        let LoadOutcome::Found(value) = load_in(temp.path(), "deploy", false).unwrap() else {
            panic!("expected config");
        };
        assert_eq!(value["target"], "custom");
        assert_eq!(value["envs"]["prod"]["require_ci_green"], true);
    }

    #[test]
    fn config_loader_monitor_normalizes_durations() {
        let temp = git_repo();
        write(
            &temp.path().join(".harness-kit/monitor.yaml"),
            r#"schema_version: 1
grace_window: 5m
poll_interval: 30s
observability:
  delegation_receipts: .harness-kit/traces/delegations.jsonl
  workflow_events: .harness-kit/work/*.jsonl
  evidence_dirs: [".evidence", ".harness-kit/monitor"]
healthcheck:
  url: https://example.com/health
  expected_status: 200
signals:
  - name: error_rate
    source: datadog
    query: "sum:errors{service:api}.as_rate()"
    threshold: "> 0.01"
"#,
        );
        let LoadOutcome::Found(value) = load_in(temp.path(), "monitor", false).unwrap() else {
            panic!("expected config");
        };
        assert_eq!(value["grace_window_seconds"], 300);
        assert_eq!(value["poll_interval_seconds"], 30);
    }

    #[test]
    fn config_loader_flywheel_normalizes_cadence() {
        let temp = git_repo();
        write(
            &temp.path().join(".harness-kit/flywheel.yaml"),
            r#"schema_version: 1
cadence: 1h
max_cycles: 3
budget_tokens: 50000
backlog_includes:
  - "052"
stop_on_monitor_alert: true
stop_on_phase_failed: true
"#,
        );
        let LoadOutcome::Found(value) = load_in(temp.path(), "flywheel", false).unwrap() else {
            panic!("expected config");
        };
        assert_eq!(value["cadence_seconds"], 3600);
    }

    #[test]
    fn config_loader_optional_and_required_missing_match_exit_contract_inputs() {
        let temp = git_repo();
        assert_eq!(
            load_in(temp.path(), "deploy", true).unwrap(),
            LoadOutcome::OptionalMissing
        );
        let LoadOutcome::RequiredMissing { path, create_path } =
            load_in(temp.path(), "deploy", false).unwrap()
        else {
            panic!("expected missing");
        };
        assert!(path.ends_with(".harness-kit/deploy.yaml"));
        assert!(create_path.ends_with(".harness-kit/deploy.yaml"));
    }

    #[test]
    fn config_loader_unknown_key_is_rejected() {
        let temp = git_repo();
        write(
            &temp.path().join(".harness-kit/deploy.yaml"),
            r#"schema_version: 1
target: fly
app: api
unknown_field: true
"#,
        );
        let error = load_in(temp.path(), "deploy", false).unwrap_err();
        assert!(error.message().contains("unknown key"));
        assert!(error.message().contains("unknown_field"));
    }

    #[test]
    fn config_loader_bad_duration_reports_field_and_units() {
        let temp = git_repo();
        write(
            &temp.path().join(".harness-kit/monitor.yaml"),
            r#"schema_version: 1
grace_window: soon
healthcheck:
  url: https://example.com/health
"#,
        );
        let error = load_in(temp.path(), "monitor", false).unwrap_err();
        assert!(error.message().contains("grace_window"));
        assert!(error.message().contains("s, m, h, d"));
    }

    #[test]
    fn config_loader_duration_accepts_shell_regex_whitespace() {
        let temp = git_repo();
        write(
            &temp.path().join(".harness-kit/flywheel.yaml"),
            r#"schema_version: 1
cadence: " 5 m "
"#,
        );
        let LoadOutcome::Found(value) = load_in(temp.path(), "flywheel", false).unwrap() else {
            panic!("expected config");
        };
        assert_eq!(value["cadence_seconds"], 300);
    }

    #[test]
    fn config_loader_empty_yaml_becomes_mapping_then_fails_schema() {
        let temp = git_repo();
        write(&temp.path().join(".harness-kit/deploy.yaml"), "");
        let error = load_in(temp.path(), "deploy", false).unwrap_err();
        assert!(error.message().contains("missing required key"));
        assert!(error.message().contains("schema_version"));
    }

    #[test]
    fn config_loader_custom_deploy_requires_commands() {
        let temp = git_repo();
        write(
            &temp.path().join(".harness-kit/deploy.yaml"),
            r#"schema_version: 1
target: custom
app: api
"#,
        );
        let error = load_in(temp.path(), "deploy", false).unwrap_err();
        assert!(error.message().contains("deploy_cmd"));
    }
}
