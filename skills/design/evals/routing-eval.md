# /design routing eval

The oracle for the composed `/design` (backlog 119, Track A). Tests the one
claim a router-with-thesis must earn: **given a design request, does the skill
reach the right specialist on the first try?**

This is a `mode-eval` A/B run, not a directory shape. Run it; grade objectively
against the answer key below; compare new `SKILL.md` against the prior revision.

## Protocol

1. **A/B drivers.** Two fresh agents, each given ONLY one `SKILL.md` revision
   (the prior `/design` vs the candidate) plus the 15 prompts. Each returns, per
   prompt, the single skill/route it would invoke and one-line why. No other
   context; the grader must not see which revision is which.
2. **Grader (objective, no preference judge).** For each prompt, the route is a
   hit iff it is the `primary` OR one of `acceptable` below. Length/structure
   preference is explicitly NOT a grading signal.
3. **Pass condition.** Candidate hits ≥13/15 **and** candidate mis-routes
   strictly fewer than the prior revision (paired). A bigger skill that does not
   route better fails.
4. **Cadence.** One-off at Track A merge; re-run on any routing-table edit and
   after a major model release (railroading re-audit).

## Answer key (15 prompts)

| # | Prompt | primary | acceptable (also a hit) | why |
|---|---|---|---|---|
| 1 | "make this look brutalist / industrial-terminal" | `leon-brutalist` | — | aesthetic preset, unambiguous vibe |
| 2 | "I want a flat, quiet Notion/Linear editorial feel" | `leon-minimalist` | — | aesthetic preset |
| 3 | "make it feel glossy / luxe / expensive agency-tier" | `leon-soft` | `leon-taste` | aesthetic preset (premium) |
| 4 | "add page transitions between routes in Next" | `vercel-react-view-transitions` | — | engineering specialist, named API |
| 5 | "is this screen accessible? check the guidelines" | `vercel-web-design-guidelines` | `impeccable audit` | a11y/guidelines review |
| 6 | "the motion feels janky — review my animations" | `emil-review-animations` | — | motion REVIEW (not authoring) |
| 7 | "what easing + duration should this modal use?" | `emil-emil-design-eng` | — | motion AUTHORING |
| 8 | "generate a landing page from scratch for this product" | `nutlope-hallmark` | `leon-gpt-tasteskill` | greenfield generation |
| 9 | "polish these cards: exact radii, shadow, optical alignment" | `jakub-make-interfaces-feel-better` | `impeccable polish` | micro-polish, exact values |
| 10 | "create a DESIGN.md / design contract for this repo" | `/design scaffold` | `leon-stitch-skill`, `@google/design.md` (Track B) | contract/tokens role |
| 11 | "refactor these components to kill the boolean-prop explosion" | `vercel-composition-patterns` | — | react architecture |
| 12 | "this looks like generic AI slop — deslop it" | `references/anti-slop.md` | `nutlope-hallmark` slop-test, `npx impeccable detect` | anti-slop ban-core / gate |
| 13 | "build the UI to match a design image I'll provide" | `leon-images-taste-skill` | — | image-first method |
| 14 | "tighten the typography hierarchy and font pairing" | `impeccable typeset` | `jakub-make-interfaces-feel-better` | typography role |
| 15 | "give me 4 structurally different directions for this dashboard" | `/design redesign` | `/design prototype` | native variation fan, stays in core |

Notes:
- A "route" = the specialist skill or the native `/design` verb the skill says
  to invoke. Naming the preset dial value (e.g. "brutalist preset → leon-brutalist")
  counts as the route.
- Prompts deliberately span all three structures the audit found: aesthetic
  presets (1-3), engineering specialists (4,11), motion author-vs-review (6,7),
  generation vs polish vs gate (8,9,12), contract (10), method (13), and the
  native core (15). A flat menu must hit each without a phase sequence.

## Run log

**2026-06-23 — Track A merge run (A/B, blind to key).** Two fresh agents routed
the 15 prompts: one given the prior `/design` (`git show master:`), one the
candidate. Graded objectively against the key above (primary ∪ acceptable).

| Revision | Hits | Mis-routes | Misses |
|---|---|---|---|
| prior `/design` | 7/15 | 8 | 1,2,3 (presets→generic redesign), 4 (→impeccable animate), 6,7 (motion collapsed to animate), 11 (→impeccable extract), 13 (→nutlope study) |
| candidate (composed) | **15/15** | **0** | — |

Pass: candidate ≥13/15 **and** mis-routes strictly < prior (0 < 8). The prior
revision's 8 misses are exactly the specialists the flat menu surfaces — direct
evidence the compose thesis (cold = undiscoverable, not unwanted) holds at the
routing layer. The standing falsifier remains 30/60-day telemetry: do those
specialists get *invoked* in real sessions now that they are reachable.

Milestone critic (fresh-context, diff+oracle only) flagged that key row 10's
primary named the not-yet-wired `@google/design.md`; corrected to `/design
scaffold` (Track B adopts the CLI). Both revisions already routed prompt 10 to
`/design scaffold`, so the 15/15 vs 7/15 delta is unchanged by the correction.
