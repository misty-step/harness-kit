---
name: vision
description: |
  Create or update root VISION.md as a first-class project north-star artifact.
  Conversational project interrogation, repo/workspace research, competitive or
  exemplar scan, lifespan clarification, philosophy distillation, and wiring
  repo-local harness primitives to read it. Use when: "vision", "vision.md",
  "project vision", "north star", "what is this project", "clarify product
  direction", "write/update VISION.md", "project philosophy", "why does this
  repo exist". Trigger: /vision, /north-star.
argument-hint: "[create|update|refresh|audit] [project-context]"
---

# /vision

Create or update root `VISION.md`: the compact project north star that gives
cold agents, maintainers, and future contributors the same answer to "what are
we building, why, and what kind of excellence matters here?"

Conversational first. Artifact always.

## Contract

- Ask the operator what the deal is unless the request already contains enough
  project intent. One good question at a time; use the interrogate-first lens
  (`harnesses/shared/references/interrogate-first.md`) when hidden decisions matter.
- Read the live repo before drafting: existing `VISION.md`, `AGENTS.md`,
  `README*`, positioning/product docs, roadmap/backlog, manifests, examples,
  demos, tests, screenshots, and repo-local skills or prompts that encode
  product behavior.
- Research just enough. Start local, then sibling/adjacent projects when they
  explain fit, then web or external exemplars only when category, audience,
  competition, or public-facing positioning is unclear.
- Clarify lifespan. Is this a spike, internal utility, consulting artifact,
  product substrate, long-lived product, or public standard? The answer changes
  tone, maintenance bar, backlog ambition, and non-goals.
- Write the root artifact at `VISION.md`. Do not tuck the canonical artifact in
  `docs/` unless repo evidence proves root is wrong.
- Let the project determine the structure. No required headings, no house
  template. Keep it tight enough that agents will actually read it.
- Capture what is uniquely load-bearing: intent, philosophy, audience,
  category, fundamentals, standards, non-goals, strategic bets, and what
  excellent looks like over the horizons the project needs.
- Wire consumers, not copies. When `VISION.md` is created or materially changed,
  update relevant repo-local `AGENTS.md`, skills, prompts, or runbooks with
  pointer lines to `VISION.md`; never duplicate vision prose into them.

## Quality Check

Load `references/vision-quality.md` when drafting, reviewing, or updating the
artifact. Use it as a taste/checklist reference, not a template.

## Output

Finish with:

1. What changed in `VISION.md`.
2. What questions were answered, deferred, or still need the operator.
3. Sources read: local files, sibling projects, external references, and what
   each changed.
4. Consumers wired: `AGENTS.md`, skills, prompts, runbooks, or explicit none.
5. Residual risk: stale assumptions, missing competitive context, or unclear
   lifespan.

## Verification

For repo edits:

```sh
test -f VISION.md
rg -n "VISION\\.md" AGENTS.md .agents .codex .claude .pi .antigravitycli skills 2>/dev/null
```

Then run the repo's named gate. In Harness Kit itself, that is:

```sh
cargo run --locked -p harness-kit-checks -- check --repo .
```

## Gotchas

- Generic mission statement. If it could describe three other repos, it failed.
- Over-prescribed structure. A beautiful template can erase the project's real
  shape.
- Backlog dump. Vision decides what belongs; it is not a sorted task list.
- Marketing voice by default. Use marketing language only when that is the
  repo's real surface.
- `docs/vision.md` drift. Root `VISION.md` is the default canonical path.
- Stale consumers. If local harness primitives keep making direction calls from
  old prose, the vision is decorative.
- Research theater. A giant competitive map is waste unless it changes a
  decision.
