use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::frontmatter;

/// A role-scoped bundle: first-party skill names and vendored external
/// aliases, both drawn from `.harness-kit/bundles.yaml` (data, never
/// duplicated skill prose).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Bundle {
    skills: Vec<String>,
    external: Vec<String>,
}

fn load_bundles(repo: &Path) -> Result<std::collections::BTreeMap<String, Bundle>> {
    let path = repo.join(".harness-kit/bundles.yaml");
    let text =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let doc: serde_yaml::Value = serde_yaml::from_str(&text)
        .with_context(|| format!("invalid YAML in {}", path.display()))?;
    let raw = doc
        .get("bundles")
        .and_then(|value| value.as_mapping())
        .with_context(|| format!("{}: missing top-level `bundles` mapping", path.display()))?;
    let mut bundles = std::collections::BTreeMap::new();
    for (name, value) in raw {
        let name = name
            .as_str()
            .with_context(|| format!("{}: bundle name must be a string", path.display()))?
            .to_string();
        let skills = string_list(value.get("skills"));
        let external = string_list(value.get("external"));
        bundles.insert(name, Bundle { skills, external });
    }
    Ok(bundles)
}

fn string_list(value: Option<&serde_yaml::Value>) -> Vec<String> {
    value
        .and_then(|value| value.as_sequence())
        .map(|seq| {
            seq.iter()
                .filter_map(|item| item.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// Resolves `bundle_name` against `.harness-kit/bundles.yaml`, filtered to
/// what actually exists in the catalog today, and validated so a bundle
/// entry that no longer resolves to a real skill fails loudly instead of
/// silently shrinking the install.
pub(crate) fn resolve_bundle(
    repo: &Path,
    bundle_name: &str,
    all_skills: &[String],
    all_externals: &[String],
) -> Result<(Vec<String>, Vec<String>)> {
    let bundles = load_bundles(repo)?;
    let bundle = bundles.get(bundle_name).with_context(|| {
        format!(
            "unknown bundle '{bundle_name}'; known bundles: {}",
            bundles.keys().cloned().collect::<Vec<_>>().join(", ")
        )
    })?;

    let skill_set: BTreeSet<_> = all_skills.iter().cloned().collect();
    let external_set: BTreeSet<_> = all_externals.iter().cloned().collect();

    let mut missing = Vec::new();
    for name in &bundle.skills {
        if !skill_set.contains(name) {
            missing.push(format!("skills/{name}"));
        }
    }
    for name in &bundle.external {
        if !external_set.contains(name) {
            missing.push(format!("skills/.external/{name}"));
        }
    }
    if !missing.is_empty() {
        bail!(
            "bundle '{bundle_name}' references skill(s) that no longer exist in the catalog: {}",
            missing.join(", ")
        );
    }

    let skills = all_skills
        .iter()
        .filter(|name| bundle.skills.contains(name))
        .cloned()
        .collect();
    let externals = all_externals
        .iter()
        .filter(|name| bundle.external.contains(name))
        .cloned()
        .collect();
    Ok((skills, externals))
}

/// Reports the projected skill count and description-byte estimate for the
/// selected scope, without touching the filesystem. Always shows the
/// full-catalog baseline; adds the bundle row when one is selected so the
/// token-savings claim (backlog.d/130) is visible before installing.
pub(crate) fn dry_run_report(
    repo: &Path,
    bundle_name: Option<&str>,
    all_skills: &[String],
    all_externals: &[String],
    scoped_skills: &[String],
    scoped_externals: &[String],
) -> Result<String> {
    let full_count = all_skills.len() + all_externals.len();
    let full_bytes = description_bytes(repo, "skills", all_skills)?
        + description_bytes(repo, "skills/.external", all_externals)?;

    let mut lines = vec![
        "\x1b[0;34mHarness Kit Bootstrap (dry run — no files written)\x1b[0m".to_string(),
        String::new(),
        format!("full catalog: {full_count} skill(s), ~{full_bytes} description bytes"),
    ];

    if let Some(name) = bundle_name {
        let bundle_count = scoped_skills.len() + scoped_externals.len();
        let bundle_bytes = description_bytes(repo, "skills", scoped_skills)?
            + description_bytes(repo, "skills/.external", scoped_externals)?;
        let saved_count = full_count.saturating_sub(bundle_count);
        let saved_bytes = full_bytes.saturating_sub(bundle_bytes);
        lines.push(format!(
            "bundle '{name}': {bundle_count} skill(s), ~{bundle_bytes} description bytes"
        ));
        lines.push(format!(
            "savings: {saved_count} fewer skill(s), ~{saved_bytes} fewer description bytes"
        ));
    }

    Ok(lines.join("\n"))
}

/// Sums the byte length of each named skill's frontmatter `description`
/// field under `repo/<category>/<name>/SKILL.md` — a cheap proxy for the
/// standing prompt-token tax a set of skills costs every session.
fn description_bytes(repo: &Path, category: &str, names: &[String]) -> Result<usize> {
    let mut total = 0usize;
    for name in names {
        let path = repo.join(category).join(name).join("SKILL.md");
        let Some(frontmatter) = frontmatter::load_frontmatter(&path)? else {
            continue;
        };
        if let Some(description) = frontmatter.get("description") {
            total += serde_yaml::to_string(description)
                .map(|text| text.len())
                .unwrap_or(0);
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bootstrap::{discover_external_skills, discover_skills};

    fn fixture_repo() -> Result<tempfile::TempDir> {
        let temp = tempfile::tempdir()?;
        for skill in ["alpha", "beta", "gamma"] {
            fs::create_dir_all(temp.path().join("skills").join(skill))?;
            fs::write(
                temp.path().join("skills").join(skill).join("SKILL.md"),
                format!("---\nname: {skill}\ndescription: {skill} does a thing\n---\n"),
            )?;
        }
        fs::create_dir_all(temp.path().join("skills/.external/delta"))?;
        fs::write(
            temp.path().join("skills/.external/delta/SKILL.md"),
            "---\nname: delta\ndescription: delta is vendored\n---\n",
        )?;
        fs::create_dir_all(temp.path().join(".harness-kit"))?;
        fs::write(
            temp.path().join(".harness-kit/bundles.yaml"),
            "version: 1\nbundles:\n  demo:\n    skills:\n      - alpha\n      - beta\n    external:\n      - delta\n",
        )?;
        Ok(temp)
    }

    #[test]
    fn resolve_bundle_filters_to_named_members_only() -> Result<()> {
        let temp = fixture_repo()?;
        let all_skills = discover_skills(temp.path())?;
        let all_externals = discover_external_skills(temp.path())?;
        let (skills, externals) = resolve_bundle(temp.path(), "demo", &all_skills, &all_externals)?;
        assert_eq!(skills, vec!["alpha".to_string(), "beta".to_string()]);
        assert_eq!(externals, vec!["delta".to_string()]);
        Ok(())
    }

    #[test]
    fn resolve_bundle_rejects_unknown_bundle_name() -> Result<()> {
        let temp = fixture_repo()?;
        let all_skills = discover_skills(temp.path())?;
        let all_externals = discover_external_skills(temp.path())?;
        let error = resolve_bundle(temp.path(), "nope", &all_skills, &all_externals)
            .unwrap_err()
            .to_string();
        assert!(error.contains("unknown bundle 'nope'"));
        assert!(error.contains("demo"));
        Ok(())
    }

    #[test]
    fn resolve_bundle_fails_loudly_when_member_no_longer_exists() -> Result<()> {
        let temp = fixture_repo()?;
        fs::write(
            temp.path().join(".harness-kit/bundles.yaml"),
            "version: 1\nbundles:\n  demo:\n    skills:\n      - alpha\n      - zeta\n    external: []\n",
        )?;
        let all_skills = discover_skills(temp.path())?;
        let all_externals = discover_external_skills(temp.path())?;
        let error = resolve_bundle(temp.path(), "demo", &all_skills, &all_externals)
            .unwrap_err()
            .to_string();
        assert!(error.contains("skills/zeta"));
        Ok(())
    }

    #[test]
    fn dry_run_reports_full_catalog_and_bundle_savings() -> Result<()> {
        let temp = fixture_repo()?;
        let all_skills = discover_skills(temp.path())?;
        let all_externals = discover_external_skills(temp.path())?;
        let (skills, externals) = resolve_bundle(temp.path(), "demo", &all_skills, &all_externals)?;
        let report = dry_run_report(
            temp.path(),
            Some("demo"),
            &all_skills,
            &all_externals,
            &skills,
            &externals,
        )?;
        assert!(report.contains("full catalog: 4 skill(s)"));
        assert!(report.contains("bundle 'demo': 3 skill(s)"));
        assert!(report.contains("savings: 1 fewer skill(s)"));
        Ok(())
    }

    #[test]
    fn dry_run_without_bundle_reports_only_full_catalog() -> Result<()> {
        let temp = fixture_repo()?;
        let all_skills = discover_skills(temp.path())?;
        let all_externals = discover_external_skills(temp.path())?;
        let report = dry_run_report(
            temp.path(),
            None,
            &all_skills,
            &all_externals,
            &all_skills,
            &all_externals,
        )?;
        assert!(report.contains("full catalog: 4 skill(s)"));
        assert!(!report.contains("bundle"));
        Ok(())
    }
}
