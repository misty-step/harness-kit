use anyhow::{Context, Result, bail};
use regex::Regex;
use serde_json::Value;

pub const FIELD: &str = "work_source_refs";

const VALID_ROLES: &[&str] = &[
    "backlog",
    "acceptance",
    "context",
    "review",
    "operator_prompt",
    "closure",
    "evidence",
];
const VALID_KINDS: &[&str] = &[
    "local_backlog",
    "local_file",
    "mcp_resource",
    "cli_resource",
    "url",
    "manual",
];
const VALID_FIELDS: &[&str] = &[
    "role",
    "kind",
    "system",
    "id",
    "uri",
    "title",
    "snapshot_ref",
    "snapshot_sha256",
    "retrieved_at",
    "closure",
];
const VALID_CLOSURE_FIELDS: &[&str] = &["mode", "capability"];
const VALID_CLOSURE_MODES: &[&str] = &[
    "none",
    "local_archive",
    "mcp_tool",
    "cli",
    "manual",
    "not_supported",
];

pub fn parse_ref(raw: &str) -> Result<Value> {
    let value: Value =
        serde_json::from_str(raw).with_context(|| format!("invalid {FIELD} JSON: {raw}"))?;
    validate_refs(std::slice::from_ref(&value), None)?;
    Ok(value)
}

pub fn validate_refs(refs: &[Value], backlog_ref: Option<&str>) -> Result<()> {
    for value in refs {
        validate_ref(value, backlog_ref)?;
    }
    Ok(())
}

fn validate_ref(value: &Value, backlog_ref: Option<&str>) -> Result<()> {
    let Some(object) = value.as_object() else {
        bail!("{FIELD} entries must be JSON objects.");
    };
    for key in object.keys() {
        if !VALID_FIELDS.contains(&key.as_str()) {
            bail!("{FIELD} entry has unknown field: {key}");
        }
    }

    let role = required_text(value, "role")?;
    if !VALID_ROLES.contains(&role) {
        bail!("{FIELD} role is invalid: {role}");
    }
    let kind = required_text(value, "kind")?;
    if !VALID_KINDS.contains(&kind) {
        bail!("{FIELD} kind is invalid: {kind}");
    }

    match kind {
        "local_backlog" => {
            let id = required_text(value, "id")?;
            if !id.chars().all(|ch| ch.is_ascii_digit()) {
                bail!("{FIELD} local_backlog id must be numeric.");
            }
            if let Some(backlog_ref) = backlog_ref
                && !backlog_ref.is_empty()
                && backlog_ref_id(backlog_ref).as_deref() != Some(id)
            {
                bail!("{FIELD} local_backlog id must match backlog_ref.");
            }
            optional_text(value, "uri")?;
        }
        "local_file" => {
            required_text(value, "uri")?;
        }
        "mcp_resource" => {
            required_text(value, "system")?;
            required_text(value, "uri")?;
        }
        "cli_resource" => {
            required_text(value, "system")?;
            if object.get("id").is_none() && object.get("uri").is_none() {
                bail!("{FIELD} cli_resource requires id or uri.");
            }
            optional_text(value, "id")?;
            optional_text(value, "uri")?;
        }
        "url" => {
            let uri = required_text(value, "uri")?;
            if !(uri.starts_with("https://") || uri.starts_with("http://")) {
                bail!("{FIELD} url uri must be http(s).");
            }
        }
        "manual" => {
            if object.get("id").is_none() && object.get("title").is_none() {
                bail!("{FIELD} manual requires id or title.");
            }
            optional_text(value, "id")?;
            optional_text(value, "title")?;
        }
        _ => unreachable!("kind validated above"),
    }

    optional_text(value, "system")?;
    optional_text(value, "title")?;
    optional_text(value, "snapshot_ref")?;
    optional_text(value, "retrieved_at")?;
    if let Some(snapshot) = value.get("snapshot_sha256") {
        let Some(snapshot) = snapshot.as_str() else {
            bail!("{FIELD} snapshot_sha256 must be a sha256 hex string.");
        };
        if !sha256_re().is_match(snapshot) {
            bail!("{FIELD} snapshot_sha256 must be a sha256 hex string.");
        }
    }
    if let Some(closure) = value.get("closure") {
        validate_closure(closure)?;
    }
    Ok(())
}

fn validate_closure(value: &Value) -> Result<()> {
    let Some(object) = value.as_object() else {
        bail!("{FIELD} closure must be an object.");
    };
    for key in object.keys() {
        if !VALID_CLOSURE_FIELDS.contains(&key.as_str()) {
            bail!("{FIELD} closure has unknown field: {key}");
        }
    }
    let mode = required_text(value, "mode")?;
    if !VALID_CLOSURE_MODES.contains(&mode) {
        bail!("{FIELD} closure mode is invalid: {mode}");
    }
    optional_text(value, "capability")?;
    Ok(())
}

fn required_text<'a>(value: &'a Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("{FIELD} {field} must be a non-empty string."))
}

fn optional_text(value: &Value, field: &str) -> Result<()> {
    if let Some(value) = value.get(field)
        && !value.is_null()
        && value.as_str().is_none_or(|text| text.trim().is_empty())
    {
        bail!("{FIELD} {field} must be a non-empty string or null.");
    }
    Ok(())
}

fn backlog_ref_id(backlog_ref: &str) -> Option<String> {
    if backlog_ref.chars().all(|ch| ch.is_ascii_digit()) {
        return Some(backlog_ref.to_string());
    }
    let name = backlog_ref.rsplit('/').next().unwrap_or(backlog_ref);
    let (prefix, _) = name.split_once('-')?;
    if prefix.chars().all(|ch| ch.is_ascii_digit()) && !prefix.is_empty() {
        Some(prefix.to_string())
    } else {
        None
    }
}

fn sha256_re() -> &'static Regex {
    static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
    RE.get_or_init(|| Regex::new(r"^[0-9a-f]{64}$").expect("static regex compiles"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn validates_local_backlog_refs_against_legacy_backlog_ref() {
        let value = json!({
            "role": "backlog",
            "kind": "local_backlog",
            "id": "023",
            "uri": "backlog.d/023-example.md",
            "closure": {"mode": "local_archive"}
        });

        assert!(validate_refs(std::slice::from_ref(&value), Some("023")).is_ok());
        assert!(
            validate_refs(
                std::slice::from_ref(&value),
                Some("backlog.d/023-example.md")
            )
            .is_ok()
        );
        assert!(
            validate_refs(
                std::slice::from_ref(&value),
                Some("backlog.d/_done/023-example.md")
            )
            .is_ok()
        );
        assert!(validate_refs(&[value], Some("024")).is_err());
    }

    #[test]
    fn validates_external_resource_shape_without_resolving_it() {
        let value = json!({
            "role": "acceptance",
            "kind": "mcp_resource",
            "system": "linear",
            "uri": "mcp://linear/issues/LIN-123",
            "snapshot_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "closure": {"mode": "mcp_tool", "capability": "linear.close_issue"}
        });

        assert!(validate_refs(&[value], None).is_ok());
    }

    #[test]
    fn rejects_baggy_or_incomplete_refs() {
        assert!(validate_refs(&[json!({"role": "backlog", "kind": "url"})], None).is_err());
        assert!(
            validate_refs(
                &[json!({"role": "backlog", "kind": "url", "uri": "file://ticket"})],
                None
            )
            .is_err()
        );
        assert!(
            validate_refs(
                &[json!({"role": "backlog", "kind": "manual", "title": "x", "extra": "y"})],
                None
            )
            .is_err()
        );
    }
}
