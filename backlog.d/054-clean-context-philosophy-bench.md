# Clean-context dispatch for philosophy bench + critic

Priority: P2
Status: superseded by 061 and 063
Estimate: S

## Goal

When `/code-review` dispatches the philosophy bench (ousterhout, carmack, grug, beck, cooper) or the generic critic, pass ONLY the PR diff + backlog acceptance criteria (if present). Strip the caller's conversation history, exploration scrollback, and any prior review turns. The bench reviewer sees the artifact, not the author's reasoning trail.

## Non-Goals

- NOT rewriting the philosophy bench personas. Each agent's `agents/*.md` stays as-is.
- NOT disabling context for other agent types. Explore, Plan, general-purpose all need the caller's context to investigate/design. This is specific to review.
- NOT changing the set of personas dispatched per review. `/code-review`'s routing table (which personas for which kinds of change) is untouched.
- NOT applying this to `/diagnose` or `/qa`. They have different context needs.

## Oracle

- [ ] `skills/code-review/SKILL.md` dispatch instructions specify: "Pass the bench reviewer ONLY the PR diff (from `git diff master...HEAD` or the explicit ref pair) and the backlog item's acceptance criteria (from `backlog.d/<id>-*.md` Oracle section if the PR references one). Do NOT summarize the session. Do NOT paste prior reviewer output. Do NOT include your own implementation reasoning."
- [ ] The instruction explicitly names the failure mode it prevents: "a same-model reviewer inheriting the author's context rationalizes the author's choices — Walden Yan / Multi-Agents What's Actually Working, 58% of clean-context bugs are severe."
- [ ] Each philosophy bench agent frontmatter (`agents/ousterhout.md`, `agents/carmack.md`, `agents/grug.md`, `agents/beck.md`, `agents/cooper.md`, `agents/critic.md`) gets a "Context expectations" section confirming it operates on diff + criteria only, and that receiving additional context is a calling-site bug.
- [ ] A trial run: dispatch bench review on the AGENTS.md restructure (051) using clean-context rules. Compare finding count + severity to a control run with full context. Capture in the closing commit trailer.
- [ ] `dagger call check --source=.` green.

## Notes

### Supersession note

This ticket preserves the old assumption that the philosophy bench is a static
set of installed persona files. The cleaner direction is now tracked in:

- `backlog.d/061-retire-global-static-agents.md`
- `backlog.d/063-dynamic-delegation-skill-contract.md`

The useful part of this ticket still stands: reviewers should get clean,
artifact-focused context. That requirement should move into `/code-review`'s
dynamic delegation contract rather than into global agent files.

### Provenance

/groom session 2026-04-23, Multi-Agents What's Actually Working (Walden Yan). Finding: reviewers with fresh context catch 58% more bugs, and 58% of those are severe. Same-model self-critique is theater — heterogeneity of context beats heterogeneity of persona. Archaeologist investigator confirmed the bench is tractable: each agent is isolated (no cross-agent references) and already has strict tool restrictions.

### Why small

This is an instruction change, not a code change. The bench personas already exist; the dispatch pattern already exists. We're tightening what gets handed to the reviewer. Estimated 30–90 minutes of editing + one trial review for validation.

### Why it's worth doing despite being local polish

The groom synthesis flagged this as "local polish only, zero downstream leverage." True — downstream repos don't use spellbook's bench directly. But the bench IS how spellbook validates its own changes. If the bench rationalizes author context, then every skill, every AGENTS.md change, every refactor has weaker review than it should. Compounds.

### The specific anti-pattern this kills

Today's dispatch often looks like: "here's the full session where I wrote this code, please review it." The reviewer receives the author's reasoning and tends to ratify it ("yes, the author considered X, the choice is defensible"). The question "is this actually good?" gets answered against the author's frame, not against the artifact. Clean context forces the reviewer to read the diff cold.

### Risk: reviewer asks for context the author didn't provide

Mitigation: that's fine. If the reviewer needs the acceptance criteria and the diff doesn't contain them, it asks for them explicitly — and that friction surfaces that the PR wasn't self-explanatory, which is useful signal. Don't pre-empt this by re-injecting context.

### Composition

- Ships after 051 (so bench personas' "Context expectations" sections can reference the L3 routing in AGENTS.md cleanly).
- Independent of 052, 053, 055.

### Execution sketch

Single PR:
1. `skills/code-review/SKILL.md` — dispatch instruction update.
2. `agents/{ousterhout,carmack,grug,beck,cooper,critic}.md` — Context expectations section added to each.
3. Trial run on a live PR (likely 051 itself) + note in commit trailer.
