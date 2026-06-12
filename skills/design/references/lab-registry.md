# The Lab Registry

The operator-confirmed format for **iterative, multi-issue design
prototyping** — anything bigger than a one-shot variation fan. Where the
variation fan answers "pick one of these," the registry runs *rounds* of
verdicts across many named issues without losing track of anything.

Origin: web-presence LAB 006 (2026-06). The operator's words: "this is
exactly what I want almost all the time whenever we're doing any kind of
design prototyping."

## The registry contract

- **One section per named issue.** An issue is a problem the operator
  stated ("the footer is cramped," "the bio is wrong"), not a component.
- **No fewer than six distinct options per section.** Distinct means
  structurally different, not palette swaps. Always include the current
  shipped state as an explicit baseline in round 1.
- **Sections never die.** Across rounds: kill weak options, mutate strong
  ones, seed genuinely unexplored directions, and refill toward six.
- **Carry the winner visibly.** Each section header shows its round
  number and running winner ("round 2 · winner: COMP-2 the card").
- Option IDs are stable and never reused (`COMP-7` after `COMP-6` even if
  1–6 are dead). Verdict notes reference IDs.

## The viewer contract

A long scrolling page of strips hides viewport-dependent problems. The
lab is a **paged viewer**:

- **Every option renders as its own full-viewport page** — real chrome,
  real content, the real design system loaded from its shipped package
  (CDN pin or installed dep), at true `100dvh` proportions.
- **The viewport is adjustable**: preset sizes (fit / 1440×900 / 1280×800
  / 1024×768 / tablet 768×1024 / phone 390×844) plus custom width ×
  height inputs. Sizes larger than the window scale down (CSS transform)
  with a readout showing real dimensions and scale percentage.
- The sidebar is the registry (sections, rounds, winners); arrow keys
  step through options; selection and viewport size persist in
  localStorage.
- Interactions stay live inside the frame: mode toggles, nav indicators,
  state animations, working links.

## File layout (zero-build, self-contained)

```
explorations/lab-NNN/
  index.html   # viewer shell: top bar (viewport controls), sidebar, iframe
  app.js       # SECTIONS manifest (ids, labels, round status, notes) + shell logic
  styles.css   # shell styles only
  frame.html   # skeleton: fonts, design-system CSS, icon sprite, <div id="mount">
  frame.js     # option builders + SPECS map; renders by location.hash
  frame.css    # styles the option screens need, nothing the shell needs
```

Options live as small **builder functions** composing shared parts
(header, nav, footer, content blocks), keyed in a `SPECS` map by ID. This
is what makes round mechanics cheap: killing is deleting a key, mutating
is editing a builder, seeding is adding one. The iframe selects via
`frame.html#ID`; the shell switches options by setting the frame's hash.

Gotchas learned the hard way:

- Demo links inside frames must not navigate the hash (use `href="#0"` and
  `preventDefault`; ignore unknown hashes in the renderer).
- Re-place nav indicators after render, on `document.fonts.ready`, and on
  resize — fallback-font metrics lie.
- Replay entrance animations by replacing the view node, not mutating it.
- **Stale-cache bite:** `python3 -m http.server` sends no `Cache-Control`,
  so Chrome heuristically caches lab JS/CSS (~10% of file age) and a new
  round can silently run the previous round's code while the server serves
  the new one. Serve with a no-cache handler AND version the asset URLs
  (`frame.js?v=N`, bumped each round) — old cache entries ignore new
  headers, only a changed URL is bulletproof. Verify which round actually
  executes (e.g., log a round marker) before trusting a sweep.
- Sims that depict ongoing work must open mid-work: pre-run the state a
  few hundred frames at init (offscreen 2×2 context works), or the first
  impression is an empty scene.
- A hidden tab reports a 0×0 viewport; canvases init at degenerate sizes
  and DOM sweeps pass anyway. Assert `innerWidth > 0` in programmatic
  sweeps and screenshot from a visible tab.

## Round mechanics

1. Operator gives verdicts per section (winner / kills / mutations /
   reactions), usually voice-dump style — mine every sentence.
2. Update the manifest in place: round status lines, killed options
   removed, mutations and new seeds added, winner first in the list.
3. Anything the operator phrased as a definite fix (not a design
   question) ships to production immediately, outside the lab.
4. Record verdicts in memory/notes so locks survive the session.

## Quality bar

Same as all lab work: real content (no lorem ipsum, no invented claims),
verified rendering at multiple viewport sizes, zero console errors,
interactions exercised before presenting. Copy candidates render in the
real chrome, never as bare text strips — type only reads in context.
