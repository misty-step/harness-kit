# Context Packet Shape

`/implement` consumes context packets. It does not produce them —
that's `/shape`'s job. This document defines the contract so
`/implement` can reject incomplete packets loudly.

## Required fields

A packet is **complete** iff all of the following are present and
concrete (not "TBD", not "see discussion"):

### `goal` (one sentence, testable)

What the change must do, from the outside. Testable = you can name an
observable outcome. Good vs bad:

- Good: "`parse_check_failures` returns a list of `GateFailure`
  dataclasses from the check() summary text"
- Good: "`skills/flywheel/SKILL.md` stays under 50 lines while
  preserving the outer-loop trigger semantics"
- Bad: "improve the build workflow"

### `oracle` (how we know it's done)

Preferably executable: a list of commands that must exit 0. Prose
checkboxes are acceptable only if each one maps to a concrete
observable (file exists, gate passes, function returns X).

Good:
```
- [ ] `cd ci && python -m pytest tests/test_heal_support.py` exits 0
- [ ] `dagger call check-frontmatter --source=.` exits 0
- [ ] `dagger call check --source=.` exits 0 (all 14 sub-gates)
- [ ] `skills/implement/SKILL.md` exists and is < 500 lines
- [ ] `rg "TODO" skills/implement/` returns no matches
```

Bad:
```
- [ ] Works well
- [ ] Code is clean
- [ ] Ready to ship
```

If the oracle is prose-only, `/implement` will either translate it
into executable form (if the translation is obvious — "passes
frontmatter" -> `dagger call check-frontmatter --source=.`) or stop
and demand a real oracle.

The final oracle command is always `dagger call check --source=.` for
any change that lands on `master`. That's the load-bearing gate.

### `implementation sequence` (ordered steps or explicit "single chunk")

Either:
- An ordered list of steps (useful for multi-behavior features)
- The literal phrase "single chunk" (for atomic changes)

If absent, `/implement` doesn't know how to decompose builder dispatch.
Stop.

## Strongly recommended fields

Not hard-gated but sharply reduce builder error rate.

### `non-goals`

Things that look in-scope but aren't. Prevents builder scope creep.
Example: "Does not modify `skills/deliver/SKILL.md` — that's a
separate ticket." Or: "Does not re-bootstrap; operator handles that
after merge."

### `constraints`

Invariants the change must preserve. Spellbook-specific:
- Cross-harness parity (Red Line) — works on Claude, Codex, AND Pi.
- Self-containment — no `../..` or `$REPO_ROOT/...` sourcing;
  libs resolve via `readlink -f` + `$SCRIPT_DIR/lib/...`.
- SKILL.md under 500 lines.
- No new claim primitives under `skills/`.
- `/deliver` composes via trigger syntax; no inlined phase logic.
- `.spellbook/deliver/<ulid>/` state files are gitignored; no
  force-adds.
- `index.yaml` is never hand-edited (pre-commit regenerates).

### `repo anchors`

Paths to read before starting. Skill examples, similar prior work,
relevant tests. Lets the builder ground itself without guessing.

Examples:
- Tested-module reference: `ci/src/spellbook_ci/heal_support.py` +
  `ci/tests/test_heal_support.py`
- Thin-harness reference form: `skills/flywheel/SKILL.md` (43 lines)
- Dagger gates: `ci/src/spellbook_ci/main.py`
- Cross-harness install pattern: `skills/tailor/SKILL.md` +
  `scripts/check-harness-agnostic-installs.sh`

### `acceptance tests`

Specific test files/cases the builder must produce. Sharpens the
oracle from "tests pass" to "these tests exist and pass."

Example: "add `ci/tests/test_<new>.py::TestClass::test_behavior_X` —
must fail before implementation, pass after."

## Packet resolution order

`/implement` looks for the packet in this order and stops at the
first hit:

1. **Explicit path argument.** `/implement path/to/packet.md` —
   caller knows exactly which packet.
2. **Backlog ID.** `/implement <NNN>` -> resolves to
   `backlog.d/<NNN>-*.md` (glob match on prefix). `backlog.d/_done/`
   is excluded.
3. **Session.** The most recent `/shape` output in the current
   conversation.
4. **Nothing found.** Stop with an instruction to run `/shape` or
   provide a path. Do not scan the backlog for "a likely candidate" —
   that's `/deliver`'s judgment, not `/implement`'s.

## Rejection examples

`/implement` stops (does not proceed) when:

- The packet has `goal` but no `oracle` -> "shape first"
- The oracle is `- [ ] ships successfully` -> unverifiable, stop
- The packet is a raw bug report with no shaping -> stop
- The packet references files that don't exist -> repo-anchor rot,
  stop
- Multiple packets match the ID prefix -> ambiguous, stop and list
- The spec requires a change that violates a constraint (e.g. adding
  `claim_acquire` under `skills/`) -> stop, flag the constraint

A loud stop is always better than a plausible half-built feature.

## Relationship to /shape

`/shape` is the upstream producer. Its output is designed to be
`/implement`'s input. If you find yourself extending `/implement` to
handle "mostly shaped" tickets, the fix is in `/shape` — not here.
Single concern, single judgment domain.
