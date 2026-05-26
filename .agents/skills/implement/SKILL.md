---
name: implement
description: |
  Atomic TDD build skill for the Spellbook repo. Takes a context packet
  (shaped ticket from /shape or backlog.d/<id>-*.md) and produces code +
  tests on a feature branch. Red -> Green -> Refactor. "Green" here
  means `dagger call check --source=.` exits 0 — the 14-gate load-bearing
  gate. Does not shape, review, QA, or ship — single concern: spec to
  green gate. Use when: "implement this spec", "build this", "TDD this",
  after /shape has produced a context packet.
  Trigger: /implement, /build (alias).
argument-hint: "[context-packet-path|ticket-id]"
---

# /implement

Spec in, green gate out. One packet, one feature branch, one concern.

## Invariants

- Trust the context packet. Do not reshape. Do not re-plan.
- If the packet is incomplete, **fail loudly** — do not invent the spec.
- The green signal is `dagger call check --source=.`. Every other test
  command (pytest, bun test, shellcheck) is a sub-gate of that one gate.
- Cross-harness parity is a Red Line. If the change touches
  `skills/<name>/SKILL.md`, it works on Claude, Codex, AND Pi.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/implement` starts by probing the
roster and dispatching two or more available roster members before code or
tests are produced. Use one lane for the bounded builder attempt and another
for independent validator/refactor scrutiny. The lead agent owns prompt
framing, conflict resolution, final verification, and receipts; it should not
write the implementation directly except for mechanical command execution,
emergency unblocks, explicit user-forbidden delegation, or an explicit waiver
when fewer than two roster members are available.

## Contract

**Input.** A context packet: goal, non-goals, constraints, repo anchors,
oracle (executable preferred), implementation sequence. Resolution order:

1. Explicit path argument (`/implement backlog.d/033-foo.md`)
2. Backlog item ID (`/implement 033`) -> resolves via `backlog.d/<id>-*.md`
3. Last `/shape` output in the current session
4. **No packet found -> stop.** Do not guess the spec from a title.

Required packet fields (hard gate — missing any = stop):
- `goal` (one sentence, testable)
- `oracle` (how we know it's done, ideally executable commands)
- `implementation sequence` (ordered steps, or explicit "single chunk")

See `references/context-packet.md` for the full shape.

**Output.**
- Code + tests on a feature branch (`<type>/<slug>` from current branch;
  `<type>` in `{feat, fix, chore, docs, refactor}`; base is `master`)
- `dagger call check --source=.` exits 0 (all 14 sub-gates green)
- Working tree clean (no debug prints, no scratch files)
- Commits in repo convention — one logical unit per commit, semantic
  prefix matching current branch type
- Final message: branch ref + oracle checklist status + gate receipt

**Stops at:** green gate + clean tree. Does not run `/code-review`,
`/qa`, `/ci`, or open a PR. Those are `/deliver`'s orchestration.

## The green signal, concretely

`dagger call check --source=.` is the one command. It fans out to 12
parallel sub-gates (from the repo brief):

| Sub-gate | Triggered by changes to |
|---|---|
| `lint-yaml` | any `*.yaml` / `*.yml` |
| `lint-shell` | any non-`ci/` shell script (shellcheck --severity=error) |
| `lint-python` | any non-`ci/` `.py` (py_compile) |
| `check-frontmatter` | `skills/*/SKILL.md`, `agents/*.md` |
| `check-index-drift` | `skills/`, `agents/` (compares against `scripts/generate-index.sh`) |
| `check-vendored-copies` | content that has canonical-vs-copy pairs |
| `test-bun` | `skills/research/` TypeScript modules |
| `check-exclusions` | source files (no `@ts-ignore`, `.skip()`, `eslint-disable`, `as any`) |
| `check-portable-paths` | anything outside `harnesses/claude/` + `.claude/hooks` (no `/Users/<name>/`, `C:\Users\`) |
| `check-harness-install-paths` | `skills/seed/`, `skills/tailor/` (install logic must not be Claude-only) |
| `check-deliver-composition` | `skills/deliver/SKILL.md` (must compose atomic phase skills via trigger syntax) |
| `check-no-claims` | anything under `skills/` (no `claims.sh`, `claim_acquire`, `claim_release`) |

The faster inner-loop commands (before the full gate):

- **Python modules in `ci/`**: `cd ci && python -m pytest tests/test_<name>.py`
  then `cd ci && python -m pytest tests/` for regression. Module under
  test: e.g. `ci/src/spellbook_ci/heal_support.py`.
- **TypeScript in `skills/research/`**: `cd skills/research && bun test`
  (mirrors `test-bun`).
- **Shell scripts under `scripts/` or `.githooks/`**: invoke directly
  against a fixture, then `dagger call lint-shell --source=.`. Prior
  art: `.githooks/test_pre_merge_commit.sh`.
- **Frontmatter / lint-only SKILL.md edits**: `dagger call
  check-frontmatter --source=.` + (if editing `/deliver`)
  `dagger call check-deliver-composition --source=.` + (if editing
  `skills/seed/` or `skills/tailor/` install code)
  `dagger call check-harness-install-paths --source=.`.

On a lint-style gate failure (`lint-yaml`, `lint-shell`, `lint-python`,
`check-frontmatter`) that blocks you after a targeted fix, the bounded
repair path is `dagger call heal --source=. --model=gpt-4.1
--attempts=2`. `HEALABLE_GATES` in `ci/src/spellbook_ci/heal_support.py`
is the source of truth for what heal can touch — do not ask it to fix
anything else.

## Workflow

### 1. Load and validate packet

Resolve the packet (order above). Parse required fields. If any are
missing or vague ("add feature X" with no oracle), stop with:

> Packet incomplete: missing <field>. Run /shape first.

Do not try to fill in the gaps. Shape is a different skill's judgment.

### 2. Create the feature branch

`git checkout -b <type>/<slug>` from the current branch. Builders never
commit to `master`. If you forget, create the branch after and
cherry-pick before handing off. Branch naming follows the brief:
`feat/*`, `fix/*`, `chore/*`, `docs/*`, `refactor/*`.

### 3. Dispatch the builder

Spawn roster-backed **builder** lanes with:
- The full context packet
- The executable oracle, with `dagger call check --source=.` as the
  final gate
- The TDD mandate (see below)
- The relevant test surface (Python? `ci/tests/`. TypeScript? `skills/research/__tests__/`.
  Shell? a sibling `test_<script>.sh`. Markdown-only skill? Dagger
  frontmatter + composition gates)
- File ownership (if the packet decomposes into disjoint chunks, spawn
  multiple builders in parallel — one per chunk, each with subset of
  oracle)

**Builder prompt must include:**
> You MUST write a failing test before production code. RED -> GREEN ->
> REFACTOR -> COMMIT. Exceptions: config files, generated code, UI
> layout, and markdown-only SKILL.md content whose test is `dagger call
> check-frontmatter` + `check-index-drift`. Document any skipped-TDD
> step inline in the commit message.

See `references/tdd-loop.md` for the full cycle and skip rules.

### 4. Verify exit conditions

Before exiting, confirm (run the commands — don't trust the builder):

- [ ] Every oracle command exits 0
- [ ] `dagger call check --source=.` exits 0 (all 14 sub-gates)
- [ ] Pre-commit hook exits 0 on the last commit (it regenerates
  `index.yaml` and runs `check-harness-agnostic-installs.sh`). If
  `index.yaml` was regenerated, the hook's amend counts as a success
  signal.
- [ ] `git status` clean (no untracked debug files; `.spellbook/deliver/<ulid>/`
  state/receipt files are gitignored and MUST NOT be force-added)
- [ ] No `TODO`/`FIXME`/`console.log`/`print("here")` added that isn't
  in the spec
- [ ] Commits are logically atomic (one concern per commit), semantic
  prefix matches branch type

If any check fails, dispatch a builder sub-agent to fix. Max 2 fix
loops, then escalate. Lint-only failures may escalate to `dagger call
heal` if they match `HEALABLE_GATES`.

### 5. Hand off

Output:
- Feature branch name
- Commit list (with semantic prefixes)
- Oracle checklist (which commands passed)
- `dagger call check --source=.` receipt (pass/fail per sub-gate)
- Completion Gate:
  - Exact end-user behavior changed
  - Live evidence that proves it
  - Exact command/path/route exercised
  - Repo-fit check
  - Residual unverified paths
- Residual risks

Do not run review, do not merge, do not push unless the packet
explicitly says so. The flywheel's outer loop pushes; builders don't.
For Spellbook-internal diffs, replace end-user behavior with the
developer/operator behavior the implementation changes.

## Scoping judgment (what the model must decide)

- **Test granularity.** One behavior per test. If you can't name the
  behavior in one short sentence, the test is too big. Examples in
  `ci/tests/test_heal_support.py`: `test_returns_all_failed_gate_details`,
  `test_rejects_unsupported_gate`, `test_commit_message_is_semantic`.
- **When to skip TDD.** Config, generated code, UI layout, pure
  exploration, and markdown-only SKILL.md content where the test is
  the Dagger frontmatter/composition gate. Document the skip in the
  commit message. Everything executable: test first.
- **When to escalate.** Builder loops on the same test failure 3+
  times, the oracle contradicts the constraints, the spec requires
  behavior that violates an invariant (cross-harness parity, skill
  self-containment, `.spellbook/deliver/` state hygiene, no-claims),
  or `dagger call heal` can't touch the failing gate. Stop and report.
- **Parallelism.** Only parallelize when file ownership is disjoint
  and oracle criteria partition cleanly. Shared SKILL.md, shared
  `ci/src/spellbook_ci/main.py` -> serial builders.
- **Refactor depth.** The refactor step in TDD is local — improve the
  code you just wrote. Broader refactors are `/refactor`'s job.

## What /implement does NOT do

- Pick tickets (caller's job, or `/deliver` / `/flywheel`)
- Shape or re-shape specs (-> `/shape`)
- Code review (-> `/code-review`)
- QA against a running app (-> `/qa`; this repo has no runtime app, so
  `/qa` is typically skipped)
- CI gate orchestration beyond running `dagger call check` (-> `/ci`)
- Simplification passes beyond TDD refactor (-> `/refactor`)
- Ship, merge, deploy (-> human, or `/settle`)
- Edit `index.yaml` (pre-commit regenerates it; manual edits are drift)
- Re-bootstrap harnesses after editing `harnesses/claude/settings.json`
  (call out the need in the hand-off; the operator re-runs `bootstrap.sh`)

## Stopping conditions

Stop with a loud report if:
- Packet is incomplete or ambiguous
- Oracle is unverifiable (prose-only checkboxes with no executable form —
  write one, or stop)
- Builder fails the same test 3+ times after targeted fix attempts
- Spec contradicts a stated invariant (cross-harness parity,
  self-containment, no-claims, `/deliver` composition, `.spellbook/deliver/`
  state hygiene)
- `dagger call check` fails a gate outside `HEALABLE_GATES` after two
  targeted builder fixes
- Tests hit an external dependency that isn't available (e.g. Dagger
  engine down; `bun` missing for `skills/research/`)

**Not** stopping conditions: spec is hard, unfamiliar codebase, initial
tests red, `dagger call heal` needed once. Those are the job.

## Gotchas

- **Reshaping inside /implement.** If the spec is wrong, stop. Don't
  silently rewrite the oracle to match what you built.
- **Declaring victory without the full gate.** "pytest passes" is not
  green — `dagger call check --source=.` is. Frontmatter, portable
  paths, harness install paths, `/deliver` composition, and `no-claims`
  fail in ways unit tests never see.
- **Editing `index.yaml` by hand.** Pre-commit regenerates it from
  `skills/` + `agents/`. A manual edit either churns on next commit or
  drifts silently.
- **Touching `harnesses/claude/settings.json` without flagging re-bootstrap.**
  Bootstrap COPIES this file (Claude mutates it at runtime). Changes
  require re-bootstrap to take effect; call it out in the hand-off.
- **Force-adding `.spellbook/deliver/<ulid>/(state|receipt).json`.**
  The pre-commit hook blocks this. It's gitignored for a reason — agent
  state, not repo content.
- **Introducing claim primitives under `skills/`.** `claims.sh`,
  `claim_acquire`, `claim_release` are dropped per backlog.d/032; the
  `check-no-claims` gate regresses on reintroduction.
- **Anchoring new behavior on one harness's feature.** Cross-harness
  parity is a Red Line. If you can't answer "what does this do on
  Codex and Pi?" the design is incomplete — raise a blocker.
- **Skills sourcing `../..` or `$REPO_ROOT/...`.** Self-containment
  invariant: libs resolve via `readlink -f` + `$SCRIPT_DIR/lib/...`.
  State roots anchor to the *invoking* project's `git rev-parse
  --show-toplevel`, not the skill's install dir.
- **Inlining phase logic in `skills/deliver/SKILL.md`.** `/deliver`
  must compose via trigger syntax (`/code-review`, `/ci`, `/qa`,
  `/implement`, `/refactor`, `/shape`). Raw `dagger call check`,
  `bunx playwright`, or direct bench-agent dispatch inside
  `skills/deliver/SKILL.md` fails `check-deliver-composition`.
- **Silent catch-and-return.** New code that swallows exceptions and
  returns fallbacks is hiding bugs. Fail loud. Test the failure mode.
  See `select_healable_failure` in `ci/src/spellbook_ci/heal_support.py`
  — raises on unsupported gate, not a silent skip.
- **Testing implementation, not behavior.** Tests that assert the
  structure of the code break on every refactor. Test what the code
  does from the outside.
- **Committing debug noise.** `console.log`, `print("here")`,
  commented-out code. The tree must be clean before exit.
- **Skipping TDD without documenting.** Config and generated code are
  fine exceptions; silently skipping because "it was simpler" is not.
- **Parallelizing coupled builders.** Two builders editing
  `skills/tailor/SKILL.md` and `skills/deliver/SKILL.md` simultaneously
  when they cross-reference = merge pain. Partition by file ownership.
- **Branch drift.** Forgetting to create the feature branch and
  committing to `master`. Always `git checkout -b` first.
- **Trusting self-reported success.** Builders say "all tests pass."
  Run `dagger call check --source=.` yourself. Agents lie
  (accidentally).
