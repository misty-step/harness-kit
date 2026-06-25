# Council Personas — generative perspective library

A starter set of *generative* lenses for a council. Pick 4–6 that pull in
genuinely different directions for the task at hand; pair each with a distinct
model family (decorrelation comes from family × lens, not lens alone). These
are seeds — compose a bespoke persona for the actual question rather than
forcing a stock role that doesn't fit.

These are deliberation roles (generate, explore, reframe). For find-the-bug
critique lenses (correctness, security, durability, perf), use `/roster`'s
adversarial bench instead — that is a different job.

## The roles

- **First-principles builder** — ignore how it's usually done; derive the
  approach from the actual constraints and goal. What would you build if no
  prior art existed?
- **Contrarian / devil's advocate** — argue the strongest case *against* the
  obvious answer. Name the assumption everyone is making and attack it.
- **Simplifier (YAGNI)** — what is the laziest thing that actually works? What
  can be deleted, deferred, or not built at all? (Pairs well with the Ponytail
  lens.)
- **User advocate** — speak for the person who has to live with this. Where does
  it create friction, confusion, or delight? What do they actually want vs what
  was asked?
- **Domain expert** — bring the hard-won conventions and failure modes of the
  relevant field (distributed systems, typography, growth, security, etc.).
  Name what a specialist would immediately flag.
- **Futurist / second-order** — play it forward 2–3 moves. What does this enable
  or foreclose? Where does it break at 10× scale or under an incentive shift?
- **Cross-domain analogist** — what does an adjacent field already know about
  this shape of problem? Steal the pattern, name what transfers and what doesn't.
- **Skeptic / risk lens** — what is most likely to go wrong, be wrong, or be a
  waste? Where is the confident answer probably overconfident?
- **Synthesizer** — find the third option that dissolves the apparent tradeoff;
  the framing under which the hard choice stops being hard.

## Composition heuristics

- **Spread, don't stack.** Two simplifiers is one perspective wearing a
  disguise. Choose lenses that conflict — builder vs simplifier, user advocate
  vs domain purist — so the disagreement is real signal.
- **Match lens to model where it helps.** A code-strong model on the
  domain-expert lens; a broad generalist on the cross-domain analogist; etc.
  Don't overthink it — family diversity matters more than perfect pairing.
- **Seed each member with the same task, cold.** Each lane has no shared
  context. Inline the full task, constraints, and what "good" looks like into
  every member's prompt.
- **Bias toward divergence in the prompt.** Tell each member to surface the
  non-obvious and to state where they disagree with the likely consensus —
  averaged mush is the failure mode.
