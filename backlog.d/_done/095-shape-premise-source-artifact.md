# Premise source artifact for `/shape`

Priority: P1
Status: shaped
Estimate: M

## Goal

Make M+ `/shape` packets name the raw premise artifact they were shaped from,
so future implementers can verify what problem was actually accepted, narrowed,
or rejected.

## Source Evidence

- Pasted article hash:
  `sha256:ff278d4ed3965ca36f2eb50dbac6712afee2ea6b3060a5a119cd39825198139c`
  `/Users/phaedrus/.codex/attachments/57bc8d3e-9224-4126-b3ec-298e9fe1cb15/pasted-text.txt`.
- Article themes: plan-first work, raw transcripts instead of summaries,
  plan-for-the-plan, and durable plans as checkpoint artifacts.
- `/shape` already requires an oracle and acceptance artifact hash when an
  acceptance source exists, but it does not require naming the upstream premise
  source for non-trivial shapes.

## Non-Goals

- Do not require every small shape to save a premise artifact.
- Do not store raw private transcripts by default.
- Do not make plan files more authoritative than tests, code, or executable
  oracles.
- Do not add a hosted planning/document review product.
- Do not rewrite `/shape` into Compound Engineering or any other plugin clone.

## Constraints / Invariants

- Raw premise artifacts may be local files, sanitized transcript excerpts,
  screenshots, issue links, PR links, or explicit waivers.
- Private/raw artifacts must remain outside public output unless redacted.
- The packet must distinguish premise source from acceptance source: the former
  explains why the shape exists; the latter proves the implementation.
- Existing context packets remain valid unless edited.

## Authority Order

tests > acceptance artifacts > code > premise artifact > docs > lore

## Repo Anchors

- `skills/shape/SKILL.md` - context packet contract and acceptance evidence
  hash requirement.
- `skills/shape/references/executable-oracles.md` - executable-oracle rules.
- `skills/shape/references/writing-plans.md` - plan-file discipline.
- `skills/implement/references/context-packet.md` - implementation handoff
  expectations.
- `skills/agent-transcript/SKILL.md` - redacted transcript excerpt handling.
- `skills/trace/SKILL.md` - refs, not raw transcripts.

## Prior Art

- Agentic engineering article: make a plan immediately; for deep work, first
  make a plan for the plan; raw transcript context beats premature summaries.
- Current `/shape` practice in `backlog.d/086-commit-qa-assistant.md`: source
  inspiration and screenshot availability were recorded in the packet, but the
  pattern is not yet enforced by the skill.
- Current `/shape` acceptance hash rule: acceptance artifacts are already
  hashable; premise artifacts should follow the same discipline when they are
  load-bearing.

## Alternatives Considered

| Option | Shape | Strength | Failure Mode | Verdict |
|---|---|---|---|---|
| Status quo | Let shape prose summarize the premise | No new fields | Summaries drift and later agents cannot inspect the source | Reject |
| Require `plan.md` for every task | All work starts from committed plan files | Strong checkpoint | Too heavy for small work and conflicts with current packet model | Reject |
| Premise source field for M+ shapes | Packet names source path/link/hash or waiver | Small, auditable, fits `/shape` | Needs careful privacy handling | Choose |
| Raw transcript archive | Store full conversation/transcript in repo | Maximum context | Privacy and token bloat risk | Reject |
| External document review product | Share plans via hosted review tool | Human-friendly | Not cross-harness or local-first | Reject |
| Only use `/trace` after delivery | Journal source context at closeout | Durable after the fact | Too late to shape implementation | Reject |
| Screenshot-only source | Attach images as premise artifacts | Good for UI issues | Insufficient for long-form/product context | Defer as one source type |

## Tradeoff Matrix

| Option | Fit | Size | Privacy | Agent-manageable | Reversible | Testable | Operating Burden |
|---|---:|---:|---:|---:|---:|---:|---:|
| Status quo | 2 | 5 | 5 | 2 | 5 | 1 | 5 |
| Require `plan.md` for every task | 3 | 2 | 3 | 4 | 4 | 4 | 3 |
| Premise source field for M+ shapes | 5 | 4 | 4 | 5 | 5 | 5 | 4 |
| Raw transcript archive | 3 | 2 | 1 | 3 | 2 | 3 | 1 |
| Hosted review product | 2 | 2 | 2 | 2 | 3 | 2 | 2 |
| `/trace` after delivery | 3 | 4 | 4 | 3 | 5 | 3 | 4 |
| Screenshot-only source | 2 | 4 | 4 | 3 | 5 | 3 | 4 |

The chosen shape adds one enforceable packet field rather than a new planning
workflow. It preserves `/shape` as the unit of handoff while making the input
auditable.

## Agent Readiness

- Profile source: `.harness-kit/agent-readiness.yaml` not needed.
- Stack feedback strength: strong if implemented through a small checker over
  packet fixtures.
- ADR decision: not required.
- Infrastructure path: local skill prose plus fixture checker.
- Gate: shape fixture self-test, `python3 scripts/check-agent-roster.py`, then
  `dagger call check --source=.`
- Evidence storage: shape fixture packets under `skills/shape/evals/` or
  equivalent.
- Mock policy impact: preserved; fixtures model local packet text.

## Delegation Evidence

- Roster providers used:
  - `claude` repo investigator, receipt
    `c5a1708e-e046-4590-8141-1d08412317a5`.
  - `pi` premise critic, receipt `fe9a2a9a-c48e-41ce-a4a2-9feea8338884`.
  - `codex` oracle critic, receipt `2920ae5b-d21c-46a6-9202-0861490134fa`.
- Accepted evidence: Claude recommended a `Premise Source:
  sha256:<digest> <path>` field for M+ shapes; Pi's duplicate warning kept the
  ticket scoped to `/shape`, not a generic plan-file clone.
- Rejected evidence: a universal plan.md-first requirement and raw transcript
  archival are too broad and privacy-heavy.
- Waivers: provider lanes did not read the pasted article; the lead hashed and
  summarized the user-provided artifact.

## Oracle

- [ ] `/shape` context packet template includes `## Premise Source` for M+
      shapes.
- [ ] The field accepts `sha256:<digest> <path-or-url>` or an explicit
      `Premise Source Waiver:` line with reason and residual risk.
- [ ] A shape fixture with no premise source fails the checker.
- [ ] A shape fixture with a missing local premise path fails the checker.
- [ ] A shape fixture with a valid local premise source and matching hash
      passes the checker.
- [ ] `skills/shape/SKILL.md` distinguishes premise source from acceptance
      evidence.
- [ ] `python3 scripts/check-agent-roster.py` passes.
- [ ] `dagger call check --source=.` passes.

## Acceptance Evidence

- Acceptance source: shape packet fixture with premise source field.
- Evidence that proves it: checker rejects missing/mismatched premise source
  and accepts valid hash/path.
- Exact command/path/route exercised: implementation should add a deterministic
  self-test command, such as `bash skills/shape/evals/check-premise-source.sh`.
- Oracle / acceptance artifact hash for this shaping input:
  `sha256:ff278d4ed3965ca36f2eb50dbac6712afee2ea6b3060a5a119cd39825198139c`
  `/Users/phaedrus/.codex/attachments/57bc8d3e-9224-4126-b3ec-298e9fe1cb15/pasted-text.txt`.
- Contract-change acknowledgment: the context packet contract intentionally
  gains a premise-source field for M+ shapes.
- Residual risk: URLs can change; local file/hash is preferred for
  load-bearing private or long-form input.

## Observability Plan

- Changed behavior to watch: future shape packets carry inspectable premise
  sources or explicit waivers.
- Named signal or evidence surface: shape checker output and packet field.
- Instrumentation debt: no fleet-wide stats until skill-invocation telemetry
  lands.

## Implementation Sequence

1. Add a `## Premise Source` block to the `/shape` packet template and guidance.
2. Add fixture packets for missing, bad-path, bad-hash, waiver, and valid
   premise source cases.
3. Add the smallest checker/self-test consistent with existing skill eval
   patterns.
4. Link `/agent-transcript` as the redaction path for private transcript
   premise sources.
5. Run the new self-test, roster check, and Dagger gate.

## Risk + Rollout

- Risk: agents over-save raw private context. Mitigate by allowing redacted
  excerpts and waivers, and by pointing to `/agent-transcript`.
- Risk: field becomes ceremonial. Mitigate with a checker for M+ fixtures and
  missing-path failures.
- Rollback: remove the field/checker and fixtures; existing packets remain
  readable.
