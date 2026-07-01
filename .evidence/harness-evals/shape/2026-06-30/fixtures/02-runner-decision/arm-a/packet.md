# Context Packet: Skill-eval bench — Rust runner in `harness-kit-checks`, or protocol + native-subagent only?

**Decision (ADR-grade): NO new Rust _execution_ runner. The bench stays protocol + native-subagent, backed by the three runner surfaces that already exist in the repo. The only Rust that may later earn itself is a tiny deterministic _objective-check grader_, and only behind a two-consumer trigger.**

Scope note: fixture pins `harness-kit@3bf0b46`. HEAD is `91c11ef`; the only diff between them is `harnesses/shared/references/image-generation.md`, so every load-bearing file below is byte-identical at the pinned SHA. All line references are from the working tree, valid at `3bf0b46`.

## Goal

Decide whether the skill-eval bench needs a new Rust runner inside `crates/harness-kit-checks`, and produce a buildable decision a cold lane can act on — so that when 112's "if a runner becomes necessary" clause is invoked, the answer cites the existing execution surfaces and the one real gap, not taste.

## Non-Goals

- Not implementing anything — no crate code, no manifests, no edits (task constraint).
- Not building a benchmark platform inside Harness Kit (112 non-goal; serious repeated arena work graduates to Daedalus).
- Not re-litigating the eval _protocol_ itself (`mode-eval.md`, `run-recipe.md`, `shape-eval.md` are settled); only the runner question.
- Not adding any subcommand that launches a provider or touches the network — that job is already owned (see Design).

## Constraints

- **Delete-first / "provider CLIs stay thin"** (shared doctrine): new surface must earn itself; the harness must not grow a second provider-launcher.
- **Gates must be tamper-evident, externally enforced, not self-attested** (Layer 1): an "objective" check graded by a subagent inside the same loop is not yet a gate.
- **112's own instruction**: "If a runner becomes necessary, implement the smallest Rust-owned helper in `crates/harness-kit-checks`; otherwise keep the first delivery as protocol plus evidence artifacts." This is a deferral default, not a license to build.
- The clean A/B knob must remain a **manifest diff, not an honor-system prompt** (`run-recipe.md` §"The clean A/B knob").

## Repo Anchors

- `skills/skill-eval/references/run-recipe.md` — defines the three drive modes: native-subagent smoke (free), `council.sh` (serious decorrelated), and the `lane_harness.v1` "clean A/B knob".
- `crates/harness-kit-checks/src/lane_harness.rs` — the `lane_harness.v1` manifest: `allowed_local_skills`, `oracle`, `provider_target`, plus `materialize_manifest` which projects only allowed skills into provider discovery roots (`provider_skill_roots`, lines 263–272) and the failure taxonomy (`FAILURE_KINDS`, lines 19–31).
- `crates/harness-kit-checks/src/main.rs:53` — `dispatch-agent` subcommand; flags `--lane-harness` (1021), `--expect-output` (1022), `--provider-target` (1005).
- `crates/harness-kit-checks/src/agent_roster.rs` — `dispatch_from_options` (123) materializes the manifest (136) then `dispatch_provider_lane` (227) probes, **spawns the provider CLI** (`Command::new`, 330), isolates config via `HOME/CODEX_HOME/CLAUDE_CONFIG_DIR/PI_HOME/GEMINI_CONFIG_DIR/XDG_CONFIG_HOME` pointed at the projection root (339–345), captures a transcript file, enforces the `--expect-output` sentinel, kills on timeout via process group, and emits a structured receipt.
- `crates/harness-kit-checks/src/summarize_delegations.rs:38–42,119–121` — `telemetry`/`summarize-delegations` already aggregate `lane_harness_ref`, `lane_harness_sha256`, `projection_status`, `output_check`, `failure_kind`.
- `skills/council/scripts/council.sh` — bash fan-out to distinct OpenRouter families via opencode/pi; caller owns composition.
- `.evidence/harness-evals/shape/2026-06-30/` — a **real, already-shipped** smoke run (arm-a/packet.md, arm-b/packet.md, blind `grader-input/{X,Y}.md` + hidden `.mapping`, `human-judge/`, `report.md`) produced with **zero new runner code**.
- `backlog.d/112-harness-eval-bench.md` (premise) and `skills/shape/evals/shape-eval.md:25` (this is fixture 2 of the shape eval itself).

## Alternatives

The fixture's framing ("Rust runner _or_ protocol-only") is a false binary. "Runner" conflates two jobs that fail differently: **R1 execution/orchestration** (spin arms, isolate skill visibility, capture transcript, receipt) and **R2 objective grading** (deterministic pass/fail over the produced artifact). R1 already exists, partly in Rust. R2 exists nowhere. Separating them is the whole decision.

| Option | What it buys | How it fails | Verdict |
|---|---|---|---|
| **A — New Rust _execution_ runner** (orchestrates arms, launches providers, one "run the A/B" command) | A single entry point for a full A/B | Re-implements `dispatch-agent` + `council.sh`; spawns a **second** provider-launcher in violation of "provider CLIs stay thin"; pushes Harness Kit toward the benchmark-platform 112 forbids. The only thing it adds over two `dispatch-agent` calls is pairing/shuffling — a shell loop, not a crate. | **Reject** |
| **B — Pure protocol forever; never any new Rust** | Ships today; zero surface; already proven on 2026-06-30 | Leaves the "objective, ~free, every-edit" tier as a **subagent reading a prose checklist** — a gate self-attested by a model inside the loop (Layer-1 violation). Fine for a one-off run, wrong as the standing every-edit gate the skill advertises. | **Reject as the end state** |
| **C — No new execution runner; one small deterministic _grader_ helper, gated by repetition, absorbing the unwired `premise-source validate`** | Keeps execution on the 3 existing surfaces; turns the objective tier into a real falsifiable command exactly where Rust belongs (deterministic policy), tamper-evident and external | Speculative unless the trigger has fired; must stay a pure function (no spawn, no network) or it slides into A | **Choose** |
| **D — Graduate the whole bench to Daedalus arena now** | Strongest repeated-eval substrate | Too much surface before ≥2 skills share task families and rubrics; 112 and `mode-eval.md` both say defer until repetition proves the need | **Defer** |

**Recommendation: Option C.** Today that means: **do nothing new in Rust** (the bench runs on native subagents + `council.sh` + `dispatch-agent --lane-harness`), and pin the grader helper as a deferred, triggered shape — not a build item.

## Design

**Execution (R1) needs no new code.** The repo already has three runner surfaces, matched to the two cadences the skill-eval names:

1. **Free smoke** → native subagents. The orchestrator spins arm A (told to invoke `/<skill>`), arm B (raw same-model), and a blind grader as fresh read-only subagents. Already done for shape; produced falsifiable evidence. No Rust.
2. **Serious decorrelated** → `council.sh`. One task file + `members.tsv` of distinct OpenRouter families; grader lane a family distinct from every worker. No Rust.
3. **Enforced skill-on-vs-off** → `dispatch-agent --lane-harness`. This is already a complete single-lane runner: `materialize_manifest` projects **only** `allowed_local_skills` into the provider's discovery roots and isolates the provider's config env at the projection root, so "skill on" (`allowed_local_skills: [shape]`) vs "skill off" (`[]`) is a **manifest diff the worker cannot ignore** — not a prompt instruction. It then launches the provider, captures the transcript, checks the `--expect-output` sentinel, classifies failures, and emits a receipt that `summarize-delegations`/`telemetry` already roll up. A paired A/B is two invocations with two manifests; shuffling for the blind grader is a `mv X Y` step.

A new Rust execution runner would re-implement `agent_roster::dispatch_provider_lane`. That is the delete-first violation this decision exists to prevent.

**Grading (R2) is the only real gap — and is deferred.** The shape-eval's "Objective checks (scriptable, pass/fail, ~free — run on every `skills/shape/**` edit)" are described in **prose**; no command runs them. Worse, `skills/shape/SKILL.md` Verification cites `harness-kit-checks premise-source validate` — but **that subcommand does not exist in the crate** (no `premise-source` arm in `main.rs`, no module). So the "objective first, ~free, every-edit, tamper-evident" tier the skill promises is currently a model reading a checklist.

When it earns itself, the helper is a **pure-function grader**, not a runner:

```
harness-kit-checks eval-grade <artifact.md> --spec <skill>-eval.md [--json|--plain]
```

- Exit 0 = all objective checks pass; exit 1 = any fail, printing `FAIL <check>` to stderr (the `print_gate_report` convention already used by `check-*` gates).
- Checks (all deterministic, no judge): skeleton sections present + non-empty; oracle contains a runnable-command token (`cargo`/`harness-kit-checks`/test invocation), not "it should work"; cited anchors resolve at the repo SHA; premise-source line well-formed (`sha256:<hex> <path>`); HTML-plan path exists; forbidden-edit globs absent from the diff.
- **No process spawn, no network** — the boundary that keeps it from becoming option A.
- It **absorbs** the dangling `premise-source validate` as one check, rather than shipping a parallel command, and reuses `source_refs.rs` (sha/ref-shape validation already lives there) + `frontmatter.rs`.

**Trigger to build it (the 112 "repetition" bar):** a **second** skill's eval reuses the same objective checklist — two consumers, not one. Until then it is speculative surface and stays a prose checklist graded by subagent (acceptable for low-volume one-off evals; explicitly _not_ acceptable if/when the bench becomes an every-edit pre-merge gate).

### CLI Surface (conditional — for the deferred helper only, not in scope now)

- **Command:** `eval-grade` — deterministic objective-check grader over an eval artifact. Primary user: both human and the eval loop.
- **Inputs:** positional `<artifact.md>`; `--spec <eval-spec.md>` (the checklist source); `--repo PATH`; optional `--diff <range>` for forbidden-edit checks.
- **Output contract:** human gate lines on stderr (`PASS/FAIL <check>`), `--json` for the loop, `--plain` stable lines. Primary verdict via exit code.
- **Exit map:** `0` all pass · `1` ≥1 objective check failed · `2` bad args/missing spec.
- **Config precedence:** flags > defaults; no env, no network, no spawn (invariant).
- **Examples:** `eval-grade .evidence/harness-evals/shape/2026-06-30/arm-a/packet.md --spec skills/shape/evals/shape-eval.md` → `PASS` lines, exit 0.

## Oracle

This is a decision packet; "done" = the decision is correct and a cold lane can act on it. Executable confirmations a stranger can run at `3bf0b46`:

- The three execution surfaces are real, not aspirational:
  - `grep -n '"dispatch-agent"' crates/harness-kit-checks/src/main.rs` → hit at line 53.
  - `grep -n 'Command::new' crates/harness-kit-checks/src/agent_roster.rs` → provider is spawned (≈330).
  - `test -f skills/council/scripts/council.sh` → exit 0.
  - `test -f .evidence/harness-evals/shape/2026-06-30/report.md` → exit 0 (protocol-only path already produced evidence).
- The named gap is real:
  - `grep -rn 'premise.source\|premise_source\|eval-grade\|eval_grade' crates/harness-kit-checks/src/` → **no match** (the cited grader is unwired; no objective-check command exists).
- The clean A/B knob is a manifest diff, not a prompt:
  - `grep -n 'allowed_local_skills' crates/harness-kit-checks/src/lane_harness.rs` and `grep -n 'CLAUDE_CONFIG_DIR\|CODEX_HOME' crates/harness-kit-checks/src/agent_roster.rs` → both hit.
- Decision is actionable: a lane invoking 112's runner clause can answer "no execution runner; defer the grader helper until a second eval reuses the checklist" and cite the rows above. A fresh critic, given this packet + `lane_harness.rs` + `agent_roster.rs` only, returns no blocking methodological flaw (per shape Contract; the adversarial focus is named below).
- Repo gate stays green: `cargo run --locked -p harness-kit-checks -- check --repo .` (no code changed, so trivially true — named so the contract is explicit if the deferred helper ever lands).

## Premise Source

Premise Source: `sha256:18efe7fbfc4013c4da9cbcbf27c4f1d3ab3b129de74ca6dafac1f0acd7693b0a backlog.d/112-harness-eval-bench.md` (carries the "if a runner becomes necessary… otherwise keep the first delivery as protocol plus evidence artifacts" clause this decision resolves). Supporting: `sha256:66934998ce8890f9dd359e13a2142afcbacba91891612456279a2bd8ef6091fa skills/skill-eval/references/run-recipe.md` (defines the three drive modes) and `sha256:1ca2a61dc566eb31d15cc427be046d212f3b244a6534a7e56928ce31d4818f6c skills/skill-eval/SKILL.md`.

## HTML Plan

HTML Plan: `/tmp/skill-eval-runner-plan.html` — authored as the planning medium and opened for rendered review. Hero states the chosen design (no new execution runner; deferred grader behind a two-consumer trigger) and the proof/stop conditions; the body carries the R1/R2 split diagram, the "what the repo already gives" table, the killed-alternatives grid, the deferred-helper shape, and the verification block.

## Risks + Rollout

| Risk | Mitigation |
|---|---|
| **Pairing/aggregation across two lanes secretly needs orchestration**, dragging a real Rust runner back in | Falsifier is explicit: name one A/B need none of the 3 surfaces serve without new execution code. Pairing = two `dispatch-agent` calls; shuffling = a `mv`; aggregation = existing `summarize-delegations`. If a critic finds a genuine gap, that reopens option A — and that is the intended trip-wire. |
| **The deferred grader is option A in disguise** | Hard invariant: the helper is a pure function — no process spawn, no network. If the shape starts launching providers, it has become A and must be rejected. |
| **Objective tier stays self-attested too long** (Layer-1 erosion) if the bench scales to an every-edit gate before the helper lands | Trigger is named (2nd eval reuses the checklist); the gap is logged here so it's a known waiver, not a silent one. Don't promote the bench to an every-edit pre-merge gate while the objective tier is subagent-graded. |
| **`premise-source validate` reference rots further** (cited in shape/SKILL.md, unwired) | Folded into the helper's scope as one check, so when it lands the dangling citation resolves instead of proliferating into a parallel command. (Tracking-only; not this packet's edit.) |

**Rollout:** nothing to ship now. The decision _is_ the deliverable. Undo cost is zero. The only future code path (the grader helper) is itself reversible: a single pure-function subcommand behind a check gate, deletable without touching execution.

**Adversarial review focus (for the fresh critic — artifact only):** Is `dispatch-agent --lane-harness` _actually_ a sufficient A/B execution runner, or does cross-lane pairing/shuffling/aggregation need orchestration that justifies a real Rust runner after all? And does the deferred `eval-grade` helper smuggle option A back in under the word "grader"?
