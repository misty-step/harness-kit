# Executable Oracles

An oracle is a check that decides success. Prose oracles drift; executable
oracles enforce. In spellbook, the oracle is what `/deliver`'s clean loop
and `dagger call check` terminate against.

## The Problem

Prose oracles require interpretation:
- "`/foo` should work" — what does "work" mean?
- "Cross-harness parity" — how verified?
- "Tests should pass" — which, where?

These decay into opinion. The builder declares victory, `/code-review`
disagrees, and nobody has a ground truth.

## The Fix: Oracles as Commands

Every oracle should be a command that returns pass/fail. For this repo
that usually means one of:

- a `dagger call check --source=.` sub-gate that turns green
- a `dagger call <other>` return value
- a `scripts/*.sh` exit code
- a `bun test` or `py_compile` run
- a pre-commit hook accepting or rejecting a crafted test commit
- a symlink-install + foreign-project invocation exiting 0

```bash
# Bad: "The new gate should fail bad commits"
# Good:
dagger call check --source=. 2>&1 | grep -q '^check-no-claims: PASS$'

# Bad: "The skill should be self-contained"
# Good:
cd /tmp && rm -rf test-consumer && mkdir test-consumer && cd test-consumer \
  && git init -q && ln -s ~/.agents/skills/foo .agents/skills/foo \
  && grep -L '\.\./\.\.' .agents/skills/foo/*.sh || exit 1
#   (no ../.. escape — self-containment invariant)

# Bad: "The pre-commit hook should reject forbidden strings"
# Good:
git checkout -b test-reject && echo 'claim_acquire' >> skills/x/SKILL.md \
  && git add -A && ! git commit -m 'test' \
  && { git checkout - && git branch -D test-reject; }
#   (exit 0 means the hook correctly blocked the commit)

# Bad: "Bootstrap should symlink the skill globally"
# Good:
./bootstrap.sh --symlink && test -L ~/.claude/skills/foo/SKILL.md \
  && readlink ~/.claude/skills/foo/SKILL.md | grep -q 'spellbook/skills/foo'
```

## Template

When writing the `## Oracle` section of a shape:

```markdown
## Oracle

- [ ] `dagger call check --source=.` green with the new `check-<name>`
      sub-gate included in the 14-gate count
- [ ] pre-commit hook on `./.githooks/pre-commit` rejects a staged
      violation (script: `scripts/test-hook-rejects-<case>.sh`)
- [ ] symlink-install into a foreign project succeeds: skill invokes
      without `../..`, `$REPO_ROOT`, or sourcing spellbook internals
- [ ] `wc -l skills/<name>/SKILL.md` returns < 500
- [ ] `scripts/check-frontmatter.py` passes on the new SKILL.md
- [ ] `index.yaml` regenerates cleanly: `scripts/generate-index.sh | diff - index.yaml` is empty
- [ ] `/deliver` on a synthetic ticket using `/<name>` produces
      merge-ready receipt (status: `merge_ready`)
```

Observable outcomes (verified by human or `/qa`) only when the artifact
has a user-visible surface. Most spellbook shapes don't — the surface is
the harness itself, and the gate is the verifier.

## When You Can't Write an Oracle

If you can't write an executable oracle, the goal isn't clear enough.
Go back to the shape's `## Goal`. Common causes in this repo:

- The shape names a mechanism without naming the failure it prevents
  (e.g. "add telemetry" with no "and here is how we know it fired").
- Success depends on subjective judgment with no proxy metric
  ("improves reflection quality"). Pick a proxy, or defer the shape.
- The test infrastructure doesn't exist yet — e.g., no `check-<name>`
  scaffold in `ci/src/spellbook_ci/main.py`. Building that scaffold IS
  the first oracle: "new `check-<name>` function exists, takes
  `source: dagger.Directory`, returns string."

## Oracle Classes Common in Spellbook Shapes

| Class | Typical Oracle |
|---|---|
| New skill | `wc -l < 500`, symlink-install test, `check-frontmatter`, invocation exit 0 |
| New Dagger gate | Sub-gate name appears in `dagger call check` output, red on crafted fail-case fixture, green on current tree |
| Pre-commit hook rule | Crafted commit is rejected; compliant commit passes |
| Doctrine change | AGENTS.md diff reviewed, plus the lint/gate that makes the doctrine structural — doctrine without enforcement is prose |
| Cross-harness mechanism | Artifact emitted for Claude + Codex + Pi from one source; each harness loads it with its native convention |
| Composition constraint | `check-deliver-composition` or analogous lint rejects a crafted violation |

The oracle makes the shape's invariants *structural* rather than
aspirational. A shape that relies on the agent remembering is a shape
that will regress.
