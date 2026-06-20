# Vision Quality

Use this reference to review a `VISION.md`. It is not a template.

## Bar

A useful vision changes what gets built, deleted, rejected, sequenced, or
polished. A cold agent should be able to read it and make better tradeoffs than
it would from `README.md` and backlog titles alone.

Good vision prose is:

- Specific to this project. It names the unusual premise, taste, constraints,
  and standards that a generic competitor would miss.
- Operational. It gives future work a decision lens, not just inspiration.
- Compact. It avoids ceremony, slogan piles, and exhaustive background.
- Honest about lifespan. Temporary tools, client artifacts, public standards,
  and long-lived products deserve different ambition and maintenance bars.
- Philosophical where philosophy matters. Fundamentals beat feature lists when
  they explain why the project should exist in this form.
- Bounded. Non-goals and rejected futures matter when they prevent drift.
- Alive. It can be revised when evidence changes, but it should not churn with
  every backlog edit.

## Review Questions

- What is this project for, and who would miss it if it disappeared?
- What must stay true even when implementation details change?
- What kind of work should this repo refuse?
- What does excellent look like in the short, medium, and long term if those
  horizons matter?
- Which adjacent projects, competitors, internal tools, or user workflows make
  the project easier to understand?
- Does the prose help an agent choose between two plausible tickets, specs,
  designs, or tradeoffs?
- Is any sentence generic enough to move unchanged into another repo?

## Failure Modes

- Wallpaper: sounds good, changes no decision.
- Substitution: repeats the README, roadmap, or backlog.
- Overreach: claims a product destiny the maintainers will not fund.
- Underspecification: hides the real audience, category, or maintenance bar.
- Fossilization: AGENTS/skills/prompts still encode older direction.
- Voice mismatch: public launch copy for an internal tool, or internal notes
  for a public-facing product.
