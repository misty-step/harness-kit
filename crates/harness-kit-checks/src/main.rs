use std::env;
use std::io::Read;
use std::path::{Path, PathBuf};

use harness_kit_checks::{
    agent_roster, backlog, bootstrap, ci_check, config_loader, docs_site, error_report,
    external_skill_lint, external_sync, frontmatter, generate_index, git_hooks, lint_gates,
    pr_reviews, quality_gates, scout_skills, skill_invocation_analytics, source_refs,
    summarize_delegations, template_check,
};
use harness_kit_hooks::claude_hooks;

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        error_report::print_error_chain(&error);
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> anyhow::Result<()> {
    let Some((command, rest)) = args.split_first() else {
        usage();
    };
    match command.as_str() {
        "check" => {
            let repo = parse_repo_arg(rest);
            for line in ci_check::run(&repo)? {
                println!("{line}");
            }
        }
        "summarize-delegations" => {
            let options = parse_summarize_delegations_args(rest);
            let path = options.path.unwrap_or_else(default_receipt_path);
            let summary = summarize_delegations::summarize_receipts(&path, &options.backlog_ref)?;
            match options.format {
                OutputFormat::Json => println!("{}", summarize_delegations::format_json(&summary)?),
                OutputFormat::Text => println!("{}", summarize_delegations::format_text(&summary)),
            }
        }
        "record-delegation" => {
            let options = parse_record_delegation_args(rest);
            summarize_delegations::validate_roster_path(&options.roster)?;
            let receipt = summarize_delegations::build_attempt_receipt(options.receipt)?;
            summarize_delegations::append_receipt(&options.receipt_output, &receipt)?;
            println!("{}", receipt["delegation_id"].as_str().unwrap_or_default());
        }
        "probe-agent-roster" => {
            let options = parse_probe_agent_roster_args(rest);
            let report = agent_roster::probe_roster(&options)?;
            for line in report.lines {
                println!("{line}");
            }
        }
        "dispatch-agent" => {
            let options = parse_dispatch_agent_args(rest);
            let receipt = agent_roster::dispatch_from_options(&options)?;
            println!(
                "{}",
                serde_json::to_string(&serde_json::Value::Object(receipt))?
            );
        }
        "check-frontmatter" => {
            let repo = parse_repo_arg(rest);
            let report = frontmatter::check_repo(&repo)?;
            for warning in &report.warnings {
                eprintln!("WARN: {warning}");
            }
            if !report.errors.is_empty() {
                for error in &report.errors {
                    eprintln!("FAIL: {error}");
                }
                report.ensure_success()?;
            }
            println!("OK: all frontmatter valid");
        }
        "generate-index" => {
            let repo = parse_repo_arg(rest);
            let report = generate_index::write_index(&repo, chrono::Utc::now())?;
            println!(
                "Generated index.yaml: {} skills, {} agents",
                report.skill_count, report.agent_count
            );
        }
        "check-index-drift" => {
            generate_index::check_drift(&parse_repo_arg(rest), chrono::Utc::now())?;
        }
        "check-docs-site" => {
            run_check_docs_site(rest);
        }
        "build-docs-site" => run_build_docs_site(rest),
        "bootstrap" => {
            let (repo, bundle, dry_run) = parse_bootstrap_args(rest);
            let mut options = bootstrap::BootstrapOptions::from_env(Some(repo))?;
            options.bundle = bundle;
            options.dry_run = dry_run;
            println!("{}", bootstrap::run(&options)?);
        }
        "test-sync-external-partial" => {
            println!("{}", external_sync::self_test_partial_sync()?);
        }
        "load-config" => {
            run_load_config(rest);
        }
        "lint-external-skills" => run_lint_external_skills(rest),
        "sync-external" => run_sync_external(rest),
        "fetch-pr-reviews" => run_fetch_pr_reviews(rest),
        "scout-skills" => run_scout_skills(rest),
        "backlog" => run_backlog(rest),
        "claude-hook" => run_claude_hook(rest),
        "git-hook" => run_git_hook(rest),
        "check-exclusions" => {
            print_gate_report(lint_gates::check_exclusions(&parse_repo_arg(rest))?)?
        }
        "check-conflict-markers" => {
            print_gate_report(lint_gates::check_conflict_markers(&parse_repo_arg(rest))?)?
        }
        "check-portable-paths" => {
            print_gate_report(lint_gates::check_portable_paths(&parse_repo_arg(rest))?)?
        }
        "check-no-claims" => {
            print_gate_report(lint_gates::check_no_claims(&parse_repo_arg(rest))?)?
        }
        "check-vendored-copies" => {
            print_gate_report(lint_gates::check_vendored_copies(&parse_repo_arg(rest))?)?
        }
        "check-harness-install-paths" => print_gate_report(
            lint_gates::check_harness_install_paths(&parse_repo_arg(rest))?,
        )?,
        "check-godfiles" => {
            let (repo, write) = parse_godfile_args(rest);
            if write {
                println!("{}", quality_gates::write_godfile_baseline(&repo)?);
            } else {
                print_gate_report(quality_gates::check_godfiles(&repo)?)?;
            }
        }
        "check-source-markers" => {
            print_gate_report(quality_gates::check_source_markers(&parse_repo_arg(rest))?)?
        }
        "check-supply-chain" => {
            print_gate_report(quality_gates::check_supply_chain(&parse_repo_arg(rest))?)?
        }
        "check-template" => print_gate_report(template_check::check_template_instantiates(
            &parse_repo_arg(rest),
        )?)?,
        "telemetry" | "skill-invocation-analytics" => run_skill_invocation_analytics(rest),
        _ => usage(),
    }
    Ok(())
}

fn run_claude_hook(args: &[String]) {
    let [hook] = args else {
        usage();
    };
    match hook.as_str() {
        "block-master-push" => {
            claude_hooks::run_block_master_push_from_stdin().unwrap_or_else(exit_error);
        }
        "check-todo-quality" => {
            claude_hooks::run_check_todo_quality_from_stdin().unwrap_or_else(exit_error);
        }
        "codex-post-feedback" => {
            claude_hooks::run_codex_post_feedback_from_stdin().unwrap_or_else(exit_error);
        }
        "codex-session-init" => {
            claude_hooks::run_codex_session_init().unwrap_or_else(exit_error);
        }
        "disk-space-guard" => {
            claude_hooks::run_disk_space_guard_from_stdin().unwrap_or_else(exit_error);
        }
        "env-var-newline-guard" => {
            claude_hooks::run_env_var_newline_guard_from_stdin().unwrap_or_else(exit_error);
        }
        "exa-research-reminder" => {
            claude_hooks::run_exa_research_reminder().unwrap_or_else(exit_error);
        }
        "exclusion-guard" => {
            claude_hooks::run_exclusion_guard_from_stdin().unwrap_or_else(exit_error);
        }
        "fix-what-you-touch" => {
            claude_hooks::run_fix_what_you_touch_from_stdin().unwrap_or_else(exit_error);
        }
        "permission-auto-approve" => {
            claude_hooks::run_permission_auto_approve_from_stdin().unwrap_or_else(exit_error);
        }
        "portable-code-guard" => {
            claude_hooks::run_portable_code_guard_from_stdin().unwrap_or_else(exit_error);
        }
        "destructive-command-guard" => {
            claude_hooks::run_destructive_command_guard_from_stdin().unwrap_or_else(exit_error);
        }
        "github-cli-guard" => {
            claude_hooks::run_github_cli_guard_from_stdin().unwrap_or_else(exit_error);
        }
        "skill-invocation-tracker" => {
            claude_hooks::run_skill_invocation_tracker_from_stdin().unwrap_or_else(exit_error);
        }
        "session-health-check" => {
            claude_hooks::run_session_health_check().unwrap_or_else(exit_error);
        }
        "shaping-ripple" => {
            claude_hooks::run_shaping_ripple_from_stdin().unwrap_or_else(exit_error);
        }
        "stop-quality-gate" => {
            claude_hooks::run_stop_quality_gate_from_stdin().unwrap_or_else(exit_error);
        }
        "time-context" => {
            claude_hooks::run_time_context().unwrap_or_else(exit_error);
        }
        _ => usage(),
    }
}

fn run_git_hook(args: &[String]) {
    let Some((hook, rest)) = args.split_first() else {
        usage();
    };
    match hook.as_str() {
        "pre-commit" => {
            let output = git_hooks::run_pre_commit(&repo_root()).unwrap_or_else(exit_error);
            if !output.is_empty() {
                println!("{output}");
            }
        }
        "pre-push" => {
            let mut stdin = String::new();
            std::io::stdin()
                .read_to_string(&mut stdin)
                .unwrap_or_else(|error| exit_error(error.into()));
            let output = git_hooks::run_pre_push(&repo_root(), &stdin).unwrap_or_else(exit_error);
            if !output.is_empty() {
                println!("{output}");
            }
        }
        "pre-merge-commit" => {
            let output = git_hooks::run_pre_merge_commit(&repo_root()).unwrap_or_else(exit_error);
            if !output.is_empty() {
                println!("{output}");
            }
        }
        "post-commit" => {
            let output = git_hooks::run_post_commit(&repo_root()).unwrap_or_else(exit_error);
            if !output.is_empty() {
                println!("{output}");
            }
        }
        "post-merge" => {
            let output = git_hooks::run_post_merge(&repo_root()).unwrap_or_else(exit_error);
            if !output.is_empty() {
                println!("{output}");
            }
        }
        "post-rewrite" => {
            let output = git_hooks::run_post_rewrite(&repo_root(), rest).unwrap_or_else(exit_error);
            if !output.is_empty() {
                println!("{output}");
            }
        }
        _ => usage(),
    }
}

fn run_backlog(args: &[String]) {
    let Some((subcommand, rest)) = args.split_first() else {
        usage();
    };
    match subcommand.as_str() {
        "trailer-keys" => {
            let [] = rest else {
                usage();
            };
            println!("{}", backlog::trailer_keys().join("\n"));
        }
        "closing-keys" => {
            let [] = rest else {
                usage();
            };
            println!("{}", backlog::closing_keys().join("\n"));
        }
        "ids-from-commit" => {
            let [commit] = rest else {
                usage();
            };
            print_lines_or_exit(backlog::ids_from_commit(Path::new("."), commit));
        }
        "ids-from-range" => {
            let [range] = rest else {
                usage();
            };
            print_lines_or_exit(backlog::ids_from_range(Path::new("."), range));
        }
        "file-for-id" => {
            let [id] = rest else {
                usage();
            };
            match backlog::file_for_id(Path::new("."), id).unwrap_or_else(exit_error) {
                Some(path) => println!("{}", path.display()),
                None => std::process::exit(1),
            }
        }
        "archive" => {
            let [id] = rest else {
                usage();
            };
            backlog::archive(Path::new("."), id).unwrap_or_else(exit_error);
        }
        _ => usage(),
    }
}

fn print_lines_or_exit(result: anyhow::Result<Vec<String>>) {
    match result {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
        }
        Err(_) => std::process::exit(1),
    }
}

fn exit_error<T>(error: anyhow::Error) -> T {
    eprintln!("{error}");
    std::process::exit(1);
}

fn print_gate_report(report: lint_gates::GateReport) -> anyhow::Result<()> {
    if report.errors.is_empty() {
        println!("{}", report.ok_message);
        return Ok(());
    }
    for error in &report.errors {
        eprintln!("{error}");
    }
    anyhow::bail!("gate check failed")
}

fn parse_repo_arg(args: &[String]) -> PathBuf {
    match args {
        [] => PathBuf::from("."),
        [flag, path] if flag == "--repo" => PathBuf::from(path),
        _ => usage(),
    }
}

fn parse_bootstrap_args(args: &[String]) -> (PathBuf, Option<String>, bool) {
    let mut repo = PathBuf::from(".");
    let mut bundle = None;
    let mut dry_run = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dry-run" => dry_run = true,
            "--repo" => {
                if let Some(path) = args.get(index + 1) {
                    repo = PathBuf::from(path);
                    index += 1;
                }
            }
            "--bundle" => {
                if let Some(name) = args.get(index + 1) {
                    bundle = Some(name.clone());
                    index += 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    (repo, bundle, dry_run)
}

fn parse_godfile_args(args: &[String]) -> (PathBuf, bool) {
    let mut repo = PathBuf::from(".");
    let mut write = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--write-baseline" => write = true,
            "--repo" => {
                if let Some(path) = args.get(index + 1) {
                    repo = PathBuf::from(path);
                    index += 1;
                }
            }
            _ => {}
        }
        index += 1;
    }
    (repo, write)
}

fn run_check_docs_site(args: &[String]) {
    let mut repo = PathBuf::from(".");
    let mut site: Option<PathBuf> = None;
    let mut self_test = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                index += 1;
                repo = PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
            }
            "--site" => {
                index += 1;
                site = Some(PathBuf::from(
                    args.get(index).cloned().unwrap_or_else(|| usage()),
                ));
            }
            "--self-test" => self_test = true,
            _ => usage(),
        }
        index += 1;
    }
    let repo = if repo.is_absolute() {
        repo
    } else {
        repo_root().join(repo)
    };
    let site = site.unwrap_or_else(|| docs_site::default_site(&repo));
    let site = if site.is_absolute() {
        site
    } else {
        repo.join(site)
    };
    let result = if self_test {
        docs_site::self_test(&repo)
    } else {
        docs_site::validate_site(&docs_site::CheckOptions {
            repo_root: repo,
            site,
            compare_to_rebuild: true,
        })
    };
    match result {
        Ok(()) => {
            println!("docs/site: valid");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_build_docs_site(args: &[String]) {
    let mut repo = repo_root();
    let mut output: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                index += 1;
                repo = PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
            }
            "--output" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index).cloned().unwrap_or_else(|| usage()),
                ));
            }
            "-h" | "--help" => {
                println!(
                    "Build Harness Kit's generated static documentation companion.\n\n\
Usage:\n  harness-kit-checks build-docs-site\n  harness-kit-checks build-docs-site --output docs/site\n  harness-kit-checks build-docs-site --repo /path/to/harness-kit --output /tmp/site\n"
                );
                std::process::exit(0);
            }
            _ => usage(),
        }
        index += 1;
    }
    let repo = if repo.is_absolute() {
        repo
    } else {
        repo_root().join(repo)
    };
    let output = output.unwrap_or_else(|| docs_site::default_site(&repo));
    let output = if output.is_absolute() {
        output
    } else {
        repo.join(output)
    };
    match docs_site::build_site(&repo, &output) {
        Ok(()) => {
            println!("Built {}", display_relative(&repo, &output));
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn display_relative(repo: &Path, path: &Path) -> String {
    path.strip_prefix(repo)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn run_fetch_pr_reviews(args: &[String]) {
    let selector = match args {
        [] => None,
        [selector] => Some(selector.as_str()),
        _ => usage(),
    };
    match pr_reviews::fetch(selector).map(|bundle| pr_reviews::render(&bundle)) {
        Ok(report) => {
            print!("{report}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_scout_skills(args: &[String]) {
    let mut options = scout_skills::ScoutOptions {
        repo_root: repo_root(),
        input: PathBuf::new(),
        output: None,
        format: scout_skills::ScoutFormat::Markdown,
        live: false,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--repo" => {
                index += 1;
                options.repo_root =
                    PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
            }
            "--input" => {
                index += 1;
                options.input = PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
            }
            "--output" => {
                index += 1;
                options.output = Some(PathBuf::from(
                    args.get(index).cloned().unwrap_or_else(|| usage()),
                ));
            }
            "--format" => {
                index += 1;
                options.format = match args.get(index).map(String::as_str) {
                    Some("markdown") => scout_skills::ScoutFormat::Markdown,
                    Some("json") => scout_skills::ScoutFormat::Json,
                    _ => usage(),
                };
            }
            "--live" => options.live = true,
            "--offline" => options.live = false,
            "-h" | "--help" => {
                println!(
                    "Harness Kit Skill Scout\n\n\
Usage:\n  harness-kit-checks scout-skills --input candidates.md --format markdown\n  harness-kit-checks scout-skills --input candidates.md --output /tmp/skill-scout-report.md\n  harness-kit-checks scout-skills --input candidates.md --format json --offline\n  harness-kit-checks scout-skills --input candidates.md --live --output /tmp/skill-scout-report.md\n"
                );
                std::process::exit(0);
            }
            _ => usage(),
        }
        index += 1;
    }
    if options.input.as_os_str().is_empty() {
        usage();
    }
    if !options.input.is_absolute() {
        options.input = options.repo_root.join(&options.input);
    }
    match scout_skills::run(&options) {
        Ok(output) => {
            print!("{output}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_lint_external_skills(args: &[String]) {
    let strict = match args {
        [] => false,
        [flag] if flag == "--strict" => true,
        [flag] if flag == "-h" || flag == "--help" => usage(),
        _ => usage(),
    };
    match external_skill_lint::lint_repo(&repo_root()) {
        Ok(report) => {
            print!("{}", external_skill_lint::render(&report));
            std::process::exit(if strict && report.dirty_count() > 0 {
                1
            } else {
                0
            });
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_sync_external(args: &[String]) {
    let mut options = external_sync::SyncOptions {
        repo_root: repo_root(),
        mode: external_sync::SyncMode::Sync,
        allow_floating: false,
        only_repo: None,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--check" => options.mode = external_sync::SyncMode::Check,
            "--allow-floating" => options.allow_floating = true,
            "--only" => {
                index += 1;
                options.only_repo = args.get(index).cloned().or_else(|| {
                    usage();
                });
            }
            "--repo" => {
                index += 1;
                options.repo_root = args.get(index).map(PathBuf::from).unwrap_or_else(|| {
                    usage();
                });
            }
            "-h" | "--help" => {
                println!(
                    "Harness Kit External-Skill Sync\n\n\
Reads registry.yaml, fetches each declared external source at a pinned ref, \
and installs selected skills into skills/.external/<alias>/.\n\n\
Usage:\n  harness-kit-checks sync-external\n  harness-kit-checks sync-external --check\n  harness-kit-checks sync-external --allow-floating\n  harness-kit-checks sync-external --only anthropics/skills\n"
                );
                std::process::exit(0);
            }
            _ => usage(),
        }
        index += 1;
    }
    match external_sync::run(&options) {
        Ok(report) => {
            for line in report.lines {
                println!("{line}");
            }
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_skill_invocation_analytics(args: &[String]) {
    let (options, format, self_test) = parse_skill_invocation_analytics_args(args);
    if self_test {
        match skill_invocation_analytics::self_test() {
            Ok(()) => {
                println!("analyze-skill-invocations self-test ok");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("{error:#}");
                std::process::exit(1);
            }
        }
    }
    match skill_invocation_analytics::analyze(&options)
        .and_then(|report| skill_invocation_analytics::render(&report, &format))
    {
        Ok(output) => {
            println!("{output}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_load_config(args: &[String]) -> ! {
    let options = parse_load_config_args(args);
    match config_loader::load(&options) {
        Ok(config_loader::LoadOutcome::Found(value)) => match config_loader::format_json(&value) {
            Ok(output) => {
                println!("{output}");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("ERROR: {error}");
                std::process::exit(1);
            }
        },
        Ok(config_loader::LoadOutcome::OptionalMissing) => {
            println!("{{}}");
            std::process::exit(0);
        }
        Ok(config_loader::LoadOutcome::RequiredMissing { path, create_path }) => {
            eprintln!(
                "ERROR: missing config file: {} (create {})",
                path.display(),
                create_path.display()
            );
            std::process::exit(2);
        }
        Err(error) => {
            eprintln!("ERROR: {}", error.message());
            std::process::exit(1);
        }
    }
}

fn parse_load_config_args(args: &[String]) -> config_loader::LoadOptions {
    let Some((name, rest)) = args.split_first() else {
        usage();
    };
    let mut options = config_loader::LoadOptions {
        name: name.clone(),
        repo: PathBuf::from("."),
        config: None,
        optional: false,
    };
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--repo" => {
                index += 1;
                let Some(value) = rest.get(index) else {
                    usage();
                };
                options.repo = PathBuf::from(value);
            }
            "--config" => {
                index += 1;
                let Some(value) = rest.get(index) else {
                    usage();
                };
                options.config = Some(PathBuf::from(value));
            }
            "--optional" => options.optional = true,
            _ => usage(),
        }
        index += 1;
    }
    options
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Json,
    Text,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SummarizeOptions {
    backlog_ref: String,
    format: OutputFormat,
    path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq)]
struct RecordOptions {
    roster: PathBuf,
    receipt_output: PathBuf,
    receipt: summarize_delegations::ReceiptInput,
}

fn parse_summarize_delegations_args(args: &[String]) -> SummarizeOptions {
    let mut options = SummarizeOptions {
        backlog_ref: String::new(),
        format: OutputFormat::Json,
        path: None,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--backlog-ref" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    usage();
                };
                options.backlog_ref = value.clone();
            }
            "--format" => {
                index += 1;
                let Some(value) = args.get(index) else {
                    usage();
                };
                options.format = match value.as_str() {
                    "json" => OutputFormat::Json,
                    "text" => OutputFormat::Text,
                    _ => usage(),
                };
            }
            value if value.starts_with('-') => usage(),
            value => {
                if options.path.is_some() {
                    usage();
                }
                options.path = Some(PathBuf::from(value));
            }
        }
        index += 1;
    }
    options
}

fn parse_probe_agent_roster_args(args: &[String]) -> agent_roster::ProbeOptions {
    let mut options = agent_roster::ProbeOptions {
        roster: default_roster_path(),
        validate_only: false,
        write_receipts: false,
        path_env: None,
        receipt_output: default_receipt_path(),
        lead_harness: "unknown".to_string(),
        lead_provider: "unknown".to_string(),
        input_ref: ".harness-kit/agents.yaml".to_string(),
        objective: "probe agent-provider roster".to_string(),
        backlog_ref: String::new(),
    };
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        match flag {
            "--validate-only" => options.validate_only = true,
            "--write-receipts" => options.write_receipts = true,
            "--roster" => {
                index += 1;
                options.roster = PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
            }
            "--receipt-output" => {
                index += 1;
                options.receipt_output =
                    PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
            }
            "--path-env" => {
                index += 1;
                options.path_env = Some(args.get(index).cloned().unwrap_or_else(|| usage()));
            }
            "--lead-harness" => {
                index += 1;
                options.lead_harness = args.get(index).cloned().unwrap_or_else(|| usage());
            }
            "--lead-provider" => {
                index += 1;
                options.lead_provider = args.get(index).cloned().unwrap_or_else(|| usage());
            }
            "--input-ref" => {
                index += 1;
                options.input_ref = args.get(index).cloned().unwrap_or_else(|| usage());
            }
            "--objective" => {
                index += 1;
                options.objective = args.get(index).cloned().unwrap_or_else(|| usage());
            }
            "--backlog-ref" => {
                index += 1;
                options.backlog_ref = args.get(index).cloned().unwrap_or_else(|| usage());
            }
            _ => usage(),
        }
        index += 1;
    }
    options
}

fn parse_u64(flag: &str, value: &str) -> u64 {
    value.parse().unwrap_or_else(|_| {
        eprintln!("{flag} must be a non-negative integer");
        std::process::exit(2);
    })
}

fn parse_f64(flag: &str, value: &str) -> f64 {
    value.parse().unwrap_or_else(|_| {
        eprintln!("{flag} must be a non-negative number");
        std::process::exit(2);
    })
}

fn default_roster_path() -> PathBuf {
    default_roster_path_for_repo(&repo_root())
}

fn default_roster_path_for_repo(repo: &Path) -> PathBuf {
    env::var_os("HARNESS_KIT_ROSTER")
        .or_else(|| env::var_os("HARNESS_KIT_ROSTER_PATH"))
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let local = repo.join(".harness-kit/agents.yaml");
            if local.exists() {
                local
            } else {
                env::var_os("HARNESS_KIT_HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| {
                        env::var_os("HOME")
                            .map(PathBuf::from)
                            .unwrap_or_else(|| PathBuf::from("."))
                            .join(".harness-kit")
                    })
                    .join("agents.yaml")
            }
        })
}

fn default_receipt_path() -> PathBuf {
    default_receipt_path_for_repo(&repo_root())
}

fn default_receipt_path_for_repo(repo: &Path) -> PathBuf {
    env::var_os("HARNESS_KIT_RECEIPTS")
        .or_else(|| env::var_os("HARNESS_KIT_RECEIPT_PATH"))
        .map(PathBuf::from)
        .unwrap_or_else(|| repo.join(".harness-kit/traces/delegations.jsonl"))
}

fn repo_root() -> PathBuf {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output();
    if let Ok(output) = output
        && output.status.success()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let trimmed = stdout.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Some(repo) = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .filter(|repo| repo.join(".harness-kit/agents.yaml").is_file())
    {
        return repo.to_path_buf();
    }
    env::current_dir().unwrap_or_else(|_| Path::new(".").to_path_buf())
}

fn usage() -> ! {
    eprintln!(
        r#"usage:
  harness-kit-checks check [--repo PATH]
  harness-kit-checks bootstrap [--repo PATH] [--bundle NAME] [--dry-run]
  harness-kit-checks check-frontmatter [--repo PATH]
  harness-kit-checks generate-index [--repo PATH]
  harness-kit-checks check-index-drift [--repo PATH]
  harness-kit-checks build-docs-site [--repo PATH] [--output PATH]
  harness-kit-checks check-docs-site [--repo PATH] [--site PATH] [--self-test]
  harness-kit-checks check-exclusions|check-conflict-markers|check-portable-paths|check-no-claims|check-vendored-copies|check-harness-install-paths [--repo PATH]
  harness-kit-checks check-godfiles [--write-baseline]|check-source-markers|check-supply-chain|check-template [--repo PATH]
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
  harness-kit-checks record-delegation --provider-target ID --provider-status STATUS --attempt-status STATUS --objective TEXT --input-ref REF --worktree-id ID [options]"#
    );
    std::process::exit(2);
}

fn parse_dispatch_agent_args(args: &[String]) -> agent_roster::DispatchOptions {
    let repo = repo_root();
    let mut options = agent_roster::DispatchOptions {
        repo: repo.clone(),
        roster: default_roster_path_for_repo(&repo),
        provider_target: String::new(),
        objective: String::new(),
        input_ref: String::new(),
        prompt_file: PathBuf::new(),
        backlog_ref: String::new(),
        lead_harness: "unknown".to_string(),
        lead_provider: "unknown".to_string(),
        model_override: None,
        timeout_s: 600.0,
        grace_s: 2.0,
        max_prompt_bytes: 128 * 1024,
        transcript_dir: default_receipt_path_for_repo(&repo)
            .parent()
            .map(|parent| parent.join("provider-lanes"))
            .unwrap_or_else(|| PathBuf::from("provider-lanes")),
        receipt_output: default_receipt_path_for_repo(&repo),
        path_env: None,
        lane_harness: None,
        keep_lane_root: false,
        expect_output: None,
    };
    let mut roster_explicit = false;
    let mut receipt_output_explicit = false;
    let mut transcript_dir_explicit = false;
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--repo" => {
                options.repo = PathBuf::from(value());
                if !roster_explicit {
                    options.roster = default_roster_path_for_repo(&options.repo);
                }
                if !receipt_output_explicit {
                    options.receipt_output = default_receipt_path_for_repo(&options.repo);
                }
                if !transcript_dir_explicit {
                    options.transcript_dir = default_receipt_path_for_repo(&options.repo)
                        .parent()
                        .map(|parent| parent.join("provider-lanes"))
                        .unwrap_or_else(|| PathBuf::from("provider-lanes"));
                }
            }
            "--roster" => {
                roster_explicit = true;
                options.roster = PathBuf::from(value());
            }
            "--receipt-output" => {
                receipt_output_explicit = true;
                options.receipt_output = PathBuf::from(value());
            }
            "--provider-target" => options.provider_target = value(),
            "--objective" => options.objective = value(),
            "--input-ref" => options.input_ref = value(),
            "--prompt-file" => options.prompt_file = PathBuf::from(value()),
            "--backlog-ref" => options.backlog_ref = value(),
            "--lead-harness" => options.lead_harness = value(),
            "--lead-provider" => options.lead_provider = value(),
            "--model-override" => options.model_override = Some(value()),
            "--timeout-s" => options.timeout_s = parse_f64(flag, &value()),
            "--grace-s" => options.grace_s = parse_f64(flag, &value()),
            "--max-prompt-bytes" => options.max_prompt_bytes = parse_u64(flag, &value()),
            "--transcript-dir" => {
                transcript_dir_explicit = true;
                options.transcript_dir = PathBuf::from(value());
            }
            "--path-env" => options.path_env = Some(value()),
            "--lane-harness" => options.lane_harness = Some(PathBuf::from(value())),
            "--expect-output" => options.expect_output = Some(value()),
            "--keep-lane-root" => {
                index -= 1;
                options.keep_lane_root = true;
            }
            _ => usage(),
        }
        index += 1;
    }
    if options.provider_target.is_empty()
        || options.objective.is_empty()
        || options.input_ref.is_empty()
        || options.prompt_file.as_os_str().is_empty()
    {
        usage();
    }
    options
}

fn parse_record_delegation_args(args: &[String]) -> RecordOptions {
    let mut roster = default_roster_path();
    let mut receipt_output = default_receipt_path();
    let mut provider_target = None;
    let mut provider_status = None;
    let mut attempt_status = None;
    let mut objective = None;
    let mut input_ref = None;
    let mut evidence_refs = Vec::new();
    let mut lead_verdict = "pending".to_string();
    let mut worktree_id = None;
    let mut backlog_ref = String::new();
    let mut lead_harness = "unknown".to_string();
    let mut lead_provider = "unknown".to_string();
    let mut summary = String::new();
    let mut model_id = None;
    let mut duration_ms = None;
    let mut transcript_bytes = None;
    let mut usage_input = summarize_delegations::UsageInput::default();
    let mut has_usage = false;
    let mut work_source_refs = Vec::new();

    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--roster" => roster = PathBuf::from(value()),
            "--receipt-output" => receipt_output = PathBuf::from(value()),
            "--provider-target" => provider_target = Some(value()),
            "--provider-status" => provider_status = Some(value()),
            "--attempt-status" => attempt_status = Some(value()),
            "--objective" => objective = Some(value()),
            "--input-ref" => input_ref = Some(value()),
            "--evidence-ref" => evidence_refs.push(value()),
            "--lead-verdict" => lead_verdict = value(),
            "--worktree-id" => worktree_id = Some(value()),
            "--backlog-ref" => backlog_ref = value(),
            "--lead-harness" => lead_harness = value(),
            "--lead-provider" => lead_provider = value(),
            "--summary" => summary = value(),
            "--model-id" => model_id = Some(value()),
            "--duration-ms" => duration_ms = Some(parse_u64(flag, &value())),
            "--transcript-bytes" => transcript_bytes = Some(parse_u64(flag, &value())),
            "--input-tokens" => {
                usage_input.input_tokens = Some(parse_u64(flag, &value()));
                has_usage = true;
            }
            "--output-tokens" => {
                usage_input.output_tokens = Some(parse_u64(flag, &value()));
                has_usage = true;
            }
            "--total-tokens" => {
                usage_input.total_tokens = Some(parse_u64(flag, &value()));
                has_usage = true;
            }
            "--cost-usd" => {
                usage_input.cost_usd = Some(parse_f64(flag, &value()));
                has_usage = true;
            }
            "--cost-source" => {
                let source = value();
                if !["provider_reported", "estimated", "manual", "unknown"]
                    .contains(&source.as_str())
                {
                    usage();
                }
                usage_input.cost_source = Some(source);
                has_usage = true;
            }
            "--work-source-ref" => match source_refs::parse_ref(&value()) {
                Ok(reference) => work_source_refs.push(reference),
                Err(error) => {
                    eprintln!("record-delegation: {error:#}");
                    std::process::exit(1);
                }
            },
            _ => usage(),
        }
        index += 1;
    }

    RecordOptions {
        roster,
        receipt_output,
        receipt: summarize_delegations::ReceiptInput {
            provider_target: provider_target.unwrap_or_else(|| usage()),
            provider_status: provider_status.unwrap_or_else(|| usage()),
            attempt_status: attempt_status.unwrap_or_else(|| usage()),
            objective: objective.unwrap_or_else(|| usage()),
            input_ref: input_ref.unwrap_or_else(|| usage()),
            evidence_refs,
            lead_verdict,
            worktree_id: worktree_id.unwrap_or_else(|| usage()),
            backlog_ref,
            lead_harness,
            lead_provider,
            summary,
            model_id,
            duration_ms,
            usage: has_usage.then_some(usage_input),
            transcript_bytes,
            lane_harness_ref: None,
            lane_harness_sha256: None,
            projection_status: None,
            failure_kind: None,
            output_check: None,
            work_source_refs,
        },
    }
}

fn parse_skill_invocation_analytics_args(
    args: &[String],
) -> (
    skill_invocation_analytics::AnalyzeOptions,
    skill_invocation_analytics::OutputFormat,
    bool,
) {
    let mut options = skill_invocation_analytics::AnalyzeOptions {
        skill_log: skill_invocation_analytics::default_skill_log(),
        work_ledger: skill_invocation_analytics::default_work_ledger(),
        delegations: skill_invocation_analytics::default_delegations(),
        since: String::new(),
        repo: String::new(),
        project: String::new(),
        skill: String::new(),
    };
    let mut format = skill_invocation_analytics::OutputFormat::Markdown;
    let mut self_test = false;
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--self-test" => {
                self_test = true;
                continue;
            }
            "--skill-log" => options.skill_log = PathBuf::from(value()),
            "--work-ledger" => options.work_ledger = PathBuf::from(value()),
            "--delegations" => options.delegations = PathBuf::from(value()),
            "--since" => options.since = value(),
            "--repo" => options.repo = value(),
            "--project" => options.project = value(),
            "--skill" => options.skill = value(),
            "--format" => {
                format = match value().as_str() {
                    "json" => skill_invocation_analytics::OutputFormat::Json,
                    "text" => skill_invocation_analytics::OutputFormat::Text,
                    "markdown" => skill_invocation_analytics::OutputFormat::Markdown,
                    _ => usage(),
                };
            }
            _ => usage(),
        }
        index += 1;
    }
    (options, format, self_test)
}
