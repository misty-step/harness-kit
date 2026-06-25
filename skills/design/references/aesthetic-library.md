# Aesthetic Library

Six operator-endorsed aesthetic directions. Each came out of a `/design`
exploration (the product was sploot, a private meme library with text→image
semantic search) and was explicitly picked by the operator as a vibe to keep.
Pull from one when a `/design` request matches its feel — they are starting
points for divergence, not a default to one-shot. Treat them like the
references in `taste-layer.md`: a specific lens, not a template to clone. DNA
travels (structure, type roles, the one distinctive move); the exact content and
palette should re-derive from the new product.

Each entry links a runnable example in `aesthetic-library/<slug>.html` (full,
self-contained, no build step — open it in a browser) and an optional
`aesthetic-library/<slug>.png` screenshot the lead adds.

These are aesthetic *presets* in the sense of the SKILL.md dials, parallel to the
vendored `leon-*` directions — but specimen-grade rather than skill-grade: read
the example to absorb the move, then build it bespoke for the new surface under
the anti-slop gate.

---

## Neo-brutalist

One-line essence: a page that admits it is a database and shows the machine
working — engineering-doc paper, hazard color, hard black borders.

- **STRUCTURE / layout** — full-bleed bordered sections stacked vertically, each
  with a `// fig.0N` index label; a scrolling ticker strip up top; a sticky
  header; a two-column hero (headline + a stacked stat sidebar). The signature
  block is a "console": a labeled title bar with LED squares over a live
  retrieval pipeline laid out as a 5-cell grid (input → tokenize → embed →
  cosine → return) feeding a meme grid that dims, highlights, and badges cells
  by match.
- **TYPE** — Archivo Black for display (uppercase, line-height ~.82, tight
  negative tracking), Space Grotesk for body, Space Mono for all labels, tags,
  and readouts (uppercase, wide letter-spacing).
- **COLOR** — ink `#0a0a0a` on unbleached-paper `#f3efe4`, with a full hazard
  set: electric blue `#1f4cff`, hot magenta `#ff2d9b`, acid yellow `#ffe600`,
  blood orange `#ff5a1f`, lime `#9cff2e`, cyan `#00e5d4`. Faint engineering grid
  on the background.
- **MOTION / interaction** — hard offset shadows (`8px 8px 0` ink) that snap on
  hover/active (button translates into its own shadow); a blinking status dot; a
  vector bar that animates on query; matched cells scale and get a lime ring.
  All motion is interaction- or status-tied, gated behind `prefers-reduced-motion`.
- **ASSET language** — inline blocky SVG glyph "memes" (geometric face/shape
  primitives), each framed and captioned; content shown as live system state
  (vector counts, sim scores, latency).
- **THE ONE distinctive move** — the search surface *is* an exposed retrieval
  pipeline: the UI narrates tokenize → embed → cosine → return as visible stages.
- **WHEN TO USE** — developer tools, data/ML products, anything that wins by
  looking honest and mechanical; high VARIANCE, high DENSITY, confident and loud.
- example: `aesthetic-library/brutalist.html` (screenshot: `brutalist.png` — lead adds)

---

## Terminal-TUI amber-glow

One-line essence: an fzf fuzzy-finder rendered as a phosphor terminal — amber on
warm black, keyboard-first, deadpan.

- **STRUCTURE / layout** — a single bordered "CRT" container with labeled panels
  (titles notched into the top border). ASCII-art wordmark banner; a segmented
  status bar with a live clock; a two-pane grid: left = recent-query history +
  stats, right = the fzf finder (`find>` prompt with a hand-drawn block caret, a
  scrolling result listbox of monospace rows, and a preview footer). A keybind
  footer (`type / ↑↓ / enter / esc`) anchors the bottom.
- **TYPE** — JetBrains Mono throughout (with IBM Plex Mono / ui-monospace
  fallbacks); ~14px, ligatures off. Everything is one monospace voice.
- **COLOR** — warm near-black `#0a0705`, primary phosphor amber `#ffb000`,
  highlight amber `#ffd27f`, tiers of dimmed amber-brown for borders and
  secondary text, one lone status-ok green `#6fcf5b`, alert vermilion `#ff6b4a`.
  Selection inverts (amber bg, dark fg).
- **MOTION / interaction** — a blinking block caret; a very subtle scanline
  overlay; selected row inverts to a solid amber bar; CTA buttons get an amber
  glow on hover. Keyboard-driven: arrows / Ctrl-N/P move, Enter opens, Esc
  clears, `/` focuses. Reduced-motion kills the caret blink and dims scanlines.
- **ASSET language** — deterministic block-glyph thumbnails (` ░▒▓█`) hashed
  from each caption; results as terminal rows with fzf-style match-character
  highlighting and a `▰▱` score spark.
- **THE ONE distinctive move** — real fzf mechanics: a subsequence fuzzy scorer
  with contiguous-run and word-boundary bonuses, live-highlighted matched
  characters, score sparks — the search *behaves* like a terminal finder.
- **WHEN TO USE** — CLI-adjacent products, dev/power-user tools, anything for a
  keyboard-first audience; maximum DENSITY, restrained MOTION, dry tone.
- example: `aesthetic-library/tui.html` (screenshot: `tui.png` — lead adds)

---

## Soft-luxe

One-line essence: a quiet museum — warm paper, serif display, one specimen
revealed per query, like a single framed plate on a gallery wall.

- **STRUCTURE / layout** — centered, generous, single-column and airy. A thin
  masthead (lowercase serif wordmark + a hairline "sign in"); a centered stage
  with eyebrow / serif headline / short subhead; a pill search field; quiet
  pill prompt chips. Below, one **specimen** plate animates in: a framed art
  card with a hairline inner mat, a museum placard (provenance line + italic
  title) and a match score. A hairline-ruled footer with tabular counts.
- **TYPE** — Fraunces (optical serif) for display and titles, used light (300)
  with italics for emphasis; Geist sans for body and UI, weights 300–400. Wide
  uppercase letter-spacing on eyebrows/metadata.
- **COLOR** — gallery neutrals: warm paper `#f4f1ea` (radial-graded to
  `#ebe6dc`), ink `#1b1a17`, soft ink `#54514a`, hairlines `#d8d2c5`, and **one**
  quiet accent used once: aged bronze `#7a5230`. No second hue.
- **MOTION / interaction** — slow, eased (`cubic-bezier(.22,.61,.36,1)`,
  0.3–0.9s). Field lifts on focus; the specimen fades/translates/scales in; the
  match score counts up to a confident 96–98%. Soft layered shadows. Fully
  reduced-motion aware.
- **ASSET language** — hand-drawn line-and-fill SVG "specimens" in the muted
  palette (a quiet plate, not a loud glyph), each with provenance metadata
  ("specimen no. 04 408 · saved oct 2021").
- **THE ONE distinctive move** — one query returns **one specimen**, framed and
  placarded like a gallery object — restraint as the whole product gesture.
- **WHEN TO USE** — premium / editorial / brand-forward surfaces, marketing
  pages, anything that should feel expensive and calm; low DENSITY, slow MOTION,
  one decisive object per view.
- example: `aesthetic-library/luxe.html` (screenshot: `luxe.png` — lead adds)

---

## Memphis postmodern

One-line essence: 80s postmodern playfulness made structural — primary-color
geometric chips, hard black outlines, and terrazzo, where the shapes *are* the
grid.

- **STRUCTURE / layout** — a centered frame of hard-bordered, offset-shadow
  blocks on a generated terrazzo background. A masthead bar (ink logo + tag +
  red sign-in). The hero is a strict 3-column grid: a left rail of colored
  geometric cells, a center headline, a right rail of colored cells — the shapes
  literally build the layout. A framed yellow search strip with full-width chip
  buttons; a result stage where the found meme drops in as a slightly-rotated
  framed card next to a stacked colored meta column (match / position / time).
- **TYPE** — Archivo Black for display/logo/labels (lowercase, tight tracking),
  Space Grotesk 500 for body. Headline mixes colored words and a yellow
  highlight stripe rotated `-1.5deg`.
- **COLOR** — cream `#f3ecd9`, ink `#16130f`, and a saturated primary-ish set:
  red `#e23b2e`, blue `#2c52d8`, yellow `#f4c531`, teal `#1ba98c`, pink
  `#ec6fa6`. 4px black borders everywhere; chunky `10–14px` offset block shadows.
- **MOTION / interaction** — buttons swap fill color and nudge on active; the
  found card eases up and settles at a `-1.4deg` rotation; the match % counts up.
  All transitions disabled under reduced-motion.
- **ASSET language** — hard-edged geometric SVG meme glyphs on a yellow plate
  (cats as triangle bodies + circle heads, etc.); a JS-generated terrazzo of
  ~120 scattered squares/triangles/circles/bars as real background texture (not
  blobs).
- **THE ONE distinctive move** — geometry is load-bearing: colored shape-cells
  build the actual grid columns, so the decoration and the layout are the same
  thing.
- **WHEN TO USE** — playful, creative, consumer brands; events, kids/education,
  anything that should feel energetic and fun without going soft; high VARIANCE,
  medium MOTION, bold and warm.
- example: `aesthetic-library/memphis.html` (screenshot: `memphis.png` — lead adds)

---

## Web-1.0 anti-design

One-line essence: a knowing 1999 GeoCities homepage — teal tile, Comic Sans,
marquee, hit counter — wrapped around one deliberately modern search box.

- **STRUCTURE / layout** — a fixed 760px centered `ridge`-bordered "page" on a
  tiled teal background. A loud navy/yellow header band with a magenta Comic
  Sans wordmark; a self-aware scrolling marquee; a `display:table` two-column
  layout: left sidebar (nav, an ironic webring box, a live odometer visitor
  counter, badge list), right main column (a CSS barber-pole "under
  construction" banner, the search, an "about" blurb). A yellow ridge CTA band
  and a navy footer of `88×31` badges.
- **TYPE** — Times New Roman body, Comic Sans MS for headers/CTA, Courier New
  for chrome/labels — *except* the search zone, which switches to a clean
  system-ui stack.
- **COLOR** — period web-safe chaos: teal `#008080`, navy `#000080`, yellow
  `#ffff00`, magenta `#ff00ff`, cyan `#00ffff`, lime `#00ff00`, silver `#c0c0c0`,
  classic link blue/purple/red — *except* the search zone, which is calm
  modern slate/blue (`#111827`, `#2563eb`, neutral grays).
- **MOTION / interaction** — a CSS marquee (pauses on hover, goes static under
  reduced-motion); a knowing `<blink>` on one NEW! tag; a blinking
  construction-pylon dot; bevel buttons that press in on `:active`. The visitor
  counter and ironic webring actually increment/persist.
- **ASSET language** — ASCII-emoticon "memes" (`=^..^=`, `( o_o)>`) on colored
  swatches; the results grid, by contrast, is clean modern cards with a green
  "TOP MATCH" hit outline.
- **THE ONE distinctive move** — a deliberate aesthetic *break*: everything is
  gloriously 1999 except the search box and results, which are conspicuously
  from the future — the contrast *is* the pitch ("the one good thing").
- **WHEN TO USE** — playful nostalgia, indie/personal projects, launch jokes, or
  any product where one modern capability should be framed against deliberate
  retro chrome; high VARIANCE, used with self-awareness, never by accident.
- example: `aesthetic-library/web1.html` (screenshot: `web1.png` — lead adds)

---

## Instrument-panel

One-line essence: a live signal-acquisition console at night — phosphor on
black, a radar scope, telemetry readouts; search = locking onto a target.

- **STRUCTURE / layout** — one dark rounded "console" panel. A monospace top
  status bar (brand `// ACQ-1`, LED + mode, corpus/index/dim stats). A two-column
  hero: left = designation label, headline, lede, and a hairline dashed **spec
  table**; right = a **scope** (CSS graticule grid + crosshair, scattered
  signal blips). An acquisition form with a `TGT>` prefix and an ACQUIRE button;
  a 4-cell **telemetry strip** (match confidence / latency / vector distance /
  lock status); a monospace **target log** table that ranks signatures.
- **TYPE** — IBM Plex Mono for all numerics/labels/headline, IBM Plex Sans for
  body. Tabular-nums on every readout; tight uppercase mono designations.
- **COLOR** — instrument dark `#070b0d` / `#0d1417`, with phosphor green
  `#3ef0a0` as the live/lock signal, amber `#ffb454` for standby/caution, cyan
  `#46d6ff` as secondary trace, red `#ff5d5d` for no-lock. Green CRT glow on
  active elements.
- **MOTION / interaction** — a pulsing status LED; a radar **sweep** that runs on
  query; a lock **reticle** that animates across the scope and snaps onto the
  winning blip; latency/confidence counters tick up during a ~900ms "scan" then
  resolve to stable numerics. Reduced-motion jumps straight to the resolved
  state.
- **ASSET language** — ASCII-face glyph "memes" (`=^=`, `o_o`, `x_x`) as colored
  swatches and as plotted blips on the scope; everything else is live
  technical telemetry (confidence %, cosine distance, lock status).
- **THE ONE distinctive move** — the search is reframed as **signal
  acquisition**: a scope scans, a reticle hunts, and a match is a "lock" with a
  confidence/distance readout — the radar metaphor carries the whole interaction.
- **WHEN TO USE** — operational dashboards, monitoring/observability, security or
  realtime tools, anything that should read as a powered live instrument; high
  DENSITY, MOTION tied strictly to state changes, dark and serious.
- example: `aesthetic-library/cockpit.html` (screenshot: `cockpit.png` — lead adds)
