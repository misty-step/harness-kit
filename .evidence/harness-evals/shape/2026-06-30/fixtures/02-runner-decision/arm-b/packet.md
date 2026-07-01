# Decision Spec: Skill-eval bench — Rust runner vs protocol + native-subagent

**Repo:** `/Users/phaedrus/Development/harness-kit` (working HEAD `91c11ef`; the requested `3bf0b46` is not the current checkout tip, but the four named files are all present and read at current HEAD — flag if you need the exact SHA checked out).

**Question:** Should the skill-eval bench gain a Rust *runner* inside `harness-kit-checks`, or stay protocol + native-subagent (+ `council.sh`) only?

## TL;DR — Decision

**Do not build a dedicated Rust eval runner/orchestrator subcommand. Stay protocol + native-subagent + `council.sh`, sitting on the Rust primitives that already exist.** The parts of an eval where determinism is load-bearing — skill on/off enforcement, input projection, secret redaction, schema validation, failure classification, receipt recording — are **already Rust-owned** in `lane_harness.rs` + `dispatch-agent`. The parts a "runner" would add — fan-out, A/B pairing, scoring, report assembly — are either already covered cheaply (council.sh / native subagents), or are judgment by design (the margin and the keep/adapt/cut verdict, which the skill explicitly forbids handing to an unanchored model judge). A Rust runner would add surface in the one zone the backlog's own non-goals fence off ("Do not build a large benchmark platform inside Harness Kit"; "add Rust helper only if repetition proves it").

Close the **one real gap** (the serious decorrelated run loses A/B enforcement) with a recipe change and an optional thin shell wrapper, not a crate subcommand. If repeated arena work later proves the orchestration must be a product, it graduates to Daedalus — which is the standing escape hatch, not a reason to grow `harness-kit-checks`.

## What already exists (this reframes the question)

The premise "should there *be* a Rust runner" is partly already answered — a single-lane Rust dispatcher exists and is wired:

- **`harness-kit-checks dispatch-agent`** (`main.rs:53`, `parse_dispatch_agent_args` at `main.rs:947`, `agent_roster::dispatch_from_options` at `agent_roster.rs:123`) already does, per lane: probe the provider and skip on failure, dispatch with timeout/grace, capture the transcript to a file, check an `--expect-output` sentinel, classify the failure (`auth_required` / `credits_exhausted` / `sentinel_mismatch` / `nonzero_exit` …), and record a delegation receipt.
- **`--lane-harness <manifest>`** (`main.rs:1021`) feeds a `lane_harness.v1` manifest into `materialize_manifest` (`lane_harness.rs:152`), which validates it (`deny_unknown_fields`, roster-checked `provider_target`, skill-existence, path-escape and symlink-escape guards, secret-like-text rejection, pinned-external-alias check) and **materializes a projection root** containing only `allowed_local_skills`, then enforces it at dispatch by repointing the provider's config dirs at that root (`agent_roster.rs:339-345`: `HOME`, `CODEX_HOME`, `CLAUDE_CONFIG_DIR`, `PI_HOME`, `GEMINI_CONFIG_DIR`, `XDG_CONFIG_HOME`).

So "skill on vs off" is **already** a tamper-evident manifest diff (`allowed_local_skills: ["x"]` vs `[]`), not an honor-system prompt — exactly as `run-recipe.md` §"The clean A/B knob" claims. There is **no** `eval` subcommand and no `eval.rs` today (subcommands are `check`, `bootstrap`, `telemetry`, `destructive-command-guard`, `dispatch-agent`, `summarize-delegations`, `record-delegation`, probe).

## The three layers of an eval, and where each belongs

| Layer | What it is | Correctness-critical? | Right owner |
|---|---|---|---|
| **1. Setup / enforcement** | skill visibility real not honor-system; inputs pinned; secrets redacted; schema valid | **Yes** — a worker can subvert it | **Rust — already done** (`lane_harness` + `dispatch-agent`) |
| **2. Execution / fan-out** | run N lanes across model families in parallel, collect outputs, report which failed | No invariant a worker can break; it's process orchestration | **Bash/native** — `council.sh` (decorrelated) + native subagents (smoke). A Rust re-impl is duplicate surface |
| **3. Grading / verdict** | objective checks → rubric → keep/adapt/cut | objective sub-tier: yes; **margin + verdict: no — judgment by design** | objective checks = per-eval **runnable commands** (already the contract); margin/verdict = **human anchor + decorrelated judge**, never a Rust scoreboard |

A Rust eval runner could only legitimately own Layer 1 (done) and the *objective sub-tier* of Layer 3. Everything else it would touch is either duplicate (Layer 2) or unautomatable on purpose (the SKILL is emphatic: "Human judgment is the ground truth"; "Agent judge ≠ ground truth"; the verdict "is signed off by the human"). The objective sub-tier doesn't need a subcommand either — the skill already mandates "oracle is a runnable command not 'it should work'", so each eval ships its own checks and the repo gate (`harness-kit-checks check`) already runs.

Net: a Rust runner has **almost no surface it could own that isn't already owned or deliberately un-ownable.**

## The one real gap (and why it doesn't need a runner)

There is a genuine architectural seam, currently undocumented:

- **Enforced A/B** (`--lane-harness`) only works through `dispatch-agent`, whose `provider_target` must be a roster CLI provider that has a projection root — `provider_skill_roots` (`lane_harness.rs:263`) emits `skills`, `.codex`, `.claude`, `.pi`, `.gemini`.
- **Decorrelated fan-out** (`council.sh`) runs `opencode`/`pi` over OpenRouter (`council.sh:89-90`) and **cannot project skills on/off** — `opencode` isn't even in the projection-root list. So the most expensive, most-trusted "serious run" currently has the **weakest A/B integrity** (honor-system skill instructions).

This is real, but it composes away without new Rust:

1. **Run the two enforced arms through `dispatch-agent --lane-harness`** (arm A skills=`[x]`, arm B skills=`[]`), decorrelating the workers with `--provider-target` / `--model-override` across roster providers.
2. **Use `council.sh` only for the blind grader** (a family distinct from both workers), or run the grader as a native subagent.

That's a **recipe** (and optionally a 30-line `eval.sh` that wraps the three `dispatch-agent` calls + pairing, exactly as `council.sh` wraps fan-out) — not a crate subcommand.

## Buildable spec — recommended path ("no runner")

**Deliverable A — fix the recipe (required, ~1 file).** Edit `skills/skill-eval/references/run-recipe.md`:
- In §"Serious run", state the enforcement seam explicitly: `council.sh` lanes are honor-system; skill visibility is only *enforced* through `dispatch-agent --lane-harness`.
- Add the composition: enforced arms via `dispatch-agent --lane-harness` (manifests differ only in `allowed_local_skills`), decorrelate via `--provider-target`/`--model-override`; grader via a distinct family (council.sh lane or native subagent).
- Note that `opencode` has no projection root, so opencode-only A/B is honor-system and is a *smoke* waiver, never the certified-margin run.

**Deliverable B — optional thin wrapper (`skills/skill-eval/scripts/eval-ab.sh`, ~30-50 lines).** Mirrors `council.sh`'s shape and license to live as bash: takes `--skill`, `--fixture`, `--arm-a-target`, `--arm-b-target`, `--grader`, writes the two `lane_harness.v1` manifests to a temp dir, invokes `dispatch-agent --lane-harness` twice + the grader once, lays the outputs into the `.evidence/harness-evals/<skill>/<date>/` packet shape `run-recipe.md` already specifies. Owns no scoring or verdict — it stops at "both arms + grader produced artifacts." Build this only if you run the serious A/B more than ~twice by hand; otherwise the recipe is enough.

**Deliverable C — none in Rust.** No `eval.rs`, no `eval` subcommand. The objective-check tier stays per-eval runnable commands in each `skills/<skill>/evals/<skill>-eval.md`, gated by the existing `harness-kit-checks check`.

**Acceptance / oracle for the recommended path:**
- `cargo run --locked -p harness-kit-checks -- check --repo .` still passes (no Rust change → trivially true; the standing oracle from backlog 112).
- A serious A/B for one skill produces an evidence packet where arms A and B were run under *enforced* manifests (`lane-harness-manifest.sha256` present in each arm's projection, differing only in `allowed_local_skills`) — provable from receipts, not prose.
- The grader family differs from both worker families (decorrelation invariant), shown in the packet.
- `run-recipe.md` names the enforcement seam and the compose-around; a fresh critic reading only the recipe + one packet finds no "honor-system serious run" methodological hole.

## When this decision flips (triggers to revisit)

Build the Rust runner only if **all** of these become true — i.e., repetition proves the platform, per backlog 112's "add Rust helper only if repetition proves it":
1. The serious A/B is run on a **schedule** (e.g. every major model release across the whole skill catalog), not ad hoc — so hand-orchestration is the bottleneck.
2. The **objective-check tier** has converged on a *shared* cross-skill grammar (same checks reused across ≥5 evals), making a generic deterministic grader a real dedup rather than premature abstraction.
3. The `eval-ab.sh` wrapper has proven insufficient (e.g. you need pairing/variance math that bash can't carry cleanly).

If those land, the right move is **graduate to Daedalus's arena loop** (the explicit standing escape hatch in both `mode-eval.md` boundaries and the SKILL's "Eval bloat" gotcha), **not** grow `harness-kit-checks` into a benchmark product. The only Rust that should *ever* land in `harness-kit-checks` for evals is more of Layer 1 (e.g. a manifest field to pin fixture SHAs) — never Layer 2 or the verdict.

## Rejected alternative — the Rust runner (for completeness)

Minimal shape if you overrule this: an `eval` subcommand that reads an eval spec, materializes arm-A/arm-B manifests, calls `dispatch_provider_lane` N times, runs declared objective-check commands, and emits `report.md` with a score matrix. **Why rejected:** (1) it re-implements `council.sh`'s fan-out in Rust (duplicate surface, violates "delete before adding"); (2) its only novel deterministic value is the objective-check sub-tier, which is already per-eval commands behind the existing gate; (3) it cannot own the load-bearing output (the margin / keep-adapt-cut), so it dresses a judgment call in a scoreboard — precisely the "rubric laundering" / "benchmark theater" failure modes the SKILL and backlog 112 warn against; (4) it pushes `harness-kit-checks` toward the "benchmark platform" the non-goals forbid. It buys a tidier CLI for work that is intentionally low-frequency and judgment-bound.

## Risks of the recommended path

| Risk | Mitigation |
|---|---|
| Recipe-only enforcement seam keeps getting forgotten in practice | Deliverable A makes it explicit; Deliverable B's wrapper makes the enforced compose the path of least resistance |
| Hand-orchestration friction tempts a premature runner | The three flip-triggers above are the explicit bar; below it, friction is correct (keeps eval frequency honestly low) |
| `opencode` lanes silently used as serious A/B | Recipe states opencode A/B is honor-system → smoke waiver only; certified margin must route enforced arms through `dispatch-agent` |

**Bottom line:** the Rust footprint for the eval bench is already at the right boundary (enforcement/validation in `lane_harness` + `dispatch-agent`). The gap is a recipe/composition gap, not a missing runner. Fix the recipe, optionally add a council-style bash wrapper, and keep the verdict where the doctrine insists it stays — with a human anchor and a decorrelated judge, not a Rust scoreboard.

**Key files:** `skills/skill-eval/references/run-recipe.md` (the edit), `crates/harness-kit-checks/src/lane_harness.rs` and `crates/harness-kit-checks/src/agent_roster.rs` (the existing Rust primitives — leave as-is), `skills/council/scripts/council.sh` (the wrapper precedent for Deliverable B), `backlog.d/112-harness-eval-bench.md` (the non-goals that gate this decision).
