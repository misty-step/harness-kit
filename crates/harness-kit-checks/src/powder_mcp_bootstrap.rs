use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const POWDER_MCP_SOURCE: &str = "/Users/phaedrus/Development/powder/crates/powder-mcp";
// harness-kit-914: `op run --env-file` resolves any `op://...` reference in
// ~/.secrets to its real value and passes any plain (non-reference) line
// through unchanged, so this works whether ~/.secrets holds raw values or
// op:// references, before or after the conversion completes. It also needs
// OP_SERVICE_ACCOUNT_TOKEN to authenticate, which a sanitized MCP-bootstrap
// context (no inherited env beyond HOME/USER/PATH) does not carry -- so the
// keychain bootstrap line runs first, live-verified under `env -i HOME=...
// USER=... PATH=...` with no pre-set token.
const POWDER_MCP_SCRIPT: &str = "export OP_SERVICE_ACCOUNT_TOKEN=\"${OP_SERVICE_ACCOUNT_TOKEN:-$(security find-generic-password -a \"$USER\" -s op-agent -w 2>/dev/null)}\"; op run --env-file ~/.secrets -- powder-mcp";
const POWDER_STAMP: &str = ".harness-kit/powder-mcp-install.sha256";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    Updated,
    Unchanged,
    Skipped,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Updated => "updated",
            Status::Unchanged => "unchanged",
            Status::Skipped => "skipped",
        }
    }
}

pub fn ensure(home: &Path, lines: &mut Vec<String>) -> Result<()> {
    lines.push(super::bootstrap::blue(
        "Ensuring Powder MCP registration...",
    ));
    let binary_status = ensure_powder_mcp_binary(home, lines)?;
    if binary_status == Status::Skipped {
        lines.push(super::bootstrap::yellow(
            "    registration skipped: powder-mcp source checkout is unavailable",
        ));
        lines.push(String::new());
        return Ok(());
    }

    ensure_claude(home, lines)?;
    ensure_codex(home, lines)?;
    ensure_opencode(home, lines)?;
    report_peer_harness_notes(home, lines);
    lines.push(String::new());
    Ok(())
}

fn ensure_powder_mcp_binary(home: &Path, lines: &mut Vec<String>) -> Result<Status> {
    let source = Path::new(POWDER_MCP_SOURCE);
    if !source.is_dir() {
        lines.push(super::bootstrap::yellow(format!(
            "    powder-mcp source missing: {}",
            source.display()
        )));
        return Ok(Status::Skipped);
    }

    let fingerprint = powder_source_fingerprint(source)?;
    let binary = cargo_bin(home).join("powder-mcp");
    let stamp = home.join(POWDER_STAMP);
    if binary.is_file() && read_trimmed(&stamp).is_ok_and(|text| text == fingerprint) {
        lines.push(super::bootstrap::green("    powder-mcp binary unchanged"));
        return Ok(Status::Unchanged);
    }

    let output = Command::new("cargo")
        .args(["install", "--locked", "--path"])
        .arg(source)
        .output()
        .context("failed to run cargo install for powder-mcp")?;
    if !output.status.success() {
        bail!(
            "cargo install --locked --path {} failed\n{}{}",
            source.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    if let Some(parent) = stamp.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&stamp, format!("{fingerprint}\n"))
        .with_context(|| format!("failed to write {}", stamp.display()))?;
    lines.push(super::bootstrap::green(
        "    powder-mcp binary installed/current",
    ));
    Ok(Status::Updated)
}

fn ensure_claude(home: &Path, lines: &mut Vec<String>) -> Result<Status> {
    let path = home.join(".claude.json");
    let status = upsert_claude_json(&path)?;
    lines.push(status_line("claude", status, "~/.claude.json"));
    Ok(status)
}

fn ensure_codex(home: &Path, lines: &mut Vec<String>) -> Result<Status> {
    let path = home.join(".codex/config.toml");
    let status = upsert_codex_toml(&path)?;
    lines.push(status_line("codex", status, "~/.codex/config.toml"));
    Ok(status)
}

fn ensure_opencode(home: &Path, lines: &mut Vec<String>) -> Result<Status> {
    let config_home = env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".config"));
    let path = config_home.join("opencode/opencode.json");
    if !path.exists() && !command_exists("opencode") {
        lines.push(status_line("opencode", Status::Skipped, "not detected"));
        return Ok(Status::Skipped);
    }
    let status = upsert_opencode_json(&path)?;
    lines.push(status_line(
        "opencode",
        status,
        "~/.config/opencode/opencode.json",
    ));
    Ok(status)
}

fn report_peer_harness_notes(home: &Path, lines: &mut Vec<String>) {
    if harness_detected(home, "pi") {
        lines.push(super::bootstrap::yellow(
            "    pi: skipped (settings/extension surface has no MCP registry)",
        ));
    }
    if harness_detected(home, "antigravity-cli")
        || harness_detected(home, "antigravity-ide")
        || harness_detected(home, "antigravity")
    {
        lines.push(super::bootstrap::yellow(
            "    antigravity: skipped (MCP lives in plugin/config packaging; no stable CLI list/health registration path)",
        ));
    }
    if command_exists("goose") {
        lines.push(super::bootstrap::yellow(
            "    goose: skipped (not currently bootstrap-projected; config uses Goose extensions)",
        ));
    }
    if command_exists("omp") {
        lines.push(super::bootstrap::yellow(
            "    omp: skipped (not currently bootstrap-projected; global config is user-owned)",
        ));
    }
}

fn status_line(harness: &str, status: Status, target: &str) -> String {
    let message = format!("    {harness}: {} ({target})", status.label());
    match status {
        Status::Updated | Status::Unchanged => super::bootstrap::green(message),
        Status::Skipped => super::bootstrap::yellow(message),
    }
}

fn upsert_claude_json(path: &Path) -> Result<Status> {
    let mut value = read_json_object_or_empty(path)?;
    let root = value
        .as_object_mut()
        .context("Claude config root must be a JSON object")?;
    let servers = root
        .entry("mcpServers")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .context("Claude mcpServers must be a JSON object")?;
    let desired = claude_powder_entry();
    if servers.get("powder") == Some(&desired) {
        return Ok(Status::Unchanged);
    }
    servers.insert("powder".to_string(), desired);
    write_json(path, &value)?;
    Ok(Status::Updated)
}

fn upsert_opencode_json(path: &Path) -> Result<Status> {
    let mut value = read_json_object_or_empty(path)?;
    let root = value
        .as_object_mut()
        .context("OpenCode config root must be a JSON object")?;
    let servers = root
        .entry("mcp")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .context("OpenCode mcp must be a JSON object")?;
    let desired = opencode_powder_entry();
    if servers.get("powder") == Some(&desired) {
        return Ok(Status::Unchanged);
    }
    servers.insert("powder".to_string(), desired);
    write_json(path, &value)?;
    Ok(Status::Updated)
}

fn upsert_codex_toml(path: &Path) -> Result<Status> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => {
            return Err(error).with_context(|| format!("failed to read {}", path.display()));
        }
    };
    let desired = codex_powder_block();
    if powder_table_is_converged(&text, &desired) {
        toml::from_str::<toml::Value>(&text)
            .with_context(|| format!("{} is not valid TOML", path.display()))?;
        return Ok(Status::Unchanged);
    }
    let mut next = remove_powder_tables(&text);
    if !next.is_empty() && !next.ends_with('\n') {
        next.push('\n');
    }
    if !next.is_empty() && !next.ends_with("\n\n") {
        next.push('\n');
    }
    next.push_str(&desired);
    toml::from_str::<toml::Value>(&next)
        .with_context(|| format!("rewritten {} is not valid TOML", path.display()))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, next).with_context(|| format!("failed to write {}", path.display()))?;
    Ok(Status::Updated)
}

fn powder_table_is_converged(text: &str, desired: &str) -> bool {
    let spans = powder_table_spans(text);
    spans.len() == 1 && text[spans[0].0..spans[0].1].trim() == desired.trim()
}

fn remove_powder_tables(text: &str) -> String {
    let spans = powder_table_spans(text);
    if spans.is_empty() {
        return text.to_string();
    }
    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for (start, end) in spans {
        out.push_str(&text[cursor..start]);
        cursor = end;
    }
    out.push_str(&text[cursor..]);
    trim_excess_blank_lines(&out)
}

fn powder_table_spans(text: &str) -> Vec<(usize, usize)> {
    let mut spans = Vec::new();
    let mut start = None;
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if is_toml_table_header(trimmed) {
            match start {
                None if is_powder_mcp_header(trimmed) => start = Some(offset),
                Some(existing_start) if !is_powder_mcp_header(trimmed) => {
                    spans.push((existing_start, offset));
                    start = if is_powder_mcp_header(trimmed) {
                        Some(offset)
                    } else {
                        None
                    };
                }
                _ => {}
            }
        }
        offset += line.len();
    }
    if let Some(existing_start) = start {
        spans.push((existing_start, text.len()));
    }
    spans
}

// Headers may carry trailing comments (`[table] # note`); requiring the line
// to END with `]` would miss them, letting a powder span swallow the table
// that follows during removal.
fn is_toml_table_header(line: &str) -> bool {
    line.starts_with('[') && line.contains(']')
}

fn is_powder_mcp_header(line: &str) -> bool {
    line.starts_with("[mcp_servers.powder]") || line.starts_with("[mcp_servers.powder.")
}

fn trim_excess_blank_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank_count = 0usize;
    for line in text.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 2 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    out.trim_end_matches('\n').to_string()
}

fn read_json_object_or_empty(path: &Path) -> Result<Value> {
    match fs::read_to_string(path) {
        Ok(text) => serde_json::from_str(&text)
            .with_context(|| format!("failed to parse {}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let rendered = serde_json::to_string_pretty(value)? + "\n";
    serde_json::from_str::<Value>(&rendered)
        .with_context(|| format!("rendered {} is not valid JSON", path.display()))?;
    fs::write(path, rendered).with_context(|| format!("failed to write {}", path.display()))
}

fn claude_powder_entry() -> Value {
    json!({
        "type": "stdio",
        "command": "bash",
        "args": ["-c", POWDER_MCP_SCRIPT],
        "env": {}
    })
}

fn opencode_powder_entry() -> Value {
    json!({
        "type": "local",
        "command": ["bash", "-c", POWDER_MCP_SCRIPT],
        "enabled": true
    })
}

fn codex_powder_block() -> String {
    format!(
        "[mcp_servers.powder]\ncommand = \"bash\"\nargs = [\"-c\", {:?}]\n",
        POWDER_MCP_SCRIPT
    )
}

fn powder_source_fingerprint(source: &Path) -> Result<String> {
    let workspace = source
        .parent()
        .and_then(Path::parent)
        .context("powder-mcp source path must be under crates/")?;
    let mut files = vec![workspace.join("Cargo.toml"), workspace.join("Cargo.lock")];
    for crate_name in ["powder-core", "powder-shell", "powder-store", "powder-mcp"] {
        let crate_dir = workspace.join("crates").join(crate_name);
        files.push(crate_dir.join("Cargo.toml"));
        collect_files(&crate_dir.join("src"), &mut files)?;
        collect_files(&crate_dir.join("tests"), &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut hasher = Sha256::new();
    for path in files.into_iter().filter(|path| path.is_file()) {
        let rel = path.strip_prefix(workspace).unwrap_or(&path);
        hasher.update(rel.to_string_lossy().as_bytes());
        hasher.update(b"\0");
        hasher
            .update(fs::read(&path).with_context(|| format!("failed to read {}", path.display()))?);
        hasher.update(b"\0");
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn collect_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let mut entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read {}", dir.display()))?
        .collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.path());
    for entry in entries {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out)?;
        } else if path.is_file() {
            out.push(path);
        }
    }
    Ok(())
}

fn cargo_bin(home: &Path) -> PathBuf {
    env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".cargo"))
        .join("bin")
}

fn read_trimmed(path: &Path) -> Result<String> {
    Ok(fs::read_to_string(path)
        .with_context(|| format!("failed to read {}", path.display()))?
        .trim()
        .to_string())
}

fn command_exists(command: &str) -> bool {
    env::var_os("PATH")
        .map(|path| env::split_paths(&path).any(|dir| dir.join(command).is_file()))
        .unwrap_or(false)
}

fn harness_detected(home: &Path, harness: &str) -> bool {
    harness_dir(home, harness).is_dir() || command_exists(harness_command(harness))
}

fn harness_dir(home: &Path, harness: &str) -> PathBuf {
    match harness {
        "antigravity-cli" => home.join(".gemini/antigravity-cli"),
        "antigravity-ide" => home.join(".gemini/antigravity-ide"),
        "antigravity" => home.join(".gemini/antigravity"),
        other => home.join(format!(".{other}")),
    }
}

fn harness_command(harness: &str) -> &str {
    match harness {
        "antigravity-cli" => "agy",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn powder_mcp_script_uses_op_run_with_keychain_bootstrap_and_toml_round_trips() {
        // harness-kit-914: consumers must resolve ~/.secrets through
        // `op run --env-file`, not a bare `source`, so an op:// reference in
        // the file resolves to its real value rather than the literal
        // reference string. The keychain bootstrap line must run first,
        // since op run itself needs OP_SERVICE_ACCOUNT_TOKEN and a sanitized
        // MCP-bootstrap context does not carry it.
        assert!(POWDER_MCP_SCRIPT.contains("op run --env-file ~/.secrets -- powder-mcp"));
        assert!(POWDER_MCP_SCRIPT.contains("OP_SERVICE_ACCOUNT_TOKEN"));
        assert!(POWDER_MCP_SCRIPT.contains("security find-generic-password"));
        assert!(!POWDER_MCP_SCRIPT.contains("source ~/.secrets"));

        // The embedded double quotes must survive a TOML round-trip
        // unmangled (codex_powder_block embeds this string as a TOML value).
        let block = codex_powder_block();
        let parsed: toml::Value = toml::from_str(&block).expect("codex_powder_block must be valid TOML");
        let args = parsed["mcp_servers"]["powder"]["args"].as_array().unwrap();
        assert_eq!(args[1].as_str().unwrap(), POWDER_MCP_SCRIPT);
    }

    #[test]
    fn claude_json_converges_powder_without_touching_other_servers() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let config = temp.path().join(".claude.json");
        fs::write(
            &config,
            r#"{"mcpServers":{"exa":{"type":"http","url":"https://example.invalid"},"powder":{"command":"old"}},"other":true}"#,
        )?;

        assert_eq!(upsert_claude_json(&config)?, Status::Updated);
        assert_eq!(upsert_claude_json(&config)?, Status::Unchanged);
        let value: Value = serde_json::from_str(&fs::read_to_string(&config)?)?;

        assert_eq!(value["other"], json!(true));
        assert_eq!(
            value["mcpServers"]["exa"]["url"],
            json!("https://example.invalid")
        );
        assert_eq!(value["mcpServers"]["powder"], claude_powder_entry());
        assert!(!fs::read_to_string(&config)?.contains("POWDER_API_KEY"));
        Ok(())
    }

    #[test]
    fn codex_toml_replaces_stale_and_duplicate_powder_tables() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let config = temp.path().join(".codex/config.toml");
        fs::create_dir_all(config.parent().unwrap())?;
        fs::write(
            &config,
            r#"[mcp_servers.unrelated]
command = "node"

[mcp_servers.powder]
command = "bash"
args = ["-lc", "old"]

[mcp_servers.powder.env]
POWDER_API_KEY = "do-not-keep"

[mcp_servers.other]
command = "python"

[mcp_servers.powder]
command = "old"
"#,
        )?;

        assert_eq!(upsert_codex_toml(&config)?, Status::Updated);
        assert_eq!(upsert_codex_toml(&config)?, Status::Unchanged);
        let text = fs::read_to_string(&config)?;

        assert!(text.contains("[mcp_servers.unrelated]\ncommand = \"node\""));
        assert!(text.contains("[mcp_servers.other]\ncommand = \"python\""));
        assert_eq!(text.matches("[mcp_servers.powder]").count(), 1);
        assert!(!text.contains("[mcp_servers.powder.env]"));
        assert!(!text.contains("do-not-keep"));
        assert!(text.contains(&codex_powder_block()));
        toml::from_str::<toml::Value>(&text)?;
        Ok(())
    }

    #[test]
    fn codex_toml_preserves_table_after_powder_when_its_header_has_a_comment() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let config = temp.path().join(".codex/config.toml");
        fs::create_dir_all(config.parent().unwrap())?;
        fs::write(
            &config,
            r#"[mcp_servers.powder]
command = "old"

[mcp_servers.keepme] # trailing comment must still terminate the powder span
command = "node"
"#,
        )?;

        assert_eq!(upsert_codex_toml(&config)?, Status::Updated);
        let text = fs::read_to_string(&config)?;

        assert!(text.contains("[mcp_servers.keepme] # trailing comment"));
        assert!(text.contains("command = \"node\""));
        assert_eq!(text.matches("[mcp_servers.powder]").count(), 1);
        assert!(text.contains(&codex_powder_block()));
        toml::from_str::<toml::Value>(&text)?;
        Ok(())
    }

    #[test]
    fn opencode_json_converges_powder_without_touching_other_servers() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let config = temp.path().join(".config/opencode/opencode.json");
        fs::create_dir_all(config.parent().unwrap())?;
        fs::write(
            &config,
            r#"{"mcp":{"exa":{"type":"remote","url":"https://example.invalid"},"powder":{"type":"local","command":["old"]}},"model":"x"}"#,
        )?;

        assert_eq!(upsert_opencode_json(&config)?, Status::Updated);
        assert_eq!(upsert_opencode_json(&config)?, Status::Unchanged);
        let value: Value = serde_json::from_str(&fs::read_to_string(&config)?)?;

        assert_eq!(value["model"], json!("x"));
        assert_eq!(value["mcp"]["exa"]["url"], json!("https://example.invalid"));
        assert_eq!(value["mcp"]["powder"], opencode_powder_entry());
        assert!(!fs::read_to_string(&config)?.contains("POWDER_API_KEY"));
        Ok(())
    }
}
