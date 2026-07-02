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

/// Materializes the template into a temp directory with sample tokens,
/// checks for unreplaced tokens, and builds the generated Rust workspace.
/// This is the template's own proof: an uncompiled `.tmpl` tree cannot drift
/// silently once this runs in the repo gate.
pub fn check_template_instantiates(repo: &Path) -> Result<GateReport> {
    let temp = tempfile::tempdir().context("failed to create temp directory")?;
    let written = materialize(repo, temp.path())?;
    if written.is_empty() {
        bail!("template materializer produced no files");
    }
    check_no_unreplaced_tokens(&written)?;
    build_generated_workspace(temp.path())?;
    Ok(GateReport::success(format!(
        "template instantiates and builds: {} file(s) materialized from {TEMPLATE_DIR}",
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
}
