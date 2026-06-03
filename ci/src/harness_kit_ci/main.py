"""Harness Kit CI pipeline — local-first quality gates via Dagger."""

from typing import Annotated

import anyio

import dagger
from dagger import DefaultPath, Doc, Ignore, dag, function, object_type

from .heal_support import (
    GateFailure,
    parse_check_failures,
    select_healable_failure,
)


def _repair_prompt(
    failure: GateFailure,
    attempt: int,
    attempts: int,
) -> str:
    """Prompt the LLM with the exact repair contract."""
    return f"""
You are repairing a failing CI gate in the Harness Kit repository.

Gate: {failure.name}
Attempt: {attempt} of {attempts}
Failure details:
{failure.detail}

Rules:
- Work only in /src.
- Fix the root cause for {failure.name}. Do not broaden scope.
- Keep edits minimal and ASCII unless the file already requires otherwise.
- Re-run the targeted gate after each meaningful edit.
- Before finishing, ensure the targeted gate passes and leave the updated repo in $repaired.
- Do not use git. Branching and committing happen after verification.

Available tool:
- $builder is a writable repo container rooted at /src with the linting tools installed.

Targeted validation commands:
- lint-yaml: find . \\( -name '*.yaml' -o -name '*.yml' \\) -not -path './ci/*' | xargs python3 -c 'import sys,yaml; [yaml.safe_load(open(f)) for f in sys.argv[1:]]'
- lint-shell: find . -name '*.sh' -not -path './ci/*' | xargs shellcheck --severity=error
- lint-python: find . -name '*.py' -not -path './ci/*' | xargs -I{{}} python3 -m py_compile {{}}
- check-frontmatter: python3 scripts/check-frontmatter.py

When the target gate passes, bind the updated container to $repaired.
""".strip()


def _lint_container(source: dagger.Directory) -> dagger.Container:
    """Base container with shellcheck and yamllint installed."""
    return (
        dag.container()
        .from_("python:3.12-slim")
        .with_exec(["apt-get", "update", "-qq"])
        .with_exec(
            [
                "apt-get",
                "install",
                "-y",
                "-qq",
                "--no-install-recommends",
                "git",
                "shellcheck",
            ]
        )
        .with_exec(["pip", "install", "-q", "yamllint"])
        .with_directory("/src", source)
        .with_workdir("/src")
    )


def _repair_container(source: dagger.Directory) -> dagger.Container:
    """Writable repair container with the repo mounted at /src."""
    return (
        dag.container()
        .from_("python:3.12-slim")
        .with_exec(["apt-get", "update", "-qq"])
        .with_exec(
            [
                "apt-get",
                "install",
                "-y",
                "-qq",
                "--no-install-recommends",
                "git",
                "shellcheck",
            ]
        )
        .with_exec(["pip", "install", "-q", "yamllint"])
        .with_directory("/src", source)
        .with_workdir("/src")
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
            _lint_container(source)
            .with_exec(["python3", "scripts/check-frontmatter.py"])
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
        """Verify index.yaml matches what generate-index.sh would produce."""
        # Strip the timestamp comment before diffing — it changes every run
        return await (
            _lint_container(source)
            .with_exec(["sh", "-c", "grep -v '^# Generated:' index.yaml > /tmp/index-committed.yaml"])
            .with_exec(["bash", "scripts/generate-index.sh"])
            .with_exec(["sh", "-c", "grep -v '^# Generated:' index.yaml > /tmp/index-generated.yaml"])
            .with_exec(["diff", "-u", "/tmp/index-committed.yaml", "/tmp/index-generated.yaml"])
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
            _lint_container(source)
            .with_exec(["bash", "scripts/check-vendored-copies.sh"])
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
            .from_("oven/bun:latest")
            .with_directory("/src", source)
            .with_workdir("/src/skills/research")
            .with_exec(["bun", "test"])
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
        script = r"""
import re, sys, pathlib

PATTERNS = [
    (r'@ts-ignore',                 'TypeScript @ts-ignore'),
    (r'@ts-expect-error',           'TypeScript @ts-expect-error'),
    (r'\bas\s+any\b',              'TypeScript as any'),
    (r':\s*any\b',                 'TypeScript : any'),
    (r'eslint-disable(?!.*--)',    'ESLint disable'),
    (r'\.skip\s*\(',              'Test .skip()'),
    (r'\bxit\s*\(',               'xit()'),
    (r'\bxdescribe\s*\(',         'xdescribe()'),
]

GLOBS = ['**/*.ts', '**/*.tsx', '**/*.js', '**/*.jsx', '**/*.py']
SKIP = {'hooks/', 'coverage/', 'dist/', '.next/', 'node_modules/', 'ci/'}

findings = []
for g in GLOBS:
    for path in pathlib.Path('.').glob(g):
        path_str = str(path)
        if any(s in path_str for s in SKIP):
            continue
        try:
            text = path.read_text()
        except Exception:
            continue
        for lineno, line in enumerate(text.splitlines(), 1):
            for regex, label in PATTERNS:
                if re.search(regex, line):
                    findings.append(f'  {path}:{lineno}: {label}')

if findings:
    print(f'Found {len(findings)} exclusion(s):', file=sys.stderr)
    print('\n'.join(findings[:20]), file=sys.stderr)
    sys.exit(1)

print('No exclusion patterns found.')
"""
        return await (
            _lint_container(source)
            .with_exec(["python3", "-c", script])
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
        script = r"""
import re, sys, pathlib

HOME_RE = re.compile(r'/Users/[a-zA-Z0-9_-]+/')
WIN_RE  = re.compile(r'C:\\Users\\[a-zA-Z0-9_-]+\\')

# Files where hardcoded paths are expected
ALLOW = {'.claude/hooks', 'coverage/', '.next/', 'dist/', 'harnesses/claude/'}

GLOBS = ['**/*.sh', '**/*.bash', '**/*.zsh', '**/Makefile', '**/.env*']

findings = []
for g in GLOBS:
    for path in pathlib.Path('.').glob(g):
        path_str = str(path)
        if any(a in path_str for a in ALLOW):
            continue
        try:
            text = path.read_text()
        except Exception:
            continue
        for lineno, line in enumerate(text.splitlines(), 1):
            m = HOME_RE.search(line) or WIN_RE.search(line)
            if m:
                findings.append(f'  {path}:{lineno}: {m.group(0)}')

if findings:
    print(f'Found {len(findings)} hardcoded path(s):', file=sys.stderr)
    print('\n'.join(findings[:20]), file=sys.stderr)
    sys.exit(1)

print('No hardcoded user paths found.')
"""
        return await (
            _lint_container(source)
            .with_exec(["python3", "-c", script])
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
            _lint_container(source)
            .with_exec(["bash", "scripts/check-harness-agnostic-installs.sh"])
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
        script = r"""
import re, sys, pathlib

TARGET = pathlib.Path('skills/deliver/SKILL.md')
if not TARGET.exists():
    print(f'{TARGET} not present; skipping deliver-composition lint.')
    sys.exit(0)

# Denylist: regex, human-readable label.
# Patterns target phase-skill internals that /deliver must delegate, not inline.
DENYLIST = [
    (r'\bsource\s+scripts/lib/claims\.sh\b',      'claims.sh sourcing (dropped primitive)'),
    (r'\bclaim_(acquire|release)\b',               'claim_acquire/claim_release (dropped primitive)'),
    (r'\bdagger\s+call\s+check\b',                 'raw `dagger call check` — use /ci instead'),
    (r'\bbunx?\s+playwright\b',                    'raw playwright invocation — use /qa instead'),
    (r'\bnpx\s+playwright\b',                      'raw playwright invocation — use /qa instead'),
    (r'Agent\s*\(\s*[\'"](?:critic|ousterhout|carmack|grug|beck)[\'"]',
     'direct bench-agent dispatch — use /code-review instead'),
    (r'subagent_type\s*=\s*[\'"](?:critic|ousterhout|carmack|grug|beck)[\'"]',
     'direct bench-agent dispatch — use /code-review instead'),
]

text = TARGET.read_text()
findings = []
for lineno, line in enumerate(text.splitlines(), 1):
    # Skip fenced/quoted example lines that document what NOT to do.
    stripped = line.lstrip()
    if stripped.startswith(('#', '>', '<!--')):
        continue
    for regex, label in DENYLIST:
        if re.search(regex, line):
            findings.append(f'  {TARGET}:{lineno}: {label}\n    {line.strip()[:120]}')

if findings:
    print(f'Found {len(findings)} inlined-phase violation(s) in {TARGET}:', file=sys.stderr)
    print('\n'.join(findings), file=sys.stderr)
    print('', file=sys.stderr)
    print('/deliver must compose atomic phase skills via trigger syntax,', file=sys.stderr)
    print('not re-implement their internals. See backlog.d/032.', file=sys.stderr)
    sys.exit(1)

print(f'{TARGET}: composition clean (no inlined-phase calls).')
"""
        return await (
            _lint_container(source)
            .with_exec(["python3", "-c", script])
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
        script = r"""
import re, sys, pathlib

ROOT = pathlib.Path('skills')
if not ROOT.exists():
    print('skills/ not present; skipping no-claims lint.')
    sys.exit(0)

PATTERNS = [
    (r'\bclaims\.sh\b',       'claims.sh reference'),
    (r'\bclaim_acquire\b',    'claim_acquire call'),
    (r'\bclaim_release\b',    'claim_release call'),
]

findings = []
for path in ROOT.rglob('*'):
    if not path.is_file():
        continue
    try:
        text = path.read_text()
    except Exception:
        continue
    for lineno, line in enumerate(text.splitlines(), 1):
        for regex, label in PATTERNS:
            if re.search(regex, line):
                findings.append(f'  {path}:{lineno}: {label}')

if findings:
    print(f'Found {len(findings)} claims-primitive reference(s) under skills/:', file=sys.stderr)
    print('\n'.join(findings[:40]), file=sys.stderr)
    print('', file=sys.stderr)
    print('Claim coordination was dropped per backlog.d/032.', file=sys.stderr)
    print('Do not reintroduce claims.sh / claim_acquire / claim_release in skills/.', file=sys.stderr)
    sys.exit(1)

print('skills/: no claims primitives found.')
"""
        return await (
            _lint_container(source)
            .with_exec(["python3", "-c", script])
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
            _lint_container(source)
            .with_exec(["python3", "skills/harness-engineering/scripts/validate-evals.py"])
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
            _lint_container(source)
            .with_exec(["python3", "scripts/check-agent-roster.py"])
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
            _lint_container(source)
            .with_exec(["python3", "scripts/review-score-trends.py", "--self-test"])
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
            _repair_container(source)
            .with_exec(["bash", ".githooks/test_pre_merge_commit.sh"])
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
            _lint_container(source)
            .with_exec(["python3", "scripts/check-evidence-blocks.py", "skills"])
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
            _lint_container(source)
            .with_exec(["python3", "scripts/check-offline-evidence-storage.py"])
            .stdout()
        )

    @function
    async def check_docs_site(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "skills/.external"]),
        ],
    ) -> str:
        """Validate generated public docs site output and drift checks."""
        return await (
            _lint_container(source)
            .with_exec(["bash", "scripts/check-docs-site.sh", "--self-test"])
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
            _lint_container(source)
            .with_exec(["bash", "scripts/test-load-harness-kit-config.sh"])
            .stdout()
        )

    @function
    async def check(
        self,
        source: Annotated[
            dagger.Directory,
            DefaultPath("/"),
            Ignore([".git", "__pycache__", ".venv", "skills/.external"]),
            Doc("Repo source directory"),
        ],
    ) -> str:
        """Run all quality gates. Exits non-zero if any fail."""
        results: list[tuple[str, bool, str]] = []

        async def run_gate(name: str, coro):
            try:
                output = await coro
                results.append((name, True, output.strip() if output else "OK"))
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
            tg.start_soon(run_gate, "check-exclusions", self.check_exclusions(source))
            tg.start_soon(run_gate, "check-portable-paths", self.check_portable_paths(source))
            tg.start_soon(
                run_gate,
                "check-harness-install-paths",
                self.check_harness_install_paths(source),
            )
            tg.start_soon(run_gate, "check-deliver-composition", self.check_deliver_composition(source))
            tg.start_soon(run_gate, "check-no-claims", self.check_no_claims(source))
            tg.start_soon(run_gate, "check-skill-evals", self.check_skill_evals(source))
            tg.start_soon(run_gate, "check-agent-roster", self.check_agent_roster(source))
            tg.start_soon(
                run_gate,
                "check-review-score-trends",
                self.check_review_score_trends(source),
            )
            tg.start_soon(run_gate, "check-git-hooks", self.check_git_hooks(source))
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

        failure = select_healable_failure(parse_check_failures(summary))
        last_error = summary
        working_source = source

        for attempt in range(1, attempts + 1):
            repaired_source = working_source
            work = (
                dag.llm()
                .with_model(model)
                .with_env(
                    dag.env()
                    .with_string_input("gate", failure.name, "the failing gate to repair")
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
                .with_prompt(_repair_prompt(failure, attempt, attempts))
            )

            try:
                repaired_container = await work.env().output("repaired").as_container().sync()
                repaired_source = repaired_container.directory("/src")
                gate_runner = getattr(self, failure.name.replace("-", "_"))
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
