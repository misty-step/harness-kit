---
name: artifact
description: |
  Produce a consistently-styled, self-contained HTML report served privately over
  Tailscale. One house template (Silver Age comic-ops palette, dark/light toggle,
  and a mandatory copy-page button) so every report an agent hands the operator
  looks and behaves the same. Use when: "make an HTML artifact/report", "serve
  this over tailscale", "write up a brief/report/dashboard as a page", or any time
  you'd otherwise dump a long analysis into chat. Trigger: /artifact.
---

# artifact

The operator reads reports as HTML pages over Tailscale, not as chat walls. This
skill is the single source of truth for how those pages look and what they can do.
Hermes-independent (replaced `~/.hermes/scripts/hermes_artifact_*`).

## The contract every artifact honors

- **One house style.** The template (CSS + JS) lives inside `scripts/artifact_create.py`
  as `HOUSE_CSS`/`HOUSE_JS` — edit there, every future report inherits it. Don't
  hand-roll a divergent stylesheet.
- **A copy-page button, always.** Every report carries a header "Copy page" button
  that copies the entire rendered document to the clipboard. Baked into the template;
  injected automatically if you pass an already-authored full HTML file.
- **Self-contained.** Inline CSS/JS, no external assets — the copied HTML is a
  complete, portable page.
- **Informational, not decorative.** Tables, callouts, diagrams that carry
  information prose can't. See the aesthetic repo for the deeper design language.

## Do it

```bash
S=~/Development/harness-kit/skills/artifact/scripts
# quick: markdown in, styled page out
python3 $S/artifact_create.py --title "Weekly Ops" --slug weekly-ops \
  --tag "Field Memo" --summary "..." --body-file report.md
# rich: author a full HTML page (best for real reports); the copy button is
# injected if you forgot it. Match HOUSE_CSS class names for consistency.
python3 $S/artifact_create.py --title "The Factory" --slug factory \
  --tag "Field Memo" --html-file factory.html
```

Output is written to `~/artifacts/public/a/<slug>/index.html` and printed as a
URL: `https://serenity.tail5f5eb4.ts.net/artifacts/a/<slug>/`. Use a 1–2 word slug.
Verify with `curl -s -o /dev/null -w '%{http_code}' <url>` before handing over the link.

## Serving

`scripts/artifact_serve.py` serves `~/artifacts/public` on `127.0.0.1:8789`; the
Tailscale route `serve /artifacts -> 127.0.0.1:8789` exposes it tailnet-privately.
It runs under launchd (`~/Library/LaunchAgents/com.phaedrus.artifacts.plist`) so it
survives reboots. Zero LLM tokens (stdlib http.server).

## Extending

Add reusable block styles (cards, timelines, phase lists) to `HOUSE_CSS` when a
report needs them, so the next report can reuse the class. Keep the template one
file; don't fork per-report CSS.
