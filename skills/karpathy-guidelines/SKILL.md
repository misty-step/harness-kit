---
name: karpathy-guidelines
description: |
  Four LLM-agent guardrails: surface assumptions, prefer simplicity, make
  surgical changes, and drive by verifiable goals. Reference for scope,
  simplicity, assumptions, or success-criteria judgment calls. Use when:
  "am I overcomplicating this", "what are my assumptions",
  "how do I verify success", "/karpathy", "/principles".
argument-hint: "[think | simple | surgical | goal]"
---

# /karpathy — behavioral guidelines

Four principles for avoiding the specific LLM-agent failure modes
Karpathy has repeatedly called out on Twitter/X: silent
assumption-picking, premature abstraction, scope creep into adjacent
code, and vague success criteria. Use as a self-check before acting,
or dispatch when an agent seems to be drifting.

**Tradeoff.** These bias toward caution over speed. For mechanical
tasks (renames, find/replace, dep bumps) they're overhead — use
judgment. The threshold is "does this need a judgment call?", same
as for delegation.

## 1. Think before coding

**Don't assume. Don't hide confusion. Surface tradeoffs.**

Before implementing:

- State your assumptions explicitly. If uncertain, ask.
- If multiple interpretations exist, present them — don't pick
  silently.
- If a simpler approach exists, say so. Push back when warranted.
- If something is unclear, stop. Name what's confusing. Ask.

The failure this prevents: the model picks one interpretation of a
vague request, runs three-hundred lines, and the user only
discovers the mismatch after reviewing the diff. Surface the fork
*before* the work, not after.

**Example.** Ticket: "add validation to the signup form."

Wrong: silently pick "validate email format + password length,"
ship that, discover the user wanted reCAPTCHA.

Right: before coding, state *"I'm going to add client-side format
validation for email and minimum-length for password. I'm not
planning to wire up reCAPTCHA, rate limiting, or server-side
checks. Confirm or redirect."*

## 2. Simplicity first

**Minimum code that solves the problem. Nothing speculative.**

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.
- If you write 200 lines and it could be 50, rewrite it.

Ask: *"Would a senior engineer say this is overcomplicated?"* If
yes, simplify.

The failure this prevents: over-engineered scaffolding that
obscures the actual change. Three design patterns, a new
abstraction layer, and a config file to solve a problem that
needed five lines.

**Example.** Ticket: "cache the user's timezone on the session."

Wrong: introduce a `CacheProvider` interface, implement it for
memory + Redis, add a config toggle, write factory functions.

Right: `session.timezone = user.timezone; session.save()`.

## 3. Surgical changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:

- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it — don't delete it.

When your changes create orphans:

- Remove imports, variables, functions that YOUR changes made
  unused.
- Don't remove pre-existing dead code unless asked.

The test: **every changed line should trace directly to the
request.**

### Reconciling with "fix what you touch"

This principle sits in productive tension with the broader
doctrine's *"Fix what you touch — including pre-existing issues
in the same area."* The tension resolves cleanly:

- **Broken** things in your working area — fix or file. No
  "pre-existing, not my problem" dodges.
- **Non-broken** things in your working area — leave alone,
  even if you'd write them differently.

Broken means: wrong output, missing guard, actually-hit bug,
fails the acceptance criteria. Not: "I'd name this differently",
"this could be a helper", "this comment is stale-ish."

## 4. Goal-driven execution

**Define success criteria. Loop until verified.**

Transform tasks into verifiable goals:

- "Add validation" → *"Write tests for invalid inputs, then make
  them pass."*
- "Fix the bug" → *"Write a test that reproduces it, then make it
  pass."*
- "Refactor X" → *"Ensure tests pass before and after. List what
  X's behavior is; preserve it bit-for-bit."*

For multi-step tasks, state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
3. [Step] → verify: [check]
```

Strong success criteria let the agent loop independently. Weak
criteria ("make it work") require constant clarification and
drift toward "works on my machine" endings.

This principle also unlocks delegation: a subagent given a vague
goal produces vague work; a subagent given a verifiable oracle
produces work you can check in 30 seconds.

## Self-check

These guidelines are working if:

- Fewer unnecessary changes appear in diffs.
- Fewer rewrites happen because the first pass was overcomplicated.
- Clarifying questions land **before** implementation, not after
  the mismatch is discovered in review.

## Attribution

Derived from `forrestchang/andrej-karpathy-skills` (MIT),
a community compilation of Andrej Karpathy's observations on
LLM coding pitfalls. Rewritten here with harness-neutral wording
and examples drawn from Harness Kit's own repo shape; the four
principles and their framing are Karpathy's.
