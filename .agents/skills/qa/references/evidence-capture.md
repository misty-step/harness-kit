# Evidence Capture Patterns

Cross-tool patterns for capturing QA evidence: screenshots, GIFs, videos, and
terminal output.

## Directory Convention

```bash
mkdir -p /tmp/qa-{slug}
# All evidence for a QA session goes here
# slug = feature name or PR number
```

## Screenshots

### Playwright MCP
```
browser_take_screenshot  →  saves to specified path
```
Full-page screenshots by default. Element screenshots via selector.

### Chrome MCP
Navigate to the state, then use `upload_image` or `gif_creator` for a
single-frame capture.

### agent-browser
```bash
# Standard screenshot
agent-browser screenshot /tmp/qa-{slug}/page.png

# Annotated screenshot (labels on interactive elements)
agent-browser screenshot --annotate /tmp/qa-{slug}/annotated.png
```
Annotated screenshots are the best format for bug reports — visible labels
map directly to actionable element refs.

### Chrome DevTools MCP
CDP-based screenshots are faster than standard approaches:
```
Take a screenshot of the current page state
```

## GIF Recordings

### Chrome MCP (claude-in-chrome)
```
1. gif_creator — start recording
2. Perform walkthrough (capture extra frames before/after actions)
3. gif_creator — stop recording
4. Name: feature-name-walkthrough.gif
```
This is the fastest path to inline-renderable GIFs for PRs.

### agent-browser → ffmpeg
```bash
# Record as WebM
agent-browser record start /tmp/qa-{slug}/walkthrough.webm
# ... interact with the app ...
agent-browser record stop

# Convert to GIF (GitHub renders GIFs inline, not WebM)
ffmpeg -y -i /tmp/qa-{slug}/walkthrough.webm \
  -vf "fps=8,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
  -loop 0 /tmp/qa-{slug}/walkthrough.gif
```

### Playwright trace → screenshots
```bash
# Traces include per-action screenshots
npx playwright show-trace trace.zip
# Export screenshots from the trace viewer
```

## Video Recordings

### agent-browser
```bash
agent-browser record start /tmp/qa-{slug}/session.webm
# ... full QA session ...
agent-browser record stop
```
Add `--pause 500` between actions for human-readable playback.

### Browserbase
Every session is automatically recorded. Access via:
- Session Inspector (web UI)
- API: download recording by session ID
- Live View for real-time observation

### Playwright
```javascript
const context = await browser.newContext({
  recordVideo: { dir: '/tmp/qa-{slug}/' }
});
// Video saved on context.close()
await context.close();
```

## CLI / Terminal Evidence

### Script capture
```bash
# Record terminal session
script -q /tmp/qa-{slug}/terminal-session.txt \
  your-cli command --args

# Or with timing for playback
script -t 2>/tmp/qa-{slug}/timing.txt /tmp/qa-{slug}/session.txt
```

### Asciinema (richer terminal recording)
```bash
asciinema rec /tmp/qa-{slug}/session.cast
# Convert to GIF:
# pip install agg  (asciinema gif generator)
agg /tmp/qa-{slug}/session.cast /tmp/qa-{slug}/terminal.gif
```

### Simple output capture
```bash
your-cli command --args > /tmp/qa-{slug}/output.txt 2>&1
echo "Exit code: $?" >> /tmp/qa-{slug}/output.txt
```

## API Evidence

```bash
# Capture response with headers and status
curl -s -w "\n\nHTTP Status: %{http_code}\nTime: %{time_total}s\n" \
  http://localhost:3000/api/endpoint | tee /tmp/qa-{slug}/api-response.json

# POST with body
curl -s -X POST http://localhost:3000/api/endpoint \
  -H "Content-Type: application/json" \
  -d '{"key": "value"}' | jq . > /tmp/qa-{slug}/api-post-response.json
```

## Evidence Naming Convention

```
/tmp/qa-{slug}/
├── 01-dashboard-home.png           # Numbered for sequence
├── 02-create-form.png
├── 03-submit-success.png
├── walkthrough.gif                  # GIF for PR embedding
├── walkthrough.webm                 # Source video (higher quality)
├── console-errors.txt               # Console output if errors found
├── network-failures.txt             # Failed network requests
├── api-response.json                # API evidence
└── cli-output.txt                   # CLI evidence
```

Number screenshots sequentially when they document a flow. Use descriptive
names that indicate what the screenshot shows.

## Uploading Evidence

Use `/demo upload` to attach evidence to PRs via draft GitHub releases.
See the `/demo` skill for the full upload protocol.
