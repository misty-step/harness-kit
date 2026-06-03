# Context Packet Shape

`/implement` consumes context packets. It does not produce them — that's
`/shape`'s job. This document defines the contract so `/implement` can
reject incomplete packets loudly.

## Required fields

A packet is **complete** iff all of the following are present and
concrete (not "TBD", not "see discussion"):

### `goal` (one sentence, testable)

What the change must do, from the outside. Testable = you can name an
observable outcome. Good vs bad:

- Good: "`/implement` loads a context packet and exits 0 when tests pass"
- Bad: "improve the build workflow"

### `oracle` (how we know it's done)

Preferably executable: a list of commands that must exit 0. Prose
checkboxes are acceptable only if each one maps to a concrete
observable (file exists, route returns 200, function returns X).

Good:
```
- [ ] `pytest tests/implement_test.py` exits 0
- [ ] `skills/implement/SKILL.md` exists and is <300 lines
- [ ] `grep -r "TODO" skills/implement/` returns no matches
```

Bad:
```
- [ ] Works well
- [ ] Code is clean
- [ ] Ready to ship
```

If the oracle is prose-only, `/implement` will either translate it into
executable form (if the translation is obvious) or stop and demand a
real oracle.

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
Example: "Does not modify /deliver's SKILL.md — that's a separate ticket."

### `constraints`

Invariants the change must preserve. Example: "SKILL.md must stay
under 300 lines," "no new dependencies."

### `repo anchors`

Paths to read before starting. Skill examples, similar prior work,
relevant tests. Lets the builder ground itself without guessing.

### `acceptance tests`

Specific test files/cases the builder must produce. Sharpens the oracle
from "tests pass" to "these tests exist and pass."

### `formal spec`

Optional, but hard-gated when present as `Formal Spec Required: yes`. The block
must include an informal spec, formal examples, acceptance oracle, hardening
budget, and waiver path. In that case `/implement` starts with a failing
acceptance test from the formal examples before unit tests or production code.
Packets without the flag stay on the ordinary red-green-refactor path.

## Packet resolution order

`/implement` looks for the packet in this order and stops at the first hit:

1. **Explicit path argument.** `/implement path/to/packet.md` — caller
   knows exactly which packet.
2. **Backlog ID.** `/implement <NNN>` → resolves to `backlog.d/<NNN>-*.md`
   (glob match on prefix).
3. **Session.** The most recent `/shape` output in the current
   conversation.
4. **Nothing found.** Stop with an instruction to run `/shape` or
   provide a path. Do not scan the backlog for "a likely candidate" —
   that's `/deliver`'s judgment, not `/implement`'s.

## Rejection examples

`/implement` stops (does not proceed) when:

- The packet has `goal` but no `oracle` → "shape first"
- The oracle is `- [ ] ships successfully` → unverifiable, stop
- The packet is a raw bug report with no shaping → stop
- The packet references files that don't exist → repo-anchor rot, stop
- Multiple packets match the ID prefix → ambiguous, stop and list

A loud stop is always better than a plausible half-built feature.

## Relationship to /shape

`/shape` is the upstream producer. Its output is designed to be
`/implement`'s input. If you find yourself extending `/implement` to
handle "mostly shaped" tickets, the fix is in `/shape` — not here.
Single concern, single judgment domain.
