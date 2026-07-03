pub fn usage() -> ! {
    eprintln!(
        r#"usage:
  harness-kit-checks check [--repo PATH]
  harness-kit-checks bootstrap [--repo PATH] [--bundle NAME] [--dry-run]
  harness-kit-checks apply-factory-mcps --profile ID|--all-profiles [--repo PATH] [--harness codex] [--project PATH] [--codex-home PATH] [--dry-run] [--check-env|--skip-env-check]
  harness-kit-checks check-frontmatter [--repo PATH]
  harness-kit-checks generate-index [--repo PATH]
  harness-kit-checks check-index-drift [--repo PATH]
  harness-kit-checks build-docs-site [--repo PATH] [--output PATH]
  harness-kit-checks check-docs-site [--repo PATH] [--site PATH] [--self-test]
  harness-kit-checks check-exclusions|check-conflict-markers|check-portable-paths|check-no-claims|check-vendored-copies|check-harness-install-paths [--repo PATH]
  harness-kit-checks check-godfiles [--write-baseline]|check-source-markers|check-supply-chain|check-supply-chain-advisories|check-template [--repo PATH]
  harness-kit-checks lint-external-skills [--strict]
  harness-kit-checks sync-external [--repo PATH] [--check] [--allow-floating] [--only owner/repo]
  harness-kit-checks test-sync-external-partial
  harness-kit-checks load-config deploy|monitor [--repo PATH] [--config PATH] [--optional]
  harness-kit-checks fetch-pr-reviews [PR]
  harness-kit-checks scout-skills --input PATH [--output PATH] [--format markdown|json] [--offline|--live]
  harness-kit-checks backlog trailer-keys|closing-keys
  harness-kit-checks backlog ids-from-commit|ids-from-range|file-for-id|archive <arg>
  harness-kit-checks claude-hook <hook-name>
  harness-kit-checks git-hook pre-commit|pre-push|pre-merge-commit|post-commit|post-merge|post-rewrite [arg...]
  harness-kit-checks telemetry [--skill-log PATH] [--since 7d|12h] [--repo NAME] [--project NAME] [--skill NAME] [--format json|text|markdown] [--self-test]
  harness-kit-checks probe-agent-roster [--validate-only] [--write-receipts] [options]
  harness-kit-checks dispatch-agent --provider-target ID --objective TEXT --input-ref REF --prompt-file PATH [--repo PATH] [options]
  harness-kit-checks summarize-delegations [--backlog-ref REF] [--format json|text] [PATH]
  harness-kit-checks record-delegation --provider-target ID --provider-status STATUS --attempt-status STATUS --objective TEXT --input-ref REF --worktree-id ID [options]
  harness-kit-checks premise-source validate PACKET [--repo PATH]|self-test"#
    );
    std::process::exit(2);
}
