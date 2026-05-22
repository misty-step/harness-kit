# Agent provider roster and delegation receipts

Priority: P1
Status: pending
Estimate: L
Aliases: agent roster, delegation receipts, provider bench
Supersedes: `backlog.d/062-provider-delegation-receipts.md`

## Goal

Make Spellbook's default operating model explicit: the lead agent talking
to the human is an agent manager, and non-trivial work should be delegated
through a configured roster of coding-agent providers rather than silently
handled solo. The unit is a cross-harness mechanism: a root-level provider
roster, one deep `delegation_receipt` boundary, a fixed local trace stream,
and shared doctrine that tells skills how to use the roster without
building a semantic orchestration platform.

The first supported roster class is local CLI coding agents:
`codex`, `claude`, `agy`, `pi`, plus conditional/manual entries for
`cursor-agent` and `grok-build` when those binaries and subscriptions are
actually available.

## Why This Isn't `/orchestrate`

The tempting solution is a new `/orchestrate` mega-skill that launches,
supervises, compares, scores, and kills every external agent. That is the
wrong first shape.

Spellbook already has lifecycle primitives:

- `backlog.d/056-agent-session-trace-lifecycle.md` owns durable session
  trace linkage.
- `backlog.d/058-work-ledger-mission-control.md` owns mission-control
  summaries and phase transitions.
- `/deliver`, `/research`, `/code-review`, `/ci`, `/qa`, `/monitor`, and
  `/reflect` already compose the actual work.

This ticket adds the missing boundary: a standard way to know which agent
tools exist, dispatch a comparable attempt when appropriate, and record
sanitized evidence about what happened. It does not create a second
workflow engine.

## Problem Challenge

The user-visible failure is not "we lack a provider adapter." The failure
is that agent delegation is currently informal and unrecoverable:

1. A lead agent may do design or implementation work solo even when fresh
   context or provider diversity would improve the result.
2. When external agents are used, their availability, prompt, worktree,
   output, and final verdict are not recorded in a consistent local
   artifact.
3. There is no durable evidence base for learning which providers perform
   well or badly on task classes.
4. Raw provider transcripts and auth/session directories are unsafe to
   commit, so "just keep logs" is not an acceptable answer.

The shape must turn delegation from an anecdote into evidence without
turning Spellbook into a process supervisor.

## Research Notes

### Exa

Exa surfaced current CLI-agent and orchestration examples, including
Antigravity CLI coverage and several community agent orchestrators. The
signal was that this ecosystem is moving fast, so Spellbook should probe
capabilities at runtime instead of assuming every provider is installed.

### xAI / Grok

xAI's Grok Build is now public as an early beta for SuperGrok Heavy
subscribers, with official pages describing a terminal coding agent,
planning, diffs, plugins, and parallel subagents:

- https://x.ai/news/grok-build-cli
- https://x.ai/cli

Its model docs list `grok-build-0.1` / `grok-code-fast-1` as an
agentic-coding model with tool and structured-output support:

- https://docs.x.ai/developers/models/grok-code-fast-1

Implication: Grok belongs in the roster as a conditional provider, not a
default dependency.

### Provider CLI Reality

Current primary candidates:

| Provider | Local command | Automation signal | Initial tier |
|---|---|---|---|
| Codex | `codex` | `codex exec`, JSON-capable automation, official docs | primary |
| Claude Code | `claude` | `claude -p`, JSON/stream output, permission modes | primary |
| Anti-Gravity | `agy` | `--print`, plugins, settings, permission bypass flag | conditional |
| Pi | `pi` | `pi -p`, `--mode json`, `--mode rpc`, skills/extensions | primary |
| Cursor | `cursor-agent` | public CLI and headless docs, not currently on `PATH` | conditional/manual |
| Grok Build | likely `grok` / `grok-build` | official beta CLI, subscriber-gated | conditional/manual |
| Gemini CLI | `gemini` | still present locally; Antigravity migration is current | fallback |
| Thinktank | `thinktank` | useful multi-agent bench wrapper over Pi | bench, not base roster |

Current local probe during shaping found `codex`, `claude`, `agy`, `pi`,
`thinktank`, and `gemini` on `PATH`; `cursor` was absent.

### Thinktank

Thinktank `research/quick` was launched against this ticket with systems
and verification lanes. It reinforced that local CLI probes are feasible,
that Thinktank itself needs async dispatch/collect treatment because runs
can take minutes, and that receipts should be local-first and append-only.
The rejected suggestion was a separate `.spellbook/provider-receipts/`
store, because that would fork trace/ledger ownership.

### Codebase

Existing repo constraints point to a narrow mechanism:

- `056` already names `.spellbook/traces/` as a viable trace store.
- `058` already names `.spellbook/work/` as the ledger/mission-control
  store.
- `skills/research/provider-adapter.ts` is search-specific and should not
  become a generic delegation manager.
- `skills/deliver/SKILL.md` and `skills/flywheel/SKILL.md` must not grow
  new state-machine internals.
- `harnesses/shared/AGENTS.md` already says default is delegation; this
  ticket should codify the machine-readable roster and receipt evidence
  that make that doctrine operational.

### External References

- OpenAI Codex CLI: https://developers.openai.com/codex/cli
- Claude Code CLI reference: https://code.claude.com/docs/en/cli-reference
- Cursor CLI: https://cursor.com/en-US/cli
- Google Antigravity CLI docs: https://antigravity.google/docs/cli-using
- xAI Grok Build: https://x.ai/news/grok-build-cli
- ABTest coding-agent evaluation paper: https://arxiv.org/abs/2604.03362

## Solution Divergence

### Option A: Deep receipt boundary plus provider roster

Add a root-level roster file and one receipt-writing boundary:
`record_delegation_attempt(...)`. The roster tells agents what can be
tried; the receipt tells the system what happened.

Failure mode: first version is modest and does not automatically supervise
long-running agents.

### Option B: Public `probe -> dispatch -> collect` lifecycle API

Expose adapter phases directly and let skills choreograph provider runs.

Failure mode: temporal decomposition. Every caller learns the lifecycle,
storage leaks through the interface, and Spellbook grows shallow glue.

### Option C: Full agent manager/control plane

Create a first-class orchestrator with process supervision, dashboards,
ranking, retries, worktree lifecycle, and provider-specific optimizations.

Failure mode: violates thin-harness doctrine and competes with `056`,
`058`, `/deliver`, `/research`, and provider-native subagent systems.

Chosen: Option A. Build the stable evidence boundary first. Let later
tickets decide whether supervision, ranking, or UI are justified by real
receipt data.

## Design

### Provider Roster

Add a root-level, repo-local roster at `.spellbook/agents.yaml`.

The roster is committed when it describes repo-approved provider classes
and safe command templates. Per-machine availability remains runtime
state. Local secrets, auth directories, expanded commands, session stores,
and raw transcripts are never committed.

Minimal provider entry:

- `id`: stable provider id, e.g. `codex`, `claude`, `agy`, `pi`,
  `cursor-agent`, `grok-build`, `manual`.
- `tier`: `primary`, `conditional`, `manual`, or `disabled`.
- `kind`: `cli`, `bench`, or `manual`.
- `probe`: safe command template used only to verify availability/version.
- `dispatch`: safe command template or `manual` marker.
- `output`: expected evidence mode, e.g. `json`, `stream-json`, `text`,
  `patch-ref`, `manual-summary`.
- `permissions`: declared permission posture, not a secret-bearing command.
- `worktree`: `required`, `recommended`, or `not_applicable`.
- `notes`: short human-readable caveat.

Initial canonical roster:

- `codex`: primary CLI provider; dispatch via non-interactive `codex exec`
  when a task can be isolated.
- `claude`: primary CLI provider; dispatch via `claude -p` with explicit
  permission/output flags.
- `pi`: primary CLI provider; dispatch via `pi -p` or `--mode json`.
- `agy`: conditional CLI provider; requires local smoke because current
  docs and product surface are new.
- `cursor-agent`: conditional CLI provider; include only when installed
  and smoke-tested locally.
- `grok-build`: conditional CLI provider; include only when installed,
  authenticated, and beta access is confirmed.
- `manual`: always available fallback for GUI-only or closed tools.

### Delegation Receipt Boundary

Expose one deep operation to callers:

```text
record_delegation_attempt(input_ref, provider_targets, objective,
evidence_refs, outcome)
```

The implementation owns probing normalization, status vocabulary,
timestamps, redaction, dedupe, schema versioning, worktree id capture, and
the storage path. Callers do not manage a public `probe -> dispatch ->
collect` sequence.

Receipt rows are append-only JSONL at:

```text
.spellbook/traces/delegations.jsonl
```

This makes `062` a trace input. `058` may summarize it in
`.spellbook/work/`, but the ledger must reference `delegation_id` values
rather than redefining provider schema.

Required receipt fields:

- `schema_version`
- `delegation_id`
- `created_at`
- `repo_root`
- `worktree_id`
- `lead_harness`
- `lead_provider`
- `backlog_ref`
- `objective`
- `input_ref`
- `provider_target`
- `provider_status`: `available`, `unavailable`, `error`, `partial`,
  `manual`
- `attempt_status`: `not_started`, `running`, `succeeded`, `failed`,
  `rejected`, `superseded`, `manual`
- `evidence_refs`: paths or ids only, never raw transcripts by default
- `summary`
- `lead_verdict`: `accepted`, `partially_accepted`, `rejected`,
  `reference_only`, or `pending`
- `redactions_applied`

### Doctrine Patch

Add shared doctrine to `harnesses/shared/AGENTS.md`:

1. The lead agent is the human-facing manager.
2. For non-trivial design, research, review, and implementation work, the
   lead agent consults the provider roster and delegates when at least one
   suitable provider is available.
3. When independent implementations or critiques would materially improve
   confidence, prefer two providers on separate worktrees and record both
   attempts.
4. Direct solo action remains correct for mechanical transforms, already
   diagnosed small fixes, bounded reads, and user-directed direct answers.
5. Delegation without a receipt is incomplete work unless the user
   explicitly requests an ephemeral experiment.

This is doctrine plus evidence. It does not force every one-line task
through multiple agents.

### Commands And Checks

Add the smallest script surface that supports the boundary:

- `scripts/probe-agent-roster.py`: validates `.spellbook/agents.yaml`,
  probes safe availability metadata, and can emit unavailable rows without
  crashing.
- `scripts/record-delegation.py`: appends normalized receipt rows after
  redaction and schema validation.
- `scripts/summarize-delegations.py`: prints descriptive counts by
  provider, task class, status, and lead verdict. It does not rank
  providers.

Add a Dagger sub-gate:

- `check-agent-roster`: validates roster syntax, receipt schema examples,
  no secret-looking values in committed roster or fixtures, and no raw
  provider session directories under committed `.spellbook/`.

Update `.gitignore` so runtime trace output is ignored while committed
configuration remains trackable:

- keep `.spellbook/agents.yaml` and schema/example fixtures eligible for
  commit,
- ignore `.spellbook/traces/*.jsonl` by default,
- keep the existing `.spellbook/deliver/` force-add guard intact.

### Experiments

First experiment class:

1. Same shaped ticket, two provider targets.
2. Two separate git worktrees.
3. Same objective and input ref.
4. Each provider emits a receipt row.
5. Lead agent records comparison: differences in approach, accepted
   output, rejected output, and residual risk.

This produces evidence for future provider selection without scoring
providers prematurely.

## Cross-Harness

The primary layer is `.spellbook/agents.yaml` plus
`.spellbook/traces/delegations.jsonl`. Claude, Codex, and Pi can all read
the same roster file and append the same receipt shape. Harness-specific
advantages are adapter details, not architecture.

Codex behavior: use `codex exec` when available and record the lead
harness/provider in receipts.

Claude behavior: use `claude -p` / output-format flags when available;
Claude settings may include helpers, but the roster remains filesystem
truth.

Pi behavior: use `pi -p` / `--mode json` when available; Pi skills/globs
may expose helper commands, but receipts remain the same JSONL shape.

Anti-Gravity, Cursor, Grok, Gemini, and Thinktank are conditional roster
entries. Missing binaries produce `unavailable` rows, not harness failure.

## Oracle

- [ ] `.spellbook/agents.yaml` exists with primary entries for `codex`,
      `claude`, and `pi`, conditional entries for `agy`, `cursor-agent`,
      and `grok-build`, plus a `manual` fallback.
- [ ] `scripts/probe-agent-roster.py` validates the roster and emits valid
      unavailable status rows when run on a machine with zero matching
      providers.
- [ ] `scripts/record-delegation.py` writes exactly one normalized receipt
      row per delegation attempt to `.spellbook/traces/delegations.jsonl`.
- [ ] Receipt rows include `schema_version`, `delegation_id`,
      `provider_target`, `provider_status`, `attempt_status`,
      `evidence_refs`, `lead_verdict`, `worktree_id`, and
      `redactions_applied`.
- [ ] Manual-import and automated provider attempts validate against the
      same receipt schema.
- [ ] A two-provider/two-worktree smoke fixture records two isolated
      attempts against the same `input_ref` without collisions.
- [ ] `scripts/summarize-delegations.py` prints descriptive counts and
      verdict deltas without provider ranking heuristics.
- [ ] `harnesses/shared/AGENTS.md` states the lead-agent manager doctrine
      and direct-action carveouts.
- [ ] `.gitignore` ignores runtime delegation traces without hiding the
      committed roster config or schema fixtures.
- [ ] `check-agent-roster` is included in `dagger call check --source=.`
      and rejects secret-looking committed roster values, raw provider
      auth/session directories, and invalid receipt examples.
- [ ] `dagger call check --source=.` passes.

## Non-Goals

- No process supervision, kill switch, timeout manager, or runaway-agent
  watchdog.
- No hosted dashboard, database, or semantic workflow DSL.
- No automatic provider ranking or model-selection policy before receipt
  volume exists.
- No provider-specific deep integration beyond command-template probes and
  dispatch templates.
- No raw transcript archival by default.
- No assumption that Cursor Composer, Grok Build, or Anti-Gravity are
  available on every machine.
- No replacement for `/research`, `/deliver`, `/monitor`, `/reflect`,
  `056`, or `058`.

## Related

- Composes with: `backlog.d/056-agent-session-trace-lifecycle.md`
- Feeds: `backlog.d/058-work-ledger-mission-control.md`
- Related: `skills/deliver/references/worktree.md`
- Related: `harnesses/shared/AGENTS.md`
