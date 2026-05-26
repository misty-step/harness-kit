---
name: ci
description: |
  Run and strengthen spellbook's Dagger gates. The gate is
  `dagger call check --source=.` — 14 parallel sub-gates covering lint,
  frontmatter, index drift, vendored copies, bun tests, exclusions,
  portable paths, harness-agnostic installs, deliver composition, and
  the no-claims regression guard. Bounded self-heal via
  `dagger call heal` for lint-style failures; non-lint failures escalate.
  Drive gates green; never lower a threshold to pass.
  Use when: "run ci", "check ci", "fix ci", "audit ci", "is ci passing",
  "run the gates", "dagger check", "why is ci failing", "strengthen ci",
  "ci is red", "gates failing", "heal ci".
  Trigger: /ci, /gates.
argument-hint: "[--audit-only|--run-only]"
---

# /ci

Confidence in correctness for the spellbook repo. The load-bearing gate
is `dagger call check --source=.` — 14 parallel sub-gates defined in
`ci/src/spellbook_ci/main.py`. Green from that call is what "CI passed"
means here. No GitHub Actions. No hosted runner. Local-first by design.

Stops at green gates. Does not review code semantics (→ `/code-review`),
does not address review comments (→ `/settle`), does not ship.

## Modes

- Default: audit → run. Full pass.
- `--audit-only`: produce audit report and gap list; do not run gates.
- `--run-only`: skip audit, just drive gates green.

## Stance

1. **Dagger is already the gate.** `dagger.json` (engineVersion `v0.20.3`,
   Python 3.12 SDK) points at `ci/`. Don't scaffold. Don't propose
   migrating to Actions. The audit is about gate *coverage and health*,
   not pipeline presence.
2. **Act, do not propose.** Mechanical strengthenings (new gate, new
   exclusion pattern, raising a threshold the code already passes) are
   applied directly via `ci/src/spellbook_ci/main.py`. Escalate only
   when the strengthening is a genuine product decision (disabling a
   green gate, materially changing scope).
3. **Fix-until-green on self-healable failures.** Use
   `dagger call heal --source=. --model=gpt-4.1 --attempts=2` for a
   single failing lint-style gate. Other failures get a structured
   diagnosis, not a silent pass.
4. **No quality lowering, ever.** Thresholds, lint rules, gate
   membership are load-bearing walls. Adding exclusions to
   `check-exclusions` / `check-portable-paths` to make a gate pass is
   the same anti-pattern as `@ts-ignore`.
5. **Composition is enforced, not encouraged.** `/deliver` cannot call
   `dagger call check` directly — `check-deliver-composition` fails the
   build if it does. `/ci` is the only skill allowed to own the gate.

## Delegation Floor

When `.spellbook/agents.yaml` exists, the lead agent uses two or more roster
lanes for audit, failure diagnosis, and gate-strengthening judgment. The lead
may run exact validation commands and apply mechanical repairs directly after
the lane decision is made. Direct lead-only CI work is limited to mechanical
command execution, emergency unblocks, explicit user-forbidden delegation, or
an explicit waiver when fewer than two roster members are available.
6. **Roster lanes can investigate, not certify.** When `.spellbook/agents.yaml`
   exists, provider lanes may inspect red gates or weak coverage, but
   `dagger call check --source=.` remains the proof.

## The 14 Gates

Defined in `ci/src/spellbook_ci/main.py`, run in parallel by
`check(source)`:

| Gate | Purpose |
|---|---|
| `lint-yaml` | Every `*.yaml`/`*.yml` at depth ≤2 parses via `yaml.safe_load`. |
| `lint-shell` | `shellcheck --severity=error` on every non-`ci/` `.sh`. |
| `lint-python` | `py_compile` on every non-`ci/` `.py`. |
| `check-frontmatter` | `scripts/check-frontmatter.py` — required fields, line limits, description length. |
| `check-index-drift` | `index.yaml` equals `scripts/generate-index.sh` output (modulo timestamp). |
| `check-vendored-copies` | Vendored files match their canonical sources (`scripts/check-vendored-copies.sh`). |
| `test-bun` | `bun test` under `skills/research/`. |
| `check-exclusions` | No `@ts-ignore`, `@ts-expect-error`, `as any`, `: any`, `eslint-disable`, `.skip()`, `xit(`, `xdescribe(` in source. |
| `check-portable-paths` | No hardcoded `/Users/<name>/` or `C:\Users\<name>\` outside `harnesses/claude/` / `.claude/hooks/`. |
| `check-harness-install-paths` | `scripts/check-harness-agnostic-installs.sh` — seed/tailor copy must not be Claude-only. |
| `check-deliver-composition` | `skills/deliver/SKILL.md` composes atomic phase skills via trigger syntax; no inlined `dagger call check` / raw playwright / direct bench-agent dispatch. |
| `check-no-claims` | Regression guard (backlog.d/032): no `claims.sh` / `claim_acquire` / `claim_release` under `skills/`. |

Each gate is a `@function` on `SpellbookCi` and can be invoked in
isolation: `dagger call lint-python --source=.`,
`dagger call check-frontmatter --source=.`, etc.

## Process

### Phase 1 — Audit (skip if `--run-only`)

The pipeline exists and is healthy. Audit is scoped to **gate coverage
and health within the Dagger module**, not "should we have CI."

Read `references/audit.md` for the full rubric. Concerns specific to
this repo:

- **New ignored-pattern source.** Was a new `as any` / `@ts-ignore`
  equivalent introduced by a recent commit? If so, add to
  `check-exclusions`.
- **New hardcoded-path vector.** New config file or shell script
  referencing `$HOME` literally? Extend `check-portable-paths` glob.
- **New generated artifact.** Something like `index.yaml` added to the
  repo that could drift? Needs a `check-*-drift` gate.
- **New skill composition contract.** If `/deliver` gained a new phase
  skill, ensure `check-deliver-composition` denylist covers direct
  invocation of its internals.
- **Heal coverage.** A new lint-style gate that would benefit from
  self-heal must be added to
  `ci/src/spellbook_ci/heal_support.py:HEALABLE_GATES`.

Emit a structured audit report:

```markdown
## CI Audit
| Gate / concern              | Status | Severity | Proposed fix |
|-----------------------------|--------|----------|--------------|
| lint-yaml                   | ok     | -        | -            |
| check-exclusions            | weak   | med      | Add `as unknown as` pattern to denylist |
| check-new-concern           | gap    | high     | New gate: scan shell scripts for raw `$REPO_ROOT/…` sourcing |
```

Apply mechanical strengthenings directly. See "Add a new gate" below.
If audit finds nothing worth fixing, say so and proceed.

### Phase 2 — Run (skip if `--audit-only`)

1. Run the gate end-to-end:

   ```bash
   dagger call check --source=.
   ```

   Dagger picks up `dagger.json` at repo root; the ignore list
   (`.git`, `__pycache__`, `.venv`, `ci`, `skills/.external`) is
   applied per-gate via `Ignore(...)` annotations.

2. Capture the structured summary. `check()` prints a sorted
   `PASS/FAIL  <gate>` block followed by indented stderr excerpts for
   each failure. See `parse_check_failures()` in
   `ci/src/spellbook_ci/heal_support.py` for the parser contract.

3. If green → emit report, exit 0.

4. If red → classify each failure per `references/self-heal.md`:

   - **Self-healable lint-style** (`lint-yaml`, `lint-shell`,
     `lint-python`, `check-frontmatter`) — exactly one failure, no
     other gates red:

     ```bash
     dagger call heal --source=. --model=gpt-4.1 --attempts=2
     ```

     Heal returns a `Directory`. Export to a worktree, diff, commit:

     ```bash
     dagger call heal --source=. export --path=./.spellbook/heal-out
     git diff --no-index -- . .spellbook/heal-out   # inspect
     # apply by rsync/mv into working tree, then:
     git add -p && git commit -m "ci: heal <gate-name>"
     dagger call check --source=.                   # verify
     ```

   - **Non-lint or multi-gate failures** — escalate immediately.
     `select_healable_failure()` rejects anything outside
     `HEALABLE_GATES` or more than one concurrent failure. Structured
     diagnosis: gate name, excerpt, candidate cause, suggested first
     file to look at.

   - **Index drift** (`check-index-drift` red) — usually means
     someone edited `index.yaml` manually or bypassed pre-commit.
     Fix by running `bash scripts/generate-index.sh` and committing.
     This is the one "self-heal outside the Dagger heal agent" case.

   - **Vendored-copy drift** (`check-vendored-copies` red) — run
     `bash scripts/check-vendored-copies.sh` locally to see which
     copy is stale; sync from canonical source; commit. Do not
     modify the copy without updating the canonical.

5. Bounded retries: heal is capped at **2 attempts** by default
   (`--attempts=2`). If it exhausts the budget, escalate — the auto-
   fixer isn't converging.

### Phase 3 — Verify

Final `dagger call check --source=.` after any fixes. Green or bust.
If a gate was added/strengthened in Phase 1, the full pipeline must
pass under the new check before `/ci` returns clean.

## Add a New Gate

1. **Write the function** in `ci/src/spellbook_ci/main.py`:

   ```python
   @function
   async def check_<name>(
       self,
       source: Annotated[
           dagger.Directory,
           DefaultPath("/"),
           Ignore([".git", "__pycache__", ".venv", "ci", "skills/.external"]),
       ],
   ) -> str:
       """One-line purpose."""
       return await (
           _lint_container(source)
           .with_exec(["bash", "scripts/check-<name>.sh"])  # or inline python
           .stdout()
       )
   ```

2. **Register in `check()`**. Add a `tg.start_soon(run_gate, "check-<name>", self.check_<name>(source))` line alongside the others.

3. **Decide on heal eligibility.** If the gate's failure mode is a
   mechanical lint fix (syntax, metadata, formatting), add it to
   `HEALABLE_GATES` in `ci/src/spellbook_ci/heal_support.py` with a
   one-line repair instruction. If the failure mode is semantic, leave
   it out — escalation is the correct behavior.

4. **Run locally**:

   ```bash
   dagger call check-<name> --source=.    # isolated
   dagger call check --source=.           # full pipeline
   ```

5. **Document** the gate in the table in this SKILL.md. Gate inventory
   and SKILL.md stay in sync.

## Pre-Commit and Pre-Merge Hooks

`.githooks/` is the hook source (configured via `core.hooksPath`,
re-applied by `bootstrap.sh`):

- **`pre-commit`** enforces three things:
  1. Blocks force-adds of `.spellbook/deliver/<ulid>/(state|receipt).json`
     (these are agent-written, never human-committed).
  2. Regenerates `index.yaml` via `scripts/generate-index.sh` when any
     `skills/*/SKILL.md` or `agents/*.md` is staged.
  3. Runs `scripts/check-harness-agnostic-installs.sh` to reject
     Claude-only seed/tailor wording.
- **`pre-merge-commit`** is the verdict gate for non-FF merges. Blocks
  the merge commit unless `/code-review` left a valid ship verdict for
  the topic branch. Escape: `SPELLBOOK_NO_REVIEW=1`.
- **`pre-push`** runs `dagger call check` before allowing push.
  Skips cleanly if Dagger or Docker are absent — do not treat a
  skipped pre-push as a passed gate. `/ci` must run the actual gate.
- **`post-commit` / `post-merge` / `post-rewrite`** re-run
  `./bootstrap.sh` when skill/agent content changed so global
  symlinks stay current.

When a gate fails because pre-commit didn't run (staged via
`--no-verify`, or bypassed), the fix is typically one of:
- `bash scripts/generate-index.sh` + `git add index.yaml`
- `bash scripts/check-harness-agnostic-installs.sh` — read the
  diagnostic, rewrite the offending copy to harness-agnostic form.

## What /ci Does NOT Do

- Review code semantics → `/code-review`
- Shape tickets or write specs → `/shape`
- Address review comments or coordinate merges → `/settle`
- Deploy or release → no deploy target in this repo
- QA against a running app → no running app in this repo
- Scaffold a Dagger module — already scaffolded (`dagger.json` +
  `ci/`). If the audit turns up "Dagger is missing," the tree is
  corrupted; stop and surface it, don't re-init.
- Lower any threshold or add exclusions to pass a gate.
- Invoke `dagger call check` from inside `skills/deliver/SKILL.md` —
  `check-deliver-composition` will block it.

## Anti-Patterns

- **Declaring green while gates are still running.** `check()` raises
  on any failure and prints the full summary. Wait for exit.
- **Reporting red and exiting** when the failure is a lint drift
  covered by `HEALABLE_GATES`. Run `dagger call heal` first.
- **Feeding heal more than one failing gate.** `select_healable_failure`
  raises; the call will fail. Fix the non-lint failures manually, then
  heal the remaining lint.
- **Lowering a threshold** or **adding an ALLOW entry to
  `check-portable-paths` / skip glob to `check-exclusions`** to make
  a gate pass. Any such change is a red flag — escalate.
- **Running raw `python scripts/check-frontmatter.py` locally** and
  calling that green. It's a fast loop during debugging, but the
  Dagger execution is the source of truth (hermetic container, pinned
  Python 3.12-slim, same versions for everyone).
- **Editing `index.yaml` by hand.** It's derived. Run
  `bash scripts/generate-index.sh`.
- **Auto-fixing a failing bun test** by deleting/`.skip()`-ing it.
  `check-exclusions` will catch the skip; the test failure is a real
  signal.
- **Silencing `check-portable-paths`** by moving the file under
  `harnesses/claude/` purely to bypass the lint. The
  `check-portable-paths` ALLOW set is for files that legitimately
  need Claude-specific paths, not an escape hatch.
- **Unbounded heal loops.** Cap is `--attempts=2`. Further failures
  escalate regardless of classification.

## Output

```markdown
## /ci Report
Audit: 0 gaps (all 14 gates green and current).
Run: dagger call check --source=. — 14/14 pass in 3m44s.
Final: green.
```

```markdown
## /ci Report — RED
Gate: lint-shell
Failure:
  scripts/generate-embeddings.sh:12: SC2086: Double-quote to prevent globbing.
Classification: self-healable (lint-style)
Action: ran `dagger call heal --source=. --model=gpt-4.1 --attempts=2`.
  Attempt 1: auto-quoted $DIR in generate-embeddings.sh. Gate passed.
  Full pipeline re-run: 14/14 pass.
Final: green.
```

```markdown
## /ci Report — RED (escalated)
Gate: test-bun
Failure:
  skills/research/tests/router.test.ts:42
  expected "planner", got "critic"
Classification: logic failure (router selection changed)
Action: escalated. Heal does not cover test-bun.
Suggested first file: skills/research/src/router.ts — route table diff since last commit.
```
