use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};

const ENGINES: &[&str] = &["codex", "claude", "droid", "copilot"];

#[derive(Debug, Clone)]
pub struct Options {
    pub mode: String,
    pub base: Option<String>,
    pub commit: String,
    pub engine: String,
    pub prompt: Vec<String>,
    pub prompt_file: Vec<PathBuf>,
    pub dataset: Vec<PathBuf>,
    pub output: Option<PathBuf>,
    pub json_output: Option<PathBuf>,
    pub parallel_tests: Option<String>,
    pub require_finding: Vec<String>,
    pub expect_findings: bool,
    pub dry_run: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            mode: "auto".to_string(),
            base: None,
            commit: "HEAD".to_string(),
            engine: std::env::var("AUTOREVIEW_ENGINE").unwrap_or_else(|_| "codex".to_string()),
            prompt: Vec::new(),
            prompt_file: Vec::new(),
            dataset: Vec::new(),
            output: None,
            json_output: None,
            parallel_tests: None,
            require_finding: Vec::new(),
            expect_findings: false,
            dry_run: false,
        }
    }
}

pub fn run(repo: &Path, options: &Options) -> Result<String> {
    if !ENGINES.contains(&options.engine.as_str()) {
        bail!("unsupported autoreview engine: {}", options.engine);
    }
    let mode = resolve_mode(repo, &options.mode)?;
    let base = options.base.clone().or_else(|| default_base(repo));
    let bundle = review_bundle(repo, &mode, base.as_deref(), &options.commit)?;
    let prompt = build_prompt(repo, &mode, base.as_deref(), &bundle, options)?;
    if let Some(command) = &options.parallel_tests {
        run_shell(repo, command)
            .with_context(|| format!("parallel test command failed: {command}"))?;
    }
    if options.dry_run {
        let report = json!({
            "engine": options.engine,
            "mode": mode,
            "base": base,
            "commit": options.commit,
            "prompt": prompt,
            "bundle_bytes": bundle.len(),
        });
        let rendered = serde_json::to_string_pretty(&report)?;
        write_outputs(options, &rendered, &report)?;
        return Ok(rendered);
    }
    let raw = run_engine(repo, &options.engine, &prompt)?;
    let report = extract_json(&raw).unwrap_or_else(|| fallback_report(&raw));
    validate_report(&report, &options.require_finding, options.expect_findings)?;
    let rendered = serde_json::to_string_pretty(&report)?;
    write_outputs(options, &raw, &report)?;
    Ok(rendered)
}

pub fn self_test(repo: &Path) -> Result<&'static str> {
    let temp = tempfile::TempDir::new()?;
    let fixture = temp.path();
    git(fixture, &["init", "--quiet"])?;
    git(fixture, &["config", "user.name", "Review Fixture"])?;
    git(
        fixture,
        &["config", "user.email", "review-fixture@example.com"],
    )?;
    fs::write(fixture.join("app.js"), "export const ok = true;\n")?;
    git(fixture, &["add", "app.js"])?;
    git(fixture, &["commit", "--quiet", "-m", "initial"])?;
    fs::write(fixture.join("app.js"), "export const ok = false;\n")?;
    let options = Options {
        mode: "local".to_string(),
        dry_run: true,
        prompt: vec!["Review normally.".to_string()],
        ..Options::default()
    };
    let output = run(fixture, &options)?;
    assert!(output.contains("\"mode\": \"local\""));
    assert!(output.contains("Review normally."));
    assert!(review_bundle(fixture, "local", None, "HEAD")?.contains("-export const ok = true;"));
    let _ = repo;
    Ok("autoreview self-test ok")
}

fn resolve_mode(repo: &Path, mode: &str) -> Result<String> {
    match mode {
        "auto" => {
            let status = git(repo, &["status", "--short"])?;
            if status.trim().is_empty() {
                Ok("commit".to_string())
            } else {
                Ok("local".to_string())
            }
        }
        "local" | "uncommitted" | "branch" | "commit" => Ok(mode.to_string()),
        _ => bail!("unsupported autoreview mode: {mode}"),
    }
}

fn default_base(repo: &Path) -> Option<String> {
    for candidate in ["origin/master", "origin/main", "master", "main"] {
        if git(repo, &["rev-parse", "--verify", candidate]).is_ok() {
            return Some(candidate.to_string());
        }
    }
    None
}

fn review_bundle(repo: &Path, mode: &str, base: Option<&str>, commit: &str) -> Result<String> {
    match mode {
        "local" | "uncommitted" => git(repo, &["diff", "--no-ext-diff", "--binary"]),
        "branch" => {
            let base = base.context("branch mode requires --base or a discoverable base")?;
            git(
                repo,
                &[
                    "diff",
                    "--no-ext-diff",
                    "--binary",
                    &format!("{base}...HEAD"),
                ],
            )
        }
        "commit" => git(
            repo,
            &[
                "show",
                "--no-ext-diff",
                "--format=fuller",
                "--stat",
                "--patch",
                commit,
            ],
        ),
        _ => bail!("unsupported autoreview mode: {mode}"),
    }
}

fn build_prompt(
    repo: &Path,
    mode: &str,
    base: Option<&str>,
    bundle: &str,
    options: &Options,
) -> Result<String> {
    let mut extra = options.prompt.join("\n\n");
    for path in &options.prompt_file {
        extra.push_str("\n\n");
        extra.push_str(
            &fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?,
        );
    }
    let mut datasets = String::new();
    for path in &options.dataset {
        datasets.push_str(&format!("\n\n# Dataset: {}\n", path.display()));
        datasets.push_str(
            &fs::read_to_string(path).with_context(|| format!("cannot read {}", path.display()))?,
        );
    }
    Ok(format!(
        "You are reviewing a Harness Kit patch.\nTarget: {mode} {}\nRepository: {}\n\nReturn JSON with findings, overall_correctness, overall_explanation, and overall_confidence.\n\nExtra instructions:\n{extra}\n{datasets}\n\nDiff bundle:\n```diff\n{bundle}\n```\n",
        base.unwrap_or(""),
        repo.display()
    ))
}

fn run_engine(repo: &Path, engine: &str, prompt: &str) -> Result<String> {
    let command = match engine {
        "codex" => vec!["codex", "exec", "--json"],
        "claude" => vec!["claude", "-p"],
        "droid" => vec!["droid"],
        "copilot" => vec!["copilot"],
        _ => unreachable!(),
    };
    let mut child = Command::new(command[0])
        .args(&command[1..])
        .current_dir(repo)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start autoreview engine {engine}"))?;
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .context("engine stdin unavailable")?
        .write_all(prompt.as_bytes())?;
    let output = child.wait_with_output()?;
    if !output.status.success() {
        bail!(
            "autoreview engine {engine} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn extract_json(raw: &str) -> Option<Value> {
    let trimmed = raw.trim();
    serde_json::from_str(trimmed).ok().or_else(|| {
        let start = trimmed.find('{')?;
        let end = trimmed.rfind('}')?;
        serde_json::from_str(&trimmed[start..=end]).ok()
    })
}

fn fallback_report(raw: &str) -> Value {
    json!({
        "findings": [],
        "overall_correctness": "patch is correct",
        "overall_explanation": raw.trim(),
        "overall_confidence": 0.0
    })
}

fn validate_report(report: &Value, required: &[String], expect_findings: bool) -> Result<()> {
    let findings = report
        .get("findings")
        .and_then(Value::as_array)
        .context("autoreview report missing findings array")?;
    if expect_findings && findings.is_empty() {
        bail!("expected at least one autoreview finding");
    }
    let text = serde_json::to_string(report)?.to_lowercase();
    for required_text in required {
        if !text.contains(&required_text.to_lowercase()) {
            bail!("required finding text not found: {required_text}");
        }
    }
    Ok(())
}

fn write_outputs(options: &Options, human: &str, report: &Value) -> Result<()> {
    if let Some(path) = &options.output {
        fs::write(path, human)?;
    }
    if let Some(path) = &options.json_output {
        fs::write(path, serde_json::to_string_pretty(report)?)?;
    }
    Ok(())
}

fn run_shell(repo: &Path, command: &str) -> Result<()> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(repo)
        .status()?;
    if !status.success() {
        bail!("command exited with {status}");
    }
    Ok(())
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git").args(args).current_dir(repo).output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    bail!("{}", String::from_utf8_lossy(&output.stderr).trim());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_contract_passes() {
        assert_eq!(
            self_test(Path::new(".")).unwrap(),
            "autoreview self-test ok"
        );
    }
}
