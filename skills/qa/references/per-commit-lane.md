# Per-Commit QA Lane

Use this lane when a commit, PR, or outer-loop cycle needs user-like scenario
evidence attached to the changed surface. It is a local-first evidence contract,
not a watcher, hosted VNC farm, or autonomous fix-PR loop.

The lane starts after `/qa` Step 0 resolves the app shape. Browser automation is
only one possible driver. CLI, library, API, MCP, and hybrid repos keep their
native QA paths.

## Inputs

- `commit_sha`: full or abbreviated commit SHA under review.
- `changed_files_summary`: short, evidence-backed summary from `git diff` or
  an equivalent PR file list.
- `app_shape`: browser, API, CLI, library, MCP, hybrid, or other concrete
  surface from `/qa` Step 0.
- `launch`: command, target URL, tool registration, package install path, or
  explicit reason no launch is needed.
- `acceptance_source`: ticket, oracle, route, README example, fixture, spec, or
  explicit absence.
- `persona`: user or operator role the scenario represents.
- `user_goal`: task the persona is trying to complete.

If the lane cannot ground those fields in live repo evidence, the result is
`inconclusive`; do not fill gaps from lore. If app shape is ambiguous or no
signals are found, return `inconclusive` and list the ambiguity instead of
falling back to browser tooling.

## Scenario Plan Schema

```json
{
  "schema_version": 1,
  "record_type": "per_commit_qa_scenario",
  "commit_sha": "59268bf",
  "changed_files_summary": ["docs/site/index.html", "skills/qa/SKILL.md"],
  "app_shape": "browser",
  "launch": {
    "kind": "url",
    "command": "pnpm dev",
    "target": "http://127.0.0.1:3000/reference/skills/qa"
  },
  "persona": "developer reviewing QA guidance",
  "user_goal": "confirm the QA page explains commit-triggered evidence",
  "steps": [
    "Open the QA reference route",
    "Find the per-commit lane section",
    "Verify the status/evidence requirements are visible"
  ],
  "expected_observable_outcomes": [
    "route responds without console errors",
    "per-commit lane heading is visible",
    "status values include pass, fail, inconclusive"
  ],
  "expected_evidence_artifacts": [
    ".evidence/<branch>/<date>/qa-report.md",
    ".evidence/<branch>/<date>/route-selection.md",
    ".evidence/<branch>/<date>/browser-screenshot.png"
  ]
}
```

For a CLI or library repo, `launch.kind` should name the native surface, such as
`command` or `sandbox_consumer`, and the steps should exercise help text,
README examples, malformed input, public imports, or equivalent repo-fit
behavior.

## Result Schema

```json
{
  "schema_version": 1,
  "record_type": "per_commit_qa_result",
  "scenario_ref": ".evidence/feat-branch/2026-06-04/scenario.json",
  "status": "inconclusive",
  "severity": "P1",
  "app_shape": "browser",
  "exact_surface_exercised": {
    "tool": "browser",
    "route": "http://127.0.0.1:3000/reference/skills/qa",
    "command": "pnpm dev"
  },
  "assertions": [
    {
      "expected": "per-commit lane heading is visible",
      "observed": "route rendered a blank page",
      "result": "fail"
    }
  ],
  "evidence_refs": [
    ".evidence/feat-branch/2026-06-04/browser-screenshot.png"
  ],
  "route_selection_transcript_ref": ".evidence/feat-branch/2026-06-04/route-selection.md",
  "qa_report_ref": ".evidence/feat-branch/2026-06-04/qa-report.md",
  "residual_risk": "Dev server launched, but no expected element was observed."
}
```

Classification must be `pass`, `fail`, or `inconclusive`. A missing expected
element or blank page is never `pass`.

`pass` is valid only when every expected observable outcome has evidence and
every assertion result is `pass`. A blank page, launch failure, missing route,
missing expected element, uninspected console/network failure, or missing
evidence artifact is `fail` or `inconclusive`, never `pass`.

Every result must include:

- exact command, path, route, endpoint, import, or tool call exercised;
- evidence refs under `.evidence/<branch>/<date>/`;
- route/scenario-selection transcript ref;
- committed `qa-report.md` or equivalent text summary ref;
- residual risk, even when the answer is "none with reason".

## Safety Boundaries

- Do not store raw screen recordings, credentials, cookies, private browser
  state, raw prompts, or raw tool outputs as analytics state.
- PR comments, draft releases, and screenshots are mirrors. The canonical
  evidence is the committed `.evidence/` package.
- Fix generation is out of scope. Follow-up fixes route through
  `/deliver --polish-only <branch|PR>`, `/code-review`, `/ci`, and clean-tree
  closeout.
- Telemetry or launch failures fail open for the agent runtime but fail closed
  for the QA verdict: report `fail` or `inconclusive`, not `pass`.

## Report Shape

The text summary should be short and auditable:

```markdown
# Commit QA Report

- Commit: `<sha>`
- App shape: `browser|API|CLI|library|MCP|hybrid`
- Scenario: `<persona>` attempting `<user_goal>`
- Status: `pass|fail|inconclusive`
- Surface exercised: `<command/path/route/tool>`
- Evidence: `<refs>`
- Assertions: `<expected -> observed -> result>`
- Residual risk: `<risk or none with reason>`
- Follow-up: route fixes through `/deliver --polish-only`; no autonomous PR.
```
