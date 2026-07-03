# Generative Media — image & short video (standing capability)

Cheap, authed, model-native. An agent that produces a complex plan, artifact,
doc, or design can render a legible, on-brand image — diagrams included — in
~4s for ~$0.03. Use it when it adds information; do not wrap it in a maintained
script (the model writes the call: see `model-native-product-primitives.md`).

## Auth — already wired

`GEMINI_API_KEY` is injected into the agent env from the 1Password **Agents**
vault: `op://Agents/GEMINI_API_KEY/credential` (service-account `op`,
non-interactive). If a harness lacks it:

```bash
export GEMINI_API_KEY=$(op read "op://Agents/GEMINI_API_KEY/credential")
```

Never print the value; pass it by env ref. Put the key in a request **header**
(`x-goog-api-key`), never in the URL or process args.

## Models (Gemini API — verified live 2026-06-30)

| Model ID | Per 1K image | Speed | Use for |
|---|---|---|---|
| `gemini-3.1-flash-lite-image` (NB2 Lite) | **$0.034** | ~4s | the default — drafts, volume, diagrams, contact sheets |
| `gemini-3.1-flash-image` (NB2) | $0.067 | 4–6s | the keeper image you'll actually ship |
| `gemini-3-pro-image` (NB Pro) | $0.134 | 20–60s | complex, text-heavy, professional asset production |
| `gemini-omni-flash-preview` | $0.10/sec video | 10s clips | audio-native video gen + conversational ("edit by talking") |

Image output is token-metered ($30/1M Lite, $60/1M NB2). Sizes `512`/`1K`/`2K`/`4K`
(uppercase K), default `1K`. **Batch API = 50% off**, async ≤24h — use it for bulk
(e.g. hundreds of images) so latency never blocks a loop. All output carries a
SynthID watermark.

## How to call

If your harness has a native image tool (Codex `image_gen`), use it. Otherwise
call the API directly — this exact request is verified working:

```bash
curl -s "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-lite-image:generateContent" \
  -H "x-goog-api-key: $GEMINI_API_KEY" -H 'Content-Type: application/json' \
  -d '{"contents":[{"parts":[{"text":"PROMPT"}]}],
       "generationConfig":{"responseModalities":["IMAGE"],"imageConfig":{"imageSize":"1K"}}}' \
| jq -r '.candidates[0].content.parts[]|select(.inlineData)|.inlineData.data' | base64 -d > out.png
```

The image bytes arrive base64 in `inlineData.data` (its `mimeType` tells you
png/jpeg). For editing or style/character consistency, add prior images as extra
`inlineData` parts (up to 14 reference images); multi-turn conversational edits
are supported. Confirm the current schema against the live doc rather than
trusting this snippet long-term: `ai.google.dev/gemini-api/docs/image-generation`.

## When it earns its place — INFORMATIONAL images, not decoration

Generate only where the image carries information prose, HTML layout, mermaid,
or a table can't:

- **`/shape` & "Think in HTML for plans"** — complex or contested plans where a
  labeled architecture / sequence / state diagram or system map materially
  reduces ambiguity (NB2 renders legible in-image text).
- **HTML artifact delivery (Hermes)** — a hero that encodes the artifact's
  thesis; status / incident posters with metrics baked in.
- **`/design`** — a contact sheet of N directions in one `--batch` (~$0.13) for
  the human to pick; the "match a design image" route.
- **`/document`** — figures and diagrams inside generated docs.
- **`/groom` & `/vision`** — a one-glance system map for a `VISION.md` or
  moonshot when relationships, scope, or sequencing are otherwise hard to hold.

**On-brand:** pass the Misty Step palette into the prompt — `#090a0f` bg,
`#11131d` panel, `#f4f1e8` ink, `#f0c36a` gold, `#8fb7ff` blue, `#81d89a` green.

## Caveats

- **SynthID watermark** on every output — fine internal; disclose for external;
  never pass off as not-AI.
- **Generated diagrams can be subtly wrong** — use for overview/vibe; keep
  mermaid or code for precise call-graphs the reader will trust.
- **~4s latency** per image — fine for one-off; `--batch` for bulk.
- **Decoration is clutter.** If the image carries nothing the prose, HTML,
  mermaid, or table can't, skip it.
