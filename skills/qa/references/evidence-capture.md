# Evidence Capture Patterns

Cross-tool patterns for capturing QA evidence: screenshots, GIFs, videos, and
terminal output.

## Directory Convention

```bash
EVIDENCE_DIR="$(cargo run --quiet --locked -p harness-kit-checks -- evidence create 2>/dev/null || printf '.evidence/manual/%s/\n' "$(date -u +%Y-%m-%d)")"
mkdir -p "$EVIDENCE_DIR"
# All evidence for a QA session goes here: .evidence/<branch>/<date>/
```

The canonical storage surface is git-native and offline-capable:
`.evidence/<branch>/<date>/`. Binary screenshots and recordings are tracked
by `.gitattributes` via Git LFS when LFS is configured; without an LFS server,
fresh clones still retain pointer files. Use temporary directories only outside
a git repo or when the target repo has no evidence convention.

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
agent-browser screenshot "$EVIDENCE_DIR/page.png"

# Annotated screenshot (labels on interactive elements)
agent-browser screenshot --annotate "$EVIDENCE_DIR/annotated.png"
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
agent-browser record start "$EVIDENCE_DIR/walkthrough.webm"
# ... interact with the app ...
agent-browser record stop

# Convert to GIF (GitHub renders GIFs inline, not WebM)
ffmpeg -y -i "$EVIDENCE_DIR/walkthrough.webm" \
  -vf "fps=8,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
  -loop 0 "$EVIDENCE_DIR/walkthrough.gif"
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
agent-browser record start "$EVIDENCE_DIR/session.webm"
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
  recordVideo: { dir: process.env.EVIDENCE_DIR }
});
// Video saved on context.close()
await context.close();
```

## CLI / Terminal Evidence

### Script capture
```bash
# Record terminal session
script -q "$EVIDENCE_DIR/terminal-session.txt" \
  your-cli command --args

# Or with timing for playback
script -t 2>"$EVIDENCE_DIR/timing.txt" "$EVIDENCE_DIR/session.txt"
```

### Asciinema (richer terminal recording)
```bash
asciinema rec "$EVIDENCE_DIR/session.cast"
# Convert to GIF:
# pip install agg  (asciinema gif generator)
agg "$EVIDENCE_DIR/session.cast" "$EVIDENCE_DIR/terminal.gif"
```

### Simple output capture
```bash
your-cli command --args > "$EVIDENCE_DIR/output.txt" 2>&1
echo "Exit code: $?" >> "$EVIDENCE_DIR/output.txt"
```

## API Evidence

```bash
# Capture response with headers and status
curl -s -w "\n\nHTTP Status: %{http_code}\nTime: %{time_total}s\n" \
  http://localhost:3000/api/endpoint | tee "$EVIDENCE_DIR/api-response.json"

# POST with body
curl -s -X POST http://localhost:3000/api/endpoint \
  -H "Content-Type: application/json" \
  -d '{"key": "value"}' | jq . > "$EVIDENCE_DIR/api-post-response.json"
```

### emulate.dev API emulator evidence

When QA uses `emulate.dev` for supported third-party APIs, capture the emulator
as part of the proof, not just the app response:

```bash
npx --yes emulate list > "$EVIDENCE_DIR/emulate-services.txt"
npx --yes emulate start --service github,stripe --seed emulate.config.yaml \
  > "$EVIDENCE_DIR/emulate-start.log" 2>&1 &
echo $! > "$EVIDENCE_DIR/emulate.pid"
# Then capture request/response pairs against the app or emulator as above.
```

Record the exact services, package/version or command output, ports/base URLs,
seed YAML/JSON path when non-secret, reset behavior, and teardown. Usage docs:
https://emulate.dev/docs.

## Evidence Naming Convention

```
.evidence/<branch>/<date>/
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

Commit `.evidence/<branch>/<date>/` on the feature branch. Mirror images to
the PR (e.g. `gh` upload or a comment) only when inline URLs are needed.
