use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use chrono::Utc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreflightLine {
    Info(String),
    Ok(String),
    Warn(String),
    Fail(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightReport {
    pub lines: Vec<PreflightLine>,
    pub fail_count: usize,
    pub warn_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightOptions {
    pub repo_root: PathBuf,
    pub path_env: OsString,
}

impl PreflightReport {
    pub fn success(&self) -> bool {
        self.fail_count == 0
    }
}

pub fn discover_repo_root(cwd: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()
        .with_context(|| "failed to run git rev-parse --show-toplevel")?;
    if !output.status.success() {
        bail!("not inside a git repository");
    }
    let root = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if root.is_empty() {
        bail!("not inside a git repository");
    }
    Ok(PathBuf::from(root))
}

pub fn run(options: &PreflightOptions) -> Result<PreflightReport> {
    let mut builder = PreflightBuilder::default();
    let root = options.repo_root.canonicalize().with_context(|| {
        format!(
            "failed to canonicalize repo root {}",
            options.repo_root.display()
        )
    })?;
    let branch = git_branch(&root).unwrap_or_else(|_| "HEAD".to_string());
    builder.info(format!("repo: {}", root.display()));
    builder.info(format!("branch: {branch}"));

    check_file(
        &mut builder,
        &root,
        "crates/harness-kit-checks/src/evidence.rs",
        "evidence helper",
    );
    check_file(
        &mut builder,
        &root,
        "crates/harness-kit-checks/src/verdicts.rs",
        "verdict helper",
    );
    check_executable(
        &mut builder,
        &root,
        ".githooks/pre-merge-commit",
        "pre-merge hook",
    );
    check_lfs_attrs(&mut builder, &root);
    if root
        .join("crates/harness-kit-checks/src/evidence.rs")
        .is_file()
    {
        builder.ok(format!(
            "evidence directory would be: {}",
            evidence_dir(&branch)
        ));
    }

    if let Some(dagger) = find_command("dagger", &options.path_env) {
        builder.ok(format!("dagger CLI found: {}", dagger.display()));
    } else {
        builder.fail("dagger CLI not found; pre-merge gate cannot run offline");
    }

    check_docker(&mut builder, &options.path_env);
    check_git_lfs(&mut builder, &options.path_env);

    Ok(builder.finish())
}

pub fn render(report: &PreflightReport) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();
    for line in &report.lines {
        match line {
            PreflightLine::Info(message) => {
                stdout.push_str(&format!("INFO: {message}\n"));
            }
            PreflightLine::Ok(message) => {
                stdout.push_str(&format!("OK: {message}\n"));
            }
            PreflightLine::Warn(message) => {
                stderr.push_str(&format!("WARN: {message}\n"));
            }
            PreflightLine::Fail(message) => {
                stderr.push_str(&format!("FAIL: {message}\n"));
            }
        }
    }
    stdout.push_str(&format!(
        "\nSummary: {} failure(s), {} warning(s)\n",
        report.fail_count, report.warn_count
    ));
    (stdout, stderr)
}

fn check_file(builder: &mut PreflightBuilder, root: &Path, rel: &str, label: &str) {
    if root.join(rel).is_file() {
        builder.ok(format!("{label}: {rel}"));
    } else {
        builder.fail(format!("{label} missing: {rel}"));
    }
}

fn check_executable(builder: &mut PreflightBuilder, root: &Path, rel: &str, label: &str) {
    let path = root.join(rel);
    if is_executable(&path) {
        builder.ok(format!("{label} executable: {rel}"));
    } else {
        builder.fail(format!("{label} missing or not executable: {rel}"));
    }
}

fn check_lfs_attrs(builder: &mut PreflightBuilder, root: &Path) {
    let attrs_path = root.join(".gitattributes");
    let Ok(attrs) = fs::read_to_string(attrs_path) else {
        builder.fail(".gitattributes does not contain required .evidence LFS rules");
        return;
    };
    if attrs.contains(".evidence/**/*.png filter=lfs")
        && attrs.contains(".evidence/**/*.webm filter=lfs")
    {
        builder.ok(".evidence binary LFS attributes present");
    } else {
        builder.fail(".gitattributes does not contain required .evidence LFS rules");
    }
}

fn check_docker(builder: &mut PreflightBuilder, path_env: &OsStr) {
    let Some(docker) = find_command("docker", path_env) else {
        builder.warn(
            "docker runtime not reachable; verify your Dagger local runtime before airplane mode",
        );
        return;
    };
    let info = Command::new(&docker).arg("info").output();
    if !matches!(info, Ok(output) if output.status.success()) {
        builder.warn(
            "docker runtime not reachable; verify your Dagger local runtime before airplane mode",
        );
        return;
    }
    builder.ok("docker runtime is reachable");
    let images = Command::new(&docker)
        .args(["image", "ls", "--format", "{{.Repository}}:{{.Tag}}"])
        .output();
    let cached = images
        .ok()
        .filter(|output| output.status.success())
        .map(|output| {
            String::from_utf8_lossy(&output.stdout)
                .lines()
                .any(|line| line.starts_with("registry.dagger.io/engine:"))
        })
        .unwrap_or(false);
    if cached {
        builder.ok("Dagger engine image appears cached locally");
    } else {
        builder.warn("no registry.dagger.io/engine:<version> image visible in docker cache");
    }
}

fn check_git_lfs(builder: &mut PreflightBuilder, path_env: &OsStr) {
    if find_command("git-lfs", path_env).is_some() {
        builder.ok("git-lfs command found");
        return;
    }
    let available = Command::new("git")
        .args(["lfs", "version"])
        .output()
        .ok()
        .is_some_and(|output| output.status.success());
    if available {
        builder.ok("git lfs is available");
    } else {
        builder.warn("git lfs not available; binary evidence may commit as full files or pointers may not hydrate");
    }
}

fn git_branch(root: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(root)
        .output()?;
    if !output.status.success() {
        bail!("failed to determine git branch");
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn evidence_dir(branch: &str) -> String {
    format!(
        ".evidence/{}/{}/",
        branch_slug(branch),
        Utc::now().format("%Y-%m-%d")
    )
}

fn branch_slug(branch: &str) -> String {
    let mut slug = String::new();
    let mut last_hyphen = false;
    for character in branch.chars() {
        let next = if character.is_ascii_alphanumeric() || matches!(character, '.' | '_' | '-') {
            character
        } else {
            '-'
        };
        if next == '-' {
            if !last_hyphen {
                slug.push(next);
            }
            last_hyphen = true;
        } else {
            slug.push(next);
            last_hyphen = false;
        }
    }
    slug.trim_matches('-').to_string()
}

fn find_command(name: &str, path_env: &OsStr) -> Option<PathBuf> {
    let paths = std::env::split_paths(path_env).collect::<Vec<_>>();
    for dir in paths {
        let candidate = dir.join(name);
        if is_executable(&candidate) {
            return Some(candidate);
        }
    }
    None
}

fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::metadata(path)
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

#[derive(Default)]
struct PreflightBuilder {
    lines: Vec<PreflightLine>,
    fail_count: usize,
    warn_count: usize,
}

impl PreflightBuilder {
    fn info(&mut self, message: impl Into<String>) {
        self.lines.push(PreflightLine::Info(message.into()));
    }

    fn ok(&mut self, message: impl Into<String>) {
        self.lines.push(PreflightLine::Ok(message.into()));
    }

    fn warn(&mut self, message: impl Into<String>) {
        self.warn_count += 1;
        self.lines.push(PreflightLine::Warn(message.into()));
    }

    fn fail(&mut self, message: impl Into<String>) {
        self.fail_count += 1;
        self.lines.push(PreflightLine::Fail(message.into()));
    }

    fn finish(self) -> PreflightReport {
        PreflightReport {
            lines: self.lines,
            fail_count: self.fail_count,
            warn_count: self.warn_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::process::Command;

    use tempfile::TempDir;

    use super::*;

    #[test]
    fn preflight_passes_with_required_files_and_dagger() {
        let fixture = Fixture::new();
        fixture.write_command("dagger", "echo dagger-stub\n");

        let report = run(&PreflightOptions {
            repo_root: fixture.root(),
            path_env: fixture.path_env(),
        })
        .unwrap();
        let (stdout, _stderr) = render(&report);

        assert!(report.success());
        assert!(stdout.contains("OK: dagger CLI found:"));
        assert!(stdout.contains(".evidence/master/"));
    }

    #[test]
    fn preflight_fails_without_dagger() {
        let fixture = Fixture::new();

        let report = run(&PreflightOptions {
            repo_root: fixture.root(),
            path_env: fixture.path_env(),
        })
        .unwrap();
        let (_stdout, stderr) = render(&report);

        assert!(!report.success());
        assert_eq!(report.fail_count, 1);
        assert!(stderr.contains("FAIL: dagger CLI not found"));
    }

    #[test]
    fn preflight_fails_without_lfs_attributes() {
        let fixture = Fixture::new();
        fixture.write_command("dagger", "echo dagger-stub\n");
        fs::write(fixture.root().join(".gitattributes"), "").unwrap();

        let report = run(&PreflightOptions {
            repo_root: fixture.root(),
            path_env: fixture.path_env(),
        })
        .unwrap();
        let (_stdout, stderr) = render(&report);

        assert!(!report.success());
        assert!(stderr.contains("required .evidence LFS rules"));
    }

    struct Fixture {
        temp: TempDir,
    }

    impl Fixture {
        fn new() -> Self {
            let temp = TempDir::new().unwrap();
            let root = temp.path();
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(root)
                .status()
                .unwrap();
            fs::create_dir_all(root.join(".empty-hooks")).unwrap();
            Command::new("git")
                .args(["config", "core.hooksPath", ".empty-hooks"])
                .current_dir(root)
                .status()
                .unwrap();
            Command::new("git")
                .args(["config", "user.name", "Test User"])
                .current_dir(root)
                .status()
                .unwrap();
            Command::new("git")
                .args(["config", "user.email", "test@example.com"])
                .current_dir(root)
                .status()
                .unwrap();
            Command::new("git")
                .args(["commit", "--allow-empty", "-m", "initial", "-q"])
                .current_dir(root)
                .status()
                .unwrap();
            fs::create_dir_all(root.join("scripts/lib")).unwrap();
            fs::create_dir_all(root.join(".githooks")).unwrap();
            fs::create_dir_all(root.join("bin")).unwrap();
            fs::create_dir_all(root.join("crates/harness-kit-checks/src")).unwrap();
            fs::write(
                root.join("crates/harness-kit-checks/src/evidence.rs"),
                "evidence_dir\n",
            )
            .unwrap();
            fs::write(
                root.join("crates/harness-kit-checks/src/verdicts.rs"),
                "verdicts\n",
            )
            .unwrap();
            fs::write(root.join(".githooks/pre-merge-commit"), "#!/bin/sh\n").unwrap();
            fs::set_permissions(
                root.join(".githooks/pre-merge-commit"),
                fs::Permissions::from_mode(0o755),
            )
            .unwrap();
            fs::write(
                root.join(".gitattributes"),
                ".evidence/**/*.png filter=lfs diff=lfs merge=lfs -text\n.evidence/**/*.webm filter=lfs diff=lfs merge=lfs -text\n",
            )
            .unwrap();
            Self { temp }
        }

        fn root(&self) -> PathBuf {
            self.temp.path().to_path_buf()
        }

        fn path_env(&self) -> OsString {
            self.temp.path().join("bin").into_os_string()
        }

        fn write_command(&self, name: &str, body: &str) {
            let path = self.temp.path().join("bin").join(name);
            fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
}
