# /critique: ad-hoc lens critic against a target

Priority: P1
Status: ready
Estimate: M

## Goal

Create a `/critique` skill that dispatches a **fresh, ad-hoc critic subagent
embodying a named lens** (from the shared lens rubric) against a target path or
module — e.g. `/critique --lens ousterhout --target src/auth`. It does not
dispatch a static agent file; it reads the lens rubric (`061`) and commissions
a critic with that perspective, scope, and evidence contract. This plugs the
gap where the operator hand-writes a detailed critic prompt in chat.

## Non-Goals

- Do NOT depend on `agents/*.md` persona files. Lenses come from the rubric
  shipped by `061`; the critic is instantiated ad-hoc.
- Do NOT be a ship gate. `/critique` is read-only architectural signal, never a
  merge verdict. The merge gate is `/code-review`.
- Do NOT duplicate `/code-review`'s parallel bench. `/critique` is a single
  targeted lens (or a small explicit set), invoked outside the PR flow.

## Constraints / Invariants

- **Resolve the trigger collision.** `/code-review`'s frontmatter currently
  claims `/critique` as a trigger. `/code-review` must drop `/critique`;
  `/critique` owns it. `/code-review` keeps `/review`. (Enforced by `076`.)
- Cross-harness: the skill instructs the primary to spawn a subagent with the
  lens prompt; it must degrade to "the primary adopts the lens itself" on
  harnesses without subagent spawning.

## Repo Anchors
- `harnesses/shared/references/lenses.md` — the lens rubric (from `061`); the
  source of `--lens` names and their "what it looks for" content.
- `skills/code-review/SKILL.md:3-8` — the `/critique` trigger to remove.
- `skills/code-review/references/bench-map.yaml` — path→lens auto-selection
  (folded in from original `079`).
- `skills/shape/SKILL.md:91-100` — existing ad-hoc bench dispatch to mirror.

## Oracle (Definition of Done)
- [x] `skills/critique/SKILL.md` authored; triggers on `/critique` only.
- [x] Accepts `--lens <name>` and `--target <path>`; `--lenses` lists the lens
      names available in the rubric.
- [x] `/critique --lens ousterhout --target <path>` dispatches a fresh
      read-only critic that embodies the ousterhout lens (from the rubric),
      reads the target, and returns structured findings (finding · evidence
      file:line · impact). No `agents/ousterhout.md` is referenced.
- [x] `/code-review` no longer claims the `/critique` trigger;
      `grep -n "/critique" skills/code-review/SKILL.md` returns nothing in the
      trigger line. `076`'s collision check passes.
- [x] `skills/code-review/references/bench-map.yaml` maps security-sensitive
      paths (auth, crypto, middleware, fetch/SSRF) to the `security` lens and
      test files to the `cooper` lens, so `/code-review` and `/critique` select
      lenses by path automatically; bench cap of 5 preserved (security lens
      REPLACES a general lens on those paths, not added on top).
- [x] `dagger call check --source=.` passes; existing `/code-review` mock runs
      show no regression.

## Implementation Sequence
1. (Depends on `061` shipping the lens rubric.) Author `skills/critique/SKILL.md`
   with the ad-hoc lens-critic dispatch contract.
2. Remove the `/critique` trigger from `skills/code-review/SKILL.md`.
3. Add/extend `bench-map.yaml` path→lens auto-selection (folds in `079`).
4. Smoke: `/critique --lenses`, `/critique --lens ousterhout --target <dir>`,
   and a `/code-review` mock run on a security path that auto-selects `security`.

## Risk + Rollout
- **Operators ignore audit output** if it lacks actionable findings. Mitigation:
  require findings to carry file:line evidence + an impact rating; offer a
  `--emit-tickets` follow-up later (out of scope here).
- **Lens drift** if the rubric and skill disagree on lens names. Mitigation:
  `--lenses` reads names directly from the rubric; no hardcoded list.

## Delegation Evidence
- agy + codex independently recommended `/critique` as a standalone philosophy
  audit decoupled from the `/code-review` merge gate, and flagged the trigger
  collision. Accepted.

## Related
- Depends on `061` (lens rubric + dispatch doctrine).
- Folds in original `079` (cooper + security bench-map wiring).
- Enforced by `076` (trigger-collision gate).
- Boundary: `/code-review` = merge gate; `/critique` = read-only lens signal.

## What Was Built

- Added `skills/critique/SKILL.md` as a focused read-only lens critique skill
  with `/critique` as its only trigger, `--lenses`, `--lens <name>`, and
  `--target <path>` contracts, plus fresh-subagent and primary-adopts-lens
  fallback instructions.
- Added a `security` lens to `harnesses/shared/references/lenses.md` and kept
  lens names in the shared rubric rather than static `agents/security.md`.
- Converted `/code-review` bench selection prose from static agent selection to
  deterministic reviewer-lens selection, including optional `replace` semantics.
- Updated `bench-map.yaml` so auth/crypto/middleware/fetch/URL paths select
  `security` and replace `grug`; test paths select `cooper`.
- Added `scripts/check-bench-map.py` and wired it into Dagger as
  `check-bench-map` to validate reviewer ids, security replacement, Cooper test
  selection, and the 5-reviewer cap.
- Added `/critique` eval scaffolding and a code-review eval case covering
  security replacement.

## Delegation Evidence

- `claude` (`2633145d-ec05-4651-a497-8c2d0f012334`) recommended adding the
  `/critique` skill, deriving `--lenses` from the rubric, adding a `security`
  lens, using optional `replace: [grug]`, and adding an executable bench-map
  checker. Accepted.
- `grok-build` (`41f38a2a-c925-413a-8800-06f2f442cde0`) independently
  converged on the same lens-not-static-agent model and the security/cooper
  bench-map updates. Accepted. Its suggestion that full Dagger was unnecessary
  was rejected because the Harness Kit repo gate is non-negotiable.

## Verification

- `python3 scripts/check-frontmatter.py`
- `python3 scripts/check-bench-map.py`
- `python3 scripts/check-agent-roster.py`
- `python3 skills/harness-engineering/scripts/validate-evals.py`
- `python3 scripts/check-evidence-blocks.py skills`
- `grep -n "/critique" skills/code-review/SKILL.md || true` -> no matches
- `bash scripts/build-docs-site.sh`
- `bash scripts/generate-index.sh`
- `bash scripts/check-docs-site.sh`
- `git diff --check`
- `dagger call check --source=.` -> 17 passed, 0 failed

## Completion Gate

- Exact end-user behavior changed: operators can invoke `/critique` for a
  single read-only lens critic, and `/code-review` now routes security/test
  paths through deterministic `security`/`cooper` reviewer lenses.
- Evidence that proves it: Dagger passed with `check-bench-map`, frontmatter,
  docs-site, eval-structure, roster, and evidence-block lanes green.
- Exact command/path/route exercised: `dagger call check --source=.` and
  `python3 scripts/check-bench-map.py`.
- Oracle / acceptance artifact hash:
  `09d0ce84067f3a12b04a1be1355da95c4070cc5ebd94429b2fa8904d2b8f1ba9`
  for this backlog item before closeout edits.
- Contract-change acknowledgment: changed `/code-review` bench-map schema by
  adding optional `replace` so a specialty security lens can take a general
  lens slot without exceeding the cap.
- Repo-fit check: follows `harnesses/shared/references/lenses.md` roles-not-files
  doctrine and Harness Kit's generated `index.yaml` / docs-site flow.
- Hardening run / waiver: no `/hardening` run; this is skill/doctrine/YAML
  routing, and the new Dagger checker covers the changed selection invariant.
- Residual risk: `/critique` execution remains an instruction-level skill, not
  a CLI parser; runtime fidelity depends on the active harness following the
  skill contract.
