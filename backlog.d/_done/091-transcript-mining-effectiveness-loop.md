# Opt-in transcript mining and effectiveness loop

Priority: P2
Status: ready
Estimate: M

## Goal

Add an opt-in, redaction-first analysis path that mines local agent transcripts
and existing evidence stores for recurring corrections, prompt debt,
skill-missed opportunities, and effectiveness signals without storing raw chat
content as analytics state.

## Why Now

The user wants to mine chat logs and understand whether Harness Kit skills are
actually useful. Existing pieces are close but not enough:

- `skills/research/references/introspect.md` describes one-off transcript
  analysis, including tool usage and activity breakdowns.
- `skills/agent-transcript/` packages redacted transcript excerpts for PRs and
  evidence.
- `skills/skillify/` extracts reusable skill candidates from sessions.
- `skills/reflect/references/distill.md` asks retros to inspect conversation,
  skill invocation logs, corrections, and prompt debt.
- `.groom/review-scores.ndjson` exists, but the live trend has too little data
  to prove effectiveness.

The missing layer is a durable, opt-in mining report that turns transcript
evidence into counts/categories/refs, not a raw transcript database.

## Non-Goals

- Do not automatically ingest all chats.
- Do not store raw transcripts, raw prompts, raw tool outputs, secrets,
  customer data, or browser/session state.
- Do not build a judge/eval system in the first slice.
- Do not auto-edit skills from mined patterns.
- Do not replace `/reflect`, `/skillify`, or `/agent-transcript`.
- Do not export transcript data to Langfuse/Phoenix/Tessl.

## Constraints / Invariants

- Explicit operator invocation required.
- Redaction happens before analysis output is persisted.
- Output is categories, counts, timestamps/session refs, and short redacted
  excerpts only when necessary.
- Secret-like strings fail closed.
- The report must name source coverage: Claude transcripts, Codex sessions,
  skill invocation logs, work ledger, receipts, review scores.
- Findings are proposals; `/reflect` or `/harness-engineering` owns any
  codification.

## Authority Order

redacted transcript refs > skill invocation logs > work ledger/receipts >
review scores > model judgment > memory

## Repo Anchors

- `skills/research/references/introspect.md` - current transcript-mining
  recipe.
- `skills/agent-transcript/SKILL.md` and
  `skills/agent-transcript/scripts/agent_transcript.py` - redaction and
  transcript package precedent.
- `skills/skillify/SKILL.md` and `skills/skillify/scripts/` - skill extraction
  precedent.
- `skills/reflect/references/distill.md` and
  `skills/reflect/references/prompt-debt.md` - reflection and prompt-debt
  destinations.
- `crates/harness-kit-checks/src/review_score_trends.rs` and `.groom/review-scores.ndjson` -
  effectiveness trend precedent.
- `crates/harness-kit-checks/src/trace_record.rs` - secret-like refusal pattern.

## Prior Art

- Langfuse's traces and metrics combine observability and evaluation traces to
  derive quality/cost/latency/volume insights; its agent API supports full-text
  search over observations:
  https://langfuse.com/docs/metrics/overview and https://langfuse.com/agents.
- Phoenix pairs traces with span/trace evaluation and human annotations:
  https://arize.com/docs/phoenix.
- Tessl scenario evals compare agent performance with and without a skill,
  which is the right inspiration for effectiveness once Harness Kit has enough
  local examples:
  https://docs.tessl.io/introduction-to-tessl/quickstart-skills-docs-rules.
- OpenTelemetry provides useful span/event vocabulary, but transcript mining is
  a higher-level qualitative analysis and should remain local first:
  https://opentelemetry.io/docs/specs/semconv/gen-ai/.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Opt-in local mining report | Redact then classify transcript/session refs | Private, actionable, fits reflect | Requires explicit runs | Choose |
| Langfuse observation search | Push/search traces externally | Strong search UX | Raw-content/privacy review required | Defer |
| Phoenix evals over traces | Score spans/traces | Strong quality loop | Needs trace export and datasets first | Defer |
| Tessl scenario lift evals | Compare with/without skills | Direct skill usefulness evidence | Needs scenario corpus and registry state | Defer |
| Always-on transcript index | Continuous local database | Rich queries | Privacy and operational burden | Reject |
| Manual reflect only | Keep current qualitative process | No new code | Patterns remain anecdotal | Reject as sufficient |

## Proposed Shape

Add a script or skill reference, likely under `/research introspect` or
`/reflect`, that:

1. Accepts explicit transcript/session paths or source roots.
2. Redacts with the existing agent-transcript redaction rules.
3. Extracts categories:
   - user corrections/redirects;
   - missing or late skill invocation;
   - repeated tool failure;
   - cost/token concern;
   - insufficient evidence claim;
   - privacy/secret risk;
   - successful skill usage pattern.
4. Joins when possible to skill invocation rows, work ledger events,
   delegation receipts, and review-score entries.
5. Emits a report:
   - top recurring correction categories;
   - candidate skill improvements;
   - skill-missed opportunities;
   - effectiveness evidence and gaps;
   - source coverage and redaction summary;
   - proposed backlog/reflection actions.

## Agent Readiness

- Profile source: no dedicated profile; use existing transcript/reflect scripts.
- Stack feedback strength: deterministic fixture transcripts and redaction
  tests.
- ADR decision: not required.
- Infrastructure path: local script invoked explicitly.
- Gate: script self-test, redaction fixture, `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .`,
  then `dagger call check --source=.`
- Evidence storage: redacted report under `.evidence/` or explicit operator
  output path; raw transcript remains external.
- Mock policy impact: preserved; fixtures model external transcript input.

## Delegation Evidence

- Roster providers used:
  - `claude`, receipt `c1bc871f-4122-4786-a2df-e04e62a03c91`, identified
    chat-log mining as recipe-only and not durable.
  - `codex`, receipt `004cb27a-ff40-4918-9ed7-40478b196a7f`, recommended a
    hybrid local-first approach and privacy boundaries.
- Native/retired bench evidence:
  - Architecture critic warned to make chat mining opt-in and artifact-backed.
  - retired bench found current reflect/review-score loops are designed but starved.
- External evidence:
  - Langfuse/Phoenix/Tessl all support trace/eval/search ideas, but none
    removes the need for Harness Kit's local privacy boundary.
- Rejected evidence:
  - Always-on raw transcript indexing.
  - Exporting transcript content before local redaction and schema proof.
- Waivers:
  - No connector access to the user's full transcript stores was used; this is
    shaped from repo scripts and public docs.

## Oracle

- [ ] A deterministic fixture transcript with secret-like text proves redaction
      happens before report output and secret-like content fails closed.
- [ ] The mining command accepts explicit transcript/session paths and refuses
      broad default ingestion unless an operator passes a source root.
- [ ] Report output contains category counts, source coverage, redaction
      summary, and evidence refs.
- [ ] Report output does not contain raw transcript text unless a redacted
      excerpt is explicitly allowed by the command.
- [ ] The report joins skill invocation rows and review-score rows when fixture
      refs match, and labels missing joins.
- [ ] `/reflect` or `/research introspect` references the durable command
      instead of recommending one-off `/tmp` scripts.
- [ ] `cargo run --locked -p harness-kit-checks -- check-agent-roster --repo .` passes.
- [ ] `dagger call check --source=.` passes.

## Observability Plan

- Changed behavior to watch: repeated corrections and skill-missed
  opportunities become counted, redacted evidence instead of impressions.
- Named signal or evidence surface: transcript-mining report, redaction
  summary, joined skill/review evidence.
- Instrumentation debt if no signal exists: no automatic evidence of skill
  usefulness until review scores and skill events are populated.

## Risk + Rollout

- Privacy leakage: fail closed on secret-like strings; store metadata and
  redacted excerpts only.
- Overfitting to one session: require counts/source coverage and distinguish
  single examples from repeated patterns.
- Scope creep into auto-tuning: output proposals only; `031` remains parked.
- Vendor lock-in: keep export out of this ticket.
- Rollback: remove script/reference and fixture; no persistent service state.
