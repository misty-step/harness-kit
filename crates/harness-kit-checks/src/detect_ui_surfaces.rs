use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectMode {
    Staged,
    Unstaged,
    Base(String),
    Paths(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiSurfaceReport {
    pub ui_surface: bool,
    pub mode: String,
    pub matches: Vec<String>,
}

pub fn detect(repo_root: &Path, mode: &DetectMode) -> Result<UiSurfaceReport> {
    let paths = collect_paths(repo_root, mode)?;
    let matches = paths
        .into_iter()
        .filter(|path| path_matches_ui_surface(path))
        .collect::<Vec<_>>();
    Ok(UiSurfaceReport {
        ui_surface: !matches.is_empty(),
        mode: mode_name(mode).to_string(),
        matches,
    })
}

pub fn collect_paths(repo_root: &Path, mode: &DetectMode) -> Result<Vec<String>> {
    match mode {
        DetectMode::Paths(paths) => Ok(paths.clone()),
        DetectMode::Staged => git_diff_names(
            repo_root,
            &["diff", "--cached", "--name-only", "--diff-filter=ACMR"],
        ),
        DetectMode::Unstaged => {
            git_diff_names(repo_root, &["diff", "--name-only", "--diff-filter=ACMR"])
        }
        DetectMode::Base(base) => {
            let range = format!("{base}...HEAD");
            git_diff_names(
                repo_root,
                &["diff", "--name-only", "--diff-filter=ACMR", &range],
            )
        }
    }
}

pub fn path_matches_ui_surface(path: &str) -> bool {
    let extension_match = [
        ".tsx", ".jsx", ".vue", ".svelte", ".css", ".scss", ".sass", ".less", ".html", ".mdx",
    ]
    .iter()
    .any(|extension| path.ends_with(extension));
    if extension_match {
        return true;
    }
    let file_name = Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or(path);
    if file_name.starts_with("tailwind.config.")
        || file_name == "components.json"
        || file_name.starts_with("tokens.")
        || file_name.starts_with("theme.")
    {
        return true;
    }
    path.starts_with("app/")
        || path.starts_with("pages/")
        || path.starts_with("components/")
        || path.starts_with("src/components/")
        || path.starts_with("stories/")
        || path.contains(".stories.")
        || path.contains(".story.")
}

pub fn mode_name(mode: &DetectMode) -> &'static str {
    match mode {
        DetectMode::Staged => "staged",
        DetectMode::Unstaged => "unstaged",
        DetectMode::Base(_) => "base",
        DetectMode::Paths(_) => "paths",
    }
}

fn git_diff_names(repo_root: &Path, args: &[&str]) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo_root)
        .output()?;
    if !output.status.success() {
        bail!("failed to collect paths for mode");
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn matches_ui_surface_paths() {
        for path in [
            "app/page.tsx",
            "components/Button.rs",
            "src/components/Card.rs",
            "theme.ts",
            "styles/site.css",
            "stories/Button.story.tsx",
            "docs/example.mdx",
        ] {
            assert!(path_matches_ui_surface(path), "{path}");
        }
        assert!(!path_matches_ui_surface("crates/lib/src/main.rs"));
    }

    #[test]
    fn detects_explicit_paths_as_json_report() {
        let report = detect(
            Path::new("."),
            &DetectMode::Paths(vec![
                "crates/lib/src/main.rs".to_string(),
                "components/Button.rs".to_string(),
            ]),
        )
        .unwrap();

        assert!(report.ui_surface);
        assert_eq!(report.mode, "paths");
        assert_eq!(report.matches, vec!["components/Button.rs"]);
        let json = serde_json::to_string(&report).unwrap();
        assert_eq!(
            json,
            r#"{"ui_surface":true,"mode":"paths","matches":["components/Button.rs"]}"#
        );
    }
}
