//! Tests for the parent `external_sync` module. Split into a sibling file (via
//! `#[path]`) so external_sync.rs stays under the god-file ratchet.

use std::fs;

use tempfile::TempDir;

use super::*;

#[test]
fn parses_registry_like_shell_and_skips_default_inactive_sources() {
    let entries = parse_registry(
        r#"
sources:
  - repo: local/repo
    default: true
  - repo: inactive/repo
    active: false
  - repo: upstream/skills
    ref: main
    skills_path: skills
    include: [one, two]
    exclude: skip
    alias_prefix: "up-"
    allow_floating: true
"#,
    )
    .unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].repo, "upstream/skills");
    assert_eq!(entries[0].ref_name, "main");
    assert_eq!(entries[0].skills_path, "skills");
    assert_eq!(entries[0].include, vec!["one", "two"]);
    assert_eq!(entries[0].exclude, vec!["skip"]);
    assert_eq!(entries[0].alias_prefix.as_deref(), Some("up-"));
    assert!(entries[0].allow_floating);
}

#[test]
fn immutable_ref_contract_matches_shell_rules() {
    assert!(is_immutable_ref("0123456789abcdef0123456789abcdef01234567"));
    assert!(is_immutable_ref("v1.2.3"));
    assert!(is_immutable_ref("1.2"));
    assert!(is_immutable_ref("release-candidate"));
    assert!(!is_immutable_ref("main"));
    assert!(!is_immutable_ref("HEAD"));
    assert!(!is_immutable_ref("trunk"));
}

#[test]
fn discovers_skills_and_copies_hidden_files_without_git_metadata() {
    let temp = TempDir::new().unwrap();
    let source = temp.path().join("checkout/skills/demo");
    fs::create_dir_all(source.join(".git")).unwrap();
    fs::create_dir_all(source.join("references")).unwrap();
    fs::write(source.join("SKILL.md"), "demo\n").unwrap();
    fs::write(source.join(".hidden"), "hidden\n").unwrap();
    fs::write(source.join(".git/HEAD"), "ignored\n").unwrap();
    fs::write(source.join("references/a.md"), "a\n").unwrap();

    assert_eq!(
        discover_skills(&temp.path().join("checkout/skills")).unwrap(),
        vec!["demo"]
    );

    let dest = temp.path().join("external/demo");
    fs::create_dir_all(&dest).unwrap();
    copy_dir(&source, &dest).unwrap();

    assert!(dest.join("SKILL.md").is_file());
    assert!(dest.join(".hidden").is_file());
    assert!(dest.join("references/a.md").is_file());
    assert!(!dest.join(".git/HEAD").exists());
}

#[test]
fn install_alias_check_mode_reports_drift_without_writing() {
    let temp = TempDir::new().unwrap();
    let src = temp.path().join("src/demo");
    fs::create_dir_all(&src).unwrap();
    fs::write(src.join("SKILL.md"), "demo\n").unwrap();
    let external = temp.path().join("skills/.external");
    fs::create_dir_all(&external).unwrap();
    let mut state = SyncState::default();

    install_alias(
        "demo",
        &src,
        "org/repo",
        "0123456789abcdef0123456789abcdef01234567",
        &external,
        SyncMode::Check,
        &mut state,
    )
    .unwrap();

    assert!(state.changed);
    assert_eq!(state.aliases, vec!["demo"]);
    assert!(!external.join("demo/SKILL.md").exists());
    assert!(state.lines[0].contains("would install/update: demo (org/repo @ 0123456)"));
}

#[test]
fn cleanup_orphans_removes_undeclared_aliases_and_keeps_checkouts() {
    let temp = TempDir::new().unwrap();
    let external = temp.path().join("skills/.external");
    fs::create_dir_all(external.join("declared")).unwrap();
    fs::create_dir_all(external.join("orphan")).unwrap();
    fs::create_dir_all(external.join("_checkouts/still-here")).unwrap();
    let mut state = SyncState {
        aliases: vec!["declared".to_string()],
        ..SyncState::default()
    };

    cleanup_orphans(&external, SyncMode::Sync, &mut state).unwrap();

    assert!(external.join("declared").is_dir());
    assert!(!external.join("orphan").exists());
    assert!(external.join("_checkouts/still-here").is_dir());
    assert!(state.changed);
}

#[test]
fn parse_registry_reads_skill_name() {
    let entries = parse_registry(
        r#"
sources:
  - repo: ogulcancelik/herdr
    ref: master
    pin: 642c6ab9eac3531c04992d2b10a7a09be6d2e06b
    skills_path: "."
    skill_name: herdr
    alias_prefix: "herdr-"
"#,
    )
    .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].skill_name.as_deref(), Some("herdr"));
    assert_eq!(entries[0].skills_path, ".");
    assert_eq!(entries[0].alias_prefix.as_deref(), Some("herdr-"));
}

#[test]
fn stage_root_skill_copies_only_skill_and_license_not_app() {
    let temp = TempDir::new().unwrap();
    let root = temp.path().join("herdr");
    // a root-level skill shares the repo with the upstream app
    fs::create_dir_all(root.join("src")).unwrap();
    fs::create_dir_all(root.join("assets")).unwrap();
    fs::create_dir_all(root.join("scripts")).unwrap();
    fs::write(root.join("SKILL.md"), "---\nname: herdr\n---\n").unwrap();
    fs::write(root.join("LICENSE"), "AGPL-3.0-or-later\n").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() {}\n").unwrap();
    fs::write(root.join("assets/logo.png"), "png\n").unwrap();
    fs::write(root.join("scripts/build.sh"), "#!/bin/sh\n").unwrap();
    fs::write(root.join("Cargo.toml"), "[package]\n").unwrap();

    let dest = temp.path().join("staged");
    stage_root_skill(&root, &dest).unwrap();

    // only SKILL.md and the upstream license are vendored
    assert!(dest.join("SKILL.md").is_file());
    assert!(dest.join("LICENSE").is_file());
    // the surrounding app (sibling dirs and build files) is never vendored
    assert!(!dest.join("src").exists());
    assert!(!dest.join("assets").exists());
    assert!(!dest.join("scripts").exists());
    assert!(!dest.join("Cargo.toml").exists());
}
