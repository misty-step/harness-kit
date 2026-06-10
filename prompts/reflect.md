---
description: Mine this session for durable lessons and write them into the files that load every session.
argument-hint: "[topic]"
---

Retrospect on this session (or the named topic) and convert findings into
artifacts, not prose. Every finding becomes exactly one of: a written change
(AGENTS.md/CLAUDE.md rule, skill edit, hook, gate, test, backlog ticket), a
concrete coaching note for the operator, or an explicit "not codifying
because …".

Separate three failure classes before assigning fixes: harness failure (the
instructions/tools should have prevented it — fix the harness), shared
ambiguity (both sides left constraints implicit), operator-spec gap (the
decisive fact lived only in the user's head — propose the tighter prompt
they could have given). Don't dump harness failures on the user, and don't
manufacture generic coaching when there's nothing high-leverage to say.

Rules earn codification by recurrence: a correction made twice is a rule
candidate; verify a candidate would have prevented a real mistake in this
session before writing it. Place each rule in the highest-leverage layer —
hook/gate/test over skill prose over always-loaded doctrine. Doctrine lines
cost every future session; spend them sparingly.

A failure note is not a lesson. Run the full ladder before writing:
investigate why it happened, verify the diagnosis against the live repo
(turn the guess into a checked fact), then distill the verified fact into a
general rule — and write it where it gets consulted, not into a graveyard
file. "Maybe X?" stored as memory is a future session re-deriving the same
mistake.

In this repo, apply edits directly and list them. In other repos, write to
the repo's own AGENTS.md/CLAUDE.md or propose a harness-kit change as a
`backlog.d/` ticket.
