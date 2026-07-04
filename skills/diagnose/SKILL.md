---
name: diagnose
description: |
  Investigate, audit, triage, and fix. Systematic debugging, incident lifecycle,
  domain auditing, and issue logging. Feedback-loop-first protocol:
  reproduce or replay before root cause, pattern analysis, hypothesis test,
  and fix.
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
- Run parallel hypothesis probes when multiple plausible causes exist.
- For design-quality root causes, dispatch an ad-hoc **lens** critic
  (`harnesses/shared/references/lenses.md`) returning `finding · evidence
  file:line · impact` — name the lens, no static agent file.

## Delegation Judgment

delegate on judgment per the shared Roster contract: native subagents
by default; add cross-model critics, roster providers, or sprite lanes
(`/sprites`) only when they answer a distinct question. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use independent evidence or hypothesis lanes for competing root causes, reproduction, config/service checks, and proposed fix validation.

## Routing

| Intent | Sub-capability |
|--------|---------------|
| Debug a bug, test failure, unexpected behavior | This file (below) |
| Need a reproduction strategy | `references/feedback-loops.md` |
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

## Learnings

Before root-cause ranking, grep known failures:
`rg -n --glob '*.md' '^(title|tags|applies_when):|<module>|<symptom>|<failure-mode>' docs/solutions`.
Open likely matches and test any applicable known cause first.

## The Iron Law

```
NO FIXES WITHOUT A FEEDBACK LOOP AND ROOT CAUSE FIRST
```

If you cannot run, replay, or inspect a signal that proves the symptom is
present, you cannot verify a fix. Build the smallest believable pass/fail loop
before hypothesizing deeply. If no loop is possible with current access, stop
and ask for a captured artifact, environment access, or temporary
instrumentation.

## Rule #1: Config Before Code

External service issues are usually config, not code. Check in order:

1. **Env vars present?** `npx convex env list --prod | grep <SERVICE>` or `vercel env ls`
2. **Env vars valid?** No trailing whitespace, correct format
3. **Endpoints reachable?** `curl -I -X POST <webhook_url>`
4. **Then** examine code

If the symptom is external-service behavior and the provider surface is
supported by `emulate.dev`, build the feedback loop against the local emulator
before blaming production config or writing code. Do not use emulation to prove
provider-only concerns such as real auth, billing, quota, or live webhook
delivery. Usage details: https://emulate.dev/docs.

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

1. **Plan loop probes** -- form 2-3 candidate observations that would
   discriminate between possible causes. These are instrumentation targets, not
   root-cause conclusions.
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
Don't use when: bug reproduces in your environment (use the main debugging
phases directly).

## The Debugging Phases

### Phase 1: Build the Feedback Loop

The feedback loop is the first deliverable. Choose the narrowest loop that
reproduces the user's symptom, not a nearby failure.

1. **Read the symptom exactly** -- full stack traces, user steps, error codes,
   wrong output, timing, and scope.
2. **Pick a loop** -- test, curl/API script, local third-party API emulator,
   CLI fixture, browser script, trace replay, bisect harness, differential run,
   or HITL log loop. See
   `references/feedback-loops.md`.
3. **Run it until you trust it** -- failure matches the reported symptom and
   repeats, or the reproduction rate is high enough for a flaky bug.
4. **Sharpen it** -- make it faster, more deterministic, and more specific
   before moving on.
5. **Stop if no loop exists** -- report what you tried and request the minimum
   missing artifact or access. Do not continue on vibes.

### Phase 2: Root Cause Investigation

Only after a loop exists:

1. **Check recent changes** -- `git diff`, `git log --oneline -10`, new deps, config
2. **Gather evidence in multi-component systems** -- log at each component boundary, run once, identify failing layer
3. **Trace data flow** -- where does the bad value originate? Trace backward to source
4. **Minimize the repro** -- reduce the loop to the smallest input/path that
   still fails.

### Phase 3: Pattern Analysis

1. **Find working examples** -- similar working code in same codebase
2. **Compare completely** -- read reference implementations fully, don't skim
3. **Identify all differences** -- however small
4. **Understand dependencies** -- settings, config, environment, assumptions

### Phase 4: Hypothesis and Testing

Scientific method. Rank 3-5 hypotheses when the cause is not obvious. Test one
prediction at a time. No stacking.

1. **Form falsifiable hypotheses** -- "If X is the cause, then changing or
   observing Y will make Z happen."
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

### Phase 5: Implementation

1. **Write failing regression first** -- use the highest seam that exercises
   the real bug pattern. If no correct seam exists, record that as an
   architecture finding and file a follow-up ticket (or run a `/critique
   --lens ousterhout` pass) rather than forcing a bad seam.
2. **Verify test fails for the right reason** -- not syntax/import errors
3. **Implement single fix** -- address root cause. ONE change at a time.
4. **Rerun original loop** -- the Phase 1 loop must pass, not only the minimized
   regression.
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
- **/research delegate**: Multi-model hypothesis validation

## Output

- **Root cause**: What's actually wrong
- **Fix**: How it was resolved
- **Verification**: Observable proof it works
- **Learning**: If the fix revealed a reusable repo-technical pattern, offer
  `/compound` while evidence is fresh.

## Gotchas

- **Fixing before investigating:** The #1 failure mode. If you haven't traced data flow, you don't know the root cause.
- **Stacking changes:** One variable per experiment. Multiple simultaneous changes make results uninterpretable.
- **Confusing symptom for root cause:** "The test fails" is a symptom. "The auth token expires before the refresh interval" is a root cause.
- **Skipping reproduction:** If you can't reproduce it, you can't verify the fix. Gather more data first.
- **Config is almost always the answer:** Env vars, endpoints, credentials. Check config before reading code.

## Verification

Semantic waiver: diagnosis quality is tied to the concrete failure and cannot
be proven by a single static fixture. Each run must cite reproduction evidence,
the hypothesis test, the fix path, and the exact command or runtime surface
that proves the symptom is gone.
