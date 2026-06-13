---
name: showcase
description: |
  Turn a working repo into credible external-facing proof: demoability audit,
  deterministic demo path, marketing site, case study, screenshots, demo video,
  launch copy, and consulting portfolio assets. Use when: "productize this",
  "make this demoable", "make this polished", "make a marketing site",
  "show this off", "demo video", "case study", "portfolio piece",
  "consulting asset", "launch page", "sales demo". Trigger: /showcase,
  /productize, /demoability.
argument-hint: "[audit|plan|site|video|case-study|polish] <repo-or-product>"
---

# /showcase

Make a real product showable without laundering weak truth.

This skill owns the gap between "it works locally" and "a serious buyer,
client, collaborator, or prospect can understand and trust it." It is not a
marketing checklist. The primitive is **showable proof**: a deterministic demo
surface, evidence-backed story, polished product moments, and assets that make
the operator look credible.

## Route

| Need | Load |
|---|---|
| Decide if the repo is demoable yet | `references/demoability-audit.md` |
| Position for consulting or portfolio use | `references/consulting-positioning.md` |
| Plan a marketing site or launch page | `references/launch-page-contract.md` |
| Script or verify a demo video | `references/demo-video-contract.md` |
| Keep external copy tied to real proof | `references/evidence-gate.md` |
| Write a reusable execution plan | `templates/showcase-plan.md` |
| Draft a landing page outline | `templates/landing-page-outline.md` |
| Draft a demo video script | `templates/demo-script.md` |
| Draft a case study | `templates/case-study.md` |

Use `/shape` when product direction is unsettled. Use `/design` for visible
artifact critique or implementation polish. Use `/qa` for live surface
verification. Use `/deliver` to build the shaped slice. This skill decides what
showcase work is worth doing and what proof must exist before it is public.

## Contract

- Product truth first. If the product cannot produce a believable live or
  replayed demo, build that before copy, brand, or video.
- One-command demo path. A showpiece needs a command, route, fixture, or
  seed/reset flow that recreates the visible state and writes evidence.
- Evidence-backed story. Every public assertion maps to a route, command,
  screenshot, video frame, CI run, dogfood packet, customer example, or explicit
  "vision" label.
- Consulting lens. Assets should prove taste, technical judgment, operator
  empathy, and ability to turn ambiguous AI systems into working software.
- Demo mode is honest. Synthetic data may be used, but it must preserve the
  product's real constraints and failure modes.
- Polish follows proof. Do not build a high-gloss shell over unverified
  behavior. Make the product moment itself credible first.
- External copy has no agent-process leakage. No closeout prose, caveat
  scaffolding, implementation apology, or "if published" meta-copy.
- The operator stays in the loop for positioning choices that affect the
  consulting offer, target buyer, pricing implication, or public claim.

## Output Shapes

**Audit**: ranked gaps with `proof gap / demo gap / polish gap / story gap`,
the smallest next slice, and the verification path.

**Plan**: a `showcase-plan.md` style packet: audience, offer, demo scenario,
asset list, gates, non-goals, and first deliverable.

**Build**: shaped tickets for demo harness, product polish, site, video, or
case study. Keep each slice independently verifiable.

**Review**: adversarial pass over public assets: what would embarrass us if a
prospect clicked, ran, or asked for proof?

## Delegation Judgment

Delegate per the shared Roster contract in `harnesses/shared/AGENTS.md`.
Useful lanes:

- Product critic: find the weakest public promise.
- Demo verifier: run the demo path cold and report where trust breaks.
- Design critic: inspect screenshots/site/video frames for hierarchy, taste,
  accessibility, and generic AI tells.
- Copy critic: remove hype, process leakage, and unsupported assertions.

Critics get the artifact plus the evidence map. They do not get the author's
intent trail.

## Gotchas

- Pretty lies are worse than ugly truth. If proof is weak, fix the proof.
- A feature tour is not a story. Show one consequential job getting done.
- "AI-powered" is table stakes. Say what changes for the operator.
- Fake data can destroy trust when it dodges the hard edge the product exists
  to handle. Seed the hard edge.
- Screenshots taken after manual poking are not a demo harness. Capture the
  reproduction command or route.
- Marketing copy often overclaims silently. Build the evidence map before
  polishing words.
- A landing page should not explain the repo's internal process. It should
  make the product, proof, and next action obvious.
- Demo videos rot. Pin the commit, command, fixture, viewport, and generated
  artifacts.
- Portfolio assets are sales assets. They must answer "why hire this operator?"
  without sounding like a resume.

## Completion Gate

See `harnesses/shared/AGENTS.md` (Completion Evidence) for the shared evidence
core. Showcase adds:

```markdown
## Showcase Gate
- Audience and offer: who this is for, and what action the asset asks for.
- Demo path: exact command, route, fixture, seed/reset flow, or waiver.
- Proof artifacts: screenshots, video, logs, CI, dogfood, release, or case-study evidence.
- Evidence map: each public assertion mapped to proof or labeled as vision.
- Product polish: visible product moment inspected, not just marketing wrapper.
- Design/copy review: artifact-backed critique result and unresolved findings.
- Fresh verifier: cold run or adversarial review result.
- Public risk: what a prospect could click, ask, or run that would still fail.
```
