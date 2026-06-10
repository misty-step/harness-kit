use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use crate::{bootstrap, ci_check, docs_site, frontmatter, generate_index, lint_gates};

pub fn run_pre_commit(repo: &Path) -> Result<String> {
    let staged = staged_paths(repo)?;
    let deliver_hits = deliver_state_hits(&staged);
    if !deliver_hits.is_empty() {
        bail!(
            "refusing to commit agent-written /deliver state files:\n{}\n\nThese files are rewritten by /deliver and must never be human-edited.\nIf /deliver is stuck, use:\n  /deliver --resume <ulid>\n  /deliver --abandon <ulid>",
            deliver_hits
                .iter()
                .map(|path| format!("  {path}"))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    let mut lines = Vec::new();
    let mut changed = false;
    if skill_changed(&staged) {
        changed = true;
        lines.push("[pre-commit] Skill content changed - regenerating index.yaml".to_string());
        generate_index::write_index(repo, chrono::Utc::now())?;
        git(repo, &["add", "index.yaml"])?;
    }

    if docs_input_changed(&staged) {
        changed = true;
        lines.push("[pre-commit] Docs source changed - regenerating docs/site".to_string());
        let site = docs_site::default_site(repo);
        docs_site::build_site(repo, &site)?;
        git(repo, &["add", "docs/site"])?;

        lines.push("[pre-commit] Verifying generated docs/site".to_string());
        docs_site::validate_site(&docs_site::CheckOptions {
            repo_root: repo.to_path_buf(),
            site,
            compare_to_rebuild: true,
        })?;
    }

    if changed {
        lines.push("[pre-commit] Verifying shared-root install wording".to_string());
        let report = lint_gates::check_harness_install_paths(repo)?;
        if !report.errors.is_empty() {
            bail!("{}", report.errors.join("\n"));
        }
    }

    Ok(lines.join("\n"))
}

pub fn run_pre_push(repo: &Path, pre_push_input: &str) -> Result<String> {
    if !repo.join("Cargo.toml").exists() {
        return Ok(String::new());
    }

    let mut lines = Vec::new();
    lines.extend(in_flight_deliver_warnings(repo)?);

    let mode = std::env::var("HARNESS_KIT_PRE_PUSH").unwrap_or_else(|_| "auto".to_string());
    match mode.as_str() {
        "off" => {
            lines.push("pre-push: skipping local gates (HARNESS_KIT_PRE_PUSH=off)".to_string());
            return Ok(lines.join("\n"));
        }
        "full" => {
            run_full_pre_push_gate(repo, &mut lines)?;
            return Ok(lines.join("\n"));
        }
        "fast" => {
            let changes = changed_paths_for_push(repo, pre_push_input)?;
            log_push_classification(&mut lines, &changes);
            if changes.force_full
                || (!changes.paths.is_empty() && !push_allows_fast_gate(&changes.paths))
            {
                bail!(
                    "pre-push: HARNESS_KIT_PRE_PUSH=fast refused because pushed paths require the full local gate"
                );
            }
            run_fast_pre_push_gate(repo, &mut lines, &changes.paths)?;
            return Ok(lines.join("\n"));
        }
        "auto" => {}
        other => bail!(
            "pre-push: invalid HARNESS_KIT_PRE_PUSH={other}; expected auto, full, fast, or off"
        ),
    }

    let changes = changed_paths_for_push(repo, pre_push_input)?;
    log_push_classification(&mut lines, &changes);
    if changes.paths.is_empty() {
        lines.push("pre-push: no pushed file changes detected; skipping local gates".to_string());
        return Ok(lines.join("\n"));
    }

    if !changes.force_full && push_allows_fast_gate(&changes.paths) {
        run_fast_pre_push_gate(repo, &mut lines, &changes.paths)?;
    } else {
        run_full_pre_push_gate(repo, &mut lines)?;
    }
    Ok(lines.join("\n"))
}

fn run_full_pre_push_gate(repo: &Path, lines: &mut Vec<String>) -> Result<()> {
    lines.push("pre-push: running harness-kit-checks check --repo .".to_string());
    lines.extend(ci_check::run(repo)?);
    Ok(())
}

fn run_fast_pre_push_gate(repo: &Path, lines: &mut Vec<String>, paths: &[String]) -> Result<()> {
    lines.push(format!(
        "pre-push: fast gate for {} low-risk file change(s)",
        paths.len()
    ));
    lines.push("pre-push: running check-frontmatter".to_string());
    frontmatter::check_repo(repo)?.ensure_success()?;

    lines.push("pre-push: running check-docs-site".to_string());
    docs_site::validate_site(&docs_site::CheckOptions {
        repo_root: repo.to_path_buf(),
        site: docs_site::default_site(repo),
        compare_to_rebuild: true,
    })?;
    Ok(())
}

struct PushChangeSet {
    paths: Vec<String>,
    source: String,
    force_full: bool,
}

fn changed_paths_for_push(repo: &Path, pre_push_input: &str) -> Result<PushChangeSet> {
    if let Some(changes) = changed_paths_from_pre_push_input(repo, pre_push_input)? {
        return Ok(changes);
    }

    for base in push_diff_bases(repo) {
        if git_output(repo, &["rev-parse", "--verify", "--quiet", &base]).is_err() {
            continue;
        }
        let output = git_output(repo, &["diff", "--name-only", &format!("{base}..HEAD")])?;
        return Ok(PushChangeSet {
            paths: sorted_paths(output.lines().map(str::to_string)),
            source: format!("fallback diff {base}..HEAD"),
            force_full: false,
        });
    }
    Ok(PushChangeSet {
        paths: Vec::new(),
        source: "no pre-push updates or diff base".to_string(),
        force_full: true,
    })
}

fn changed_paths_from_pre_push_input(
    repo: &Path,
    pre_push_input: &str,
) -> Result<Option<PushChangeSet>> {
    let updates = parse_pre_push_updates(pre_push_input);
    if updates.is_empty() {
        return Ok(None);
    }

    let mut paths = BTreeSet::new();
    let mut force_full = false;
    let mut sources = Vec::new();
    for update in updates {
        if is_zero_oid(&update.local_oid) {
            force_full = true;
            sources.push(format!("delete {}", update.remote_ref));
            continue;
        }
        if is_zero_oid(&update.remote_oid) {
            force_full = true;
            sources.push(format!("new {}", update.local_ref));
            continue;
        }
        let range = format!("{}..{}", update.remote_oid, update.local_oid);
        let output = git_output(repo, &["diff", "--name-only", &range])?;
        paths.extend(output.lines().map(str::to_string));
        sources.push(format!("pre-push {range}"));
    }

    Ok(Some(PushChangeSet {
        paths: paths.into_iter().collect(),
        source: sources.join(", "),
        force_full,
    }))
}

#[derive(Debug, PartialEq, Eq)]
struct PrePushUpdate {
    local_ref: String,
    local_oid: String,
    remote_ref: String,
    remote_oid: String,
}

fn parse_pre_push_updates(input: &str) -> Vec<PrePushUpdate> {
    input
        .lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace();
            let update = PrePushUpdate {
                local_ref: parts.next()?.to_string(),
                local_oid: parts.next()?.to_string(),
                remote_ref: parts.next()?.to_string(),
                remote_oid: parts.next()?.to_string(),
            };
            parts.next().is_none().then_some(update)
        })
        .collect()
}

fn is_zero_oid(oid: &str) -> bool {
    !oid.is_empty() && oid.chars().all(|character| character == '0')
}

fn sorted_paths(paths: impl Iterator<Item = String>) -> Vec<String> {
    paths.collect::<BTreeSet<_>>().into_iter().collect()
}

fn log_push_classification(lines: &mut Vec<String>, changes: &PushChangeSet) {
    lines.push(format!(
        "pre-push: classified pushed changes from {}",
        changes.source
    ));
    lines.push(format!(
        "pre-push: changed paths: {}",
        summarize_changed_paths(&changes.paths)
    ));
    if changes.paths.is_empty() {
        lines.push("pre-push: decision no-op (no changed paths)".to_string());
    } else if changes.force_full {
        lines.push("pre-push: full gate required by conservative ref classification".to_string());
    } else if push_allows_fast_gate(&changes.paths) {
        lines.push("pre-push: decision fast gate (docs/backlog allowlist only)".to_string());
    } else {
        lines.push(
            "pre-push: decision full local gate (source or harness path changed)".to_string(),
        );
    }
}

fn summarize_changed_paths(paths: &[String]) -> String {
    if paths.is_empty() {
        return "<none>".to_string();
    }
    let mut shown = paths.iter().take(12).cloned().collect::<Vec<_>>();
    if paths.len() > shown.len() {
        shown.push(format!("... and {} more", paths.len() - shown.len()));
    }
    shown.join(", ")
}

fn push_diff_bases(repo: &Path) -> Vec<String> {
    let mut bases = Vec::new();
    if let Ok(upstream) = git_output(
        repo,
        &[
            "rev-parse",
            "--abbrev-ref",
            "--symbolic-full-name",
            "@{upstream}",
        ],
    ) {
        let upstream = upstream.trim();
        if !upstream.is_empty() {
            bases.push(upstream.to_string());
        }
    }
    if let Ok(Some(branch)) = current_branch(repo) {
        bases.push(format!("origin/{branch}"));
    }
    bases.push("origin/master".to_string());
    bases.sort();
    bases.dedup();
    bases
}

fn push_allows_fast_gate(paths: &[String]) -> bool {
    !paths.is_empty() && paths.iter().all(|path| path_allows_fast_gate(path))
}

fn path_allows_fast_gate(path: &str) -> bool {
    if path.starts_with("backlog.d/") && path.ends_with(".md") {
        return true;
    }
    if path.starts_with("docs/site/") || path.starts_with("docs/copy/") {
        return true;
    }
    if path.starts_with("docs/") {
        return path.ends_with(".md")
            || path.ends_with(".json")
            || path.ends_with(".svg")
            || path.ends_with(".html")
            || path.ends_with(".txt")
            || path.ends_with(".css");
    }
    matches!(path, "README.md" | "CHANGELOG.md")
}

pub fn run_pre_merge_commit(repo: &Path) -> Result<String> {
    run_pre_merge_commit_with_env(repo, env_map())
}

pub fn run_post_commit(repo: &Path) -> Result<String> {
    let changed = git_output(repo, &["diff", "--name-only", "HEAD~1", "HEAD"]).unwrap_or_default();
    if !bootstrap_relevant_changed(changed.lines()) {
        return Ok(String::new());
    }
    let output = bootstrap::run(&bootstrap::BootstrapOptions::from_env(Some(
        repo.to_path_buf(),
    ))?)?;
    Ok(format!(
        "[post-commit] Harness Kit harness content changed - re-linking\n{output}"
    ))
}

pub fn run_post_merge(repo: &Path) -> Result<String> {
    bootstrap::run(&bootstrap::BootstrapOptions::from_env(Some(
        repo.to_path_buf(),
    ))?)
}

pub fn run_post_rewrite(repo: &Path, args: &[String]) -> Result<String> {
    if args.first().map(String::as_str) != Some("rebase") {
        return Ok(String::new());
    }
    run_post_merge(repo)
}

fn run_pre_merge_commit_with_env(repo: &Path, env: BTreeMap<String, String>) -> Result<String> {
    let mut lines = Vec::new();
    match current_branch(repo)?.as_deref() {
        Some("master" | "main") => {}
        _ => return Ok(lines.join("\n")),
    }
    if !repo.join("Cargo.toml").exists() {
        return Ok(lines.join("\n"));
    }
    if env.get("HARNESS_KIT_NO_CI").map(String::as_str) == Some("1") {
        lines.push("pre-merge-commit: bypassing CI gate (HARNESS_KIT_NO_CI=1)".to_string());
        return Ok(lines.join("\n"));
    }

    lines.push("pre-merge-commit: running harness-kit-checks check --repo .".to_string());
    lines.extend(ci_check::run(repo)?);
    Ok(lines.join("\n"))
}

fn staged_paths(repo: &Path) -> Result<Vec<String>> {
    let output = git_output(repo, &["diff", "--cached", "--name-only"])?;
    Ok(output.lines().map(str::to_string).collect())
}

fn deliver_state_hits(paths: &[String]) -> Vec<String> {
    paths
        .iter()
        .filter(|path| {
            path.starts_with(".harness-kit/deliver/")
                && (path.ends_with("/state.json") || path.ends_with("/receipt.json"))
        })
        .cloned()
        .collect()
}

fn skill_changed(paths: &[String]) -> bool {
    paths.iter().any(|path| {
        (path.starts_with("skills/") && path.ends_with("/SKILL.md"))
            || (path.starts_with("agents/") && path.ends_with(".md"))
    })
}

fn docs_input_changed(paths: &[String]) -> bool {
    paths.iter().any(|path| {
        (path.starts_with("skills/") && path.ends_with("/SKILL.md"))
            || (path.starts_with("agents/") && path.ends_with(".md"))
            || path == "AGENTS.md"
            || path == "harnesses/shared/AGENTS.md"
            || path.starts_with("docs/copy/")
            || path == "docs/positioning.md"
            || (path.starts_with("backlog.d/") && path.ends_with(".md"))
            || path == "bootstrap.sh"
            || path == "crates/harness-kit-checks/src/ci_check.rs"
            || path.starts_with("crates/harness-kit-checks/src/docs_site.rs")
            || path.starts_with("crates/harness-kit-checks/src/generate_index.rs")
    })
}

fn bootstrap_relevant_changed<'a>(paths: impl IntoIterator<Item = &'a str>) -> bool {
    paths.into_iter().any(|path| {
        path.starts_with("skills/")
            || path.starts_with("agents/")
            || path.starts_with("harnesses/")
            || path.starts_with(".githooks/")
            || path == "bootstrap.sh"
    })
}

fn in_flight_deliver_warnings(repo: &Path) -> Result<Vec<String>> {
    let root = repo.join(".harness-kit/deliver");
    if !root.is_dir() {
        return Ok(Vec::new());
    }
    let mut warnings = Vec::new();
    for state in state_files(&root)? {
        let Ok(text) = fs::read_to_string(&state) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            continue;
        };
        let Some(status) = value.get("status").and_then(Value::as_str) else {
            continue;
        };
        if status == "merge_ready" {
            continue;
        }
        let ulid = state
            .parent()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        warnings.push(format!(
            "pre-push: in-flight /deliver run detected (ulid={ulid}, status={status})"
        ));
        warnings.push(format!("  -> /deliver --resume {ulid}    # finish the run"));
        warnings.push(format!(
            "  -> /deliver --abandon {ulid}   # clear state, keep branch"
        ));
    }
    Ok(warnings)
}

fn state_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    for run in fs::read_dir(root).with_context(|| format!("failed to read {}", root.display()))? {
        let run = run?;
        if !run.file_type()?.is_dir() {
            continue;
        }
        let state = run.path().join("state.json");
        if state.is_file() {
            files.push(state);
        }
    }
    files.sort();
    Ok(files)
}

fn current_branch(repo: &Path) -> Result<Option<String>> {
    match git_output(repo, &["symbolic-ref", "--quiet", "--short", "HEAD"]) {
        Ok(output) => Ok(Some(output.trim().to_string()).filter(|branch| !branch.is_empty())),
        Err(_) => Ok(None),
    }
}

fn env_map() -> BTreeMap<String, String> {
    std::env::vars().collect()
}

fn git(repo: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .args(args)
        .current_dir(repo)
        .status()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !status.success() {
        bail!("git {} failed with {status}", args.join(" "));
    }
    Ok(())
}

fn git_output(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn detects_deliver_state_files() {
        let paths = vec![
            ".harness-kit/deliver/01/state.json".to_string(),
            ".harness-kit/deliver/01/receipt.json".to_string(),
            ".harness-kit/traces/delegations.jsonl".to_string(),
        ];
        assert_eq!(
            deliver_state_hits(&paths),
            vec![
                ".harness-kit/deliver/01/state.json".to_string(),
                ".harness-kit/deliver/01/receipt.json".to_string()
            ]
        );
    }

    #[test]
    fn classifies_index_and_docs_inputs() {
        assert!(skill_changed(&["skills/ci/SKILL.md".to_string()]));
        assert!(skill_changed(&["agents/a11y-auditor.md".to_string()]));
        assert!(!skill_changed(&[
            "skills/ci/references/audit.md".to_string()
        ]));

        assert!(docs_input_changed(
            &["backlog.d/090-example.md".to_string()]
        ));
        assert!(docs_input_changed(&[
            "crates/harness-kit-checks/src/docs_site.rs".to_string()
        ]));
        assert!(docs_input_changed(&[
            "crates/harness-kit-checks/src/ci_check.rs".to_string()
        ]));
        assert!(!docs_input_changed(&["README.md".to_string()]));
    }

    #[test]
    fn detects_bootstrap_relevant_changes() {
        assert!(bootstrap_relevant_changed(["skills/ci/SKILL.md"]));
        assert!(bootstrap_relevant_changed(["harnesses/shared/AGENTS.md"]));
        assert!(bootstrap_relevant_changed([".githooks/pre-push"]));
        assert!(bootstrap_relevant_changed(["bootstrap.sh"]));
        assert!(!bootstrap_relevant_changed(["README.md"]));
    }

    #[test]
    fn classifies_pre_push_fast_gate_paths() {
        assert!(push_allows_fast_gate(&[
            "backlog.d/102-ci-dx-fast-path-and-dagger-coarsening.md".to_string(),
            "docs/site/manifest.json".to_string(),
            "docs/positioning.md".to_string(),
        ]));
        assert!(push_allows_fast_gate(&["README.md".to_string()]));

        assert!(!push_allows_fast_gate(&["skills/ci/SKILL.md".to_string()]));
        assert!(!push_allows_fast_gate(&[
            "crates/harness-kit-checks/src/git_hooks.rs".to_string()
        ]));
        assert!(!push_allows_fast_gate(&[".githooks/pre-push".to_string()]));
        assert!(!push_allows_fast_gate(&[
            "crates/harness-kit-checks/src/ci_check.rs".to_string()
        ]));
        assert!(!push_allows_fast_gate(&[]));
    }

    #[test]
    fn parses_pre_push_updates_from_stdin() {
        assert_eq!(
            parse_pre_push_updates(
                "refs/heads/master abc123 refs/heads/master def456\n\
                 refs/heads/topic 111 refs/heads/topic 000000\n"
            ),
            vec![
                PrePushUpdate {
                    local_ref: "refs/heads/master".to_string(),
                    local_oid: "abc123".to_string(),
                    remote_ref: "refs/heads/master".to_string(),
                    remote_oid: "def456".to_string(),
                },
                PrePushUpdate {
                    local_ref: "refs/heads/topic".to_string(),
                    local_oid: "111".to_string(),
                    remote_ref: "refs/heads/topic".to_string(),
                    remote_oid: "000000".to_string(),
                },
            ]
        );
        assert!(parse_pre_push_updates("malformed line").is_empty());
        assert!(is_zero_oid("0000000000"));
        assert!(!is_zero_oid("abc000"));
    }

    #[test]
    fn summarizes_push_classification_decision() {
        let changes = PushChangeSet {
            paths: vec![
                "backlog.d/102-ci-dx-fast-path-and-dagger-coarsening.md".to_string(),
                "crates/harness-kit-checks/src/git_hooks.rs".to_string(),
            ],
            source: "pre-push old..new".to_string(),
            force_full: false,
        };
        let mut lines = Vec::new();
        log_push_classification(&mut lines, &changes);
        assert!(lines.iter().any(|line| line.contains("pre-push old..new")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("crates/harness-kit-checks/src/git_hooks.rs"))
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("decision full local gate"))
        );
    }

    #[test]
    fn reads_in_flight_deliver_warnings_without_python() {
        let temp = TempDir::new().unwrap();
        let deliver = temp.path().join(".harness-kit/deliver");
        fs::create_dir_all(deliver.join("01")).unwrap();
        fs::create_dir_all(deliver.join("02")).unwrap();
        fs::write(deliver.join("01/state.json"), r#"{"status":"running"}"#).unwrap();
        fs::write(deliver.join("02/state.json"), r#"{"status":"merge_ready"}"#).unwrap();

        let warnings = in_flight_deliver_warnings(temp.path()).unwrap();
        assert_eq!(
            warnings,
            vec![
                "pre-push: in-flight /deliver run detected (ulid=01, status=running)".to_string(),
                "  -> /deliver --resume 01    # finish the run".to_string(),
                "  -> /deliver --abandon 01   # clear state, keep branch".to_string()
            ]
        );
    }
}
