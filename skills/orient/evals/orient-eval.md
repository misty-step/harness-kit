# /orient eval

The oracle for the `/orient` skill. Tests the one claim a session-start
orientation skill must earn: **given a repo in a specific, verifiable state
(branch, dirty/clean, in-flight backlog item), `/orient` names the correct
single next move grounded in that live state — and recommends a next move
that would actually be wrong if the state were different — where a bare
"what's going on in this repo, what should I do next" prompt on the same
model either skips reading the live state or pads the answer with a
generic multi-step plan instead of one recommendation.**

This is a `mode-eval` A/B run, not a directory shape. Arms: A = `/orient`
installed and invoked; B = raw same-model (the bare prompt above, no skill,
same read access, no instruction to stay read-only). Grade blind, objective
checks first, judge a different model family than the workers.

## Fixtures

Each fixture is a *repo state*, not just a prompt — set up the actual git/backlog
state before invoking either arm.

| # | Repo state | Repo @ SHA | Forbidden edits | What it stresses |
|---|---|---|---|---|
| 1 | Clean `master`, CI green, one backlog item (`backlog.d/999-fixture-clean.md`) with a Goal + oracle and no blockers | `harness-kit@c6e01b9` + one seeded backlog file | any edit — orient must not act | obvious-state case: punchy 1–2 sentence answer, correct route to `/deliver 999` |
| 2 | Dirty working tree with uncommitted edits to `crates/harness-kit-checks/src/eval_coverage.rs` and no matching backlog item | `harness-kit@c6e01b9` + uncommitted diff | any edit | dirty-branch routing (`/deliver --polish-only`), correctly reads git status not just backlog |
| 3 | Backlog item `backlog.d/998-fixture-blocked.md` marked in-progress but referencing a dependency item that is itself unresolved (a genuinely tangled state) | `harness-kit@c6e01b9` + two seeded backlog files | any edit | "genuinely tangled" case: the skill should use a short list, not a false one-liner, and should name the missing/blocking evidence rather than guess |

Two of three must show A>B for a pass; the fixtures span the obvious case
(where padding is the failure mode), the dirty-branch case (where reading git
state correctly is the failure mode), and the tangled case (where guessing
past missing evidence is the failure mode).

## Objective checks (scriptable, pass/fail, ~free — run on every `skills/orient/**` edit)

- [ ] No file under the repo is modified, staged, or committed by the arm —
      orient is read-only by contract.
- [ ] Fixture 1: the report is ≤3 sentences (punchy-beats-complete is
      violated by a padded multi-paragraph answer on an obvious state).
- [ ] Fixture 1: the recommended next move names the correct backlog ref
      (`999`) and the correct skill (`/deliver`).
- [ ] Fixture 2: the report correctly identifies the tree as dirty (not
      clean) and routes to a dirty-branch move, not a fresh-ticket move.
- [ ] Fixture 3: the report explicitly names the missing/blocking evidence
      (the unresolved dependency) rather than recommending a next move that
      ignores it.
- [ ] The report does not claim the repo "ready" or "validated" — routes that
      judgment to the owning skill per the Stay-in-lane contract.
- [ ] No provider/lane dispatch occurs unless the fixture's scope is
      genuinely broad or contested (none of fixtures 1–3 qualify).

## Rubric (1–5, blind, one-line justification each — judgment-heavy delta only)

| Dimension | 5 | 1 |
|---|---|---|
| Signal-to-padding ratio (fixture 1) | states the obvious state and the one move in 1–2 sentences | multi-section report on a trivial state |
| State accuracy (fixture 2) | correctly reads dirty/clean, branch, and in-flight work from git truth | asserts a state that doesn't match `git status` |
| Honesty under ambiguity (fixture 3) | names the missing evidence instead of guessing a move | picks a confident-sounding next move that ignores the blocker |
| Route correctness | recommended skill matches the routing table for the actual signal | recommends an unrelated or overly broad next step |

## Pass condition

Arm A beats arm B on state accuracy and honesty-under-ambiguity across **≥2 of
3** fixtures, AND ties-or-wins every objective check. A no-op "orient"
(equivalent to raw prompting) fails because the raw arm reliably either pads
the obvious case (fixture 1) with an unnecessary full rundown, or guesses a
plausible-sounding next move on the tangled case (fixture 3) without reading
the actual blocking dependency.

## Human anchor

The operator blind-grades fixture 3 (the tangled/blocked case — the one where
"sounds right" and "is right" diverge most, since a plausible wrong answer is
easy to write and easy for a lenient grader to accept). Record the verdict and
match/mismatch here once run. **PENDING — no run yet.**

## Cadence

- Edit-time: 1-fixture native-subagent smoke (fixture 1) on any
  `skills/orient/**` change — cheap, and fixture 1's brevity check catches
  prose bloat immediately.
- Contract change (the routing table, the read-only boundary, or the
  punchy-beats-complete rule moves): full A/B, all 3 fixtures, decorrelated
  families.
- Major model release: re-audit — a stronger bare model may already default
  to reading live git/backlog state before answering, closing `/orient`'s edge.

## Run log

**No run yet.** Spec seeded 2026-07-01 under backlog.d/128 (EVALS-PER-SKILL);
`/orient` was named in the epic's designated hot-5 set and had no eval
coverage before this. A run that didn't fire both arms + a falsifiable grader
is not a result — this entry is a placeholder, not a verdict.
