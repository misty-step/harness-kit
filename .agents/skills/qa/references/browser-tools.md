# Browser Tools for Agent-Driven QA

Detailed setup and usage patterns for each browser automation tool.

## Playwright MCP

The most mature browser automation MCP. Accessibility tree snapshots instead
of screenshots — structured text, no vision model needed.

### Setup

```json
{
  "mcpServers": {
    "playwright": {
      "command": "npx",
      "args": ["@playwright/mcp@latest"]
    }
  }
}
```

### Key tools

- `browser_navigate` — load URL, get accessibility snapshot
- `browser_click` / `browser_type` — interact by element ref
- `browser_take_screenshot` — full page or element screenshot
- `browser_snapshot` — current accessibility tree
- `browser_wait_for_network_idle` — wait for async loads
- `browser_generate_playwright_test` — convert exploration to test code

### Playwright CLI (token-efficient alternative)

Playwright CLI saves snapshots and screenshots to disk instead of streaming
them into context. **4x cheaper** on tokens than MCP for the same capability.

```bash
npx @playwright/cli@latest
```

Use CLI when embedded in a large coding workflow. Use MCP when the agent needs
a persistent interactive browser loop.

### Playwright Test Agents (v1.56+)

Three built-in agents for test lifecycle:

**Planner:** Explores the app, creates a structured test plan.
```
Input: URL + feature description
Output: Step-by-step test plan with expected outcomes
```

**Generator:** Converts a plan into executable Playwright tests with validated
selectors and assertions.
```
Input: Test plan from Planner
Output: TypeScript Playwright test file
```

**Healer:** Repairs failing tests by inspecting the live UI and patching
locators/assertions that broke due to UI changes.
```
Input: Failing test + error
Output: Patched test with updated selectors
```

### Screenshot and video

```javascript
// Screenshot
await page.screenshot({ path: 'screenshot.png', fullPage: true });

// Video recording (set in context)
const context = await browser.newContext({
  recordVideo: { dir: '/tmp/qa-videos/' }
});

// Trace (includes screenshots, DOM snapshots, network)
await context.tracing.start({ screenshots: true, snapshots: true });
// ... do things ...
await context.tracing.stop({ path: 'trace.zip' });
// View: npx playwright show-trace trace.zip
```

---

## Chrome MCP (claude-in-chrome)

Live browser control via the claude-in-chrome extension. Best for exploratory
QA with existing auth state and GIF demo capture.

### Key tools

- `tabs_context_mcp` — **always call first** to get current tab state
- `navigate` — go to URL
- `read_page` / `get_page_text` — read page content
- `find` — locate elements
- `form_input` — fill forms
- `computer` — click, type, scroll at coordinates
- `gif_creator` — record GIF walkthrough
- `read_console_messages` — check for errors
- `read_network_requests` — check for failed calls
- `javascript_tool` — run JS in page context

### GIF recording workflow

1. Navigate to starting state
2. Start GIF recording via `gif_creator`
3. Perform the walkthrough — capture extra frames before/after actions
4. Stop recording
5. Name meaningfully: `feature-name-walkthrough.gif`

### Best for

- Exploratory QA with existing login/cookies
- Demo GIF recording
- Quick visual verification
- Console/network error checking

### Limitations

- Tied to the user's Chrome instance
- Cannot run headless or in CI
- GIF output only (no video)

---

## agent-browser (Vercel Labs)

Rust CLI for AI agents. Compact text output, lowest token usage.

### Install

```bash
# Via npm
npm install -g agent-browser

# Or direct binary
curl -fsSL https://agent-browser.dev/install | sh
```

### Key features

- **Ref-based snapshots** — accessibility tree with element refs (like Playwright MCP)
- **Annotated screenshots** — screenshots with visible labels bound to element refs
- **Video recording** — WebM with start/stop control
- **Snapshot diffing** — compare before/after states
- **Pixel diffing** — compare screenshots for visual regression
- **Sessions** — persistent browser state across commands

### Usage patterns

```bash
# Navigate and get snapshot
agent-browser navigate https://localhost:3000
agent-browser snapshot

# Annotated screenshot (labels on interactive elements)
agent-browser screenshot --annotate /tmp/qa-slug/annotated.png

# Video recording
agent-browser record start /tmp/qa-slug/walkthrough.webm
# ... interact with the page ...
agent-browser record stop

# Snapshot diff
agent-browser snapshot --save before.json
# ... make changes ...
agent-browser snapshot --diff before.json
```

### Token efficiency

82% less context than Playwright MCP for equivalent tasks. Snapshots are
compact by design — built for AI agent consumption.

### Best for

- Token-conscious QA sessions
- Annotated bug report screenshots
- Video evidence with precise start/stop
- Visual regression via pixel diffing

---

## Stagehand / Browserbase

Browserbase is hosted browser infrastructure. Stagehand is the AI framework
on top, with natural-language-friendly primitives.

### Setup (MCP)

```json
{
  "mcpServers": {
    "browserbase": {
      "command": "npx",
      "args": ["@browserbasehq/mcp-server@latest"],
      "env": {
        "BROWSERBASE_API_KEY": "<key>",
        "BROWSERBASE_PROJECT_ID": "<project>"
      }
    }
  }
}
```

### Stagehand primitives

- `act("click the login button")` — natural language action
- `extract("get the price from this page")` — structured data extraction
- `observe("what's on this page?")` — semantic page understanding
- `agent` — multi-step autonomous workflow

### Hosted features

- **Session recordings** — every session recorded as video automatically
- **Live View** — watch the agent in real time
- **Session Inspector** — replay with timeline, logs, network
- **Stealth mode** — anti-bot bypass, residential proxies
- **Caching** — run AI once, cache into deterministic code for reruns

### Best for

- QA on hostile/anti-bot sites
- Shared session recordings for team review
- Human-in-the-loop via Live View
- Production agent workflows at scale

---

## Chrome DevTools MCP

Deep debugging access to a live Chrome instance via DevTools Protocol.

### Setup

```json
{
  "mcpServers": {
    "chrome-devtools": {
      "command": "npx",
      "args": ["@anthropic-ai/chrome-devtools-mcp@latest"]
    }
  }
}
```

### Key capabilities

- **Performance traces** — CPU profiling, rendering analysis
- **Network inspection** — request/response details, timing, failures
- **Console messages** — errors, warnings, log output (use `pattern` for filtering)
- **Screenshots** — via CDP (faster than standard approaches)
- **Script evaluation** — run JS in any frame context
- **Device emulation** — viewport, user agent, geolocation

### When to use

Chrome DevTools MCP is a **diagnostic tool**, not a primary QA driver. Use when:
- A page is janky and you need a perf trace
- Network calls are failing and you need request/response details
- Console errors need investigation
- You need to understand render path changes

### Privacy note

Google Chrome DevTools MCP collects usage statistics by default. Performance
tools may send trace URLs to the CrUX API. For internal/sensitive QA, review
the privacy settings.
