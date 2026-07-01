# Repo-local QA skills: rollout + bridge wiring

Priority: P1 · Status: in progress · Estimate: M

## What shipped (2026-06-30)
The Cramer pattern (project-local QA skills = highest-value local agent
investment) as a Harness Kit primitive + a seeded flagship set.

- **Template / contract:** `skills/qa/templates/repo-qa-skill.md` — the canonical
  shape (Surfaces map → Start local runtime w/ exact commands+ports+env → per-
  surface steps → Evidence/Report). Thin-first, "refine through use", concrete-
  over-generic, interview-the-operator, `<repo>-qa` naming.
- **Scaffold path:** `/qa` gap-flag now routes to a real scaffold recipe (explore
  run surface → interview → write from template → self-verify).
- **Audit affordance:** `/harness-engineering` "Repo-QA audit" — enumerate active
  repos, check for `<repo>-qa`, close the gap thin-first (never a deterministic
  blast; line-173 failure mode).

## Seeded (grounded, thin, uncommitted — one per repo)
| repo | skill | surface | deterministic gate (necessary, not sufficient) |
|---|---|---|---|
| cerberus | `cerberus-qa` (exemplar) | review runner (Rust + OpenCode/OMP) | `./scripts/verify.sh` (fixtures replay canned model output) |
| bitterblossom | `bb-qa` | `bb` CLI + plane + Fly Sprite exec | `./scripts/verify.sh` (stubs + local substrate) |
| memory-engine | `memory-engine-qa` | FSRS kernel + Axum API/UI + gen brain | `bun run ci` (fixtures can't prove the model brain) |
| daedalus | `daedalus-qa` | Rust CLI eval foundry | `bin/gate` (grader calibration is the real QA) |
| canary | `canary-qa` | Axum obs service + CLI + TS SDK | `./bin/validate` (Dagger; live write-path unproven) |
| sploot | `sploot-qa` | Next.js meme app + API + extension | `pnpm lint/type-check/test/build` (jsdom, not the app) |

`laboratory` already had a `qa` skill (pre-existing) — rename to `laboratory-qa`
for naming consistency (minor).

## BLOCKER: skill bridge unwired across the fleet
Claude Code loads `.claude/skills/`; canonical source is `.agents/skills/`;
daybook bridges them (`.claude/skills -> ../.agents/skills` symlink). **No
flagship consumer repo has this bridge** (bitterblossom, daedalus, cerberus,
simons all have `.agents/skills` but no `.claude/skills`; memory-engine/canary/
sploot/landmark have neither). Only `laboratory` is wired (`.claude/skills` +
`.codex/skills`) — and it's the only one that had a working QA skill. **Until
bridged, all six new skills are correct files that do not surface in-harness.**
Decide + wire: `.claude/skills -> ../.agents/skills` (and `.codex/skills` where
Codex is used) per repo, or route through harness-kit bootstrap/seed. Note
`sploot`'s `AGENTS.md` forbids vendoring skill catalogs — the repo-local QA skill
is the sanctioned exception; add a one-line carve-out there.

## Tail — on-demand (do NOT batch; scaffold each on its next real QA pass)
- **Products (high QA value, likely next):** chrondle, doomscrum, brainrot, linejam
- **Infra/tools:** simons, landmark, standby, dexter, crucible, allie, bastion,
  curb, vanity, conviction, workbench, daybook
- **Content/ops (low QA value — may never need one):** misty-step, web-presence,
  aesthetic, hermes-skins, nassau-ops

## Done when
- [x] Template + /qa scaffold + harness-engineering audit shipped; gate green.
- [x] Five flagships seeded + cerberus exemplar.
- [ ] Bridge decision made + wired so the skills load.
- [ ] Tail scaffolded on-demand (tracked here, not blasted).
