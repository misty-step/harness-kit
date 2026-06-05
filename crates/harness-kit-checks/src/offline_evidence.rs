use std::collections::HashSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use regex::Regex;

const REQUIRED_ATTRS: [&str; 3] = [
    ".evidence/**/*.png filter=lfs diff=lfs merge=lfs -text",
    ".evidence/**/*.gif filter=lfs diff=lfs merge=lfs -text",
    ".evidence/**/*.webm filter=lfs diff=lfs merge=lfs -text",
];

const NO_TMP_FILES: [&str; 10] = [
    "skills/qa/SKILL.md",
    "skills/qa/references/evidence-capture.md",
    "skills/qa/references/scaffold.md",
    "skills/demo/SKILL.md",
    "skills/demo/references/pr-evidence-upload.md",
    "skills/demo/references/remotion.md",
    "skills/demo/references/tts-narration.md",
    "skills/demo/references/scaffold.md",
    "skills/deliver/references/evidence.md",
    "skills/deliver/references/receipt.md",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineEvidenceReport {
    pub errors: Vec<String>,
}

pub fn check_repo(root: &Path) -> Result<OfflineEvidenceReport> {
    let mut errors = Vec::new();

    let attrs_path = root.join(".gitattributes");
    if !attrs_path.exists() {
        errors.push(".gitattributes is missing".to_string());
    } else {
        let attrs: HashSet<String> = fs::read_to_string(&attrs_path)?
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(ToOwned::to_owned)
            .collect();
        let mut missing: Vec<&str> = REQUIRED_ATTRS
            .iter()
            .copied()
            .filter(|required| !attrs.contains(*required))
            .collect();
        missing.sort();
        if !missing.is_empty() {
            errors.push(format!(".gitattributes missing LFS rules: {missing:?}"));
        }
    }

    let gitignore = root.join(".gitignore");
    if gitignore.exists() {
        let ignored =
            Regex::new(r"(?m)^\s*\.evidence/?\s*$")?.is_match(&fs::read_to_string(&gitignore)?);
        if ignored {
            errors.push(".gitignore must not ignore .evidence/".to_string());
        }
    }

    for rel in NO_TMP_FILES {
        let text = read_required(root, rel)?;
        for bad in ["/tmp/qa", "/tmp/demo-slug", "/tmp/demo-evidence"] {
            if text.contains(bad) {
                errors.push(format!("{rel} still references {bad}"));
            }
        }
    }

    let deliver_evidence = read_required(root, "skills/deliver/references/evidence.md")?;
    if deliver_evidence.contains("NOT\nversion-controlled") {
        errors.push(
            "deliver evidence doctrine still says evidence is not version-controlled".to_string(),
        );
    }
    if !deliver_evidence.contains(".evidence/<branch>/<date>/") {
        errors.push("deliver evidence doctrine must name .evidence/<branch>/<date>/".to_string());
    }

    let demo = read_required(root, "skills/demo/SKILL.md")?;
    if !demo.contains("Draft GitHub release (`gh release create --draft`) | Optional mirror") {
        errors.push("demo surfaces table must make draft releases optional mirrors".to_string());
    }

    let pr_upload = read_required(root, "skills/demo/references/pr-evidence-upload.md")?;
    if !pr_upload.contains("HARNESS_EVIDENCE_GITHUB=1") {
        errors.push(
            "PR evidence upload reference must gate GitHub uploads behind HARNESS_EVIDENCE_GITHUB=1"
                .to_string(),
        );
    }

    let code_review = read_required(root, "skills/code-review/SKILL.md")?;
    if !(code_review.contains(".evidence/<branch>/<date>/review-synthesis.md")
        && code_review.contains(".evidence/<branch>/<date>/verdict.json"))
    {
        errors.push("code-review must write synthesis and verdict into .evidence".to_string());
    }

    let ship = read_required(root, "skills/ship/SKILL.md")?;
    if !(ship.contains("QA-Evidence:") && ship.contains("harness-kit-checks -- evidence trailer")) {
        errors.push(
            "ship must preserve QA-Evidence trailers from non-empty .evidence dirs".to_string(),
        );
    }

    let helper = read_required(root, "crates/harness-kit-checks/src/evidence.rs")?;
    if !helper.contains("evidence_dir_create") {
        errors.push("evidence helper missing evidence_dir_create".to_string());
    }
    if !helper.contains("evidence_trailer") {
        errors.push("evidence helper missing evidence_trailer".to_string());
    }

    Ok(OfflineEvidenceReport { errors })
}

fn read_required(root: &Path, rel: &str) -> Result<String> {
    fs::read_to_string(root.join(rel)).with_context(|| format!("failed to read {rel}"))
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    #[test]
    fn valid_contract_passes() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());

        let report = check_repo(temp.path()).unwrap();
        assert_eq!(report.errors, Vec::<String>::new());
    }

    #[test]
    fn missing_lfs_rule_fails() {
        let temp = tempfile::tempdir().unwrap();
        write_valid_fixture(temp.path());
        fs::write(
            temp.path().join(".gitattributes"),
            ".evidence/**/*.png filter=lfs diff=lfs merge=lfs -text\n",
        )
        .unwrap();

        let report = check_repo(temp.path()).unwrap();
        assert!(report.errors[0].contains(".gitattributes missing LFS rules"));
        assert!(report.errors[0].contains(".evidence/**/*.gif"));
        assert!(report.errors[0].contains(".evidence/**/*.webm"));
    }

    fn write_valid_fixture(root: &Path) {
        fs::write(
            root.join(".gitattributes"),
            REQUIRED_ATTRS.join("\n") + "\n",
        )
        .unwrap();
        fs::write(root.join(".gitignore"), "target/\n").unwrap();
        for rel in NO_TMP_FILES {
            write_file(root, rel, "durable evidence text\n");
        }
        write_file(
            root,
            "skills/deliver/references/evidence.md",
            ".evidence/<branch>/<date>/\nversion-controlled evidence\n",
        );
        write_file(
            root,
            "skills/demo/SKILL.md",
            "Draft GitHub release (`gh release create --draft`) | Optional mirror\n",
        );
        write_file(
            root,
            "skills/demo/references/pr-evidence-upload.md",
            "HARNESS_EVIDENCE_GITHUB=1\n",
        );
        write_file(
            root,
            "skills/code-review/SKILL.md",
            ".evidence/<branch>/<date>/review-synthesis.md\n.evidence/<branch>/<date>/verdict.json\n",
        );
        write_file(
            root,
            "skills/ship/SKILL.md",
            "QA-Evidence:\nharness-kit-checks -- evidence trailer\n",
        );
        write_file(
            root,
            "crates/harness-kit-checks/src/evidence.rs",
            "evidence_dir_create\nevidence_trailer\n",
        );
    }

    fn write_file(root: &Path, rel: &str, contents: &str) {
        let path = root.join(rel);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
}
