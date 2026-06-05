---
name: browser
description: |
  Pick browser automation for web/Electron: CI E2E, scripted flows, scraping,
  visual regression, exploratory QA, persona walks, monitoring, or browser
  agents. Deterministic Playwright first; harden exploratory findings into
  repeatable tests. Use for "automate the browser", "test this web app",
  "test this electron app", "Playwright or Stagehand", "scrape this site",
  "browser agent", "visual regression", "E2E tests".
  Trigger: /browser.
argument-hint: "[describe the task]"
---

# /browser

Browser automation is a **testing pyramid**, not a single tool. Pick the
layer first, then the tool for that layer.

## The Pyramid

| Layer | Purpose | Tooling |
|-------|---------|---------|
| **4. Continuous QA** | scheduled agents, autonomous bug filing, synthetic monitoring against staging/prod | Custom Browser Use / Stagehand loops, bugAgent, QA.tech, Mabl, supaguard |
| **3. Exploratory / Persona-driven** | cold-start exploration with a persona, charters, SBTM/PROOF reports, UX-gap discovery | Browser Use, Stagehand `agent()`, agent-browser, custom persona harnesses |
| **2. Hybrid / AI-assisted** | Playwright body + AI for fragile steps, self-healing, AI-authored tests committed to repo | Playwright 1.56+ Planner/Generator/Healer, Stagehand atomic primitives, QA Wolf / Octomind (generate code), visual regression |
| **1. Deterministic Playwright E2E** | critical user journeys, CI gate, 99%+ reliable, regression floor | Raw Playwright code, Playwright Test, fixtures, `getByRole` |

**Findings flow down, not up.** The exploratory layer is a *discovery
instrument*. When an agent persona finds a real bug or UX gap, harden
the repro into a Layer 1 or Layer 2 test. Do not use runtime agents as
release gates — they're stochastic by construction.

**Generate-once vs run-every-time.** QA Wolf / Octomind / Playwright's
Planner use AI at **authoring time** and produce deterministic Playwright
code committed to the repo. Stagehand atomic / Browser Use use AI at
**runtime**, paying LLM cost on every execution. Authoring-time AI is
the production sweet spot; runtime AI is for fragile surfaces, one-off
flows, and exploration.

For the full pyramid rationale, persona patterns, SBTM/PROOF reporting,
and commercial landscape, read `references/pyramid.md`.

## The Four Tool Layers

Orthogonal to the pyramid, every browser-automation stack assembles four
layers. Pick one per layer.

1. **Driver** — what speaks to the browser.
   Playwright (universal — default), Puppeteer (Chrome-only — stay on
   it if you're on it), Selenium (legacy, multi-language — only for
   existing Java/C# investment).

2. **Mode** — who drives actions.
   Scripted (selectors; cheap; breaks on UI change); hybrid (code + LLM-
   resolved step per action — Stagehand's `act("click login")`);
   full agent (LLM plans and drives — Browser Use).

3. **Wrapper** — the library or MCP the agent codes against.
   Raw Playwright / Puppeteer / Selenium (zero LLM cost); **Stagehand**
   (four primitives on top of Playwright/Puppeteer/CDP, TS-first);
   **Browser Use** (full-agent wrapper on Playwright, Python-first);
   **agent-browser** (Vercel Rust CLI, compact output, lowest tokens);
   **Playwright MCP** (Microsoft's official, ax-tree snapshots).

4. **Infrastructure** — where the browser actually runs.
   Local Chromium (free, private, CI-friendly); **Browserbase** (hosted
   with stealth, proxies, auto-CAPTCHA, session recording/inspector);
   **Real Chrome attached** (Claude-in-Chrome extension, or
   `@playwright-repl/mcp` Dramaturg — not a CI tool by design).

Most production stacks: **Playwright driver → chosen mode → wrapper →
local or Browserbase**.

For per-tool setup and capability detail, read `references/tools.md`.

## The Three Playwrights

"Playwright" means three different things with very different cost
profiles. Don't conflate them.

| Surface | Tokens per action | What's happening |
|---------|-------------------|------------------|
| **Raw Playwright (code)** | 0 | You write TS/Python. No LLM in the loop. |
| **Playwright CLI** (`@playwright/cli`) | ~0 | Snapshots/screenshots write to disk; agent reads files on demand. |
| **Playwright MCP** (`@playwright/mcp`) | 200–400 per snapshot, builds up across a session | Accessibility tree streams into context every call. The "heavy" one. |

Raw Playwright is the most token-efficient option in the entire space.
Playwright MCP is the "heavy" one people complain about. Reach
accordingly.

## Decision Matrix (by pyramid layer)

| Layer | Task | Stack |
|-------|------|-------|
| 1 | CI E2E on known flows | Playwright Test, raw code, local Chromium |
| 1 | Electron app E2E | Playwright `_electron` + `electron-playwright-helpers` |
| 2 | AI-authored tests committed to repo | Playwright Planner/Generator/Healer (1.56+), or QA Wolf / Octomind (commercial) |
| 2 | Self-healing flaky selectors | Stagehand atomic primitives on the fragile step only |
| 2 | Visual regression | Playwright `toHaveScreenshot()`, Applitools, or agent-browser pixel diff |
| 2 | Agent debugging a live app | Playwright MCP + Chrome DevTools MCP |
| 3 | Structured extraction from fragile UI (TS) | Stagehand `extract()` with Zod schema |
| 3 | Persona-driven exploratory QA | Browser Use or Stagehand `agent()` with persona prompt, SBTM/PROOF report |
| 3 | Token-conscious exploration loop | agent-browser CLI (compact snapshots by design) |
| 3 | Exploratory QA in my logged-in Chrome | Claude-in-Chrome MCP or `@playwright-repl/mcp` (Dramaturg) |
| 4 | Scheduled staging/prod QA with bug filing | Browser Use or Stagehand in CI/cron + bug-filing integration |
| 4 | Synthetic monitoring with AI classification | supaguard, or custom Playwright + Claude classifier |
| any | Anti-bot site, scale, session replay | Any of the above **+ Browserbase** |

For full stack walk-throughs per scenario, read `references/stacks.md`.

## Load-Bearing Gotchas

- **Spectron is deprecated.** Electron = Playwright `_electron` +
  `electron-playwright-helpers`. See `references/electron.md`.
- **Stagehand's `agent()` is the expensive failure mode.** One-shot goals
  can balloon to 500k+ tokens. Atomic `act`/`extract`/`observe` runs at
  ~7k tokens/step with caching. In the pyramid, `agent()` belongs in
  Layer 3 (exploration), not Layer 2 (production flows).
- **Stagehand, Browser Use, and Browserbase stack — they don't compete.**
  Stagehand wraps Playwright; Browser Use wraps Playwright via CDP;
  Browserbase is infrastructure under any of them. "Stagehand vs
  Browserbase" is a category error.
- **Browser Use is autonomy-optimized, not token-optimized.** It burns
  2–5s per action and $0.02–0.30 per task on planning overhead. Worth
  it for genuinely open-ended exploration; wasteful if you already know
  the steps.
- **Runtime agents are not release gates.** They're stochastic. Use them
  for discovery (Layer 3/4); harden findings into deterministic tests
  at Layer 1/2. "Agentic runtime as CI gate" is the pattern reddit
  practitioners warn against.
- **Claude-in-Chrome and Dramaturg are not CI tools.** They attach to
  your *real* Chrome — auth, cookies, open tabs persist. That's the
  feature.
- **"AI beats selectors" is situational.** On stable UIs, a CSS selector
  is faster, cheaper, and 100% deterministic. LLM-resolved actions cost
  $0.002–0.02 each. Reach for AI only when selectors genuinely break
  often (15–25%/month on fast-moving UIs) or the UI is ambiguous.
- **Puppeteer has no Firefox or WebKit.** New project = Playwright.
- **Language ecosystem is a real constraint.** Stagehand is TS-first
  (Python bindings lag). Browser Use is Python-first. Choose to match
  your codebase.
- **Commercial SaaS QA platforms** (QA.tech, Mabl, Rainforest, Momentic)
  compress the pyramid into one vendor. Convenient, but lock you in
  and often can't export tests as Playwright code. Octomind and QA Wolf
  explicitly output deterministic Playwright — prefer those if you
  want escape-hatch ownership.
- **Browserbase's value is anti-bot + observability, not just hosting.**
  No bot detection + no session replay need = local Chromium is cheaper.

## Existing Harness Kit Integrations

- **Vercel `agent-browser`** registered in `registry.yaml` (pin
  `4cc6ca40`) as `vercel-agent-browser`. Layer 3 default for
  token-conscious exploration loops.
- **Vercel `dogfood`** (`vercel-dogfood`) pairs with `agent-browser` for
  "dogfood your own product" — repro-first bug documentation inside a
  running app. Layer 3 pattern.
- **`/qa`** has a QA-scoped setup reference at
  `skills/qa/references/browser-tools.md` for the tools above used in
  QA evidence capture. This skill is the broader selection judgment;
  `/qa` is the workflow that consumes the tools.

## Output

When routing a task, state:
1. Which **pyramid layer** the task belongs to.
2. Which **tool layers** (driver/mode/wrapper/infra) you're picking.
3. The stack — one line per tool layer.
4. The gotcha most likely to bite.
5. Cost estimate if any runtime-LLM tool is in the stack.

## Verification

Semantic waiver: this router selects external browser tooling per target app
and cannot be proven by a repo-local static fixture today. Validate source
shape with `python3 scripts/check-frontmatter.py`; workflow correctness is
proven by the consuming `/qa` or app-specific browser test artifact.
