# /triage

Fix production issues. Audit, investigate, fix, verify, postmortem.

## Usage

```bash
/triage                        # Audit and fix highest priority (default)
/triage investigate INC-456    # Deep dive on a specific production incident
/triage investigate-ci 12345   # Deep dive on specific CI run failure
/triage fix                    # Create PR for current fix
/triage verify                 # Verify fix with observable proof
/triage postmortem INC-456     # Generate postmortem after merge
```

## Stage 1: Production Audit

Audit these sources in parallel:
1. **Incident platform** -- unresolved incidents/issues from Canary, Sentry, or equivalent
2. **Vercel logs** -- Recent errors in stream
3. **Health endpoints** -- `/api/health` response
4. **GitHub CI/CD** -- Failed workflow runs

If all clean: "All systems nominal. No action required."

## Stage 2: Investigate

### Incident Issues (`/triage investigate ISSUE-ID`)

1. Fetch full incident context from the project's incident platform (API, CLI, or connector)
2. Parse stack trace, file paths, breadcrumbs, affected users
3. Create branch: `fix/incident-$(date +%Y%m%d-%H%M)`
4. Load affected files, check `git log --oneline -10` for causal commit
5. Form root cause hypothesis

### CI/CD Failures (`/triage investigate-ci RUN-ID`)

1. `gh run view RUN-ID --log-failed`
2. Identify failed step and error
3. Create branch: `fix/ci-[workflow]-[date]`
4. Check recent commits for regression

### Delegation

- **Builder sub-agent**: Stack trace analysis, code archaeology
- **/research**: Prior art, known issues, similar incidents
- **/research delegate**: Validate proposed fix before implementing
- **Parallel investigators**: When >2 plausible root causes (see `/diagnose`)

### Multi-Hypothesis Mode

When >2 plausible root causes:
1. Create agent team with 3-5 investigators
2. Each teammate gets one hypothesis to prove/disprove
3. Teammates challenge each other's findings
4. Lead synthesizes consensus root cause

## Stage 3: Fix (`/triage fix`)

Prerequisites: On `fix/` branch with changes.

1. **Write failing test first** -- reproduce the error BEFORE fixing
2. Run tests to verify fix
3. Create PR with the incident/ticket link for traceability
4. If fix cannot be verified within 30 min, revert the causal commit:
   ```bash
   git revert <causal-commit> --no-edit
   git push
   ```

## Stage 4: Verify (`/triage verify`)

**MANDATORY** -- a fix is just a hypothesis until proven by metrics.

### Verification Protocol

1. **Define observable success criteria:**
   ```
   SUCCESS CRITERIA:
   - [ ] Log entry: "[specific message]"
   - [ ] Metric change: [metric] from [X] to [Y]
   - [ ] Database state: [field] = [expected]
   - [ ] API response: [endpoint] returns [expected]
   ```

2. **Trigger test event:**
   ```bash
   stripe events resend [event_id] --webhook-endpoint [endpoint_id]
   curl -X POST [endpoint] -d '[test payload]'
   ```

3. **Observe results:**
   ```bash
   vercel logs [app] --json | grep [pattern]
   npx convex logs --prod | grep [pattern]
   ```

4. **Verify database state:**
   ```bash
   npx convex run --prod [query] '{"id": "[id]"}'
   ```

5. **Document evidence:**
   ```
   VERIFICATION EVIDENCE:
   - Timestamp: [when]
   - Test performed: [what]
   - Log observed: [paste]
   - Metric before/after: [values]
   - Database confirmed: [yes/no]
   VERDICT: [VERIFIED / NOT VERIFIED]
   ```

### Red Flags (NOT Verified)

- "The code looks right now"
- "It should work"
- "Let's wait and see"
- No log entry observed, metrics unchanged

### If Verification Fails

1. Don't panic -- hypothesis was wrong
2. Revert if fix made things worse
3. Loop back to investigation
4. Question assumptions

## Stage 5: Postmortem (`/triage postmortem ISSUE-ID`)

Prerequisites: Fix deployed (PR merged).

### Blameless Postmortem Structure

- **Summary**: One paragraph -- what, impact, resolution
- **Timeline**: Key events in UTC
- **Root Cause**: Actual underlying cause, not symptoms
- **5 Whys**: Dig to systemic factors
- **What Went Well**: Good practices during response
- **What Went Wrong**: Honest assessment, no blame
- **Follow-up Actions**: Concrete items with ownership

Write to `docs/postmortems/YYYY-MM-DD-ISSUE-ID.md` or update `INCIDENT-{timestamp}.md`.

### Stage 6: Prevent

If systemic issue:
1. Create prevention issue
2. Optionally `/deliver` the fix
3. Codify learnings (regression test, agent update, monitoring rule)

## Auto-Detected Issues

Issues labeled `auto-detected` + `bug` are created by the observability pipeline.
Treat as P0 unless evidence suggests otherwise.

## Incident Platform Integration

When incident-platform observability is configured, use these sources per phase:

| Phase | Tool | Use |
|-------|------|-----|
| Audit | Issue search, timeline API, or alert feed | Find fresh incident classes and regressions |
| Investigate | Full issue details, traces, webhook payloads | Recover stack trace, breadcrumbs, and correlated context |
| Verify | Post-deploy issue search plus production smoke checks | Confirm the failing flow is now healthy |
| Manage | Incident workflow | Resolve, suppress, classify, or assign incidents |

Include the incident link in the PR for traceability. If the platform supports
auto-resolution on deploy, use its canonical closing reference as well.

## Useful Commands

```bash
# GitHub CI
gh run list --branch main --status failure --limit 10
gh run view RUN-ID --log-failed
gh run rerun RUN-ID --failed

# Health check
curl -fsS https://<host>/api/health

# Vercel logs
vercel logs <app> --json

# Convex logs
npx convex logs --prod
```

## Environment Variables

```bash
INCIDENT_PLATFORM_TOKEN  # Example: CANARY_API_KEY or SENTRY_AUTH_TOKEN
INCIDENT_PLATFORM_TARGET # Example: project slug, service name, or API endpoint
INCIDENT_WEBHOOK_SECRET  # Optional when reproducing webhook delivery failures
VERCEL_TOKEN             # Optional for vercel logs
```

## References

- `references/investigation-protocol.md` -- Root-cause workflow and incident work log
- `references/fix.md` -- How to land and verify the fix once the cause is proven
- `references/log-issues.md` -- Convert recurring findings into GitHub issues

## Related

- `/diagnose` -- Systematic debugging protocol
- `/log-issues <domain>` -- Create GitHub issues from recurring findings
- `/reflect` -- Session retrospective and codification
