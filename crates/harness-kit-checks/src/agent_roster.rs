use std::collections::BTreeSet;
use std::env;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use chrono::Utc;
use regex::Regex;
use serde_json::{Map, Value};
use serde_yaml::{Mapping, Value as YamlValue};
use uuid::Uuid;
use wait_timeout::ChildExt;

use crate::summarize_delegations::{self, ReceiptInput};

#[cfg(unix)]
use std::os::unix::process::{CommandExt, ExitStatusExt};

pub const ROSTER_PROVIDER_IDS: &[&str] = &[
    "codex",
    "claude",
    "pi",
    "agy",
    "cursor-agent",
    "grok-build",
    "manual",
];

const VALID_TIERS: &[&str] = &["primary", "conditional", "manual", "disabled"];
const VALID_KINDS: &[&str] = &["cli", "bench", "manual"];
const VALID_OUTPUTS: &[&str] = &["json", "stream-json", "text", "patch-ref", "manual-summary"];
const VALID_WORKTREE: &[&str] = &["required", "recommended", "not_applicable"];
const REQUIRED_PROVIDER_FIELDS: &[&str] = &[
    "tier",
    "kind",
    "probe",
    "dispatch",
    "output",
    "permissions",
    "worktree",
    "notes",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeOptions {
    pub roster: PathBuf,
    pub validate_only: bool,
    pub write_receipts: bool,
    pub path_env: Option<String>,
    pub receipt_output: PathBuf,
    pub lead_harness: String,
    pub lead_provider: String,
    pub input_ref: String,
    pub objective: String,
    pub backlog_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeReport {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DispatchOptions {
    pub roster: PathBuf,
    pub provider_target: String,
    pub objective: String,
    pub input_ref: String,
    pub prompt_file: PathBuf,
    pub backlog_ref: String,
    pub lead_harness: String,
    pub lead_provider: String,
    pub model_override: Option<String>,
    pub timeout_s: f64,
    pub grace_s: f64,
    pub max_prompt_bytes: u64,
    pub transcript_dir: PathBuf,
    pub receipt_output: PathBuf,
    pub path_env: Option<String>,
}

pub fn probe_roster(options: &ProbeOptions) -> Result<ProbeReport> {
    let roster = load_roster(&options.roster)?;
    validate_roster(&roster)?;
    if options.validate_only {
        return Ok(ProbeReport {
            lines: vec![format!("{}: roster valid", options.roster.display())],
        });
    }

    let receipts = build_probe_receipts(
        &roster,
        options.path_env.as_deref(),
        &options.lead_harness,
        &options.lead_provider,
        &options.input_ref,
        &options.objective,
        &options.backlog_ref,
    )?;
    let mut lines = Vec::with_capacity(receipts.len());
    for receipt in receipts {
        if options.write_receipts {
            summarize_delegations::append_receipt(&options.receipt_output, &receipt)?;
        }
        lines.push(serde_json::to_string(&Value::Object(receipt))?);
    }
    Ok(ProbeReport { lines })
}

pub fn dispatch_from_options(options: &DispatchOptions) -> Result<Map<String, Value>> {
    let metadata = fs::metadata(&options.prompt_file)
        .with_context(|| format!("failed to stat {}", options.prompt_file.display()))?;
    if metadata.len() > options.max_prompt_bytes {
        bail!(
            "--prompt-file exceeds --max-prompt-bytes ({})",
            options.max_prompt_bytes
        );
    }
    let prompt = fs::read_to_string(&options.prompt_file)
        .with_context(|| format!("failed to read {}", options.prompt_file.display()))?;
    let roster = load_roster(&options.roster)?;
    dispatch_provider_lane(
        &roster,
        &options.provider_target,
        &prompt,
        DispatchRequest {
            objective: &options.objective,
            input_ref: &options.input_ref,
            transcript_dir: &options.transcript_dir,
            receipt_output: &options.receipt_output,
            timeout_s: options.timeout_s,
            grace_s: options.grace_s,
            lead_harness: &options.lead_harness,
            lead_provider: &options.lead_provider,
            backlog_ref: &options.backlog_ref,
            path_env: options.path_env.as_deref(),
            model_override: options.model_override.as_deref(),
        },
    )
}

pub struct DispatchRequest<'a> {
    pub objective: &'a str,
    pub input_ref: &'a str,
    pub transcript_dir: &'a Path,
    pub receipt_output: &'a Path,
    pub timeout_s: f64,
    pub grace_s: f64,
    pub lead_harness: &'a str,
    pub lead_provider: &'a str,
    pub backlog_ref: &'a str,
    pub path_env: Option<&'a str>,
    pub model_override: Option<&'a str>,
}

pub fn dispatch_provider_lane(
    roster: &YamlValue,
    provider_target: &str,
    prompt: &str,
    request: DispatchRequest<'_>,
) -> Result<Map<String, Value>> {
    validate_roster(roster)?;
    let providers = roster
        .as_mapping()
        .and_then(|root| root.get(YamlValue::String("providers".to_string())))
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| anyhow!("roster must define providers mapping."))?;
    let provider = providers
        .get(YamlValue::String(provider_target.to_string()))
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| anyhow!("unknown provider target: {provider_target}"))?;
    let worktree_id = current_worktree_id();

    if required_string(provider_target, provider, "kind")? == "manual" {
        let receipt = summarize_delegations::build_attempt_receipt(ReceiptInput {
            provider_target: provider_target.to_string(),
            provider_status: "manual".to_string(),
            attempt_status: "manual".to_string(),
            objective: request.objective.to_string(),
            input_ref: request.input_ref.to_string(),
            evidence_refs: Vec::new(),
            lead_verdict: "reference_only".to_string(),
            worktree_id,
            backlog_ref: request.backlog_ref.to_string(),
            lead_harness: request.lead_harness.to_string(),
            lead_provider: request.lead_provider.to_string(),
            summary: "manual provider cannot be dispatched by CLI".to_string(),
            model_id: None,
            duration_ms: None,
            usage: None,
            transcript_bytes: None,
        })?;
        summarize_delegations::append_receipt(request.receipt_output, &receipt)?;
        return Ok(receipt);
    }

    let provider_status = probe_status(provider_target, provider, request.path_env);
    if provider_status != "available" {
        let receipt = summarize_delegations::build_attempt_receipt(ReceiptInput {
            provider_target: provider_target.to_string(),
            provider_status: provider_status.clone(),
            attempt_status: "failed".to_string(),
            objective: request.objective.to_string(),
            input_ref: request.input_ref.to_string(),
            evidence_refs: Vec::new(),
            lead_verdict: "rejected".to_string(),
            worktree_id,
            backlog_ref: request.backlog_ref.to_string(),
            lead_harness: request.lead_harness.to_string(),
            lead_provider: request.lead_provider.to_string(),
            summary: format!("provider probe was {provider_status}; dispatch skipped"),
            model_id: None,
            duration_ms: None,
            usage: None,
            transcript_bytes: None,
        })?;
        summarize_delegations::append_receipt(request.receipt_output, &receipt)?;
        return Ok(receipt);
    }

    let command = dispatch_command(provider_target, provider, prompt, request.model_override)?;
    let transcript = transcript_path(request.transcript_dir, provider_target);
    if let Some(parent) = transcript.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let output = File::create(&transcript)
        .with_context(|| format!("failed to create {}", transcript.display()))?;
    let stderr = output
        .try_clone()
        .with_context(|| format!("failed to clone {}", transcript.display()))?;
    let mut process = Command::new(&command[0]);
    process
        .args(&command[1..])
        .stdin(Stdio::null())
        .stdout(Stdio::from(output))
        .stderr(Stdio::from(stderr));
    if let Some(path_env) = request.path_env {
        process.env("PATH", path_env);
    }
    #[cfg(unix)]
    {
        process.process_group(0);
    }

    let started_at = Instant::now();
    let mut child = process
        .spawn()
        .with_context(|| format!("failed to start provider command {}", command[0]))?;
    let timeout = Duration::from_secs_f64(request.timeout_s.max(0.0));
    let (timed_out, return_code, cleanup_note) = match child.wait_timeout(timeout)? {
        Some(status) => (false, exit_code(status), None),
        None => {
            let note = terminate_process_group(child.id(), request.grace_s);
            let status = child
                .wait_timeout(Duration::from_secs(1))?
                .map(exit_code)
                .unwrap_or(-libc::SIGKILL);
            (true, status, Some(note))
        }
    };
    let duration_ms = started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let transcript_bytes = fs::metadata(&transcript)
        .ok()
        .map(|metadata| metadata.len());

    let (provider_status, attempt_status, lead_verdict, summary) = if timed_out {
        (
            "error".to_string(),
            "failed".to_string(),
            "rejected".to_string(),
            format!(
                "provider dispatch timed out after {}s; {}",
                format_seconds(request.timeout_s),
                cleanup_note.unwrap_or_else(|| "process group killed".to_string())
            ),
        )
    } else if return_code == 0 {
        let mut summary = "provider dispatch exited 0".to_string();
        if let Some(model_override) = request.model_override {
            summary.push_str(&format!(
                "; model_override={}",
                resolve_model_override(provider, model_override)
            ));
        }
        (
            "available".to_string(),
            "succeeded".to_string(),
            "pending".to_string(),
            summary,
        )
    } else {
        (
            "available".to_string(),
            "failed".to_string(),
            "rejected".to_string(),
            format!("provider dispatch exited {return_code}"),
        )
    };

    let receipt = summarize_delegations::build_attempt_receipt(ReceiptInput {
        provider_target: provider_target.to_string(),
        provider_status,
        attempt_status,
        objective: request.objective.to_string(),
        input_ref: request.input_ref.to_string(),
        evidence_refs: vec![transcript.display().to_string()],
        lead_verdict,
        worktree_id,
        backlog_ref: request.backlog_ref.to_string(),
        lead_harness: request.lead_harness.to_string(),
        lead_provider: request.lead_provider.to_string(),
        summary,
        model_id: receipt_model_id(provider, request.model_override),
        duration_ms: Some(duration_ms),
        usage: None,
        transcript_bytes,
    })?;
    summarize_delegations::append_receipt(request.receipt_output, &receipt)?;
    Ok(receipt)
}

pub fn load_roster(path: &Path) -> Result<YamlValue> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read roster {}", path.display()))?;
    let mut value: YamlValue = serde_yaml::from_str(&text)
        .with_context(|| format!("failed to parse roster {}", path.display()))?;
    resolve_yaml_merges(&mut value);
    Ok(value)
}

fn resolve_yaml_merges(value: &mut YamlValue) {
    match value {
        YamlValue::Mapping(mapping) => {
            for child in mapping.values_mut() {
                resolve_yaml_merges(child);
            }
            let merge_key = YamlValue::String("<<".to_string());
            let Some(merge_value) = mapping.remove(&merge_key) else {
                return;
            };
            let mut merged = Mapping::new();
            match merge_value {
                YamlValue::Mapping(base) => merge_mapping(&mut merged, base),
                YamlValue::Sequence(sequence) => {
                    for item in sequence {
                        if let YamlValue::Mapping(base) = item {
                            merge_mapping(&mut merged, base);
                        }
                    }
                }
                _ => {}
            }
            let explicit = std::mem::take(mapping);
            merge_mapping_override(&mut merged, explicit);
            *mapping = merged;
        }
        YamlValue::Sequence(sequence) => {
            for child in sequence {
                resolve_yaml_merges(child);
            }
        }
        _ => {}
    }
}

fn merge_mapping(target: &mut Mapping, mut source: Mapping) {
    for value in source.values_mut() {
        resolve_yaml_merges(value);
    }
    for (key, value) in source {
        target.entry(key).or_insert(value);
    }
}

fn merge_mapping_override(target: &mut Mapping, mut source: Mapping) {
    for value in source.values_mut() {
        resolve_yaml_merges(value);
    }
    for (key, value) in source {
        target.insert(key, value);
    }
}

pub fn validate_roster(roster: &YamlValue) -> Result<()> {
    let root = roster
        .as_mapping()
        .ok_or_else(|| anyhow!("roster must contain a YAML mapping."))?;
    if root
        .get(YamlValue::String("version".to_string()))
        .and_then(YamlValue::as_i64)
        != Some(1)
    {
        bail!("roster version must be 1.");
    }
    let providers = root
        .get(YamlValue::String("providers".to_string()))
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| anyhow!("roster must define providers mapping."))?;

    let expected: BTreeSet<_> = ROSTER_PROVIDER_IDS.iter().copied().collect();
    let actual: Result<BTreeSet<_>> = providers
        .keys()
        .map(|key| {
            key.as_str()
                .ok_or_else(|| anyhow!("roster provider ids must be strings."))
        })
        .collect();
    let actual = actual?;
    let missing: Vec<_> = expected.difference(&actual).copied().collect();
    let extra: Vec<_> = actual.difference(&expected).copied().collect();
    if !missing.is_empty() {
        bail!("roster missing providers: {}", missing.join(", "));
    }
    if !extra.is_empty() {
        bail!("roster contains unknown providers: {}", extra.join(", "));
    }

    for (provider_id, provider) in providers {
        let provider_id = provider_id
            .as_str()
            .ok_or_else(|| anyhow!("roster provider ids must be strings."))?;
        let provider = provider
            .as_mapping()
            .ok_or_else(|| anyhow!("{provider_id}: provider entry must be a mapping."))?;
        validate_provider(provider_id, provider)?;
    }
    Ok(())
}

pub fn build_probe_receipts(
    roster: &YamlValue,
    path_env: Option<&str>,
    lead_harness: &str,
    lead_provider: &str,
    input_ref: &str,
    objective: &str,
    backlog_ref: &str,
) -> Result<Vec<Map<String, Value>>> {
    validate_roster(roster)?;
    let providers = roster
        .as_mapping()
        .and_then(|root| root.get(YamlValue::String("providers".to_string())))
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| anyhow!("roster must define providers mapping."))?;
    let mut receipts = Vec::with_capacity(providers.len());
    for provider_id in ROSTER_PROVIDER_IDS {
        let provider = providers
            .get(YamlValue::String((*provider_id).to_string()))
            .and_then(YamlValue::as_mapping)
            .ok_or_else(|| anyhow!("roster missing providers: {provider_id}"))?;
        receipts.push(summarize_delegations::build_attempt_receipt(
            ReceiptInput {
                provider_target: (*provider_id).to_string(),
                provider_status: probe_status(provider_id, provider, path_env),
                attempt_status: "not_started".to_string(),
                objective: objective.to_string(),
                input_ref: input_ref.to_string(),
                evidence_refs: Vec::new(),
                lead_verdict: "pending".to_string(),
                worktree_id: current_worktree_id(),
                backlog_ref: backlog_ref.to_string(),
                lead_harness: lead_harness.to_string(),
                lead_provider: lead_provider.to_string(),
                summary: format!("probe: {provider_id}"),
                model_id: None,
                duration_ms: None,
                usage: None,
                transcript_bytes: None,
            },
        )?);
    }
    Ok(receipts)
}

fn validate_provider(provider_id: &str, provider: &Mapping) -> Result<()> {
    let present: BTreeSet<_> = provider.keys().filter_map(YamlValue::as_str).collect();
    let missing: Vec<_> = REQUIRED_PROVIDER_FIELDS
        .iter()
        .copied()
        .filter(|field| !present.contains(field))
        .collect();
    if !missing.is_empty() {
        bail!("{provider_id}: missing fields: {}", missing.join(", "));
    }
    validate_enum(provider_id, provider, "tier", VALID_TIERS)?;
    validate_enum(provider_id, provider, "kind", VALID_KINDS)?;
    validate_enum(provider_id, provider, "output", VALID_OUTPUTS)?;
    validate_enum(provider_id, provider, "worktree", VALID_WORKTREE)?;
    if provider_id == "manual"
        && (required_string(provider_id, provider, "kind")? != "manual"
            || required_string(provider_id, provider, "tier")? != "manual")
    {
        bail!("manual provider must use tier=manual and kind=manual.");
    }

    let secret_re = secret_re();
    let shell_meta_re = Regex::new(r"[;&|`$<>]").expect("static regex compiles");
    for field in ["probe", "dispatch", "permissions", "notes"] {
        let value = required_string(provider_id, provider, field)?;
        if secret_re.is_match(value) {
            bail!("{provider_id}: {field} contains secret-like text.");
        }
        if matches!(field, "probe" | "dispatch") && shell_meta_re.is_match(value) {
            bail!("{provider_id}: {field} contains shell metacharacters.");
        }
    }

    if let Some(variants) = provider.get(YamlValue::String("model_variants".to_string())) {
        let variants = variants
            .as_mapping()
            .ok_or_else(|| anyhow!("{provider_id}: model_variants must be a mapping."))?;
        for (name, model) in variants {
            let name = name.as_str().ok_or_else(|| {
                anyhow!("{provider_id}: model_variants keys must be non-empty strings.")
            })?;
            if name.trim().is_empty() {
                bail!("{provider_id}: model_variants keys must be non-empty strings.");
            }
            let model = model.as_str().ok_or_else(|| {
                anyhow!("{provider_id}: model_variants values must be non-empty strings.")
            })?;
            if model.trim().is_empty() {
                bail!("{provider_id}: model_variants values must be non-empty strings.");
            }
            if secret_re.is_match(model) {
                bail!("{provider_id}: model_variants contains secret-like text.");
            }
        }
    }
    Ok(())
}

fn validate_enum(
    provider_id: &str,
    provider: &Mapping,
    field: &str,
    valid_values: &[&str],
) -> Result<()> {
    let value = required_string(provider_id, provider, field)?;
    if !valid_values.contains(&value) {
        bail!(
            "{provider_id}: {field} must be one of: {}.",
            valid_values.join(", ")
        );
    }
    Ok(())
}

fn required_string<'a>(provider_id: &str, provider: &'a Mapping, field: &str) -> Result<&'a str> {
    let value = provider
        .get(YamlValue::String(field.to_string()))
        .and_then(YamlValue::as_str)
        .ok_or_else(|| anyhow!("{provider_id}: {field} must be a non-empty string."))?;
    if value.trim().is_empty() {
        bail!("{provider_id}: {field} must be a non-empty string.");
    }
    Ok(value)
}

fn probe_status(provider_id: &str, provider: &Mapping, path_env: Option<&str>) -> String {
    if provider_id == "manual"
        || required_string(provider_id, provider, "kind").is_ok_and(|kind| kind == "manual")
    {
        return "manual".to_string();
    }
    let probe = match required_string(provider_id, provider, "probe") {
        Ok(probe) => probe,
        Err(_) => return "error".to_string(),
    };
    let command = match shell_words::split(probe) {
        Ok(command) if !command.is_empty() => command,
        _ => return "error".to_string(),
    };
    let search_path = path_env
        .map(str::to_string)
        .or_else(|| env::var("PATH").ok())
        .unwrap_or_default();
    if !binary_available(&command[0], &search_path) {
        return "unavailable".to_string();
    }
    let mut child = match Command::new(&command[0])
        .args(&command[1..])
        .env("PATH", &search_path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return "error".to_string(),
    };
    match child.wait_timeout(Duration::from_secs(5)) {
        Ok(Some(status)) if status.success() => "available".to_string(),
        Ok(Some(_)) => "error".to_string(),
        Ok(None) => {
            let _ = child.kill();
            let _ = child.wait();
            "error".to_string()
        }
        Err(_) => "error".to_string(),
    }
}

fn resolve_model_override(provider: &Mapping, model_override: &str) -> String {
    provider
        .get(YamlValue::String("model_variants".to_string()))
        .and_then(YamlValue::as_mapping)
        .and_then(|variants| variants.get(YamlValue::String(model_override.to_string())))
        .and_then(YamlValue::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| model_override.to_string())
}

fn dispatch_command(
    provider_id: &str,
    provider: &Mapping,
    prompt: &str,
    model_override: Option<&str>,
) -> Result<Vec<String>> {
    let dispatch = required_string(provider_id, provider, "dispatch")?;
    let mut command = shell_words::split(dispatch)
        .with_context(|| format!("{provider_id}: dispatch command could not be parsed"))?;
    if command.is_empty() {
        bail!("{provider_id}: dispatch command could not be parsed");
    }
    if let Some(model_override) = model_override {
        let model = resolve_model_override(provider, model_override);
        let dispatch_model = model
            .strip_prefix("openrouter/")
            .unwrap_or(&model)
            .to_string();
        if let Some(model_index) = command.iter().position(|part| part == "--model") {
            if model_index == command.len() - 1 {
                bail!("dispatch command has --model without a value.");
            }
            command[model_index + 1] = dispatch_model;
        } else {
            command.extend(["--model".to_string(), dispatch_model]);
        }
    }
    command.push(prompt.to_string());
    Ok(command)
}

fn receipt_model_id(provider: &Mapping, model_override: Option<&str>) -> Option<String> {
    if let Some(model_override) = model_override {
        return Some(resolve_model_override(provider, model_override));
    }
    let dispatch = provider
        .get(YamlValue::String("dispatch".to_string()))
        .and_then(YamlValue::as_str)?;
    let command = shell_words::split(dispatch).ok()?;
    if let Some(model_index) = command.iter().position(|part| part == "--model")
        && model_index < command.len() - 1
    {
        return Some(command[model_index + 1].clone());
    }
    provider
        .get(YamlValue::String("model".to_string()))
        .and_then(YamlValue::as_str)
        .filter(|model| !model.trim().is_empty())
        .map(str::to_string)
}

fn transcript_path(transcript_dir: &Path, provider_target: &str) -> PathBuf {
    let stamp = Utc::now().format("%Y%m%dT%H%M%S.%6fZ");
    let safe_provider = Regex::new(r"[^A-Za-z0-9_.-]")
        .expect("static regex compiles")
        .replace_all(provider_target, "_");
    transcript_dir.join(format!("{stamp}-{safe_provider}-{}.txt", short_uuid()))
}

fn format_seconds(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn short_uuid() -> String {
    Uuid::new_v4().simple().to_string()[..8].to_string()
}

fn exit_code(status: std::process::ExitStatus) -> i32 {
    status.code().unwrap_or_else(|| {
        #[cfg(unix)]
        {
            -status.signal().unwrap_or(libc::SIGKILL)
        }
        #[cfg(not(unix))]
        {
            -1
        }
    })
}

#[cfg(unix)]
fn terminate_process_group(pid: u32, grace_s: f64) -> String {
    let pgid = pid as libc::pid_t;
    if unsafe { libc::kill(-pgid, libc::SIGTERM) } != 0 {
        return match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ESRCH) => "process group exited before SIGTERM".to_string(),
            Some(libc::EPERM) => "permission denied during SIGTERM cleanup".to_string(),
            _ => "process group exited before cleanup".to_string(),
        };
    }
    if grace_s > 0.0 {
        thread::sleep(Duration::from_secs_f64(grace_s));
    }
    if unsafe { libc::kill(-pgid, libc::SIGKILL) } != 0 {
        return match std::io::Error::last_os_error().raw_os_error() {
            Some(libc::ESRCH) => "process group exited after SIGTERM".to_string(),
            Some(libc::EPERM) => "permission denied during SIGKILL cleanup".to_string(),
            _ => "process group exited after SIGTERM".to_string(),
        };
    }
    "process group killed".to_string()
}

#[cfg(not(unix))]
fn terminate_process_group(_pid: u32, _grace_s: f64) -> String {
    "process group cleanup unsupported on this platform".to_string()
}

fn binary_available(binary: &str, search_path: &str) -> bool {
    let binary_path = Path::new(binary);
    if binary_path.components().count() > 1 {
        return is_executable(binary_path);
    }
    env::split_paths(search_path).any(|dir| is_executable(&dir.join(binary)))
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.is_file()
        && path
            .metadata()
            .is_ok_and(|metadata| metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
}

fn current_worktree_id() -> String {
    env::current_dir()
        .ok()
        .and_then(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn secret_re() -> Regex {
    Regex::new(
        r"(?i)(api[_-]?key|access[_-]?token|refresh[_-]?token|auth[_-]?token|secret|password|bearer|xai_api_key|exa_api_key|anthropic_api_key)",
    )
    .expect("static regex compiles")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::thread;
    use std::time::Instant;

    use super::*;

    #[test]
    fn validates_committed_roster_shape() -> Result<()> {
        let roster = load_roster(Path::new("../../.harness-kit/agents.yaml"))?;

        validate_roster(&roster)?;

        Ok(())
    }

    #[test]
    fn load_roster_expands_yaml_merge_keys_like_python_safe_load() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("agents.yaml");
        fs::write(
            &path,
            r#"
version: 1
providers:
  codex: &cli
    tier: primary
    kind: cli
    probe: fake-agent --version
    dispatch: fake-agent run
    output: text
    permissions: default
    worktree: recommended
    notes: fixture
  claude: {<<: *cli, tier: primary}
  pi: {<<: *cli, tier: primary}
  agy: {<<: *cli, tier: conditional}
  cursor-agent: {<<: *cli, tier: conditional}
  grok-build: {<<: *cli, tier: conditional}
  manual:
    tier: manual
    kind: manual
    probe: manual
    dispatch: manual
    output: manual-summary
    permissions: default
    worktree: not_applicable
    notes: fixture
"#,
        )?;

        let roster = load_roster(&path)?;

        validate_roster(&roster)?;
        Ok(())
    }

    #[test]
    fn rejects_secret_like_command_values() {
        let roster = fixture_roster("echo ACCESS_TOKEN=abc123", "fake-agent run");

        let error = validate_roster(&roster).unwrap_err().to_string();

        assert!(error.contains("secret-like"), "{error}");
    }

    #[test]
    fn rejects_yaml_scalar_coercion_for_string_fields() {
        let mut roster = fixture_roster("fake-agent --version", "fake-agent run");
        let provider = provider_mut(&mut roster, "codex");
        provider.insert(
            YamlValue::String("notes".to_string()),
            YamlValue::Bool(true),
        );

        let error = validate_roster(&roster).unwrap_err().to_string();

        assert!(
            error.contains("notes must be a non-empty string"),
            "{error}"
        );
    }

    #[test]
    fn builds_unavailable_probe_receipts_for_empty_path() -> Result<()> {
        let roster = fixture_roster("fake-agent --version", "fake-agent run");

        let receipts = build_probe_receipts(
            &roster,
            Some(""),
            "codex",
            "gpt-5",
            ".harness-kit/agents.yaml",
            "probe fixture",
            "",
        )?;

        assert_eq!(receipts.len(), ROSTER_PROVIDER_IDS.len());
        assert!(receipts.iter().any(|receipt| {
            receipt["provider_target"] == "manual" && receipt["provider_status"] == "manual"
        }));
        assert!(
            receipts
                .iter()
                .filter(|receipt| receipt["provider_target"] != "manual")
                .all(|receipt| receipt["provider_status"] == "unavailable"
                    && receipt["attempt_status"] == "not_started")
        );
        Ok(())
    }

    #[test]
    fn probe_executes_side_effect_free_command_and_reports_error_status() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let tool = dir.path().join("fake-agent");
        fs::write(&tool, "#!/usr/bin/env sh\nexit 7\n")?;
        let mut mode = fs::metadata(&tool)?.permissions();
        mode.set_mode(mode.mode() | 0o100);
        fs::set_permissions(&tool, mode)?;
        let roster = fixture_roster("fake-agent --version", "fake-agent run");

        let receipts = build_probe_receipts(
            &roster,
            Some(&dir.path().display().to_string()),
            "codex",
            "gpt-5",
            ".harness-kit/agents.yaml",
            "probe fixture",
            "",
        )?;

        assert!(
            receipts
                .iter()
                .filter(|receipt| receipt["provider_target"] != "manual")
                .all(|receipt| receipt["provider_status"] == "error")
        );
        Ok(())
    }

    #[test]
    fn probe_inherits_non_path_environment() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let tool = dir.path().join("fake-agent");
        fs::write(&tool, "#!/bin/sh\n[ -n \"$HOME\" ]\n")?;
        let mut mode = fs::metadata(&tool)?.permissions();
        mode.set_mode(mode.mode() | 0o100);
        fs::set_permissions(&tool, mode)?;
        let roster = fixture_roster("fake-agent --version", "fake-agent run");

        let receipts = build_probe_receipts(
            &roster,
            Some(&dir.path().display().to_string()),
            "codex",
            "gpt-5",
            ".harness-kit/agents.yaml",
            "probe fixture",
            "",
        )?;

        assert!(
            receipts
                .iter()
                .filter(|receipt| receipt["provider_target"] != "manual")
                .all(|receipt| receipt["provider_status"] == "available")
        );
        Ok(())
    }

    #[test]
    fn dispatch_refuses_unavailable_provider_before_running() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let transcript_dir = dir.path().join("traces");
        let receipt_path = dir.path().join("delegations.jsonl");
        let roster = fixture_roster("missing-agent --version", "missing-agent --run");

        let receipt = dispatch_provider_lane(
            &roster,
            "codex",
            "hello",
            DispatchRequest {
                objective: "unavailable provider fixture",
                input_ref: "prompt.txt",
                transcript_dir: &transcript_dir,
                receipt_output: &receipt_path,
                timeout_s: 1.0,
                grace_s: 0.1,
                lead_harness: "codex",
                lead_provider: "codex",
                backlog_ref: "backlog.d/072-bounded-roster-lane-dispatch.md",
                path_env: Some(""),
                model_override: None,
            },
        )?;

        assert_eq!(receipt["provider_status"], "unavailable");
        assert_eq!(receipt["attempt_status"], "failed");
        assert!(!transcript_dir.exists());
        assert_eq!(
            summarize_delegations::summarize_receipts(&receipt_path, "")?.total,
            1
        );
        Ok(())
    }

    #[test]
    fn dispatch_appends_prompt_after_agy_print_flag() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let bin_dir = dir.path().join("bin");
        fs::create_dir(&bin_dir)?;
        let argv_path = dir.path().join("argv.txt");
        let fake = bin_dir.join("fake-agy");
        fs::write(
            &fake,
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"--help\" ]; then exit 0; fi\nprintf '%s\\n' \"$@\" > {}\nlast=''\nfor arg do last=\"$arg\"; done\nprintf '%s\\n' \"$last\"\n",
                argv_path.display()
            ),
        )?;
        make_executable(&fake)?;
        let receipt_path = dir.path().join("delegations.jsonl");
        let transcript_dir = dir.path().join("traces");
        let roster = fixture_roster(
            "fake-agy --help",
            "fake-agy --dangerously-skip-permissions --print-timeout 10m --print",
        );

        let receipt = dispatch_provider_lane(
            &roster,
            "agy",
            "sentinel prompt",
            DispatchRequest {
                objective: "agy argv fixture",
                input_ref: "prompt.txt",
                transcript_dir: &transcript_dir,
                receipt_output: &receipt_path,
                timeout_s: 1.0,
                grace_s: 0.1,
                lead_harness: "codex",
                lead_provider: "codex",
                backlog_ref: "",
                path_env: Some(&bin_dir.display().to_string()),
                model_override: None,
            },
        )?;

        assert_eq!(receipt["attempt_status"], "succeeded");
        let argv = fs::read_to_string(&argv_path)?;
        let args: Vec<_> = argv.lines().collect();
        assert_eq!(args[args.len() - 2..], ["--print", "sentinel prompt"]);
        assert!(
            args.iter().position(|arg| *arg == "--print-timeout")
                < args.iter().position(|arg| *arg == "--print")
        );
        let transcript = fs::read_to_string(receipt["evidence_refs"][0].as_str().unwrap())?;
        assert!(transcript.contains("sentinel prompt"));
        Ok(())
    }

    #[test]
    fn dispatch_model_override_uses_roster_variant() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let bin_dir = dir.path().join("bin");
        fs::create_dir(&bin_dir)?;
        let argv_path = dir.path().join("argv.txt");
        let fake = bin_dir.join("fake-pi");
        fs::write(
            &fake,
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then exit 0; fi\nprintf '%s\\n' \"$@\" > {}\nprintf '%s\\n' \"$#\"\n",
                argv_path.display()
            ),
        )?;
        make_executable(&fake)?;
        let mut roster = fixture_roster(
            "fake-pi --version",
            "fake-pi -p --provider openrouter --model moonshotai/kimi-k2.6 --tools read",
        );
        provider_mut(&mut roster, "pi").insert(
            YamlValue::String("model_variants".to_string()),
            serde_yaml::from_str("long_context: openrouter/deepseek/deepseek-v4-pro\n")?,
        );
        let receipt_path = dir.path().join("delegations.jsonl");
        let transcript_dir = dir.path().join("traces");

        let receipt = dispatch_provider_lane(
            &roster,
            "pi",
            "sentinel prompt",
            DispatchRequest {
                objective: "pi model override fixture",
                input_ref: "prompt.txt",
                transcript_dir: &transcript_dir,
                receipt_output: &receipt_path,
                timeout_s: 1.0,
                grace_s: 0.1,
                lead_harness: "codex",
                lead_provider: "codex",
                backlog_ref: "",
                path_env: Some(&bin_dir.display().to_string()),
                model_override: Some("long_context"),
            },
        )?;

        assert_eq!(receipt["attempt_status"], "succeeded");
        let args: Vec<_> = fs::read_to_string(&argv_path)?
            .lines()
            .map(str::to_string)
            .collect();
        let model_index = args.iter().position(|arg| arg == "--model").unwrap();
        assert_eq!(args[model_index + 1], "deepseek/deepseek-v4-pro");
        assert_eq!(receipt["model_id"], "openrouter/deepseek/deepseek-v4-pro");
        assert!(
            receipt["summary"]
                .as_str()
                .unwrap()
                .contains("model_override=openrouter/deepseek/deepseek-v4-pro")
        );
        Ok(())
    }

    #[test]
    fn dispatch_timeout_kills_process_group_and_records_transcript() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let bin_dir = dir.path().join("bin");
        fs::create_dir(&bin_dir)?;
        let pid_file = dir.path().join("child.pid");
        let fake = bin_dir.join("fake-agent");
        fs::write(
            &fake,
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then exit 0; fi\ntrap '' TERM\n/bin/sh -c 'trap \"\" TERM; sleep 60' &\nprintf '%s' \"$!\" > {}\necho started child $!\nwhile true; do sleep 1; done\n",
                pid_file.display()
            ),
        )?;
        make_executable(&fake)?;
        let receipt_path = dir.path().join("delegations.jsonl");
        let transcript_dir = dir.path().join("traces");
        let roster = fixture_roster("fake-agent --version", "fake-agent run");

        let started = Instant::now();
        let receipt = dispatch_provider_lane(
            &roster,
            "codex",
            "hello",
            DispatchRequest {
                objective: "timeout fixture",
                input_ref: "prompt.txt",
                transcript_dir: &transcript_dir,
                receipt_output: &receipt_path,
                timeout_s: 0.2,
                grace_s: 0.1,
                lead_harness: "codex",
                lead_provider: "codex",
                backlog_ref: "backlog.d/072-bounded-roster-lane-dispatch.md",
                path_env: Some(&bin_dir.display().to_string()),
                model_override: None,
            },
        )?;

        assert!(started.elapsed().as_secs_f64() < 2.0);
        assert_eq!(receipt["provider_status"], "error");
        assert_eq!(receipt["attempt_status"], "failed");
        assert!(receipt["summary"].as_str().unwrap().contains("timed out"));
        let transcript = receipt["evidence_refs"][0].as_str().unwrap();
        assert!(fs::read_to_string(transcript)?.contains("started child"));
        assert_eq!(
            summarize_delegations::summarize_receipts(&receipt_path, "")?.total,
            1
        );

        let child_pid: i32 = fs::read_to_string(&pid_file)?.parse()?;
        for _ in 0..20 {
            if !pid_exists(child_pid) {
                break;
            }
            thread::sleep(Duration::from_millis(50));
        }
        assert!(!pid_exists(child_pid));
        Ok(())
    }

    fn fixture_roster(probe: &str, dispatch: &str) -> YamlValue {
        let mut providers = Mapping::new();
        for provider_id in ROSTER_PROVIDER_IDS {
            let mut provider = Mapping::new();
            provider.insert(
                YamlValue::String("tier".to_string()),
                YamlValue::String(
                    if matches!(*provider_id, "codex" | "claude" | "pi") {
                        "primary"
                    } else if *provider_id == "manual" {
                        "manual"
                    } else {
                        "conditional"
                    }
                    .to_string(),
                ),
            );
            provider.insert(
                YamlValue::String("kind".to_string()),
                YamlValue::String(
                    if *provider_id == "manual" {
                        "manual"
                    } else {
                        "cli"
                    }
                    .to_string(),
                ),
            );
            provider.insert(
                YamlValue::String("probe".to_string()),
                YamlValue::String(
                    if *provider_id == "manual" {
                        "manual"
                    } else {
                        probe
                    }
                    .to_string(),
                ),
            );
            provider.insert(
                YamlValue::String("dispatch".to_string()),
                YamlValue::String(
                    if *provider_id == "manual" {
                        "manual"
                    } else {
                        dispatch
                    }
                    .to_string(),
                ),
            );
            provider.insert(
                YamlValue::String("output".to_string()),
                YamlValue::String(
                    if *provider_id == "manual" {
                        "manual-summary"
                    } else {
                        "text"
                    }
                    .to_string(),
                ),
            );
            provider.insert(
                YamlValue::String("permissions".to_string()),
                YamlValue::String("default".to_string()),
            );
            provider.insert(
                YamlValue::String("worktree".to_string()),
                YamlValue::String(
                    if *provider_id == "manual" {
                        "not_applicable"
                    } else {
                        "recommended"
                    }
                    .to_string(),
                ),
            );
            provider.insert(
                YamlValue::String("notes".to_string()),
                YamlValue::String("fixture".to_string()),
            );
            providers.insert(
                YamlValue::String((*provider_id).to_string()),
                YamlValue::Mapping(provider),
            );
        }
        let mut root = Mapping::new();
        root.insert(
            YamlValue::String("version".to_string()),
            YamlValue::Number(1.into()),
        );
        root.insert(
            YamlValue::String("providers".to_string()),
            YamlValue::Mapping(providers),
        );
        YamlValue::Mapping(root)
    }

    fn provider_mut<'a>(roster: &'a mut YamlValue, provider_id: &str) -> &'a mut Mapping {
        roster
            .as_mapping_mut()
            .unwrap()
            .get_mut(YamlValue::String("providers".to_string()))
            .unwrap()
            .as_mapping_mut()
            .unwrap()
            .get_mut(YamlValue::String(provider_id.to_string()))
            .unwrap()
            .as_mapping_mut()
            .unwrap()
    }

    fn make_executable(path: &Path) -> Result<()> {
        let mut mode = fs::metadata(path)?.permissions();
        mode.set_mode(mode.mode() | 0o100);
        fs::set_permissions(path, mode)?;
        Ok(())
    }

    #[cfg(unix)]
    fn pid_exists(pid: i32) -> bool {
        unsafe { libc::kill(pid, 0) == 0 }
    }

    #[cfg(not(unix))]
    fn pid_exists(_pid: i32) -> bool {
        false
    }
}
