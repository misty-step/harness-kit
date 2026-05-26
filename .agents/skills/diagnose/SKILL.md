---
name: diagnose
description: |
  Investigate, audit, triage, and fix. Systematic debugging, incident lifecycle,
  domain auditing, and issue logging. Four-phase protocol: root cause → pattern
  analysis → hypothesis test → fix.
  Use for: any bug, test failure, production incident, error spikes, audit,
  triage, postmortem, "diagnose", "why is this broken", "debug this",
  "production down", "is production ok", "audit stripe", "log issues".
  Trigger: /diagnose.
argument-hint: <symptoms or domain> e.g. "error in auth" or "audit stripe"
---

# /diagnose

Find root cause. Fix it. Prove it works.

## Execution Stance

You are the executive orchestrator.
- Keep hypothesis ranking, root-cause proof, and fix selection on the lead model.
- Delegate bounded evidence gathering and implementation to focused subagents.

## Delegation Floor

When `.spellbook/agents.yaml` exists, `/diagnose` starts by probing the
roster and dispatching two or more available roster members for independent
evidence or hypothesis lanes before selecting a fix. Use separate lanes for
competing root causes, reproduction, config/service checks, or proposed fix
validation. The lead agent owns the root-cause verdict, fix choice, proof,
and receipts. Direct lead-only diagnosis is limited to mechanical command
execution, emergency unblocks, explicit user-forbidden delegation, or an
explicit waiver when fewer than two roster members are available.
- Run parallel hypothesis probes when multiple plausible causes exist.

## Routing

| Intent | Sub-capability |
|--------|---------------|
| Debug a bug, test failure, unexpected behavior | This file (below) |
| Flaky test investigation | `references/flaky-test-investigation.md` |
| Incident lifecycle: triage, investigate, postmortem | `references/triage.md` |
| Domain audit: "audit stripe", "audit quality" | `references/audit.md` |
| Audit then fix highest priority issue | `references/fix.md` |
| Create GitHub issues from audit findings | `references/log-issues.md` |

If first argument matches a domain name (stripe, quality, etc.), route to `references/audit.md`.
If "triage", "incident", "postmortem", "production down" → `references/triage.md`.
If "flaky", "flake", "intermittent", "nondeterministic test" → `references/flaky-test-investigation.md`.
If "fix" → `references/fix.md`. If "log issues" → `references/log-issues.md`.
Otherwise, this is a debugging session — continue below.

**The user's symptoms:** $ARGUMENTS

## The Iron Law

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

If you haven't completed Phase 1, you cannot propose fixes.

## Rule #1: Config Before Code

External service issues are usually config, not code. Check in order:

1. **Env vars present?** `npx convex env list --prod | grep <SERVICE>` or `vercel env ls`
2. **Env vars valid?** No trailing whitespace, correct format
3. **Endpoints reachable?** `curl -I -X POST <webhook_url>`
4. **Then** examine code

## Sub-Agent Patterns

### Quick investigation (default)

Spawn at least two roster-backed **Explore** lanes to gather evidence. Tell
them to investigate the symptoms, reproduce the issue, trace data flow, and
report back with root cause + evidence + proposed fix. They should NOT
implement the fix — just report.
You review, decide if root cause is proven, then dispatch a **builder** for
the fix or dig deeper.

### Multi-Hypothesis Mode

When >2 plausible root causes and a single investigation would anchor on one:
spawn parallel **Explore** subagents, one per hypothesis. Each gets one
hypothesis to prove or disprove by tracing a specific subsystem. They report
back with confirmed/disproved + evidence. You synthesize into a consensus root
cause, then dispatch a **builder** (general-purpose) for the fix.

Use when: ambiguous stack trace, multiple services, flaky failures.
Don't use when: obvious single cause, config issue, simple regression.

### What you keep vs what you delegate

| You (lead) | Sub-agents (investigators) |
|------------|---------------------------|
| Ranking hypotheses | Tracing one subsystem |
| Declaring root cause proven | Comparing working vs broken |
| Choosing the fix | Gathering logs and reproductions |
| Deciding when evidence is sufficient | Running targeted test cases |

## Instrumented Reproduction Loop

When you can't reproduce the bug yourself (auth-gated, mobile, timing-dependent,
hardware-specific, user-flow-dependent):

```
INSTRUMENT → USER REPRODUCES → READ LOGS → REFINE → REPEAT
```

1. **Hypothesize** -- form 2-3 candidate root causes from symptoms
2. **Instrument** -- add targeted logging that discriminates between hypotheses.
   Write to a log file the user can share back:
   ```bash
   LOG_FILE="${HOME}/Desktop/debug-$(date +%s).log"
   ```
   Log at decision points: function entry/exit, branch taken, values at boundaries.
   Tag each log line with the hypothesis it tests: `[H1] auth token expired: ${token.exp}`
3. **Hand off** -- tell user: "Reproduce the bug, then say done." Give exact steps if known.
4. **Read & analyze** -- when user signals done, read the log file. For each hypothesis:
   - Supported? Design next experiment to narrow further.
   - Disproved? Eliminate, remove its instrumentation, add new hypothesis.
   - Insufficient data? Add more targeted logging at the next layer.
5. **Iterate** -- repeat until one hypothesis survives all evidence. Max 3 rounds —
   if still ambiguous after 3, escalate to Multi-Hypothesis Mode (agent teams).
6. **Clean up** -- remove all instrumentation before fixing. Instrumentation is diagnostic,
   not the fix.

Use when: flaky tests, user-reported bugs you can't trigger, environment-specific issues.
Don't use when: bug reproduces in your environment (just use Phase 1-4 directly).

## The Four Phases

### Phase 1: Root Cause Investigation

BEFORE attempting ANY fix:

1. **Read error messages carefully** -- full stack traces, line numbers, error codes
2. **Reproduce consistently** -- exact steps. If not reproducible, gather more data
3. **Check recent changes** -- `git diff`, `git log --oneline -10`, new deps, config
4. **Gather evidence in multi-component systems** -- log at each component boundary, run once, identify failing layer
5. **Trace data flow** -- where does the bad value originate? Trace backward to source

### Phase 2: Pattern Analysis

1. **Find working examples** -- similar working code in same codebase
2. **Compare completely** -- read reference implementations fully, don't skim
3. **Identify all differences** -- however small
4. **Understand dependencies** -- settings, config, environment, assumptions

### Phase 3: Hypothesis and Testing

Scientific method. One experiment at a time. No stacking.

1. **Form single hypothesis** -- "I think X causes Y because Z" (write it down explicitly)
2. **Design experiment** -- What will prove or disprove this? Justify: why this experiment,
   what will it tell us? Smallest possible change, one variable only.
3. **Run experiment** -- observe result
4. **Evaluate**:
   - **Disproved** → eliminate this cause, form NEW hypothesis. This step matters —
     ruling things out is progress, not failure.
   - **Supported** → design next experiment to increase confidence. Not proven until
     you can explain the full causal chain.
   - **Ambiguous** → experiment was too broad. Narrow scope and rerun.
5. **Repeat** until root cause is proven or confidence is high enough to act

Never skip justification. "Just try X" is a red flag — if you can't explain what
you'll learn from an experiment, you don't understand the problem yet.

### Phase 4: Implementation

1. **Write failing test first** -- reproduce the bug in a test before any fix
2. **Verify test fails for the right reason** -- not syntax/import errors
3. **Implement single fix** -- address root cause. ONE change at a time.
4. **Verify** -- test passes, no other tests broken, issue resolved.
5. **If 3+ fixes failed** -- STOP. Question the architecture. See `references/systematic-debugging.md`.

## Root Cause Discipline

For each hypothesis, categorize:
- **ROOT**: Fixing this removes the fundamental cause
- **SYMPTOM**: Fixing this masks an underlying issue

Post-fix question: "If we revert in 6 months, does the problem return?"

## Demand Observable Proof

Before declaring "fixed", show:
- Log entry proving the fix worked
- Metric that changed
- Database state confirming resolution

Mark as **UNVERIFIED** until observables confirm.

## Classification

| Type | Signals | Approach |
|------|---------|----------|
| Test failure | Assertion error | Read test, trace expectation |
| Runtime error | Exception, crash | Stack trace -> source -> state |
| Type error | TS complaint | Read error, check types |
| Build failure | Bundler error | Check deps, config |
| Behavior mismatch | "Does Y, should do X" | Trace code path |
| Performance | Slow, timeout | Add timing instrumentation |
| Production incident | Incident tracker, alerts | Create INCIDENT.md, timeline |

## Investigation Work Log (Production Issues)

For non-trivial production issues, create `INCIDENT-{timestamp}.md`:
- **Timeline**: What happened when (UTC)
- **Evidence**: Logs, metrics, configs checked
- **Hypotheses**: Ranked by likelihood
- **Actions**: What tried, what learned
- **Root cause**: When found
- **Fix**: What resolved it

## Red Flags -- STOP and Return to Phase 1

- "Quick fix for now, investigate later"
- "Just try changing X and see"
- Multiple simultaneous changes
- Proposing solutions before tracing data flow
- "One more fix attempt" (when 2+ already tried)
- Each fix reveals new problem in different place

## Toolkit

- **Incident platform**: Canary timeline/report endpoints, Sentry issue details, or equivalent incident tooling
- **Git**: bisect, blame, recent deploys
- **Observability**: platform logs, incident tracker signals, monitoring dashboards
- **Sub-agents**: Parallel hypothesis investigation (see above)
- **/research thinktank**: Multi-model hypothesis validation

## Output

- **Root cause**: What's actually wrong
- **Fix**: How it was resolved
- **Verification**: Observable proof it works

## Gotchas

- **Fixing before investigating:** The #1 failure mode. If you haven't traced data flow, you don't know the root cause.
- **Stacking changes:** One variable per experiment. Multiple simultaneous changes make results uninterpretable.
- **Confusing symptom for root cause:** "The test fails" is a symptom. "The auth token expires before the refresh interval" is a root cause.
- **Skipping reproduction:** If you can't reproduce it, you can't verify the fix. Gather more data first.
- **Config is almost always the answer:** Env vars, endpoints, credentials. Check config before reading code.
