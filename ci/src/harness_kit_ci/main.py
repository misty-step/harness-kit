"""Harness Kit CI pipeline — local-first quality gates via Dagger."""

import json
from typing import Annotated

import anyio

import dagger
from dagger import DefaultPath, Doc, Ignore, dag, function, object_type

PYTHON_IMAGE = "public.ecr.aws/docker/library/python:3.12-slim"
RUST_IMAGE = "public.ecr.aws/docker/library/rust:1.88-slim"


async def _repair_prompt(
    source: dagger.Directory,
    failure: dict[str, str],
    attempt: int,
    attempts: int,
) -> str:
    """Build the LLM repair prompt via the Rust CLI contract."""
    return await (
        _rust_container(source)
        .with_exec(
            [
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "heal-support",
                "repair-prompt",
                failure["name"],
                failure["detail"],
                str(attempt),
                str(attempts),
            ]
        )
        .stdout()
    )


async def _select_healable_failure(
    source: dagger.Directory,
    summary: str,
) -> dict[str, str]:
    """Select one healable failure via the Rust CLI contract."""
    raw = await (
        _rust_container(source)
        .with_exec(
            [
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "heal-support",
                "select-healable-json",
                summary,
            ]
        )
        .stdout()
    )
    parsed = json.loads(raw)
    if not isinstance(parsed, dict):
        raise ValueError("heal-support select-healable-json returned non-object JSON")
    name = parsed.get("name")
    detail = parsed.get("detail")
    if not isinstance(name, str) or not isinstance(detail, str):
        raise ValueError("heal-support select-healable-json returned invalid failure JSON")
    return {"name": name, "detail": detail}


def _lint_container(source: dagger.Directory) -> dagger.Container:
    """Base container with Python YAML parsing and shellcheck installed."""
    return (
        dag.container()
        .from_(PYTHON_IMAGE)
        .with_exec(
            ["sh", "-c", "apt-get update -qq && apt-get install -y -qq --no-install-recommends git shellcheck && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*"]
        )
        .with_exec(["sh", "-c", "pip install -q PyYAML && rm -rf /root/.cache/pip"])
        .with_directory("/src", source)
        .with_workdir("/src")
    )


def _repair_container(source: dagger.Directory) -> dagger.Container:
    """Writable repair container that can run every heal prompt command."""
    return (
        _rust_container(source)
        .with_exec(
            ["sh", "-c", "apt-get update -qq && apt-get install -y -qq --no-install-recommends shellcheck && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*"]
        )
    )


def _rust_container(source: dagger.Directory) -> dagger.Container:
    """Base container for Rust checks that still need local Python helpers."""
    return (
        dag.container()
        .from_(RUST_IMAGE)
        .with_env_variable("CARGO_TERM_QUIET", "true")
        .with_env_variable("CARGO_TERM_PROGRESS_WHEN", "never")
        .with_exec(
            ["sh", "-c", "apt-get update -qq && apt-get install -y -qq --no-install-recommends git python3 python3-yaml && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*"]
        )
        .with_exec(["rustup", "component", "add", "rustfmt", "clippy"])
        .with_directory("/src", source)
        .with_workdir("/src")
        .with_exec(["cargo", "fetch", "--locked"])
        .with_env_variable("CARGO_NET_OFFLINE", "true")
    )


@object_type
class HarnessKitCi:
    """Local CI pipeline for the Harness Kit repo."""

    @function
    async def lint_yaml(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
            Doc("Repo source directory"),
        ],
    ) -> str:
        """Validate YAML files parse correctly."""
        # Discover yaml files, pass as argv (not f-string interpolation)
        return await (
            _lint_container(source)
            .with_exec([
                "sh", "-c",
                "find . \\( -name '*.yaml' -o -name '*.yml' \\) "
                "-not -path './ci/*' "
                "| xargs python3 -c "
                "'import sys,yaml; [yaml.safe_load(open(f)) for f in sys.argv[1:]]'",
            ])
            .stdout()
        )

    @function
    async def lint_shell(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Run shellcheck on all bash scripts (errors only)."""
        # Discover .sh files from filesystem
        return await (
            _lint_container(source)
            .with_exec([
                "sh", "-c",
                "find . -name '*.sh' -not -path './ci/*' "
                "| xargs shellcheck --severity=error",
            ])
            .stdout()
        )

    @function
    async def lint_python(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Syntax-check all Python files via py_compile."""
        # Discover .py files from filesystem
        return await (
            _lint_container(source)
            .with_exec([
                "sh", "-c",
                "find . -name '*.py' -not -path './ci/*' "
                "| xargs -I{} python3 -m py_compile {}",
            ])
            .stdout()
        )

    @function
    async def check_frontmatter(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate SKILL.md and agent frontmatter: required fields, line limits."""
        return await (
            _rust_container(source)
            .with_exec(
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "harness-kit-checks",
                    "--",
                    "check-frontmatter",
                    "--repo",
                    ".",
                ]
            )
            .stdout()
        )

    @function
    async def check_index_drift(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Verify index.yaml matches what the Rust index generator would produce."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-index-drift",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def check_vendored_copies(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Verify vendored copies match their canonical sources."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-vendored-copies",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def test_bun(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Run Bun tests for the research skill."""
        return await (
            dag.container()
            .from_(PYTHON_IMAGE)
            .with_exec(
                ["sh", "-c", "apt-get update -qq && apt-get install -y -qq --no-install-recommends bash ca-certificates curl unzip && rm -rf /var/lib/apt/lists/* /var/cache/apt/archives/*"]
            )
            .with_env_variable("BUN_INSTALL", "/opt/bun")
            .with_exec(["bash", "-c", "curl -fsSL https://bun.sh/install | bash"])
            .with_directory("/src", source)
            .with_workdir("/src/skills/research")
            .with_exec(["sh", "-c", "PATH=/opt/bun/bin:$PATH bun test"])
            .stdout()
        )

    @function
    async def test_trace_record(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Run the trace work-record helper self-test."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "test",
                "--quiet",
                "--workspace",
                "--locked",
                "trace_record",
            ])
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "trace-record",
                "--self-test",
            ])
            .with_exec([
                "sh",
                "-c",
                "store=$(mktemp -d)/work-records.jsonl && "
                "cargo run --locked -p harness-kit-checks -- trace-record append "
                "--store \"$store\" --backlog 056 "
                "--branch deliver/056-agent-session-trace-lifecycle "
                "--commit abc1234 "
                "--reviewer-verdict-ref .harness-kit/traces/delegations.jsonl#abc "
                "--qa-ref .evidence/qa/056.md "
                "--demo-ref .evidence/demo/056.gif "
                "--transcript-ref .harness-kit/traces/transcripts/056.md "
                "--shipped-ref master@deadbeef "
                "--metadata source=self-test >/tmp/trace-receipt.json && "
                "python3 - \"$store\" <<'PY'\n"
                "import json, sys\n"
                "from pathlib import Path\n"
                "rows = [json.loads(line) for line in Path(sys.argv[1]).read_text().splitlines()]\n"
                "assert len(rows) == 1\n"
                "row = rows[0]\n"
                "assert row['record_type'] == 'agent-session-trace'\n"
                "assert row['trace_id'].startswith('trace-')\n"
                "assert row['metadata'] == {'source': 'self-test'}\n"
                "assert row['transcript_refs'] == ['.harness-kit/traces/transcripts/056.md']\n"
                "PY",
            ])
            .stdout()
        )

    @function
    async def check_exclusions(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Scan source files for exclusion patterns (@ts-ignore, .skip, eslint-disable, etc.)."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-exclusions",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def check_conflict_markers(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Reject unresolved Git conflict markers in committed text files."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-conflict-markers",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def check_portable_paths(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Scan shell scripts and configs for hardcoded user home paths."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-portable-paths",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def check_harness_install_paths(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Reject stale or Claude-only install instructions."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-harness-install-paths",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def test_work_ledger(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Run the work-ledger helper self-test."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "test",
                "--quiet",
                "--workspace",
                "--locked",
                "work_ledger",
            ])
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "work-ledger",
                "--self-test",
            ])
            .stdout()
        )

    @function
    async def test_bootstrap_agent_allowlist(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Verify bootstrap installs only allowlisted global agents."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "test-bootstrap-agent-allowlist",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def test_sync_external_partial(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Verify partial external sync does not remove other synced skills."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "test-sync-external-partial",
            ])
            .stdout()
        )

    @function
    async def check_runtime_primitives(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate runtime hooks, settings, and skill invocation protocols."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-runtime-primitives",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def test_agent_readiness_profile(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Run the agent-readiness profile CRUD smoke test."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "test",
                "--quiet",
                "--workspace",
                "--locked",
                "agent_readiness_profile",
            ])
            .stdout()
        )

    @function
    async def check_deliver_composition(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Forbid inlined phase-skill internals in skills/deliver/SKILL.md.

        /deliver must compose atomic phase skills via their trigger syntax
        (/code-review, /ci, /qa, /implement, /refactor, /shape) — not
        re-implement them by dispatching phase agents or running raw
        phase tooling. This lint catches composer regressions where
        inlined logic creeps back in.
        """
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-deliver-composition",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def check_no_claims(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Regression guard: forbid claim-coordination primitives under skills/.

        claims.sh / claim_acquire / claim_release were dropped per 032.
        Any reappearance under skills/ is a regression and must fail CI.
        """
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-no-claims",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def check_skill_evals(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate structure for existing skill eval suites."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-skill-evals",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def test_design_evals(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Run deterministic design eval grader self-tests."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "design-eval",
                "--self-test",
            ])
            .stdout()
        )

    @function
    async def check_agent_roster(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate agent roster config, receipt fixtures, and trace ignore policy."""
        return await (
            _rust_container(source)
            .with_exec(
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "harness-kit-checks",
                    "--",
                    "check-agent-roster",
                    "--repo",
                    ".",
                ]
            )
            .stdout()
        )

    @function
    async def test_rust(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external", "target"]),
        ],
    ) -> str:
        """Run Rust format and unit tests."""
        return await (
            _rust_container(source)
            .with_exec(["cargo", "fmt", "--all", "--check"])
            .with_exec(["cargo", "test", "--quiet", "--workspace", "--locked"])
            .with_exec(["cargo", "clippy", "--workspace", "--all-targets", "--locked", "--", "-D", "warnings"])
            .stdout()
        )

    @function
    async def check_review_score_trends(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate review-score trend analyzer self-test."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "test",
                "--quiet",
                "--workspace",
                "--locked",
                "review_score",
            ])
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "review-score-trends",
                "--self-test",
            ])
            .stdout()
        )

    @function
    async def check_shape_renderer(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate the /shape static HTML context-packet renderer."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "shape-render",
                "--self-test",
            ])
            .stdout()
        )

    @function
    async def check_git_hooks(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Run git hook behavior tests."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "test",
                "--locked",
                "-p",
                "harness-kit-checks",
                "git_hooks",
            ])
            .stdout()
        )

    @function
    async def check_bench_map(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate code-review bench-map reviewer ids and replacement fixtures."""
        return await (
            _rust_container(source)
            .with_exec(
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "harness-kit-checks",
                    "--",
                    "check-bench-map",
                    "--repo",
                    ".",
                ]
            )
            .stdout()
        )

    @function
    async def check_evidence_blocks(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate Completion Gate and Acceptance Evidence templates."""
        return await (
            _rust_container(source)
            .with_exec(
                [
                    "cargo",
                    "run",
                    "--locked",
                    "-p",
                    "harness-kit-checks",
                    "--",
                    "check-evidence-blocks",
                    "skills",
                ]
            )
            .stdout()
        )

    @function
    async def check_offline_evidence_storage(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Validate the git-native offline evidence storage contract."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-offline-evidence-storage",
                "--repo",
                ".",
            ])
            .stdout()
        )

    @function
    async def check_docs_site(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "skills/.external", ".harness-kit/tmp/lane-harness"]),
        ],
    ) -> str:
        """Validate generated public docs site output and drift checks."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "run",
                "--locked",
                "-p",
                "harness-kit-checks",
                "--",
                "check-docs-site",
                "--repo",
                ".",
                "--self-test",
            ])
            .stdout()
        )

    @function
    async def test_harness_kit_config_loader(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
        ],
    ) -> str:
        """Run loader regression tests for .harness-kit/*.yaml contracts."""
        return await (
            _rust_container(source)
            .with_exec([
                "cargo",
                "test",
                "--quiet",
                "--workspace",
                "--locked",
                "config_loader",
            ])
            .stdout()
        )

    @function
    async def check(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "skills/.external", ".harness-kit/tmp/lane-harness"]),
            Doc("Repo source directory"),
        ],
    ) -> str:
        """Run all quality gates. Exits non-zero if any fail."""
        results: list[tuple[str, bool, str]] = []
        gate_limit = anyio.Semaphore(4)

        async def run_gate(name: str, coro):
            async with gate_limit:
                try:
                    await coro
                    results.append((name, True, "OK"))
                except dagger.ExecError as e:
                    detail = (e.stdout or e.stderr or str(e)).strip()
                    results.append((name, False, detail or str(e)))
                except Exception as e:
                    results.append((name, False, str(e)))

        async with anyio.create_task_group() as tg:
            tg.start_soon(run_gate, "lint-yaml", self.lint_yaml(source))
            tg.start_soon(run_gate, "lint-shell", self.lint_shell(source))
            tg.start_soon(run_gate, "lint-python", self.lint_python(source))
            tg.start_soon(run_gate, "check-frontmatter", self.check_frontmatter(source))
            tg.start_soon(run_gate, "check-index-drift", self.check_index_drift(source))
            tg.start_soon(run_gate, "check-vendored-copies", self.check_vendored_copies(source))
            tg.start_soon(run_gate, "test-bun", self.test_bun(source))
            tg.start_soon(run_gate, "test-trace-record", self.test_trace_record(source))
            tg.start_soon(run_gate, "check-exclusions", self.check_exclusions(source))
            tg.start_soon(run_gate, "check-conflict-markers", self.check_conflict_markers(source))
            tg.start_soon(run_gate, "check-portable-paths", self.check_portable_paths(source))
            tg.start_soon(run_gate, "test-work-ledger", self.test_work_ledger(source))
            tg.start_soon(
                run_gate,
                "check-harness-install-paths",
                self.check_harness_install_paths(source),
            )
            tg.start_soon(
                run_gate,
                "test-bootstrap-agent-allowlist",
                self.test_bootstrap_agent_allowlist(source),
            )
            tg.start_soon(
                run_gate,
                "test-sync-external-partial",
                self.test_sync_external_partial(source),
            )
            tg.start_soon(
                run_gate,
                "check-runtime-primitives",
                self.check_runtime_primitives(source),
            )
            tg.start_soon(
                run_gate,
                "test-agent-readiness-profile",
                self.test_agent_readiness_profile(source),
            )
            tg.start_soon(run_gate, "check-deliver-composition", self.check_deliver_composition(source))
            tg.start_soon(run_gate, "check-no-claims", self.check_no_claims(source))
            tg.start_soon(run_gate, "check-skill-evals", self.check_skill_evals(source))
            tg.start_soon(run_gate, "test-design-evals", self.test_design_evals(source))
            tg.start_soon(run_gate, "check-agent-roster", self.check_agent_roster(source))
            tg.start_soon(run_gate, "test-rust", self.test_rust(source))
            tg.start_soon(
                run_gate,
                "check-review-score-trends",
                self.check_review_score_trends(source),
            )
            tg.start_soon(run_gate, "check-shape-renderer", self.check_shape_renderer(source))
            tg.start_soon(run_gate, "check-git-hooks", self.check_git_hooks(source))
            tg.start_soon(run_gate, "check-bench-map", self.check_bench_map(source))
            tg.start_soon(run_gate, "check-evidence-blocks", self.check_evidence_blocks(source))
            tg.start_soon(
                run_gate,
                "check-offline-evidence-storage",
                self.check_offline_evidence_storage(source),
            )
            tg.start_soon(run_gate, "check-docs-site", self.check_docs_site(source))
            tg.start_soon(
                run_gate,
                "test-harness-kit-config-loader",
                self.test_harness_kit_config_loader(source),
            )

        # Format results
        lines = ["Harness Kit CI Results", "=" * 40]
        passed = 0
        failed = 0
        for name, ok, msg in sorted(results):
            status = "PASS" if ok else "FAIL"
            if ok:
                passed += 1
            else:
                failed += 1
            lines.append(f"  {status}  {name}")
            if not ok:
                for line in msg.splitlines()[:5]:
                    lines.append(f"         {line}")
        lines.append("=" * 40)
        lines.append(f"{passed} passed, {failed} failed")

        summary = "\n".join(lines)

        if failed > 0:
            raise Exception(summary)

        return summary

    @function
    async def heal(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
            Doc("Repo source directory"),
        ],
        model: Annotated[str, Doc("LLM model for the repair agent")] = "gpt-4.1",
        attempts: Annotated[int, Doc("Maximum repair attempts before escalation")] = 2,
    ) -> dagger.Directory:
        """Repair one failing lint-style gate and return the updated repo directory."""
        if attempts < 1:
            raise ValueError("attempts must be at least 1.")

        try:
            summary = await self.check(source)
        except Exception as error:
            summary = str(error)
        else:
            return source

        failure = await _select_healable_failure(source, summary)
        last_error = summary
        working_source = source

        for attempt in range(1, attempts + 1):
            repaired_source = working_source
            work = (
                dag.llm()
                .with_model(model)
                .with_env(
                    dag.env()
                    .with_string_input("gate", failure["name"], "the failing gate to repair")
                    .with_string_input("failure_summary", last_error, "latest failure summary")
                    .with_container_input(
                        "builder",
                        _repair_container(working_source),
                        "a writable repo container rooted at /src with lint tools installed",
                    )
                    .with_container_output(
                        "repaired",
                        "the updated repo container after the gate passes",
                    )
                )
                .with_system_prompt(
                    "You are a minimal repair agent. Fix the failing CI gate with the smallest correct change."
                )
                .with_prompt(await _repair_prompt(working_source, failure, attempt, attempts))
            )

            try:
                repaired_container = await work.env().output("repaired").as_container().sync()
                repaired_source = repaired_container.directory("/src")
                gate_runner = getattr(self, failure["name"].replace("-", "_"))
                await gate_runner(repaired_source)
                await self.check(repaired_source)
                return repaired_source
            except Exception as error:
                last_error = str(error)
                if attempt == attempts:
                    break
                working_source = repaired_source

        raise Exception(
            "heal exhausted its repair budget after "
            f"{attempts} attempt(s).\n"
            f"Last error:\n{last_error}"
        )
