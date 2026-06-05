---
name: a11y
description: |
  Accessibility audit, remediation, and verification. WCAG 2.2 AA compliance.
  Three-agent protocol: audit (find issues) → remediate (fix them) → critique (verify fixes).
  Use when: "accessibility audit", "a11y", "WCAG", "screen reader",
  "keyboard navigation", "contrast check", "aria fix", "accessibility sprint",
  "audit accessibility", "fix accessibility", "a11y issues", "a11y check".
  Trigger: /a11y
argument-hint: "[audit|fix|verify] [route|component|--scope full]"
---

# /a11y

Audit, fix, and verify accessibility. WCAG 2.2 AA.

**Target:** $ARGUMENTS

## Execution Stance

You are the executive orchestrator.
- Keep severity decisions, scope tradeoffs, and final PASS/FAIL judgment on the lead model.
- Delegate audit, remediation, and critique to separate focused subagents.
- Prefer parallel exploration for independent checks; keep remediation sequential when fixes interact.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use specialized lanes for accessibility auditor,
remediation builder, keyboard/screen-reader verifier, and cold WCAG critic.
Prefer different providers for audit and critique when available; native
in-thread subagents may supplement but do not satisfy the roster floor.

## Routing

| Intent | Action |
|--------|--------|
| `/a11y` or `/a11y audit` | Full triad: audit → remediate → critique |
| `/a11y audit <target>` | Audit only — read `references/audit.md` |
| `/a11y fix <target>` | Remediate only — read `references/remediate.md` |
| `/a11y verify` | Critique recent changes — read `references/critique.md` |

If no sub-command, run the full triad below.

## Three-Agent Protocol

### Phase 1: Audit (read-only)

Launch an Explore subagent to find issues. It does NOT fix anything.

**Automated scan:**
- If a dev server is running, scan with Playwright + axe-core
  (`wcag2a`, `wcag2aa`, `wcag22aa` tags)
- If no server, scan component source statically

**Static analysis — grep for anti-patterns:**
- `div onClick` or `span onClick` without `role` + `tabIndex` + `onKeyDown`
- `<img` without `alt=`
- `<table>` without `<caption>` or `aria-label`
- `<Button>` with only icon children and no `aria-label`
- `<input` or `<select` without associated `<label>` or `aria-label`
- `tabindex` values > 0
- `outline: none` or `outline: 0` without replacement focus style

**Structural checks:**
- Landmarks: `<main>`, `<nav aria-label>`, `<header>`, `<footer>`
- Skip-to-content link as first focusable element
- Focus management on SPA route change
- `aria-required` on required form fields
- `aria-sort` on sortable table columns
- Dialog focus restoration on close
- `prefers-reduced-motion` media query for animations

**Output:** structured findings as:
```
## [SEVERITY] WCAG [criterion]: [title]
File: [path]:[line]
Issue: [what's wrong]
Fix: [specific change needed]
```

Ranked: critical → serious → moderate → minor.

Read `references/audit.md` for the full protocol.

### Phase 2: Remediate (writes code)

Launch a builder/carmack subagent with the audit findings.

**Priority order** (from fixing-accessibility community skill):
1. Accessible names (critical) — every interactive control needs a name
2. Keyboard access (critical) — no div-as-button, all elements reachable via Tab
3. Focus and dialogs (critical) — trap focus in modals, restore on close
4. Semantics (high) — native elements over ARIA roles
5. Forms and errors (high) — link errors via aria-describedby, mark required
6. Announcements (medium) — aria-live for dynamic content, aria-expanded
7. Contrast and states (medium) — sufficient contrast, visible focus
8. Media and motion (low) — alt text, prefers-reduced-motion
9. Tool boundaries (critical) — minimal changes, don't rewrite unrelated code

**Rules:**
- Minimal surgical changes. Never rewrite large UI sections.
- Native HTML over ARIA workarounds. `<button>` not `<div role="button">`.
- Run vitest-axe after each fix to verify.
- Defer moderate/minor issues if timeline is tight — log them.

Read `references/remediate.md` for the full protocol.

### Phase 3: Critique (read-only, cold review)

Launch a critic subagent. **No shared context with the implementer.**

- Re-run axe scan on modified files/routes
- Keyboard-test: tab through modified components, verify focus order
- Check for regressions: new violations, broken focus, removed a11y features
- Read the git diff and axe results independently
- Verdict: **PASS** or **FAIL** with specific issues

If FAIL → back to Phase 2 with the critic's issues.

Read `references/critique.md` for the full protocol.

## Gotchas

1. **axe-core catches ~50-60% of issues** — never declare "accessible" from automated scans alone
2. **Radix/Headless UI portals render outside component tree** — jsdom-based tests miss portal content; Playwright catches it
3. **SPA focus is invisible to axe** — focus-on-navigate, skip links, dialog focus restoration need manual verification
4. **aria-sort on `<th>` is not enough** — the sort trigger must be a focusable `<button>` inside `<th>`
5. **`opacity-50` on disabled elements** drops contrast below 4.5:1 — test disabled states explicitly
6. **Dialog `onCloseAutoFocus` with `preventDefault()`** kills focus restoration — let the headless library handle it
7. **`aria-required` and HTML `required` serve different purposes** — use both
8. **Don't wrap library-managed ARIA** — libraries like cmdk manage their own; verify, don't duplicate
9. **Terminology/i18n systems** — dynamic text means labels and alt text may change; test with overridden terms
10. **Fixed headers can obscure focused elements** — WCAG 2.4.11 Focus Not Obscured (scroll-margin-top)

## WCAG 2.2 New Criteria Quick Reference

| Criterion | Level | What to check |
|-----------|-------|---------------|
| 2.4.11 Focus Not Obscured | AA | Fixed headers/footers don't cover focused element |
| 2.4.13 Focus Appearance | AAA | Focus indicator >=2px, 3:1 contrast |
| 2.5.7 Dragging Movements | AA | Drag-and-drop has non-drag alternative |
| 2.5.8 Target Size | AA | Interactive targets >= 24x24 CSS px |
| 3.2.6 Consistent Help | A | Help in same relative position across pages |
| 3.3.7 Redundant Entry | A | Don't ask for same info twice in multi-step flows |
| 3.3.8 Accessible Auth | AA | No cognitive function tests for login |

## Testing Stack

| Tool | Layer | When |
|------|-------|------|
| `eslint-plugin-jsx-a11y` | Lint | Every save |
| `vitest-axe` | Unit test | Every component test |
| `@axe-core/playwright` | E2E | CI + targeted scans |
| axe DevTools (browser) | Dev | Manual spot-checks |
| VoiceOver / NVDA | Manual | Critical flow validation |
| Keyboard-only | Manual | Every interactive change |
