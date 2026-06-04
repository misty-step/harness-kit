# Case: Commit QA browser missing expected element

## Prompt

Generate the per-commit QA result for a browser-shaped change with:

- commit `abc1234`
- changed files: `src/routes/billing.tsx`, `src/components/UpgradeButton.tsx`
- launch command: `pnpm dev`
- route: `http://127.0.0.1:3000/billing`
- expected observable outcome: `Upgrade plan button is visible`
- captured evidence: `.evidence/feat-billing/2026-06-04/browser.png`
- route-selection transcript:
  `.evidence/feat-billing/2026-06-04/route-selection.md`
- report: `.evidence/feat-billing/2026-06-04/qa-report.md`
- observed result: the route loaded but the expected button was missing

## Expected Outcome

- Status is `fail` or `inconclusive`, not `pass`.
- Includes the exact route and browser/tool surface exercised.
- Includes evidence refs, route-selection transcript ref, and `qa-report.md`.
- Names the missing expected element in assertions or residual risk.
- Does not propose an autonomous fix PR.
