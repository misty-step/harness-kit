# Shape eval — run report (2026-06-30, fixture 1 smoke)

**Eval:** `skills/shape/evals/shape-eval.md` · **Skill under test:** `/shape` ·
**Driver:** `skills/skill-eval` (native-subagent smoke per `references/run-recipe.md`)

## Setup

- **Fixture:** `01-telemetry-top` — "Add a `--top <N>` flag to the `telemetry`
  subcommand that limits the report to the N most-used skills." Repo
  `harness-kit@3bf0b46`, spec-only (no source edits).
- **Arms:** A = `/shape` (agent read & followed `skills/shape/SKILL.md`);
  B = raw ("flesh out this spec into something buildable", no skill). Same model,
  same repo, same fixture.
- **Blinding:** grader saw `X.md` / `Y.md`, not the arm labels. Mapping (revealed
  post-grade): **X = arm B (raw), Y = arm A (shape).**
- **Family:** arms + grader all Claude. **Smoke waiver** — this proves the loop
  fires and the *direction* of the delta, not a certified margin. Certification
  needs the decorrelated paid run (council, distinct families per arm/grader).

## Objective checks (blind)

| Check | B (raw) | A (shape) |
|---|---|---|
| 10 sections present | FAIL (no Alternatives / Premise Source / HTML Plan / Risks+Rollout) | PASS |
| Oracle has runnable token | PASS | PASS |
| Repo Anchors 3–10 real paths | PASS | PASS (7, verified live) |
| Alternatives ≥2 + verdict + one pick | FAIL | PASS (A/B/C/D + CHOSEN) |
| Premise Source line | FAIL | PASS (explicit waiver + residual risk) |
| HTML Plan path | FAIL | PASS |
| No forbidden edits | PASS | PASS |
| CLI-design block | PARTIAL | PASS |

## Rubric (blind, 1–5)

| Dimension | B (raw) | A (shape) |
|---|---|---|
| Premise challenge | 4 | 5 |
| Alternatives fail-differently + pick | 2 | 5 |
| Architecture depth | **5** | 4 |
| Tooling + verification | 5 | 5 |
| Executability by a stranger | 4 | 5 |
| Artifact quality | 4 | 5 |
| **Total** | **24/30** | **29/30** |

## Verdict

**Arm A (shape) is more buildable. Pass on fixture 1** (A beats B on aggregate
rubric AND ties-or-wins every objective check).

The win is on-claim: shape supplied the scope-lock (skills-table vs whole-report
ambiguity flagged with a stop condition), the alternatives-with-a-pick, the
premise-source waiver, and the HTML plan — the scaffolding a cold stranger needs
to confirm they're building the right-scoped thing. Raw produced a precise diff
but no decision scaffolding.

**Honest counter-evidence (why this isn't a rubber-stamp):**
- Raw **won architecture depth** (5 vs 4).
- Raw **caught a real gotcha shape missed**: at `skill_invocation_analytics.rs:291`
  the `PathBuf`s are moved into the first `analyze` call, so shape's third
  `self_test` call would hit a use-after-move (compiler-caught, but a stranger
  following shape verbatim trips it).
- The grader explicitly considered and rejected a tie.

## Variance / limitations

- **n=1 fixture.** Skill pass condition is ≥2 of 3. Fixture 1 is the *easiest*
  case for raw (small, well-scoped CLI). Shape's edge should widen on fixture 2
  (architecture call) and fixture 3 (high-risk money/auth) — unrun.
- **Shared model family** (smoke waiver) — margin uncertified.

## Decision label

**`/shape`: needs-more-tasks** (fixture 1 → A>B on-claim; run fixtures 2–3 and one
decorrelated paired run before `keep`).

**`skill-eval` skill:** the loop fired end-to-end and returned a blind,
falsifiable, on-claim verdict that could have gone the other way. The mechanism
works.
