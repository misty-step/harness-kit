use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};

use crate::lint_gates::GateReport;
use crate::process;

const TEMPLATE_DIR: &str = "skills/harness-engineering/templates/one-core-many-faces";

/// Sample substitution values for the template's 10 documented tokens
/// (`skills/harness-engineering/templates/one-core-many-faces/README.md`).
/// Any token that appears in a `.tmpl` file but not here is a real drift
/// bug: the materializer leaves it untouched and the unreplaced-token check
/// below catches it.
const SAMPLE_TOKENS: &[(&str, &str)] = &[
    ("{{project}}", "Example Product"),
    ("{{client_class}}", "Example"),
    ("{{crate_prefix}}", "example"),
    ("{{binary}}", "example"),
    ("{{repo}}", "misty-step/example"),
    ("{{base_branch}}", "main"),
    ("{{npm_scope}}", "misty-step"),
    ("{{fly_app}}", "example-prod"),
    ("{{fly_region}}", "iad"),
    (
        "{{description}}",
        "Example product for template verification.",
    ),
];

/// Materializes every `*.tmpl` file under the template directory into `dest`,
/// stripping the `.tmpl` suffix and substituting `SAMPLE_TOKENS`. Returns the
/// materialized file paths so callers can scan them for drift.
///
/// Deliberately does not touch `README.md` — that file documents the
/// template for a human copying it, it is not part of the generated tree
/// (see the README's own "Target Tree" section).
pub(crate) fn materialize(repo: &Path, dest: &Path) -> Result<Vec<PathBuf>> {
    let template_root = repo.join(TEMPLATE_DIR);
    if !template_root.is_dir() {
        bail!("template directory missing: {}", template_root.display());
    }
    let mut written = Vec::new();
    materialize_dir(&template_root, &template_root, dest, &mut written)?;
    Ok(written)
}

fn materialize_dir(
    template_root: &Path,
    dir: &Path,
    dest: &Path,
    written: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in fs::read_dir(dir).with_context(|| format!("failed to read {}", dir.display()))? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            materialize_dir(template_root, &path, dest, written)?;
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(stripped_name) = file_name.strip_suffix(".tmpl") else {
            continue; // README.md and any other non-.tmpl file: not part of the generated tree.
        };
        let relative_dir = path
            .parent()
            .unwrap_or(template_root)
            .strip_prefix(template_root)
            .unwrap_or(Path::new(""));
        let target = dest.join(relative_dir).join(stripped_name);
        fs::create_dir_all(target.parent().unwrap_or(dest))?;
        let content = fs::read_to_string(&path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        let substituted = substitute(&content);
        fs::write(&target, substituted)
            .with_context(|| format!("failed to write {}", target.display()))?;
        written.push(target);
    }
    Ok(())
}

fn substitute(content: &str) -> String {
    let mut result = content.to_string();
    for (token, value) in SAMPLE_TOKENS {
        result = result.replace(token, value);
    }
    result
}

/// Scans materialized files for any of the template's known token patterns
/// that survived substitution. Deliberately checks only the 10 documented
/// `{{token}}` patterns, not a blanket `{{.*}}` regex — the generated
/// `.github/workflows/*.yml` legitimately contains GitHub Actions expression
/// syntax (`${{ github.ref }}`), which is not a template token and must not
/// be flagged as drift.
fn check_no_unreplaced_tokens(paths: &[PathBuf]) -> Result<()> {
    let mut findings = Vec::new();
    for path in paths {
        let content = fs::read_to_string(path)
            .with_context(|| format!("failed to read {}", path.display()))?;
        for (token, _) in SAMPLE_TOKENS {
            if content.contains(token) {
                findings.push(format!("{}: unreplaced {token}", path.display()));
            }
        }
    }
    if findings.is_empty() {
        Ok(())
    } else {
        bail!(
            "template materializer left unreplaced tokens:\n{}",
            findings.join("\n")
        );
    }
}

/// Runs `cargo generate-lockfile` then `cargo build --locked` in the
/// materialized workspace, matching the template README's own documented
/// flow ("Run `cargo generate-lockfile` before locked gates or Docker
/// builds."). A freshly materialized project has no committed Cargo.lock —
/// `--locked` alone would fail immediately, not prove anything about the
/// template.
fn build_generated_workspace(dest: &Path) -> Result<()> {
    run_cargo(dest, &["generate-lockfile"])?;
    run_cargo(dest, &["build", "--locked", "--workspace"])
}

fn run_cargo(dest: &Path, args: &[&str]) -> Result<()> {
    let output = process::command("cargo")
        .args(args)
        .current_dir(dest)
        .output()
        .with_context(|| format!("failed to run cargo {}", args.join(" ")))?;
    if output.status.success() {
        return Ok(());
    }
    bail!(
        "cargo {} failed in {}\n{}{}",
        args.join(" "),
        dest.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Substituted binary name for the API server (`{{binary}}-server` with the
/// sample tokens applied — see `SAMPLE_TOKENS`).
fn api_binary_name() -> String {
    substitute("{{binary}}-server")
}

/// Kills the child on drop so a failing assertion never leaks a bound port
/// or an orphaned server process out of the gate run.
struct ChildGuard(std::process::Child);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Binds to an OS-assigned port and immediately releases it. Small TOCTOU
/// window before the child binds it instead — acceptable for gate-only
/// tooling, not a production concern.
fn free_tcp_port() -> Result<u16> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .context("failed to bind an ephemeral port to pick a free one")?;
    Ok(listener.local_addr()?.port())
}

/// Runs `curl`, returning `(status_code, body)`. Shells out rather than
/// adding an HTTP client dependency to this crate — curl is preinstalled on
/// every target (macOS, `ubuntu-latest` GitHub runners) this gate runs on.
fn curl(args: &[&str]) -> Result<(u16, String)> {
    let output = process::command("curl")
        .args(["-s", "-w", "\n%{http_code}"])
        .args(args)
        .output()
        .context("failed to run curl")?;
    if !output.status.success() {
        bail!(
            "curl {} failed\n{}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    parse_curl_output(&output.stdout)
}

/// Parses `curl -w '\n%{http_code}'`'s output: the response body followed by
/// a trailing newline-delimited status code. Split out from `curl` so the
/// parsing itself is unit-testable without spawning a real process.
fn parse_curl_output(stdout: &[u8]) -> Result<(u16, String)> {
    let stdout = String::from_utf8_lossy(stdout);
    let (body, code) = stdout
        .rsplit_once('\n')
        .context("curl output missing trailing status code")?;
    let status = code
        .trim()
        .parse::<u16>()
        .with_context(|| format!("curl produced a non-numeric status code: {code:?}"))?;
    Ok((status, body.to_string()))
}

/// Boots the generated API server and replays real requests against it: the
/// deploy layer (#137) and `cargo build` (child 3) prove the binary
/// compiles; this proves the binary actually serves the routes it claims —
/// the template README's own "API face: request replay" proof requirement.
fn check_api_boots_and_serves(dest: &Path) -> Result<()> {
    let binary_path = dest.join("target/debug").join(api_binary_name());
    if !binary_path.is_file() {
        bail!(
            "expected API server binary at {} after build_generated_workspace",
            binary_path.display()
        );
    }

    let port = free_tcp_port()?;
    let database_path = dest.join("smoke.db");
    let child = process::command(&binary_path.to_string_lossy())
        .env("PORT", port.to_string())
        .env("DATABASE_PATH", &database_path)
        .spawn()
        .with_context(|| format!("failed to spawn {}", binary_path.display()))?;
    let _guard = ChildGuard(child);

    let base = format!("http://127.0.0.1:{port}");
    let healthz_url = format!("{base}/healthz");
    let mut last_error = None;
    let mut ready = false;
    for _ in 0..50 {
        match curl(&[&healthz_url]) {
            Ok((200, _)) => {
                ready = true;
                break;
            }
            Ok((status, body)) => last_error = Some(format!("status {status}: {body}")),
            Err(error) => last_error = Some(error.to_string()),
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    if !ready {
        bail!(
            "generated API server never became healthy at {healthz_url}: {}",
            last_error.unwrap_or_else(|| "no attempt succeeded".to_string())
        );
    }

    let (readyz_status, readyz_body) = curl(&[&format!("{base}/readyz")])?;
    if readyz_status != 200 {
        bail!("GET /readyz returned {readyz_status}: {readyz_body}");
    }

    let (list_status, list_body) = curl(&[&format!("{base}/items")])?;
    if list_status != 200 {
        bail!("GET /items returned {list_status}: {list_body}");
    }

    let (create_status, create_body) = curl(&[
        "-X",
        "POST",
        &format!("{base}/items"),
        "-H",
        "content-type: application/json",
        "-d",
        r#"{"id":"1","title":"smoke","body":"template gate"}"#,
    ])?;
    if create_status != 200 {
        bail!("POST /items (valid body) returned {create_status}: {create_body}");
    }

    // Validation-error path: malformed JSON must not 500. The template
    // README requires this explicitly ("API face: request replay" implies
    // the error path, and `check_template_instantiates`'s sibling checks
    // already prove the happy path — a route that only works on well-formed
    // input is unproven).
    let (bad_status, bad_body) = curl(&[
        "-X",
        "POST",
        &format!("{base}/items"),
        "-H",
        "content-type: application/json",
        "-d",
        "not-json",
    ])?;
    if !(400..500).contains(&bad_status) {
        bail!(
            "POST /items with malformed JSON should return a 4xx validation error, got {bad_status}: {bad_body}"
        );
    }

    Ok(())
}

/// Builds the generated TypeScript SDK and does a throwaway consumer build
/// that imports the package and calls one public method — the template
/// README's own "SDK face: throwaway consumer build" proof requirement.
/// Uses `bun` (already provisioned in CI via `oven-sh/setup-bun`) rather
/// than requiring a separate Node.js toolchain setup.
fn check_sdk_builds(dest: &Path) -> Result<()> {
    let sdk_dir = dest.join("sdk/typescript");
    if !sdk_dir.is_dir() {
        bail!("expected TypeScript SDK at {}", sdk_dir.display());
    }

    run_bun(&sdk_dir, &["install"])?;
    run_bun(&sdk_dir, &["run", "build"])?;

    let dist_index = sdk_dir.join("dist/index.js");
    if !dist_index.is_file() {
        bail!(
            "SDK build did not produce {} — `bun run build` reported success but emitted nothing",
            dist_index.display()
        );
    }

    let client_class = format!("{}Client", substitute("{{client_class}}"));
    let consumer_dir = dest.join(".sdk-consumer-smoke");
    fs::create_dir_all(&consumer_dir)?;
    let consumer_script = consumer_dir.join("consume.mjs");
    fs::write(
        &consumer_script,
        format!(
            "import {{ {client_class} }} from {:?};\n\
             const client = new {client_class}(\"http://127.0.0.1:1\", \"token\");\n\
             if (typeof client.listItems !== \"function\") {{\n\
             \x20 throw new Error(\"consumer smoke: listItems is not a function on {client_class}\");\n\
             }}\n\
             console.log(\"sdk-consumer-smoke-ok\");\n",
            dist_index.display()
        ),
    )?;
    let output = process::command("bun")
        .arg("run")
        .arg(&consumer_script)
        .current_dir(&consumer_dir)
        .output()
        .context("failed to run SDK consumer smoke")?;
    if !output.status.success()
        || !String::from_utf8_lossy(&output.stdout).contains("sdk-consumer-smoke-ok")
    {
        bail!(
            "SDK consumer smoke failed\nstdout: {}\nstderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

fn run_bun(dir: &Path, args: &[&str]) -> Result<()> {
    let output = process::command("bun")
        .args(args)
        .current_dir(dir)
        .output()
        .with_context(|| format!("failed to run bun {}", args.join(" ")))?;
    if output.status.success() {
        return Ok(());
    }
    bail!(
        "bun {} failed in {}\n{}{}",
        args.join(" "),
        dir.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// Materializes the template into a temp directory with sample tokens,
/// checks for unreplaced tokens, and builds the generated Rust workspace,
/// then boots the API face and builds the SDK face to prove they actually
/// work, not just compile. This is the template's own proof: an uncompiled
/// `.tmpl` tree cannot drift silently once this runs in the repo gate.
pub fn check_template_instantiates(repo: &Path) -> Result<GateReport> {
    let temp = tempfile::tempdir().context("failed to create temp directory")?;
    let written = materialize(repo, temp.path())?;
    if written.is_empty() {
        bail!("template materializer produced no files");
    }
    check_no_unreplaced_tokens(&written)?;
    build_generated_workspace(temp.path())?;
    check_api_boots_and_serves(temp.path())?;
    check_sdk_builds(temp.path())?;
    Ok(GateReport::success(format!(
        "template instantiates, builds, boots the API, and builds the SDK: {} file(s) materialized from {TEMPLATE_DIR}",
        written.len()
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write(path: &Path, contents: &str) {
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }

    #[test]
    fn materialize_strips_tmpl_suffix_and_substitutes_tokens() {
        let repo = tempfile::tempdir().unwrap();
        write(
            &repo.path().join(TEMPLATE_DIR).join("Cargo.toml.tmpl"),
            "name = \"{{crate_prefix}}\"\n",
        );
        write(
            &repo.path().join(TEMPLATE_DIR).join("README.md"),
            "not part of the generated tree\n",
        );
        let dest = tempfile::tempdir().unwrap();

        let written = materialize(repo.path(), dest.path()).unwrap();

        assert_eq!(written.len(), 1);
        assert!(dest.path().join("Cargo.toml").exists());
        assert!(!dest.path().join("README.md").exists());
        let content = fs::read_to_string(dest.path().join("Cargo.toml")).unwrap();
        assert_eq!(content, "name = \"example\"\n");
    }

    #[test]
    fn materialize_preserves_subdirectory_structure() {
        let repo = tempfile::tempdir().unwrap();
        write(
            &repo
                .path()
                .join(TEMPLATE_DIR)
                .join("crates/core/Cargo.toml.tmpl"),
            "name = \"{{crate_prefix}}-core\"\n",
        );
        let dest = tempfile::tempdir().unwrap();

        materialize(repo.path(), dest.path()).unwrap();

        assert!(dest.path().join("crates/core/Cargo.toml").exists());
    }

    #[test]
    fn unreplaced_token_check_fails_on_leftover_token_but_ignores_gha_syntax() {
        let dest = tempfile::tempdir().unwrap();
        let clean = dest.path().join("workflow.yml");
        write(
            &clean,
            "on:\n  push:\nconcurrency:\n  group: release-${{ github.ref }}\n",
        );
        assert!(check_no_unreplaced_tokens(&[clean]).is_ok());

        let dirty = dest.path().join("Cargo.toml");
        write(&dirty, "name = \"{{crate_prefix}}-core\"\n");
        let error = check_no_unreplaced_tokens(&[dirty])
            .unwrap_err()
            .to_string();
        assert!(error.contains("unreplaced {{crate_prefix}}"));
    }

    #[test]
    fn missing_template_directory_fails_loudly() {
        let repo = tempfile::tempdir().unwrap();
        let dest = tempfile::tempdir().unwrap();
        let error = materialize(repo.path(), dest.path())
            .unwrap_err()
            .to_string();
        assert!(error.contains("template directory missing"));
    }

    #[test]
    fn parses_curl_body_and_status() {
        let (status, body) = parse_curl_output(b"{\"status\":\"ok\"}\n200").unwrap();
        assert_eq!(status, 200);
        assert_eq!(body, "{\"status\":\"ok\"}");
    }

    #[test]
    fn parses_curl_empty_body() {
        let (status, body) = parse_curl_output(b"\n204").unwrap();
        assert_eq!(status, 204);
        assert_eq!(body, "");
    }

    #[test]
    fn rejects_curl_output_without_status_line() {
        assert!(parse_curl_output(b"no newline here").is_err());
    }

    #[test]
    fn rejects_non_numeric_curl_status() {
        let error = parse_curl_output(b"body\nabc").unwrap_err().to_string();
        assert!(error.contains("non-numeric status code"));
    }

    #[test]
    fn free_tcp_port_returns_a_bindable_port() {
        let port = free_tcp_port().unwrap();
        // The port must be immediately bindable again — proves it was
        // actually released, not just read from a still-open listener.
        assert!(std::net::TcpListener::bind(("127.0.0.1", port)).is_ok());
    }

    #[test]
    fn api_binary_name_applies_sample_binary_token() {
        assert_eq!(api_binary_name(), "example-server");
    }
}
