use std::env;
use std::path::{Path, PathBuf};

use harness_kit_checks::{
    agent_readiness_profile, agent_roster, agent_transcript, autoreview, backlog, bench_map,
    bootstrap, bootstrap_agent_allowlist, check_agent_roster, claude_hooks, config_loader,
    critique_eval, design_eval, detect_ui_surfaces, docs_site, embeddings, eval_graders, events,
    evidence, evidence_blocks, external_skill_lint, external_sync, frontmatter, generate_index,
    git_hooks, heal_commit, heal_support, lane_harness, lint_gates, offline_evidence,
    offline_validation_preflight, pr_reviews, premise_source, reflect_checkpoint, reflect_evidence,
    repo_skill, review_score_trends, runtime_primitives, shape_renderer, skill_audit, skill_evals,
    skill_invocation_analytics, skillify_classify, skillify_skill_crud, skillify_transcript,
    summarize_delegations, trace_record, transcript_effectiveness, verdicts, work_ledger,
};

fn main() {
    if let Err(error) = run(env::args().skip(1).collect()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> anyhow::Result<()> {
    let Some((command, rest)) = args.split_first() else {
        usage();
    };
    match command.as_str() {
        "check-agent-roster" => {
            let repo = parse_check_agent_roster_args(rest);
            let report = check_agent_roster::run(&repo)?;
            for line in report.lines {
                println!("{line}");
            }
        }
        "autoreview" => run_autoreview(rest),
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
        "materialize-lane-harness" => {
            let options = parse_materialize_lane_harness_args(rest);
            let roster = agent_roster::load_roster(&options.roster)?;
            let report = lane_harness::materialize_manifest(
                &options.repo,
                &roster,
                &options.manifest,
                options.root.as_deref(),
            )?;
            println!("{}", lane_harness::format_materialize_report(&report));
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
        "check-bench-map" => {
            let repo = parse_repo_arg(rest);
            let report = bench_map::run(&repo)?;
            for line in report.lines {
                println!("{line}");
            }
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
            let repo = parse_repo_arg(rest);
            let options = bootstrap::BootstrapOptions::from_env(Some(repo))?;
            println!("{}", bootstrap::run(&options)?);
        }
        "generate-embeddings" => run_generate_embeddings(rest),
        "search-embeddings" => run_search_embeddings(rest),
        "test-bootstrap-agent-allowlist" => {
            let repo = parse_repo_arg(rest);
            match bootstrap_agent_allowlist::run(&repo) {
                Ok(message) => println!("{message}"),
                Err(error) => {
                    eprintln!("{error:#}");
                    std::process::exit(1);
                }
            }
        }
        "test-sync-external-partial" => {
            println!("{}", external_sync::self_test_partial_sync()?);
        }
        "check-runtime-primitives" => {
            println!("{}", runtime_primitives::run(&parse_repo_arg(rest))?);
        }
        "design-eval" => {
            run_design_eval(rest);
        }
        "critique-eval" => {
            run_critique_eval(rest);
        }
        "eval-grader" => {
            run_eval_grader(rest);
        }
        "load-config" => {
            run_load_config(rest);
        }
        "work-ledger" => {
            run_work_ledger(rest);
        }
        "trace-record" => {
            run_trace_record(rest);
        }
        "review-score-trends" => {
            run_review_score_trends(rest);
        }
        "repo-skill" => run_repo_skill(rest),
        "detect-ui-surfaces" => run_detect_ui_surfaces(rest),
        "offline-validation-preflight" => run_offline_validation_preflight(rest),
        "reflect-gather-evidence" => run_reflect_gather_evidence(rest),
        "reflect-checkpoint" => run_reflect_checkpoint(rest),
        "premise-source" => run_premise_source(rest),
        "mine-transcript-effectiveness" => run_mine_transcript_effectiveness(rest),
        "lint-external-skills" => run_lint_external_skills(rest),
        "sync-external" => run_sync_external(rest),
        "fetch-pr-reviews" => run_fetch_pr_reviews(rest),
        "agent-transcript" => run_agent_transcript(rest),
        "backlog" => run_backlog(rest),
        "claude-hook" => run_claude_hook(rest),
        "evidence" => run_evidence(rest),
        "event" => run_event(rest),
        "git-hook" => run_git_hook(rest),
        "heal-support" => run_heal_support(rest),
        "heal-commit" => {
            heal_commit::run(rest, &repo_root()).unwrap_or_else(exit_error);
        }
        "verdict" => run_verdict(rest),
        "check-evidence-blocks" => {
            let paths = parse_paths_arg(rest, &[PathBuf::from("skills")]);
            let errors = evidence_blocks::check_paths(&paths)?;
            if !errors.is_empty() {
                eprintln!("Evidence block check failed:");
                for error in errors {
                    eprintln!("  {error}");
                }
                anyhow::bail!("evidence block check failed");
            }
            println!("Evidence blocks valid.");
        }
        "check-exclusions" => {
            print_gate_report(lint_gates::check_exclusions(&parse_repo_arg(rest))?)?
        }
        "check-conflict-markers" => {
            print_gate_report(lint_gates::check_conflict_markers(&parse_repo_arg(rest))?)?
        }
        "check-portable-paths" => {
            print_gate_report(lint_gates::check_portable_paths(&parse_repo_arg(rest))?)?
        }
        "check-deliver-composition" => print_gate_report(lint_gates::check_deliver_composition(
            &parse_repo_arg(rest),
        )?)?,
        "check-no-claims" => {
            print_gate_report(lint_gates::check_no_claims(&parse_repo_arg(rest))?)?
        }
        "check-offline-evidence-storage" => {
            let report = offline_evidence::check_repo(&parse_repo_arg(rest))?;
            if !report.errors.is_empty() {
                eprintln!("Offline evidence storage contract failed:");
                for error in report.errors {
                    eprintln!("- {error}");
                }
                anyhow::bail!("offline evidence storage contract failed");
            }
            println!("offline evidence storage contract valid");
        }
        "check-vendored-copies" => {
            print_gate_report(lint_gates::check_vendored_copies(&parse_repo_arg(rest))?)?
        }
        "check-harness-install-paths" => print_gate_report(
            lint_gates::check_harness_install_paths(&parse_repo_arg(rest))?,
        )?,
        "check-skill-evals" => {
            let report = skill_evals::check_repo(&parse_repo_arg(rest))?;
            if !report.errors.is_empty() {
                for error in &report.errors {
                    eprintln!("FAIL: {error}");
                }
                std::process::exit(1);
            }
            println!("{}", skill_evals::format_success(&report));
        }
        "audit-skills" => run_audit_skills(rest),
        "agent-readiness-profile" => run_agent_readiness_profile(rest),
        "shape-render" => run_shape_render(rest),
        "skill-invocation-analytics" => run_skill_invocation_analytics(rest),
        "skillify-classify" => run_skillify_classify(rest),
        "skillify-skill-crud" => run_skillify_skill_crud(rest),
        "skillify-parse-transcript" => run_skillify_parse_transcript(rest),
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

fn run_autoreview(args: &[String]) {
    if args == ["--self-test"] {
        println!(
            "{}",
            autoreview::self_test(&repo_root()).unwrap_or_else(exit_error)
        );
        return;
    }
    let mut options = autoreview::Options::default();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--mode" => {
                index += 1;
                options.mode = args.get(index).unwrap_or_else(|| usage()).clone();
            }
            "--base" => {
                index += 1;
                options.base = Some(args.get(index).unwrap_or_else(|| usage()).clone());
            }
            "--commit" => {
                index += 1;
                options.commit = args.get(index).unwrap_or_else(|| usage()).clone();
            }
            "--engine" => {
                index += 1;
                options.engine = args.get(index).unwrap_or_else(|| usage()).clone();
            }
            "--prompt" => {
                index += 1;
                options
                    .prompt
                    .push(args.get(index).unwrap_or_else(|| usage()).clone());
            }
            "--prompt-file" => {
                index += 1;
                options
                    .prompt_file
                    .push(PathBuf::from(args.get(index).unwrap_or_else(|| usage())));
            }
            "--dataset" => {
                index += 1;
                options
                    .dataset
                    .push(PathBuf::from(args.get(index).unwrap_or_else(|| usage())));
            }
            "--output" => {
                index += 1;
                options.output = Some(PathBuf::from(args.get(index).unwrap_or_else(|| usage())));
            }
            "--json-output" => {
                index += 1;
                options.json_output =
                    Some(PathBuf::from(args.get(index).unwrap_or_else(|| usage())));
            }
            "--parallel-tests" => {
                index += 1;
                options.parallel_tests = Some(args.get(index).unwrap_or_else(|| usage()).clone());
            }
            "--require-finding" => {
                index += 1;
                options
                    .require_finding
                    .push(args.get(index).unwrap_or_else(|| usage()).clone());
            }
            "--expect-findings" => options.expect_findings = true,
            "--dry-run" => options.dry_run = true,
            _ => usage(),
        }
        index += 1;
    }
    println!(
        "{}",
        autoreview::run(&repo_root(), &options).unwrap_or_else(exit_error)
    );
}

fn run_event(args: &[String]) {
    let Some((subcommand, rest)) = args.split_first() else {
        usage();
    };
    match subcommand.as_str() {
        "kinds" => {
            let [] = rest else {
                usage();
            };
            println!("{}", events::known_kinds().join("\n"));
        }
        "emit" => {
            let [log, kind, phase, agent, payload] = rest else {
                usage();
            };
            events::emit_event(Path::new(log), kind, phase, agent, payload)
                .unwrap_or_else(exit_error);
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
            let output = git_hooks::run_pre_push(&repo_root()).unwrap_or_else(exit_error);
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

fn run_heal_support(args: &[String]) {
    let Some((subcommand, rest)) = args.split_first() else {
        usage();
    };
    match subcommand.as_str() {
        "first-failed-gate" => {
            let [summary] = rest else {
                usage();
            };
            if let Some(gate) = heal_support::first_failed_gate(summary) {
                println!("{gate}");
            }
        }
        "repair-branch-name" => {
            let [gate] = rest else {
                usage();
            };
            println!("{}", heal_support::repair_branch_name(gate));
        }
        "repair-commit-message" => {
            let [gate] = rest else {
                usage();
            };
            println!("{}", heal_support::repair_commit_message(gate));
        }
        "snapshot-delta" => {
            let [before, after] = rest else {
                usage();
            };
            let (stage, remove) = heal_support::snapshot_delta(Path::new(before), Path::new(after))
                .unwrap_or_else(exit_error);
            for path in remove {
                println!("D\t{path}");
            }
            for path in stage {
                println!("S\t{path}");
            }
        }
        "parse-failures-json" => {
            let [summary] = rest else {
                usage();
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&heal_support::parse_check_failures(summary))
                    .map_err(anyhow::Error::from)
                    .unwrap_or_else(exit_error)
            );
        }
        "select-healable-json" => {
            let [summary] = rest else {
                usage();
            };
            let failures = heal_support::parse_check_failures(summary);
            let failure =
                heal_support::select_healable_failure(&failures).unwrap_or_else(exit_error);
            println!(
                "{}",
                serde_json::to_string_pretty(&failure)
                    .map_err(anyhow::Error::from)
                    .unwrap_or_else(exit_error)
            );
        }
        "repair-prompt" => {
            let [name, detail, attempt, attempts] = rest else {
                usage();
            };
            let attempt = attempt
                .parse::<usize>()
                .map_err(|error| anyhow::anyhow!("invalid attempt: {error}"))
                .unwrap_or_else(exit_error);
            let attempts = attempts
                .parse::<usize>()
                .map_err(|error| anyhow::anyhow!("invalid attempts: {error}"))
                .unwrap_or_else(exit_error);
            let failure = heal_support::GateFailure {
                name: name.to_string(),
                detail: detail.to_string(),
            };
            println!(
                "{}",
                heal_support::repair_prompt(&failure, attempt, attempts).unwrap_or_else(exit_error)
            );
        }
        _ => usage(),
    }
}

fn run_evidence(args: &[String]) {
    let Some((subcommand, rest)) = args.split_first() else {
        usage();
    };
    match subcommand.as_str() {
        "branch-slug" => {
            let branch = optional_value(rest);
            println!(
                "{}",
                evidence::branch_slug(Path::new("."), branch).unwrap_or_else(exit_error)
            );
        }
        "date" => {
            let [] = rest else {
                usage();
            };
            println!("{}", evidence::evidence_date());
        }
        "dir" => {
            let (branch, day) = optional_pair(rest);
            println!(
                "{}",
                evidence::evidence_dir(Path::new("."), branch, day).unwrap_or_else(exit_error)
            );
        }
        "create" => {
            let (branch, day) = optional_pair(rest);
            println!(
                "{}",
                evidence::evidence_dir_create(Path::new("."), branch, day)
                    .unwrap_or_else(exit_error)
            );
        }
        "trailer" => {
            let (dir, branch, day) = optional_triple(rest);
            if let Some(trailer) = evidence::evidence_trailer(Path::new("."), dir, branch, day)
                .unwrap_or_else(exit_error)
            {
                println!("{trailer}");
            }
        }
        _ => usage(),
    }
}

fn optional_value(args: &[String]) -> Option<&str> {
    match args {
        [] => None,
        [value] => Some(value),
        _ => usage(),
    }
}

fn optional_pair(args: &[String]) -> (Option<&str>, Option<&str>) {
    match args {
        [] => (None, None),
        [first] => (Some(first), None),
        [first, second] => (Some(first), Some(second)),
        _ => usage(),
    }
}

fn optional_triple(args: &[String]) -> (Option<&str>, Option<&str>, Option<&str>) {
    match args {
        [] => (None, None, None),
        [first] => (Some(first), None, None),
        [first, second] => (Some(first), Some(second), None),
        [first, second, third] => (Some(first), Some(second), Some(third)),
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

fn run_verdict(args: &[String]) {
    let Some((subcommand, rest)) = args.split_first() else {
        usage();
    };
    match subcommand.as_str() {
        "write" => {
            let [branch, json] = rest else {
                usage();
            };
            verdicts::write(Path::new("."), branch, json).unwrap_or_else(exit_error);
        }
        "read" => {
            let [branch] = rest else {
                usage();
            };
            let json = verdicts::read(Path::new("."), branch).unwrap_or_else(exit_error);
            print!("{json}");
        }
        "validate" => {
            let [branch] = rest else {
                usage();
            };
            verdicts::validate(Path::new("."), branch).unwrap_or_else(exit_error);
        }
        "check-landable" => {
            let [branch] = rest else {
                usage();
            };
            match verdicts::check_landable(Path::new("."), branch).unwrap_or_else(exit_error) {
                verdicts::Landable::Yes => {}
                verdicts::Landable::No => std::process::exit(1),
                verdicts::Landable::DontShip => std::process::exit(2),
            }
        }
        "delete" => {
            let [branch] = rest else {
                usage();
            };
            verdicts::delete(Path::new("."), branch).unwrap_or_else(exit_error);
        }
        "list" => {
            let [] = rest else {
                usage();
            };
            print!(
                "{}",
                verdicts::list(Path::new(".")).unwrap_or_else(exit_error)
            );
        }
        "push" => {
            let remote = optional_remote(rest);
            verdicts::push(Path::new("."), remote).unwrap_or_else(exit_error);
        }
        "fetch" => {
            let remote = optional_remote(rest);
            verdicts::fetch(Path::new("."), remote).unwrap_or_else(exit_error);
        }
        _ => usage(),
    }
}

fn optional_remote(args: &[String]) -> &str {
    match args {
        [] => "origin",
        [remote] => remote,
        _ => usage(),
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

fn parse_check_agent_roster_args(args: &[String]) -> PathBuf {
    parse_repo_arg(args)
}

fn parse_repo_arg(args: &[String]) -> PathBuf {
    match args {
        [] => PathBuf::from("."),
        [flag, path] if flag == "--repo" => PathBuf::from(path),
        _ => usage(),
    }
}

fn parse_paths_arg(args: &[String], defaults: &[PathBuf]) -> Vec<PathBuf> {
    if args.is_empty() {
        defaults.to_vec()
    } else if args.iter().any(|arg| arg.starts_with('-')) {
        usage()
    } else {
        args.iter().map(PathBuf::from).collect()
    }
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

fn run_design_eval(args: &[String]) {
    if args == ["--self-test"] {
        match design_eval::self_test() {
            Ok(message) => {
                println!("{message}");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("{error:#}");
                std::process::exit(1);
            }
        }
    }
    let (mode, path) = match args {
        [path] => (
            design_eval::DesignEvalMode::RenderedCritique,
            PathBuf::from(path),
        ),
        [mode, path] => {
            let Some(mode) = design_eval::DesignEvalMode::parse(mode) else {
                usage();
            };
            (mode, PathBuf::from(path))
        }
        _ => usage(),
    };
    match design_eval::grade(mode, &path) {
        Ok(message) => {
            println!("{message}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_critique_eval(args: &[String]) {
    if args == ["--self-test"] {
        match critique_eval::self_test() {
            Ok(message) => {
                println!("{message}");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("{error:#}");
                std::process::exit(1);
            }
        }
    }
    let path = match args {
        [path] => PathBuf::from(path),
        _ => usage(),
    };
    match critique_eval::grade(&path) {
        Ok(message) => {
            println!("{message}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_eval_grader(args: &[String]) {
    if args == ["--self-test"] {
        match eval_graders::self_test() {
            Ok(message) => {
                println!("{message}");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("{error:#}");
                std::process::exit(1);
            }
        }
    }
    let (grader, path) = match args {
        [grader, path] => {
            let Some(grader) = eval_graders::EvalGrader::parse(grader) else {
                usage();
            };
            (grader, PathBuf::from(path))
        }
        _ => usage(),
    };
    match eval_graders::grade(grader, &path) {
        Ok(message) => {
            println!("{message}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_audit_skills(args: &[String]) {
    let options = parse_audit_skills_args(args);
    let root = match skill_audit::repo_root(&options.repo) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    let report = match skill_audit::audit_repo(&root)
        .map(|(audits, routing_paths)| skill_audit::render_report(&audits, &routing_paths))
    {
        Ok(report) => report,
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    };
    if let Some(output) = options.output {
        let output = if output.is_absolute() {
            output
        } else {
            root.join(output)
        };
        if let Some(parent) = output.parent()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            eprintln!("failed to create {}: {error}", parent.display());
            std::process::exit(1);
        }
        if let Err(error) = std::fs::write(&output, report) {
            eprintln!("failed to write {}: {error}", output.display());
            std::process::exit(1);
        }
    } else {
        print!("{report}");
    }
}

struct AuditSkillsOptions {
    repo: PathBuf,
    output: Option<PathBuf>,
}

fn parse_audit_skills_args(args: &[String]) -> AuditSkillsOptions {
    let mut options = AuditSkillsOptions {
        repo: PathBuf::from("."),
        output: None,
    };
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--repo" => options.repo = PathBuf::from(value()),
            "--output" => options.output = Some(PathBuf::from(value())),
            _ => usage(),
        }
        index += 1;
    }
    options
}

fn run_repo_skill(args: &[String]) {
    let Some((command, rest)) = args.split_first() else {
        usage();
    };
    let result = match command.as_str() {
        "scaffold" => {
            let options = parse_repo_skill_scaffold_args(rest);
            repo_skill::scaffold(&options).map(|paths| {
                paths
                    .into_iter()
                    .map(|path| path.display().to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            })
        }
        "validate" => {
            let Some(path) = rest.first() else {
                usage();
            };
            if rest.len() != 1 {
                usage();
            }
            repo_skill::validate(Path::new(path))
        }
        "self-test" => {
            no_args(rest).unwrap_or_else(|_| usage());
            repo_skill::self_test()
        }
        _ => usage(),
    };
    match result {
        Ok(message) => println!("{message}"),
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn parse_repo_skill_scaffold_args(args: &[String]) -> repo_skill::ScaffoldOptions {
    let Some((name, rest)) = args.split_first() else {
        usage();
    };
    let mut root = PathBuf::from(".");
    let mut kind = repo_skill::SkillKind::Generic;
    let mut index = 0;
    while index < rest.len() {
        let flag = rest[index].as_str();
        index += 1;
        let value = || rest.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--repo" => root = PathBuf::from(value()),
            "--kind" => {
                kind = repo_skill::SkillKind::parse(&value()).unwrap_or_else(|error| {
                    eprintln!("{error}");
                    std::process::exit(2);
                });
            }
            _ => usage(),
        }
        index += 1;
    }
    repo_skill::ScaffoldOptions {
        root,
        name: name.clone(),
        kind,
    }
}

fn run_agent_readiness_profile(args: &[String]) {
    let (profile, command, rest) = parse_agent_readiness_prefix(args);
    let result = match command.as_str() {
        "create" => {
            let (repo_root, force) = parse_agent_readiness_create_args(&rest);
            agent_readiness_profile::create(&agent_readiness_profile::CreateOptions {
                profile,
                repo_root,
                force,
            })
        }
        "read" => no_args(&rest).and_then(|()| agent_readiness_profile::read(&profile)),
        "validate" => no_args(&rest).and_then(|()| agent_readiness_profile::validate(&profile)),
        "update" => {
            let options = parse_agent_readiness_update_args(profile, &rest);
            agent_readiness_profile::update(&options)
        }
        "delete" => {
            let waiver_id = parse_agent_readiness_delete_args(&rest);
            agent_readiness_profile::delete(&profile, &waiver_id)
        }
        _ => usage(),
    };
    match result {
        Ok(message) => {
            println!("{message}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("profile-crud: {}", error.message());
            std::process::exit(2);
        }
    }
}

fn parse_agent_readiness_prefix(args: &[String]) -> (PathBuf, String, Vec<String>) {
    let mut profile = agent_readiness_profile::default_profile_path();
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--profile" {
            index += 1;
            profile = PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
            index += 1;
        } else if args[index].starts_with('-') {
            usage();
        } else {
            return (profile, args[index].clone(), args[index + 1..].to_vec());
        }
    }
    usage()
}

fn parse_agent_readiness_create_args(args: &[String]) -> (PathBuf, bool) {
    let mut repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let mut force = false;
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        match flag {
            "--repo-root" => {
                repo_root = PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
                index += 1;
            }
            "--force" => force = true,
            _ => usage(),
        }
    }
    (repo_root, force)
}

fn parse_agent_readiness_update_args(
    profile: PathBuf,
    args: &[String],
) -> agent_readiness_profile::UpdateOptions {
    let mut options = agent_readiness_profile::UpdateOptions {
        profile,
        waiver_id: String::new(),
        scope: String::new(),
        reason: String::new(),
        expires_on: String::new(),
        adr: String::new(),
        readiness_state: None,
    };
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--waiver-id" => options.waiver_id = value(),
            "--scope" => options.scope = value(),
            "--reason" => options.reason = value(),
            "--expires-on" => options.expires_on = value(),
            "--adr" => options.adr = value(),
            "--readiness-state" => {
                let state = value();
                if !["improved", "preserved", "regressed", "unknown"].contains(&state.as_str()) {
                    usage();
                }
                options.readiness_state = Some(state);
            }
            _ => usage(),
        }
        index += 1;
    }
    if options.waiver_id.is_empty()
        || options.scope.is_empty()
        || options.reason.is_empty()
        || options.expires_on.is_empty()
        || options.adr.is_empty()
    {
        usage();
    }
    options
}

fn parse_agent_readiness_delete_args(args: &[String]) -> String {
    let mut waiver_id = String::new();
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        match flag {
            "--waiver-id" => {
                waiver_id = args.get(index).cloned().unwrap_or_else(|| usage());
                index += 1;
            }
            _ => usage(),
        }
    }
    if waiver_id.is_empty() {
        usage();
    }
    waiver_id
}

fn no_args(args: &[String]) -> agent_readiness_profile::ProfileResult<()> {
    if args.is_empty() { Ok(()) } else { usage() }
}

fn run_agent_transcript(args: &[String]) {
    let options = parse_agent_transcript_args(args);
    if options.self_test {
        match agent_transcript::self_test() {
            Ok(message) => {
                println!("{message}");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("{error:#}");
                std::process::exit(1);
            }
        }
    }
    let input = match options.input {
        Some(path) => std::fs::read_to_string(&path),
        None => {
            let mut buffer = String::new();
            match std::io::Read::read_to_string(&mut std::io::stdin(), &mut buffer) {
                Ok(_) => Ok(buffer),
                Err(error) => Err(error),
            }
        }
    };
    let input = match input {
        Ok(input) => input,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    };
    match agent_transcript::render_block(&input, &options.title) {
        Ok(rendered) => {
            if let Some(output) = options.output {
                if let Err(error) = std::fs::write(&output, rendered) {
                    eprintln!("{error}");
                    std::process::exit(1);
                }
            } else {
                print!("{rendered}");
            }
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

struct AgentTranscriptOptions {
    input: Option<PathBuf>,
    output: Option<PathBuf>,
    title: String,
    self_test: bool,
}

fn parse_agent_transcript_args(args: &[String]) -> AgentTranscriptOptions {
    let mut options = AgentTranscriptOptions {
        input: None,
        output: None,
        title: "Agent Transcript".to_string(),
        self_test: false,
    };
    let mut index = 0;
    if args.first().is_some_and(|command| command == "render") {
        index = 1;
    }
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--input" | "-i" => options.input = Some(PathBuf::from(value())),
            "--output" | "-o" => options.output = Some(PathBuf::from(value())),
            "--title" => options.title = value(),
            "--self-test" => {
                options.self_test = true;
                index -= 1;
            }
            _ => usage(),
        }
        index += 1;
    }
    options
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

fn run_reflect_gather_evidence(args: &[String]) {
    let count = match reflect_evidence::parse_commit_count(args) {
        Ok(count) => count,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    print!("{}", reflect_evidence::gather(&cwd, count));
    std::process::exit(0);
}

fn run_reflect_checkpoint(args: &[String]) {
    if args == ["--self-test"] {
        println!(
            "{}",
            reflect_checkpoint::self_test().unwrap_or_else(exit_error)
        );
        return;
    }
    let Some((command, rest)) = args.split_first() else {
        usage();
    };
    if command != "validate" {
        usage();
    }
    let mut checkpoint = None;
    let mut gate = None;
    let mut packet = None;
    let mut index = 0;
    while index < rest.len() {
        match rest[index].as_str() {
            "--gate" => {
                index += 1;
                gate = Some(rest.get(index).unwrap_or_else(|| usage()).clone());
            }
            "--packet" => {
                index += 1;
                packet = Some(PathBuf::from(rest.get(index).unwrap_or_else(|| usage())));
            }
            value if checkpoint.is_none() => checkpoint = Some(PathBuf::from(value)),
            _ => usage(),
        }
        index += 1;
    }
    if let Some(topic) = gate {
        reflect_checkpoint::gate(checkpoint.as_deref(), &topic, packet.as_deref())
            .unwrap_or_else(exit_error);
    } else {
        let Some(path) = checkpoint else {
            usage();
        };
        reflect_checkpoint::validate_path(&path).unwrap_or_else(exit_error);
    }
}

fn run_premise_source(args: &[String]) {
    let Some((command, rest)) = args.split_first() else {
        usage();
    };
    match command.as_str() {
        "self-test" => println!(
            "{}",
            premise_source::self_test(&repo_root()).unwrap_or_else(exit_error)
        ),
        "validate" => {
            let [packet] = rest else {
                usage();
            };
            premise_source::validate_packet(&repo_root(), &PathBuf::from(packet))
                .unwrap_or_else(exit_error);
            println!("PASS: premise source valid in {packet}");
        }
        _ => usage(),
    }
}

fn run_mine_transcript_effectiveness(args: &[String]) {
    if args == ["--self-test"] {
        println!(
            "{}",
            transcript_effectiveness::self_test().unwrap_or_else(exit_error)
        );
        return;
    }
    let mut options = transcript_effectiveness::Options {
        transcripts: Vec::new(),
        source_roots: Vec::new(),
        skill_log: env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".claude/skill-invocations.jsonl"),
        work_ledger: PathBuf::from(".harness-kit/work/ledger.jsonl"),
        delegations: PathBuf::from(".harness-kit/traces/delegations.jsonl"),
        review_scores: PathBuf::from(".groom/review-scores.ndjson"),
        allow_redacted_excerpts: false,
    };
    let mut format = "markdown".to_string();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--transcript" => {
                index += 1;
                options
                    .transcripts
                    .push(PathBuf::from(args.get(index).unwrap_or_else(|| usage())));
            }
            "--source-root" => {
                index += 1;
                options
                    .source_roots
                    .push(PathBuf::from(args.get(index).unwrap_or_else(|| usage())));
            }
            "--skill-log" => {
                index += 1;
                options.skill_log = PathBuf::from(args.get(index).unwrap_or_else(|| usage()));
            }
            "--work-ledger" => {
                index += 1;
                options.work_ledger = PathBuf::from(args.get(index).unwrap_or_else(|| usage()));
            }
            "--delegations" => {
                index += 1;
                options.delegations = PathBuf::from(args.get(index).unwrap_or_else(|| usage()));
            }
            "--review-scores" => {
                index += 1;
                options.review_scores = PathBuf::from(args.get(index).unwrap_or_else(|| usage()));
            }
            "--allow-redacted-excerpts" => options.allow_redacted_excerpts = true,
            "--format" => {
                index += 1;
                format = args.get(index).unwrap_or_else(|| usage()).clone();
            }
            _ => usage(),
        }
        index += 1;
    }
    let report = transcript_effectiveness::build_report(&options).unwrap_or_else(exit_error);
    match format.as_str() {
        "json" => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(anyhow::Error::from)
                .unwrap_or_else(exit_error)
        ),
        "markdown" => println!("{}", transcript_effectiveness::render_markdown(&report)),
        _ => usage(),
    }
}

fn run_offline_validation_preflight(args: &[String]) {
    if !args.is_empty() {
        usage();
    }
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let repo_root = match offline_validation_preflight::discover_repo_root(&cwd) {
        Ok(root) => root,
        Err(error) => {
            eprintln!("FAIL: {error}");
            println!("\nSummary: 1 failure(s), 0 warning(s)");
            std::process::exit(1);
        }
    };
    let path_env = env::var_os("PATH").unwrap_or_default();
    match offline_validation_preflight::run(&offline_validation_preflight::PreflightOptions {
        repo_root,
        path_env,
    }) {
        Ok(report) => {
            let (stdout, stderr) = offline_validation_preflight::render(&report);
            print!("{stdout}");
            eprint!("{stderr}");
            std::process::exit(if report.success() { 0 } else { 1 });
        }
        Err(error) => {
            eprintln!("FAIL: {error:#}");
            println!("\nSummary: 1 failure(s), 0 warning(s)");
            std::process::exit(1);
        }
    }
}

fn run_detect_ui_surfaces(args: &[String]) {
    let mode = parse_detect_ui_surfaces_args(args);
    match detect_ui_surfaces::detect(&repo_root(), &mode)
        .and_then(|report| serde_json::to_string(&report).map_err(Into::into))
    {
        Ok(json) => {
            println!("{json}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    }
}

fn parse_detect_ui_surfaces_args(args: &[String]) -> detect_ui_surfaces::DetectMode {
    match args {
        [] => detect_ui_surfaces::DetectMode::Unstaged,
        [flag] if flag == "--staged" => detect_ui_surfaces::DetectMode::Staged,
        [flag] if flag == "--unstaged" => detect_ui_surfaces::DetectMode::Unstaged,
        [flag, base] if flag == "--base" => detect_ui_surfaces::DetectMode::Base(base.clone()),
        [flag, paths @ ..] if flag == "--paths" && !paths.is_empty() => {
            detect_ui_surfaces::DetectMode::Paths(paths.to_vec())
        }
        [flag] if flag == "-h" || flag == "--help" => usage(),
        _ => usage(),
    }
}

fn run_skillify_classify(args: &[String]) {
    let options = parse_skillify_classify_args(args);
    match skillify_classify::classify(&options)
        .and_then(|value| serde_json::to_string_pretty(&value).map_err(Into::into))
    {
        Ok(json) => {
            println!("{json}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn parse_skillify_classify_args(args: &[String]) -> skillify_classify::ClassifyOptions {
    let Some((packet, rest)) = args.split_first() else {
        usage();
    };
    if packet.starts_with('-') {
        usage();
    }
    let mut options = skillify_classify::ClassifyOptions {
        packet_path: PathBuf::from(packet),
        roster_path: PathBuf::from(".harness-kit/agents.yaml"),
        repo_root: PathBuf::from("."),
        providers: Vec::new(),
        dry_run: false,
        timeout_s: 120,
    };
    let mut index = 0;
    while index < rest.len() {
        let flag = rest[index].as_str();
        index += 1;
        let value = || rest.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--roster" => options.roster_path = PathBuf::from(value()),
            "--repo-root" => options.repo_root = PathBuf::from(value()),
            "--provider" => options.providers.push(value()),
            "--dry-run" => {
                options.dry_run = true;
                index -= 1;
            }
            "--timeout-s" => options.timeout_s = parse_u64(flag, &value()),
            _ => usage(),
        }
        index += 1;
    }
    options
}

fn run_skillify_skill_crud(args: &[String]) {
    let options = parse_skillify_skill_crud_args(args);
    let result = match options.command.as_str() {
        "create" => skillify_skill_crud::create_skill(
            &options.skills_root,
            &options.name,
            &options.description,
            &options.body,
        ),
        "read" => skillify_skill_crud::read_skill(&options.skills_root, &options.name),
        "update" => skillify_skill_crud::update_skill(
            &options.skills_root,
            &options.name,
            (!options.description.is_empty()).then_some(options.description.as_str()),
            (!options.body.is_empty()).then_some(options.body.as_str()),
        ),
        "delete" => skillify_skill_crud::delete_skill(&options.skills_root, &options.name),
        "validate" => skillify_skill_crud::validate_skill(&options.skills_root, &options.name),
        _ => usage(),
    };
    match result.and_then(|value| serde_json::to_string(&value).map_err(Into::into)) {
        Ok(json) => {
            println!("{json}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

struct SkillifySkillCrudOptions {
    command: String,
    skills_root: PathBuf,
    name: String,
    description: String,
    body: String,
}

fn parse_skillify_skill_crud_args(args: &[String]) -> SkillifySkillCrudOptions {
    let Some((command, rest)) = args.split_first() else {
        usage();
    };
    if !["create", "read", "update", "delete", "validate"].contains(&command.as_str()) {
        usage();
    }
    let mut options = SkillifySkillCrudOptions {
        command: command.clone(),
        skills_root: PathBuf::from("skills"),
        name: String::new(),
        description: String::new(),
        body: String::new(),
    };
    let mut index = 0;
    while index < rest.len() {
        let flag = rest[index].as_str();
        index += 1;
        let value = || rest.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--skills-root" => options.skills_root = PathBuf::from(value()),
            "--name" => options.name = value(),
            "--description" => options.description = value(),
            "--body" => options.body = value(),
            _ => usage(),
        }
        index += 1;
    }
    if options.name.is_empty() {
        usage();
    }
    options
}

fn run_skillify_parse_transcript(args: &[String]) {
    let options = parse_skillify_parse_transcript_args(args);
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let transcript = match skillify_transcript::resolve_transcript(
        options.transcript,
        options.from_current,
        &options.claude_projects_dir,
        &cwd,
    ) {
        Ok(transcript) => transcript,
        Err(error) => {
            eprintln!("parse-transcript: {error}");
            std::process::exit(2);
        }
    };
    match skillify_transcript::parse_transcript(&transcript)
        .and_then(|packet| serde_json::to_string_pretty(&packet).map_err(Into::into))
    {
        Ok(json) => {
            println!("{json}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("parse-transcript: {error:#}");
            std::process::exit(2);
        }
    }
}

struct SkillifyParseTranscriptOptions {
    transcript: Option<PathBuf>,
    from_current: bool,
    claude_projects_dir: PathBuf,
}

fn parse_skillify_parse_transcript_args(args: &[String]) -> SkillifyParseTranscriptOptions {
    let mut transcript = None;
    let mut from_current = false;
    let mut claude_projects_dir = env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude/projects");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--from-current" => {
                from_current = true;
                index += 1;
            }
            "--claude-projects-dir" => {
                index += 1;
                claude_projects_dir =
                    PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
                index += 1;
            }
            value if value.starts_with('-') => usage(),
            value => {
                if transcript.is_some() {
                    usage();
                }
                transcript = Some(PathBuf::from(value));
                index += 1;
            }
        }
    }
    SkillifyParseTranscriptOptions {
        transcript,
        from_current,
        claude_projects_dir,
    }
}

fn run_shape_render(args: &[String]) {
    if args == ["--self-test"] {
        match shape_renderer::self_test() {
            Ok(()) => {
                println!("shape context packet renderer self-test ok");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("{error:#}");
                std::process::exit(1);
            }
        }
    }
    let mut source: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--output" => {
                index += 1;
                output = Some(PathBuf::from(
                    args.get(index).cloned().unwrap_or_else(|| usage()),
                ));
            }
            value if value.starts_with('-') => usage(),
            value => {
                if source.is_some() {
                    usage();
                }
                source = Some(PathBuf::from(value));
            }
        }
        index += 1;
    }
    let Some(source) = source else { usage() };
    let Some(output) = output else { usage() };
    let repo_root = default_repo_root();
    let source = if source.is_absolute() {
        source
    } else {
        repo_root.join(source)
    };
    let output = if output.is_absolute() {
        output
    } else {
        repo_root.join(output)
    };
    match shape_renderer::render(&shape_renderer::RenderOptions {
        repo_root: repo_root.clone(),
        source,
        output,
    }) {
        Ok(report) => {
            let label = report
                .output
                .strip_prefix(&repo_root)
                .unwrap_or(&report.output)
                .display();
            println!("Rendered {label}");
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("{error:#}");
            std::process::exit(1);
        }
    }
}

fn run_generate_embeddings(args: &[String]) {
    let mut options = embeddings::GenerateOptions {
        repo_root: repo_root(),
        dimensions: embeddings::DEFAULT_DIMS,
        dry_run: false,
        local_only: false,
        output: None,
        metadata_path: None,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--dimensions" => {
                index += 1;
                options.dimensions = args
                    .get(index)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| usage());
                index += 1;
            }
            "--dry-run" => {
                options.dry_run = true;
                index += 1;
            }
            "--local-only" => {
                options.local_only = true;
                index += 1;
            }
            "--output" => {
                index += 1;
                options.output = Some(PathBuf::from(
                    args.get(index).cloned().unwrap_or_else(|| usage()),
                ));
                index += 1;
            }
            "--metadata-path" => {
                index += 1;
                options.metadata_path = Some(PathBuf::from(
                    args.get(index).cloned().unwrap_or_else(|| usage()),
                ));
                index += 1;
            }
            _ => usage(),
        }
    }
    embeddings::generate_embeddings(&options).unwrap_or_else(exit_error);
}

fn run_search_embeddings(args: &[String]) {
    let mut options = embeddings::SearchOptions {
        repo_root: repo_root(),
        query: None,
        project_dir: None,
        top: 15,
        item_type: None,
        json: false,
        dimensions: embeddings::DEFAULT_DIMS,
    };
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--top" => {
                index += 1;
                options.top = args
                    .get(index)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| usage());
                index += 1;
            }
            "--type" => {
                index += 1;
                options.item_type = Some(args.get(index).cloned().unwrap_or_else(|| usage()));
                index += 1;
            }
            "--project-dir" => {
                index += 1;
                options.project_dir = Some(PathBuf::from(
                    args.get(index).cloned().unwrap_or_else(|| usage()),
                ));
                index += 1;
            }
            "--json" => {
                options.json = true;
                index += 1;
            }
            "--dimensions" => {
                index += 1;
                options.dimensions = args
                    .get(index)
                    .and_then(|value| value.parse().ok())
                    .unwrap_or_else(|| usage());
                index += 1;
            }
            value if value.starts_with('-') => usage(),
            value => {
                if options.query.is_some() {
                    usage();
                }
                options.query = Some(value.to_string());
                index += 1;
            }
        }
    }
    if options.query.is_none() && options.project_dir.is_none() {
        eprintln!("Usage: harness-kit-checks search-embeddings <query> | --project-dir <path>");
        std::process::exit(1);
    }
    embeddings::search_embeddings(&options).unwrap_or_else(exit_error);
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

fn default_repo_root() -> PathBuf {
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
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
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

fn run_work_ledger(args: &[String]) -> ! {
    if args == ["--self-test"] {
        match work_ledger::self_test() {
            Ok(()) => {
                println!("work-ledger self-test ok");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("work-ledger: {error:#}");
                std::process::exit(1);
            }
        }
    }
    let Some((command, rest)) = args.split_first() else {
        usage();
    };
    match command.as_str() {
        "append" => {
            let options = parse_work_ledger_append_args(rest);
            match work_ledger::append(&options) {
                Ok(receipt) => {
                    println!("{}", serde_json::to_string(&receipt).unwrap_or_default());
                    std::process::exit(0);
                }
                Err(error) => {
                    eprintln!("work-ledger: {error:#}");
                    std::process::exit(1);
                }
            }
        }
        "summary" => {
            let store = parse_work_ledger_summary_args(rest);
            match work_ledger::summary(&store) {
                Ok(summary) => {
                    println!("{summary}");
                    std::process::exit(0);
                }
                Err(error) => {
                    eprintln!("work-ledger: {error:#}");
                    std::process::exit(2);
                }
            }
        }
        _ => usage(),
    }
}

fn run_trace_record(args: &[String]) -> ! {
    if args == ["--self-test"] {
        match trace_record::self_test() {
            Ok(()) => {
                println!("trace_record self-test ok");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("trace_record: {error:#}");
                std::process::exit(2);
            }
        }
    }
    let Some((command, rest)) = args.split_first() else {
        usage();
    };
    match command.as_str() {
        "append" => {
            let options = parse_trace_record_append_args(rest);
            match trace_record::append(&options) {
                Ok(receipt) => {
                    println!("{}", serde_json::to_string(&receipt).unwrap_or_default());
                    std::process::exit(0);
                }
                Err(error) => {
                    eprintln!("trace_record: {error:#}");
                    std::process::exit(2);
                }
            }
        }
        _ => usage(),
    }
}

fn run_review_score_trends(args: &[String]) -> ! {
    if args == ["--self-test"] {
        match review_score_trends::self_test() {
            Ok(()) => {
                println!("OK: review-score trend self-test passed");
                std::process::exit(0);
            }
            Err(error) => {
                eprintln!("FAIL: {error:#}");
                std::process::exit(1);
            }
        }
    }
    let path = match args {
        [] => review_score_trends::default_path(),
        [path] if !path.starts_with('-') => PathBuf::from(path),
        _ => usage(),
    };
    match review_score_trends::load_rows(&path) {
        Ok(rows) => {
            println!("{}", review_score_trends::report(&rows, &path));
            std::process::exit(0);
        }
        Err(error) => {
            eprintln!("FAIL: {error:#}");
            std::process::exit(1);
        }
    }
}

fn parse_trace_record_append_args(args: &[String]) -> trace_record::AppendOptions {
    let mut options = trace_record::AppendOptions {
        store: trace_record::default_store(),
        backlog: String::new(),
        spec_ref: String::new(),
        branch: String::new(),
        commits: Vec::new(),
        reviewer_verdict_refs: Vec::new(),
        qa_refs: Vec::new(),
        demo_refs: Vec::new(),
        transcript_refs: Vec::new(),
        shipped_ref: String::new(),
        waiver_reason: String::new(),
        metadata: Vec::new(),
    };
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--store" => options.store = PathBuf::from(value()),
            "--backlog" => options.backlog = value(),
            "--spec-ref" => options.spec_ref = value(),
            "--branch" => options.branch = value(),
            "--commit" => options.commits.push(value()),
            "--reviewer-verdict-ref" => options.reviewer_verdict_refs.push(value()),
            "--qa-ref" => options.qa_refs.push(value()),
            "--demo-ref" => options.demo_refs.push(value()),
            "--transcript-ref" => options.transcript_refs.push(value()),
            "--shipped-ref" => options.shipped_ref = value(),
            "--waiver-reason" => options.waiver_reason = value(),
            "--metadata" => options.metadata.push(value()),
            _ => usage(),
        }
        index += 1;
    }
    if options.backlog.is_empty() || options.branch.is_empty() {
        usage();
    }
    options
}

fn parse_work_ledger_append_args(args: &[String]) -> work_ledger::AppendOptions {
    let mut options = work_ledger::AppendOptions {
        store: work_ledger::default_store(),
        event_type: String::new(),
        work_id: String::new(),
        parent_work_id: String::new(),
        backlog: String::new(),
        branch: String::new(),
        owning_skill: String::new(),
        phase: String::new(),
        evidence_refs: Vec::new(),
        blockers: Vec::new(),
        spawned_agents: Vec::new(),
        trace_refs: Vec::new(),
        next_action: String::new(),
        status: "active".to_string(),
        usage: None,
    };
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--store" => options.store = PathBuf::from(value()),
            "--event-type" => options.event_type = value(),
            "--work-id" => options.work_id = value(),
            "--parent-work-id" => options.parent_work_id = value(),
            "--backlog" => options.backlog = value(),
            "--branch" => options.branch = value(),
            "--owning-skill" => options.owning_skill = value(),
            "--phase" => options.phase = value(),
            "--evidence-ref" => options.evidence_refs.push(value()),
            "--blocker" => options.blockers.push(value()),
            "--spawned-agent" => options.spawned_agents.push(value()),
            "--trace-ref" => options.trace_refs.push(value()),
            "--next-action" => options.next_action = value(),
            "--status" => options.status = value(),
            "--usage-json" => {
                let raw = value();
                match work_ledger::parse_usage_json(Some(&raw)) {
                    Ok(usage) => options.usage = usage,
                    Err(error) => {
                        eprintln!("work-ledger: {error:#}");
                        std::process::exit(1);
                    }
                }
            }
            _ => usage(),
        }
        index += 1;
    }
    if options.event_type.is_empty()
        || options.work_id.is_empty()
        || options.backlog.is_empty()
        || options.branch.is_empty()
        || options.owning_skill.is_empty()
        || options.phase.is_empty()
        || options.next_action.is_empty()
    {
        usage();
    }
    options
}

fn parse_work_ledger_summary_args(args: &[String]) -> PathBuf {
    let mut store = work_ledger::default_store();
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--store" => {
                index += 1;
                store = PathBuf::from(args.get(index).cloned().unwrap_or_else(|| usage()));
            }
            _ => usage(),
        }
        index += 1;
    }
    store
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
        },
    }
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

struct MaterializeLaneHarnessOptions {
    repo: PathBuf,
    roster: PathBuf,
    manifest: PathBuf,
    root: Option<PathBuf>,
}

fn parse_materialize_lane_harness_args(args: &[String]) -> MaterializeLaneHarnessOptions {
    let mut options = MaterializeLaneHarnessOptions {
        repo: repo_root(),
        roster: default_roster_path(),
        manifest: PathBuf::new(),
        root: None,
    };
    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        index += 1;
        let value = || args.get(index).cloned().unwrap_or_else(|| usage());
        match flag {
            "--repo" => options.repo = PathBuf::from(value()),
            "--roster" => options.roster = PathBuf::from(value()),
            "--manifest" => options.manifest = PathBuf::from(value()),
            "--root" => options.root = Some(PathBuf::from(value())),
            _ => usage(),
        }
        index += 1;
    }
    if options.manifest.as_os_str().is_empty() {
        usage();
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
  harness-kit-checks check-agent-roster [--repo PATH]
  harness-kit-checks autoreview [--mode auto|local|branch|commit] [--base REF] [--engine codex|claude|droid|copilot] [--dry-run]
  harness-kit-checks autoreview --self-test
  harness-kit-checks check-frontmatter [--repo PATH]
  harness-kit-checks check-bench-map [--repo PATH]
  harness-kit-checks generate-index [--repo PATH]
  harness-kit-checks generate-embeddings [--dimensions N] [--dry-run] [--local-only] [--output PATH] [--metadata-path PATH]
  harness-kit-checks search-embeddings <query> [--top N] [--type skill|agent] [--json]
  harness-kit-checks search-embeddings --project-dir PATH [--top N] [--json]
  harness-kit-checks check-index-drift [--repo PATH]
  harness-kit-checks bootstrap [--repo PATH]
  harness-kit-checks build-docs-site [--repo PATH] [--output PATH]
  harness-kit-checks check-docs-site [--repo PATH] [--site PATH] [--self-test]
  harness-kit-checks test-bootstrap-agent-allowlist [--repo PATH]
  harness-kit-checks test-sync-external-partial
  harness-kit-checks check-runtime-primitives [--repo PATH]
  harness-kit-checks design-eval --self-test
  harness-kit-checks design-eval [rendered-critique|scaffold-contract|design-contract-maintenance|token-only-critique] <candidate-output>
  harness-kit-checks critique-eval --self-test
  harness-kit-checks critique-eval <candidate-output>
  harness-kit-checks eval-grader --self-test
  harness-kit-checks eval-grader code-review-entrypoint|code-review-repo-fit|create-repo-skill|orient-session-start|qa-cli-smoke|qa-browser-missing-selector|qa-non-browser <candidate-output>
  harness-kit-checks load-config deploy|monitor|flywheel [--repo PATH] [--config PATH] [--optional]
  harness-kit-checks work-ledger --self-test
  harness-kit-checks work-ledger append [options]
  harness-kit-checks work-ledger summary [--store PATH]
  harness-kit-checks trace-record --self-test
  harness-kit-checks trace-record append [options]
  harness-kit-checks review-score-trends [--self-test|PATH]
  harness-kit-checks repo-skill scaffold NAME [--kind qa|persona-acceptance|generic] [--repo PATH]
  harness-kit-checks repo-skill validate .agents/skills/NAME
  harness-kit-checks repo-skill self-test
  harness-kit-checks shape-render SOURCE --output OUTPUT
  harness-kit-checks shape-render --self-test
  harness-kit-checks skill-invocation-analytics [--skill-log PATH] [--work-ledger PATH] [--delegations PATH] [--since 7d|12h] [--repo NAME] [--project NAME] [--skill NAME] [--format json|text|markdown] [--self-test]
  harness-kit-checks detect-ui-surfaces [--staged|--unstaged|--base REF|--paths PATH...]
  harness-kit-checks offline-validation-preflight
  harness-kit-checks reflect-gather-evidence [N]
  harness-kit-checks reflect-checkpoint validate [CHECKPOINT] [--gate TOPIC] [--packet PATH]
  harness-kit-checks reflect-checkpoint --self-test
  harness-kit-checks premise-source validate PACKET
  harness-kit-checks premise-source self-test
  harness-kit-checks mine-transcript-effectiveness [--transcript PATH|--source-root PATH] [--format json|markdown] [--self-test]
  harness-kit-checks lint-external-skills [--strict]
  harness-kit-checks sync-external [--repo PATH] [--check] [--allow-floating] [--only owner/repo]
  harness-kit-checks fetch-pr-reviews [PR]
  harness-kit-checks agent-transcript [render] [--input PATH] [--output PATH] [--title TEXT] [--self-test]
  harness-kit-checks skillify-classify PACKET [--roster PATH] [--repo-root PATH] [--provider ID] [--dry-run] [--timeout-s N]
  harness-kit-checks skillify-skill-crud create|read|update|delete|validate --name NAME [--skills-root PATH] [--description TEXT] [--body TEXT]
  harness-kit-checks skillify-parse-transcript [TRANSCRIPT] [--from-current] [--claude-projects-dir PATH]
  harness-kit-checks check-evidence-blocks [PATH ...]
  harness-kit-checks check-exclusions [--repo PATH]
  harness-kit-checks check-conflict-markers [--repo PATH]
  harness-kit-checks check-portable-paths [--repo PATH]
  harness-kit-checks check-deliver-composition [--repo PATH]
  harness-kit-checks check-no-claims [--repo PATH]
  harness-kit-checks check-offline-evidence-storage [--repo PATH]
  harness-kit-checks check-vendored-copies [--repo PATH]
  harness-kit-checks check-harness-install-paths [--repo PATH]
  harness-kit-checks check-skill-evals [--repo PATH]
  harness-kit-checks audit-skills [--repo PATH] [--output PATH]
  harness-kit-checks agent-readiness-profile [--profile PATH] create [--repo-root PATH] [--force]
  harness-kit-checks agent-readiness-profile [--profile PATH] read|validate
  harness-kit-checks agent-readiness-profile [--profile PATH] update --waiver-id ID --scope TEXT --reason TEXT --expires-on YYYY-MM-DD --adr TEXT [--readiness-state STATE]
  harness-kit-checks agent-readiness-profile [--profile PATH] delete --waiver-id ID
  harness-kit-checks backlog trailer-keys|closing-keys
  harness-kit-checks backlog ids-from-commit|ids-from-range|file-for-id|archive <arg>
  harness-kit-checks claude-hook block-master-push|check-todo-quality|codex-post-feedback|codex-session-init|destructive-command-guard|disk-space-guard|env-var-newline-guard|exa-research-reminder|exclusion-guard|fix-what-you-touch|github-cli-guard|permission-auto-approve|portable-code-guard|session-health-check|shaping-ripple|skill-invocation-tracker|stop-quality-gate|time-context
  harness-kit-checks evidence branch-slug|date|dir|create|trailer [arg...]
  harness-kit-checks event kinds
  harness-kit-checks event emit LOG KIND PHASE AGENT PAYLOAD_JSON
  harness-kit-checks git-hook pre-commit|pre-push|pre-merge-commit|post-commit|post-merge|post-rewrite [arg...]
  harness-kit-checks heal-support first-failed-gate|repair-branch-name|repair-commit-message <arg>
  harness-kit-checks heal-support snapshot-delta BEFORE AFTER
  harness-kit-checks heal-support parse-failures-json|select-healable-json SUMMARY
  harness-kit-checks heal-support repair-prompt NAME DETAIL ATTEMPT ATTEMPTS
  harness-kit-checks heal-commit [dagger heal args...]
  harness-kit-checks verdict write|read|validate|check-landable|delete <branch>
  harness-kit-checks verdict list|push|fetch [remote]
  harness-kit-checks probe-agent-roster [--validate-only] [--write-receipts] [options]
  harness-kit-checks materialize-lane-harness --manifest PATH [--root PATH] [--repo PATH] [--roster PATH]
  harness-kit-checks dispatch-agent --provider-target ID --objective TEXT --input-ref REF --prompt-file PATH [--repo PATH] [--lane-harness PATH] [--keep-lane-root] [options]
  harness-kit-checks summarize-delegations [--backlog-ref REF] [--format json|text] [PATH]
  harness-kit-checks record-delegation --provider-target ID --provider-status STATUS --attempt-status STATUS --objective TEXT --input-ref REF --worktree-id ID [options]"#
    );
    std::process::exit(2);
}
