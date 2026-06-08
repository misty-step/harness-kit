use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use regex::Regex;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillKind {
    Qa,
    PersonaAcceptance,
    Generic,
}

impl SkillKind {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "qa" => Ok(Self::Qa),
            "persona" | "persona-acceptance" => Ok(Self::PersonaAcceptance),
            "generic" => Ok(Self::Generic),
            _ => bail!("unknown repo skill kind {value:?}; expected qa|persona-acceptance|generic"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Qa => "qa",
            Self::PersonaAcceptance => "persona-acceptance",
            Self::Generic => "generic",
        }
    }
}

pub struct ScaffoldOptions {
    pub root: PathBuf,
    pub name: String,
    pub kind: SkillKind,
}

pub fn scaffold(options: &ScaffoldOptions) -> Result<Vec<PathBuf>> {
    validate_skill_name(&options.name)?;
    let skill_dir = options.root.join(".agents/skills").join(&options.name);
    if skill_dir.exists() {
        bail!("{} already exists", skill_dir.display());
    }
    let eval_case = match options.kind {
        SkillKind::Qa => "qa-smoke",
        SkillKind::PersonaAcceptance => "persona-acceptance",
        SkillKind::Generic => "repo-workflow",
    };
    let created = vec![
        skill_dir.join("SKILL.md"),
        skill_dir.join("evals/README.md"),
        skill_dir
            .join("evals/cases")
            .join(format!("{eval_case}.md")),
        skill_dir.join("evals/graders/README.md"),
    ];
    fs::create_dir_all(skill_dir.join("evals/cases"))?;
    fs::create_dir_all(skill_dir.join("evals/graders"))?;
    fs::write(
        &created[0],
        skill_template(&options.name, options.kind, eval_case),
    )?;
    fs::write(
        &created[1],
        format!(
            "# {} evals\n\nReplace this scaffold with the generated skill's runnable oracle.\n",
            options.name
        ),
    )?;
    fs::write(
        &created[2],
        format!(
            "# Case: {eval_case}\n\n- Target repo evidence: REPLACE_WITH_PATH\n- Command/path/route exercised: REPLACE_WITH_COMMAND_OR_ROUTE\n- Expected artifact: REPLACE_WITH_ARTIFACT\n"
        ),
    )?;
    fs::write(
        &created[3],
        "Replace this scaffold with a repo-local rubric or executable grader command.\n",
    )?;
    Ok(created)
}

pub fn validate(skill_dir: &Path) -> Result<String> {
    if !skill_dir.is_dir() {
        bail!(
            "{} is not a generated repo skill directory",
            skill_dir.display()
        );
    }
    let skill_md = skill_dir.join("SKILL.md");
    let text = read_required(&skill_md)?;
    reject_placeholders(&text, &skill_md)?;
    validate_frontmatter(&text, &skill_md)?;
    for required in [
        "Use when:",
        "Trigger:",
        "Completion Gate",
        "Exact behavior verified",
        "Evidence that proves it",
        "Exact command/path/route exercised",
        "Repo-fit check",
        "Residual risk",
    ] {
        if !text.contains(required) {
            bail!(
                "{} missing required section or phrase {required:?}",
                skill_md.display()
            );
        }
    }
    if !has_concrete_repo_anchor(&text) {
        bail!(
            "{} must name at least one concrete command, route, endpoint, or repo path",
            skill_md.display()
        );
    }
    require_file(skill_dir.join("evals/README.md"))?;
    require_nonempty_dir(skill_dir.join("evals/cases"), "eval case")?;
    require_nonempty_dir(skill_dir.join("evals/graders"), "eval grader or rubric")?;
    reject_placeholders_in_tree(skill_dir)?;
    validate_bridges(skill_dir)?;
    Ok(format!(
        "{}: generated repo skill valid",
        skill_dir.display()
    ))
}

pub fn self_test() -> Result<String> {
    let temp = tempfile::tempdir()?;
    let options = ScaffoldOptions {
        root: temp.path().to_path_buf(),
        name: "persona-acceptance".to_string(),
        kind: SkillKind::PersonaAcceptance,
    };
    scaffold(&options)?;
    let generated = temp.path().join(".agents/skills/persona-acceptance");
    let error = validate(&generated).unwrap_err().to_string();
    if !error.contains("placeholder") {
        bail!("scaffold should require agent fill before validation");
    }
    fs::write(
        generated.join("SKILL.md"),
        r#"---
name: persona-acceptance
description: |
  Test clinic intake value proposition against live repo behavior.
  Use when: "persona acceptance", "clinic intake QA".
  Trigger: /persona-acceptance.
---

# Persona Acceptance

Run `npm run test:e2e -- --project=chromium`.
Exercise route `/intake`.

## Completion Gate
- Exact behavior verified: front-desk coordinator finds a new intake.
- Evidence that proves it: `.evidence/persona-acceptance/report.md`.
- Exact command/path/route exercised: `npm run test:e2e -- --project=chromium`, `/intake`.
- Repo-fit check: Playwright route follows repo test setup.
- Residual risk: production writes are not exercised.
"#,
    )?;
    fs::write(
        generated.join("evals/README.md"),
        "# Evals\n\nRun the persona acceptance case against demo data.\n",
    )?;
    fs::write(
        generated.join("evals/cases/persona-acceptance.md"),
        "# Case\n\nRun `npm run test:e2e -- --project=chromium` against `/intake`.\n",
    )?;
    fs::write(
        generated.join("evals/graders/README.md"),
        "# Grader\n\nPass only with evidence under `.evidence/persona-acceptance/`.\n",
    )?;
    validate(&generated)?;
    Ok("repo-skill self-test ok".to_string())
}

fn skill_template(name: &str, kind: SkillKind, eval_case: &str) -> String {
    format!(
        r#"---
name: {name}
description: |
  REPLACE_WITH_REPO_SPECIFIC_TRIGGER_CLASSIFIER.
  Use when: "REPLACE_WITH_TRIGGER".
  Trigger: /{name}.
---

# {name}

Repo-local {kind} skill. Agent fills this from live repo discovery.

## Completion Gate
- Exact behavior verified: REPLACE_WITH_BEHAVIOR.
- Evidence that proves it: REPLACE_WITH_EVIDENCE_PATH.
- Exact command/path/route exercised: REPLACE_WITH_COMMAND_OR_ROUTE.
- Repo-fit check: REPLACE_WITH_REPO_CONVENTION.
- Residual risk: REPLACE_WITH_RISK_OR_NONE.

## Eval
- Case: `evals/cases/{eval_case}.md`
"#,
        kind = kind.as_str()
    )
}

fn validate_skill_name(name: &str) -> Result<()> {
    let valid = Regex::new(r"^[a-z0-9][a-z0-9-]{0,63}$").expect("static regex compiles");
    if !valid.is_match(name) {
        bail!("skill name must be lowercase kebab-case, got {name:?}");
    }
    Ok(())
}

fn read_required(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("{} missing or unreadable", path.display()))
}

fn require_file(path: PathBuf) -> Result<()> {
    if !path.is_file() {
        bail!("{} missing", path.display());
    }
    Ok(())
}

fn require_nonempty_dir(path: PathBuf, label: &str) -> Result<()> {
    if !path.is_dir() {
        bail!("{} missing {label} directory", path.display());
    }
    let has_file = fs::read_dir(&path)?
        .filter_map(Result::ok)
        .any(|entry| entry.path().is_file());
    if !has_file {
        bail!("{} contains no {label} files", path.display());
    }
    Ok(())
}

fn validate_frontmatter(text: &str, path: &Path) -> Result<()> {
    if !text.starts_with("---\n") {
        bail!("{} missing YAML frontmatter", path.display());
    }
    let Some(end) = text[4..].find("\n---\n") else {
        bail!("{} has unterminated YAML frontmatter", path.display());
    };
    let frontmatter = &text[4..4 + end];
    for required in ["name:", "description:"] {
        if !frontmatter.contains(required) {
            bail!("{} frontmatter missing {required}", path.display());
        }
    }
    Ok(())
}

fn reject_placeholders_in_tree(skill_dir: &Path) -> Result<()> {
    for relative in ["evals/README.md", "evals/cases", "evals/graders"] {
        let path = skill_dir.join(relative);
        if path.is_file() {
            reject_placeholders(&read_required(&path)?, &path)?;
        } else if path.is_dir() {
            for entry in fs::read_dir(&path)? {
                let path = entry?.path();
                if path.is_file() {
                    reject_placeholders(&read_required(&path)?, &path)?;
                }
            }
        }
    }
    Ok(())
}

fn reject_placeholders(text: &str, path: &Path) -> Result<()> {
    let placeholder =
        Regex::new(r"(?i)TODO|\[fill in\]|your-app|REPLACE_|placeholder|lorem ipsum|example\.com")
            .expect("static regex compiles");
    if placeholder.is_match(text) {
        bail!("{} contains placeholder text", path.display());
    }
    Ok(())
}

fn has_concrete_repo_anchor(text: &str) -> bool {
    let patterns = [
        r"`[^`]*(npm|pnpm|bun|yarn|cargo|go test|pytest|python|swift|xcodebuild|make|just|deno|node)[^`]*`",
        r"`/[^`]+`",
        r"`\.[A-Za-z0-9_./-]+`",
        r"`[A-Za-z0-9_./-]+\.(rs|ts|tsx|js|jsx|py|go|swift|md|yaml|yml|toml|json)`",
        r"https?://",
    ];
    patterns.iter().any(|pattern| {
        Regex::new(pattern)
            .expect("static regex compiles")
            .is_match(text)
    })
}

fn validate_bridges(skill_dir: &Path) -> Result<()> {
    let Some(name) = skill_dir.file_name().and_then(|name| name.to_str()) else {
        bail!("skill directory must have a valid name");
    };
    let Some(agents_skills) = skill_dir.parent() else {
        return Ok(());
    };
    let Some(agents_dir) = agents_skills.parent() else {
        return Ok(());
    };
    let Some(repo) = agents_dir.parent() else {
        return Ok(());
    };
    for harness in [".claude", ".codex", ".pi"] {
        let bridge = repo.join(harness).join("skills").join(name);
        if bridge.exists() {
            let metadata = fs::symlink_metadata(&bridge)?;
            if metadata.file_type().is_symlink() {
                let target = fs::read_link(&bridge)?;
                if target.as_os_str().is_empty() {
                    bail!("{} has empty symlink target", bridge.display());
                }
                let resolved = if target.is_absolute() {
                    target
                } else {
                    bridge
                        .parent()
                        .unwrap_or_else(|| Path::new("."))
                        .join(target)
                };
                let resolved = resolved
                    .canonicalize()
                    .with_context(|| format!("{} has dangling symlink target", bridge.display()))?;
                let expected = skill_dir.canonicalize()?;
                if resolved != expected {
                    bail!(
                        "{} should point to {}, not {}",
                        bridge.display(),
                        expected.display(),
                        resolved.display()
                    );
                }
            } else if bridge.is_dir() {
                bail!(
                    "{} should be a bridge symlink to .agents/skills/{name}, not a copied directory",
                    bridge.display()
                );
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn self_test_exercises_scaffold_and_validate() {
        assert_eq!(self_test().unwrap(), "repo-skill self-test ok");
    }

    #[cfg(unix)]
    #[test]
    fn rejects_misdirected_harness_bridge() {
        let temp = tempfile::tempdir().unwrap();
        let skill_dir = temp.path().join(".agents/skills/qa");
        fs::create_dir_all(skill_dir.join("evals/cases")).unwrap();
        fs::create_dir_all(skill_dir.join("evals/graders")).unwrap();
        fs::create_dir_all(temp.path().join(".claude/skills")).unwrap();
        fs::create_dir_all(temp.path().join(".agents/skills/other")).unwrap();
        std::os::unix::fs::symlink(
            "../../.agents/skills/other",
            temp.path().join(".claude/skills/qa"),
        )
        .unwrap();
        fs::write(
            skill_dir.join("SKILL.md"),
            r#"---
name: qa
description: |
  QA this repo.
  Use when: "qa".
  Trigger: /qa.
---

# QA

Run `cargo test`.

## Completion Gate
- Exact behavior verified: cargo test runs for the generated QA skill.
- Evidence that proves it: `.evidence/qa/report.md`.
- Exact command/path/route exercised: `cargo test`.
- Repo-fit check: follows Cargo test setup.
- Residual risk: none.
"#,
        )
        .unwrap();
        fs::write(skill_dir.join("evals/README.md"), "# Evals\n").unwrap();
        fs::write(
            skill_dir.join("evals/cases/qa.md"),
            "# Case\n`cargo test`\n",
        )
        .unwrap();
        fs::write(
            skill_dir.join("evals/graders/README.md"),
            "# Grader\n`.evidence/qa/report.md`\n",
        )
        .unwrap();

        let error = validate(&skill_dir).unwrap_err().to_string();
        assert!(error.contains("should point to"));
    }
}
