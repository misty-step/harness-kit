use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow, bail};
use regex::Regex;
use serde::Deserialize;
use serde_yaml::{Mapping, Value as YamlValue};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::agent_roster::{self, ROSTER_PROVIDER_IDS};

pub const SCHEMA: &str = "lane_harness.v1";
pub const PROJECTION_STATUS_PROJECTED: &str = "projected";
pub const PROJECTION_STATUS_FAILED: &str = "failed";
pub const PROJECTION_STATUS_NOT_REQUESTED: &str = "not_requested";

pub const FAILURE_KINDS: &[&str] = &[
    "missing_binary",
    "probe_failed",
    "probe_timeout",
    "spawn_failed",
    "auth_required",
    "credits_exhausted",
    "entitlement_missing",
    "dispatch_timeout",
    "nonzero_exit",
    "sentinel_mismatch",
    "projection_failed",
];

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LaneHarnessManifest {
    pub schema: String,
    pub role: String,
    pub provider_target: String,
    #[serde(default)]
    pub model_override: Option<String>,
    #[serde(default)]
    pub allowed_local_skills: Vec<String>,
    #[serde(default)]
    pub allowed_external_aliases: Vec<String>,
    #[serde(default)]
    pub allowed_tools: Vec<String>,
    pub oracle: LaneHarnessOracle,
    pub evidence_expectations: Vec<String>,
    pub fallback: LaneHarnessFallback,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LaneHarnessOracle {
    pub kind: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LaneHarnessFallback {
    pub on_provider_failure: String,
    pub replacement_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneHarnessReport {
    pub manifest_path: PathBuf,
    pub manifest_sha256: String,
    pub provider_target: String,
    pub model_override: Option<String>,
    pub root: PathBuf,
    pub visible_skills: Vec<String>,
}

pub fn read_manifest(path: &Path) -> Result<LaneHarnessManifest> {
    let text =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_yaml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn manifest_sha256(path: &Path) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(fs::read(path)?)))
}

pub fn validate_manifest(
    repo: &Path,
    roster: &YamlValue,
    manifest: &LaneHarnessManifest,
) -> Result<()> {
    agent_roster::validate_roster(roster)?;
    if manifest.schema != SCHEMA {
        bail!("lane harness schema must be {SCHEMA}");
    }
    validate_non_empty(&manifest.role, "role")?;
    validate_identifier(&manifest.provider_target, "provider_target")?;
    if !ROSTER_PROVIDER_IDS.contains(&manifest.provider_target.as_str()) {
        bail!("lane harness provider_target is not a known provider id.");
    }
    if let Some(model_override) = &manifest.model_override {
        validate_non_empty(model_override, "model_override")?;
        validate_model_override(roster, &manifest.provider_target, model_override)?;
    }
    validate_unique_names(&manifest.allowed_local_skills, "allowed_local_skills")?;
    validate_unique_names(
        &manifest.allowed_external_aliases,
        "allowed_external_aliases",
    )?;
    validate_unique_names(&manifest.allowed_tools, "allowed_tools")?;
    validate_non_empty(&manifest.oracle.kind, "oracle.kind")?;
    validate_non_empty(&manifest.oracle.value, "oracle.value")?;
    validate_no_secret_like(&manifest.oracle.value, "oracle.value")?;
    if manifest.evidence_expectations.is_empty() {
        bail!("lane harness evidence_expectations must not be empty.");
    }
    for expectation in &manifest.evidence_expectations {
        validate_non_empty(expectation, "evidence_expectations")?;
        validate_no_secret_like(expectation, "evidence_expectations")?;
    }
    validate_non_empty(
        &manifest.fallback.on_provider_failure,
        "fallback.on_provider_failure",
    )?;
    validate_non_empty(
        &manifest.fallback.replacement_policy,
        "fallback.replacement_policy",
    )?;
    if manifest.fallback.on_provider_failure != "record_and_return" {
        bail!("lane harness fallback.on_provider_failure must be record_and_return.");
    }
    if manifest.fallback.replacement_policy != "lead_explicit" {
        bail!("lane harness fallback.replacement_policy must be lead_explicit.");
    }

    for skill in &manifest.allowed_local_skills {
        validate_skill_name(skill, "allowed_local_skills")?;
        let skill_dir = repo.join("skills").join(skill);
        let skill_file = skill_dir.join("SKILL.md");
        if !skill_file.is_file() {
            bail!("lane harness references unknown local skill: {skill}");
        }
    }
    validate_external_aliases(repo, &manifest.allowed_external_aliases)?;
    Ok(())
}

pub fn validate_manifest_path(repo: &Path, roster: &YamlValue, path: &Path) -> Result<()> {
    let manifest = read_manifest(path)?;
    validate_manifest(repo, roster, &manifest)
}

pub fn materialize_manifest(
    repo: &Path,
    roster: &YamlValue,
    manifest_path: &Path,
    root: Option<&Path>,
) -> Result<LaneHarnessReport> {
    let repo =
        fs::canonicalize(repo).with_context(|| format!("failed to resolve {}", repo.display()))?;
    let manifest = read_manifest(manifest_path)?;
    validate_manifest(&repo, roster, &manifest)?;
    let manifest_sha256 = manifest_sha256(manifest_path)?;
    let root = projection_root(&repo, root)?;
    let skill_sources = local_skill_sources(&repo, &manifest.allowed_local_skills)?;
    if root.exists() {
        fs::remove_dir_all(&root).with_context(|| format!("failed to clear {}", root.display()))?;
    }
    fs::create_dir_all(&root).with_context(|| format!("failed to create {}", root.display()))?;

    if let Err(error) = link_local_skills(&root, &skill_sources) {
        let _ = fs::remove_dir_all(&root);
        return Err(error);
    }
    fs::write(
        root.join("lane-harness-manifest.sha256"),
        format!("{manifest_sha256}\n"),
    )?;
    Ok(LaneHarnessReport {
        manifest_path: manifest_path.to_path_buf(),
        manifest_sha256,
        provider_target: manifest.provider_target,
        model_override: manifest.model_override,
        root,
        visible_skills: skill_sources
            .iter()
            .map(|source| source.name.clone())
            .collect(),
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalSkillSource {
    name: String,
    canonical_source: PathBuf,
}

fn local_skill_sources(repo: &Path, skills: &[String]) -> Result<Vec<LocalSkillSource>> {
    let skills_root =
        fs::canonicalize(repo.join("skills")).context("failed to resolve skills root")?;
    let mut sources = Vec::with_capacity(skills.len());
    for skill in skills {
        let source = repo.join("skills").join(skill);
        let canonical_source = fs::canonicalize(&source)
            .with_context(|| format!("failed to resolve {}", source.display()))?;
        if !canonical_source.starts_with(&skills_root) {
            bail!("lane harness local skill escapes skills root: {skill}");
        }
        sources.push(LocalSkillSource {
            name: skill.clone(),
            canonical_source,
        });
    }
    Ok(sources)
}

fn link_local_skills(root: &Path, skill_sources: &[LocalSkillSource]) -> Result<()> {
    for source in skill_sources {
        for skill_root in provider_skill_roots(root) {
            fs::create_dir_all(&skill_root)
                .with_context(|| format!("failed to create {}", skill_root.display()))?;
            link_or_copy_dir(&source.canonical_source, &skill_root.join(&source.name))?;
        }
    }
    Ok(())
}

fn projection_root(repo: &Path, root: Option<&Path>) -> Result<PathBuf> {
    let base = repo.join(".harness-kit/tmp/lane-harness");
    let candidate = root
        .map(|root| {
            if root.is_absolute() {
                root.to_path_buf()
            } else {
                repo.join(root)
            }
        })
        .unwrap_or_else(|| base.join(short_id()));
    let base = normalize_path(&base);
    let candidate = normalize_path(&candidate);
    if candidate == base || !candidate.starts_with(&base) {
        bail!(
            "lane harness projection root must be under {}",
            base.display()
        );
    }
    Ok(candidate)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}

pub fn provider_skill_roots(root: &Path) -> Vec<PathBuf> {
    vec![
        root.join("skills"),
        root.join(".codex/skills"),
        root.join(".claude/skills"),
        root.join(".pi/skills"),
        root.join(".gemini/antigravity-cli/skills"),
        root.join(".gemini/config/skills"),
    ]
}

pub fn format_materialize_report(report: &LaneHarnessReport) -> String {
    format!(
        "lane_harness_ref: {}\nlane_harness_sha256: {}\nprovider_target: {}\nprojection_root: {}\nvisible_skills: {}",
        report.manifest_path.display(),
        report.manifest_sha256,
        report.provider_target,
        report.root.display(),
        if report.visible_skills.is_empty() {
            "none".to_string()
        } else {
            report.visible_skills.join(",")
        }
    )
}

pub fn validate_projection_status(value: &str) -> bool {
    matches!(
        value,
        PROJECTION_STATUS_PROJECTED | PROJECTION_STATUS_FAILED | PROJECTION_STATUS_NOT_REQUESTED
    )
}

pub fn validate_failure_kind(value: &str) -> bool {
    FAILURE_KINDS.contains(&value)
}

pub fn classify_failure(return_code: i32, transcript: Option<&str>) -> &'static str {
    let text = transcript.unwrap_or("").to_ascii_lowercase();
    if text.contains("spend limit")
        || text.contains("credit limit")
        || text.contains("credits exhausted")
        || text.contains("out of credits")
    {
        "credits_exhausted"
    } else if text.contains("auth required")
        || text.contains("authentication")
        || text.contains("unauthorized")
        || text.contains("api key")
    {
        "auth_required"
    } else if text.contains("entitlement") || text.contains("not entitled") {
        "entitlement_missing"
    } else if return_code == 0 {
        "sentinel_mismatch"
    } else {
        "nonzero_exit"
    }
}

fn validate_external_aliases(repo: &Path, aliases: &[String]) -> Result<()> {
    if aliases.is_empty() {
        return Ok(());
    }
    let registry_path = repo.join("registry.yaml");
    let registry: YamlValue = serde_yaml::from_str(
        &fs::read_to_string(&registry_path)
            .with_context(|| format!("failed to read {}", registry_path.display()))?,
    )
    .with_context(|| format!("failed to parse {}", registry_path.display()))?;
    let sources = registry
        .as_mapping()
        .and_then(|root| root.get(YamlValue::String("sources".to_string())))
        .and_then(YamlValue::as_sequence)
        .ok_or_else(|| anyhow!("registry.yaml must define sources."))?;

    for alias in aliases {
        validate_skill_name(alias, "allowed_external_aliases")?;
        let mut matched = false;
        for source in sources.iter().filter_map(YamlValue::as_mapping) {
            let Some(prefix) = source
                .get(YamlValue::String("alias_prefix".to_string()))
                .and_then(YamlValue::as_str)
            else {
                continue;
            };
            let Some(skill_name) = alias.strip_prefix(prefix) else {
                continue;
            };
            let pinned = source
                .get(YamlValue::String("pin".to_string()))
                .and_then(YamlValue::as_str)
                .is_some_and(|pin| !pin.trim().is_empty());
            if !pinned {
                bail!("lane harness external alias is not pinned in registry.yaml: {alias}");
            }
            if let Some(include) = source
                .get(YamlValue::String("include".to_string()))
                .and_then(YamlValue::as_sequence)
            {
                let allowed = include
                    .iter()
                    .filter_map(YamlValue::as_str)
                    .any(|name| name == skill_name);
                if !allowed {
                    continue;
                }
            }
            matched = true;
            break;
        }
        if !matched {
            bail!("lane harness references unknown external alias: {alias}");
        }
    }
    Ok(())
}

fn validate_model_override(
    roster: &YamlValue,
    provider_target: &str,
    model_override: &str,
) -> Result<()> {
    let provider = roster_provider(roster, provider_target)?;
    let mut allowed = BTreeSet::new();
    if let Some(model) = provider
        .get(YamlValue::String("model".to_string()))
        .and_then(YamlValue::as_str)
        .filter(|model| !model.trim().is_empty())
    {
        allowed.insert(model.to_string());
    }
    if let Some(variants) = provider
        .get(YamlValue::String("model_variants".to_string()))
        .and_then(YamlValue::as_mapping)
    {
        for (name, model) in variants {
            if let Some(name) = name.as_str().filter(|name| !name.trim().is_empty()) {
                allowed.insert(name.to_string());
            }
            if let Some(model) = model.as_str().filter(|model| !model.trim().is_empty()) {
                allowed.insert(model.to_string());
            }
        }
    }
    if !allowed.contains(model_override) {
        bail!(
            "lane harness model_override must match provider model or model_variants for {provider_target}: {model_override}"
        );
    }
    Ok(())
}

fn roster_provider<'a>(roster: &'a YamlValue, provider_target: &str) -> Result<&'a Mapping> {
    roster
        .as_mapping()
        .and_then(|root| root.get(YamlValue::String("providers".to_string())))
        .and_then(YamlValue::as_mapping)
        .and_then(|providers| providers.get(YamlValue::String(provider_target.to_string())))
        .and_then(YamlValue::as_mapping)
        .ok_or_else(|| anyhow!("roster missing provider: {provider_target}"))
}

fn validate_unique_names(values: &[String], field: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    for value in values {
        validate_non_empty(value, field)?;
        validate_no_secret_like(value, field)?;
        if !seen.insert(value) {
            bail!("lane harness {field} contains duplicate value: {value}");
        }
    }
    Ok(())
}

fn validate_skill_name(value: &str, field: &str) -> Result<()> {
    validate_non_empty(value, field)?;
    if value.contains('/') || value.contains('\\') || value == "." || value == ".." {
        bail!("lane harness {field} contains path escape: {value}");
    }
    validate_identifier(value, field)
}

fn validate_identifier(value: &str, field: &str) -> Result<()> {
    let re = Regex::new(r"^[A-Za-z0-9][A-Za-z0-9_.-]*$").expect("static regex compiles");
    if !re.is_match(value) {
        bail!("lane harness {field} has invalid identifier: {value}");
    }
    Ok(())
}

fn validate_non_empty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("lane harness {field} must be non-empty.");
    }
    validate_no_secret_like(value, field)
}

fn validate_no_secret_like(value: &str, field: &str) -> Result<()> {
    let secret_re = Regex::new(
        r"(?i)(api[_-]?key|secret|token|password|bearer\s+[A-Za-z0-9._-]{8,}|sk-[A-Za-z0-9_-]{8,})",
    )
    .expect("static regex compiles");
    if secret_re.is_match(value) {
        bail!("lane harness {field} contains secret-like text.");
    }
    Ok(())
}

#[cfg(unix)]
fn link_or_copy_dir(source: &Path, destination: &Path) -> Result<()> {
    std::os::unix::fs::symlink(source, destination).with_context(|| {
        format!(
            "failed to symlink {} -> {}",
            destination.display(),
            source.display()
        )
    })
}

#[cfg(not(unix))]
fn link_or_copy_dir(source: &Path, destination: &Path) -> Result<()> {
    copy_dir(source, destination)
}

#[cfg(not(unix))]
fn copy_dir(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let target = destination.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            fs::copy(&path, &target)?;
        }
    }
    Ok(())
}

fn short_id() -> String {
    Uuid::new_v4().simple().to_string()[..12].to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_yaml::{Mapping, Value as YamlValue};

    use super::*;

    #[test]
    fn validates_manifest_and_rejects_bad_inputs() -> Result<()> {
        let dir = fixture_repo()?;
        let roster = fixture_roster();
        let good: LaneHarnessManifest = serde_yaml::from_str(&manifest_yaml("ci"))?;
        validate_manifest(dir.path(), &roster, &good)?;

        let unknown_provider =
            manifest_yaml("ci").replace("provider_target: codex", "provider_target: nope");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&unknown_provider)?;
        assert!(
            validate_manifest(dir.path(), &roster, &manifest)
                .unwrap_err()
                .to_string()
                .contains("known provider")
        );

        let unknown_skill = manifest_yaml("missing-skill");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&unknown_skill)?;
        assert!(
            validate_manifest(dir.path(), &roster, &manifest)
                .unwrap_err()
                .to_string()
                .contains("unknown local skill")
        );

        let duplicate_skill = manifest_yaml("ci").replace("- ci", "- ci\n  - ci");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&duplicate_skill)?;
        assert!(
            validate_manifest(dir.path(), &roster, &manifest)
                .unwrap_err()
                .to_string()
                .contains("duplicate")
        );

        let secret = manifest_yaml("ci").replace("commands_read", "token sk-test_123456789");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&secret)?;
        assert!(
            validate_manifest(dir.path(), &roster, &manifest)
                .unwrap_err()
                .to_string()
                .contains("secret-like")
        );

        let unknown_field = manifest_yaml("ci") + "\nextra: nope\n";
        let error = serde_yaml::from_str::<LaneHarnessManifest>(&unknown_field)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown field"));
        Ok(())
    }

    #[test]
    fn rejects_path_escape_and_unpinned_external_alias() -> Result<()> {
        let dir = fixture_repo()?;
        let roster = fixture_roster();
        let escape = manifest_yaml("../ci");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&escape)?;
        assert!(
            validate_manifest(dir.path(), &roster, &manifest)
                .unwrap_err()
                .to_string()
                .contains("path escape")
        );

        let external = manifest_yaml("ci").replace(
            "allowed_external_aliases: []",
            "allowed_external_aliases:\n  - floating-design",
        );
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&external)?;
        assert!(
            validate_manifest(dir.path(), &roster, &manifest)
                .unwrap_err()
                .to_string()
                .contains("not pinned")
        );
        Ok(())
    }

    #[test]
    fn model_override_must_match_provider_roster_model_or_variant() -> Result<()> {
        let dir = fixture_repo()?;
        let roster = fixture_roster();

        let default_model =
            manifest_yaml("ci").replace("model_override: null", "model_override: fixture-model");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&default_model)?;
        validate_manifest(dir.path(), &roster, &manifest)?;

        let variant_name =
            manifest_yaml("ci").replace("model_override: null", "model_override: alternate");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&variant_name)?;
        validate_manifest(dir.path(), &roster, &manifest)?;

        let variant_value =
            manifest_yaml("ci").replace("model_override: null", "model_override: fixture-alt");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&variant_value)?;
        validate_manifest(dir.path(), &roster, &manifest)?;

        let off_roster =
            manifest_yaml("ci").replace("model_override: null", "model_override: other-model");
        let manifest: LaneHarnessManifest = serde_yaml::from_str(&off_roster)?;
        assert!(
            validate_manifest(dir.path(), &roster, &manifest)
                .unwrap_err()
                .to_string()
                .contains("model_override must match provider model")
        );
        Ok(())
    }

    #[test]
    fn materializes_only_allowed_skills_into_provider_discovery_roots() -> Result<()> {
        let dir = fixture_repo()?;
        let roster = fixture_roster();
        let manifest_path = dir.path().join("lane.yaml");
        fs::write(&manifest_path, manifest_yaml("ci"))?;
        let root = fs::canonicalize(dir.path())?.join(".harness-kit/tmp/lane-harness/test");

        let report = materialize_manifest(dir.path(), &roster, &manifest_path, Some(&root))?;

        assert_eq!(report.visible_skills, vec!["ci"]);
        for skill_root in provider_skill_roots(&root) {
            assert!(skill_root.join("ci/SKILL.md").exists());
            assert!(skill_root.join("ci/references/nested.md").exists());
            assert!(!skill_root.join("shape").exists());
            assert!(!skill_root.join("groom").exists());
        }
        assert!(!dir.path().join("index.yaml").exists());
        assert!(!dir.path().join(".codex/skills").exists());
        Ok(())
    }

    #[test]
    fn rejects_projection_root_outside_runtime_directory() -> Result<()> {
        let dir = fixture_repo()?;
        let roster = fixture_roster();
        let manifest_path = dir.path().join("lane.yaml");
        fs::write(&manifest_path, manifest_yaml("ci"))?;

        let error = materialize_manifest(dir.path(), &roster, &manifest_path, Some(dir.path()))
            .unwrap_err()
            .to_string();

        assert!(error.contains("projection root must be under"));
        assert!(dir.path().join("skills/ci/SKILL.md").exists());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn rejects_skill_symlink_escape() -> Result<()> {
        let dir = fixture_repo()?;
        let outside = tempfile::tempdir()?;
        fs::write(outside.path().join("SKILL.md"), "name: ci\n")?;
        fs::remove_dir_all(dir.path().join("skills/ci"))?;
        std::os::unix::fs::symlink(outside.path(), dir.path().join("skills/ci"))?;
        let roster = fixture_roster();
        let manifest_path = dir.path().join("lane.yaml");
        fs::write(&manifest_path, manifest_yaml("ci"))?;
        let root = fs::canonicalize(dir.path())?.join(".harness-kit/tmp/lane-harness/escaped");

        let error = materialize_manifest(dir.path(), &roster, &manifest_path, Some(&root))
            .unwrap_err()
            .to_string();

        assert!(error.contains("escapes skills root"));
        assert!(!root.exists());
        Ok(())
    }

    fn fixture_repo() -> Result<tempfile::TempDir> {
        let dir = tempfile::tempdir()?;
        for skill in ["ci", "shape", "groom"] {
            let skill_dir = dir.path().join("skills").join(skill);
            fs::create_dir_all(skill_dir.join("references"))?;
            fs::write(skill_dir.join("SKILL.md"), format!("name: {skill}\n"))?;
            fs::write(skill_dir.join("references/nested.md"), "nested")?;
        }
        fs::write(
            dir.path().join("registry.yaml"),
            "sources:\n  - repo: example/floating\n    ref: main\n    alias_prefix: floating-\n    include: [design]\n  - repo: example/pinned\n    ref: main\n    pin: 0000000000000000000000000000000000000000\n    alias_prefix: pinned-\n    include: [design]\n",
        )?;
        Ok(dir)
    }

    fn fixture_roster() -> YamlValue {
        let mut providers = Mapping::new();
        for provider_id in ROSTER_PROVIDER_IDS {
            let mut provider = Mapping::new();
            provider.insert(
                YamlValue::String("tier".to_string()),
                YamlValue::String(
                    if *provider_id == "manual" {
                        "manual"
                    } else {
                        "primary"
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
                YamlValue::String("echo ok".to_string()),
            );
            provider.insert(
                YamlValue::String("dispatch".to_string()),
                YamlValue::String("echo run".to_string()),
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
                YamlValue::String("recommended".to_string()),
            );
            provider.insert(
                YamlValue::String("notes".to_string()),
                YamlValue::String("fixture".to_string()),
            );
            provider.insert(
                YamlValue::String("model".to_string()),
                YamlValue::String("fixture-model".to_string()),
            );
            let mut variants = Mapping::new();
            variants.insert(
                YamlValue::String("alternate".to_string()),
                YamlValue::String("fixture-alt".to_string()),
            );
            provider.insert(
                YamlValue::String("model_variants".to_string()),
                YamlValue::Mapping(variants),
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

    fn manifest_yaml(skill: &str) -> String {
        format!(
            r#"schema: lane_harness.v1
role: critic
provider_target: codex
model_override: null
allowed_local_skills:
  - {skill}
allowed_external_aliases: []
allowed_tools:
  - shell.readonly
oracle:
  kind: path
  value: backlog.d/101-focused-lane-harness-projection.md
evidence_expectations:
  - commands_read
fallback:
  on_provider_failure: record_and_return
  replacement_policy: lead_explicit
"#
        )
    }
}
