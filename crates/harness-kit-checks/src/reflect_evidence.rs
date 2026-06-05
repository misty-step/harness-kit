use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Result, bail};
use regex::Regex;

pub fn gather(repo_root: &Path, commit_count: usize) -> String {
    let mut output = String::new();
    output.push_str("## Recent Commits\n");
    output.push_str(&git_or_fallback(
        repo_root,
        &["log", "--oneline", &format!("-{commit_count}")],
        "(not in a git repo)",
    ));
    output.push_str("\n\n## Changed Files\n");
    output.push_str(&git_or_fallback(
        repo_root,
        &["diff", "--stat", &format!("HEAD~{commit_count}")],
        "(no git history)",
    ));
    output.push_str("\n\n## Uncommitted\n");
    output.push_str(&git_or_fallback(
        repo_root,
        &["status", "--short"],
        "(not in a git repo)",
    ));
    output.push_str("\n\n## Environment Hints\n");
    output.push_str("### .env files (existence only, no values)\n");
    for path in env_files(repo_root).into_iter().take(10) {
        output.push_str(&format!("{}\n", path.display()));
    }
    output.push_str("\n### Dev server config\n");
    for path in dev_server_configs(repo_root).into_iter().take(5) {
        output.push_str(&format!("{}\n", path.display()));
    }
    output
}

pub fn parse_commit_count(args: &[String]) -> Result<usize> {
    match args {
        [] => Ok(10),
        [value] => value.parse().map_err(Into::into),
        _ => bail!("usage: reflect-gather-evidence [N]"),
    }
}

fn git_or_fallback(repo_root: &Path, args: &[&str], fallback: &str) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output();
    match output {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.trim_end().to_string()
        }
        _ => fallback.to_string(),
    }
}

fn env_files(repo_root: &Path) -> Vec<PathBuf> {
    let mut matches = Vec::new();
    collect_env_files(repo_root, repo_root, 0, &mut matches);
    matches.sort();
    matches
        .into_iter()
        .map(|path| relative(repo_root, &path))
        .collect()
}

fn collect_env_files(root: &Path, dir: &Path, depth: usize, matches: &mut Vec<PathBuf>) {
    if depth > 2 || dir.file_name().and_then(|name| name.to_str()) == Some("node_modules") {
        return;
    }
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        if metadata.is_dir() {
            collect_env_files(root, &path, depth + 1, matches);
        } else if metadata.is_file()
            && path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(".env"))
            && path.starts_with(root)
        {
            matches.push(path);
        }
    }
}

fn dev_server_configs(repo_root: &Path) -> Vec<PathBuf> {
    let pattern = Regex::new(r#"dev.*server|"dev""#).unwrap();
    let mut candidates = Vec::new();
    candidates.push(repo_root.join("package.json"));
    if let Ok(entries) = fs::read_dir(repo_root) {
        for entry in entries.flatten() {
            let path = entry.path().join("package.json");
            if path.is_file() {
                candidates.push(path);
            }
        }
    }
    candidates
        .into_iter()
        .filter(|path| {
            fs::read_to_string(path)
                .map(|text| pattern.is_match(&text))
                .unwrap_or(false)
        })
        .map(|path| relative(repo_root, &path))
        .collect()
}

fn relative(root: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(root).unwrap_or(path).to_path_buf()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn gathers_fallback_sections_outside_git_repo() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join(".env.local"), "SECRET=hidden").unwrap();
        fs::write(
            temp.path().join("package.json"),
            r#"{"scripts":{"dev":"vite"}}"#,
        )
        .unwrap();

        let output = gather(temp.path(), 3);

        assert!(output.contains("## Recent Commits\n(not in a git repo)"));
        assert!(output.contains("## Changed Files\n(no git history)"));
        assert!(output.contains(".env.local"));
        assert!(!output.contains("SECRET=hidden"));
        assert!(output.contains("package.json"));
    }

    #[test]
    fn rejects_extra_arguments() {
        let error = parse_commit_count(&["1".to_string(), "2".to_string()]).unwrap_err();

        assert!(error.to_string().contains("usage: reflect-gather-evidence"));
    }
}
