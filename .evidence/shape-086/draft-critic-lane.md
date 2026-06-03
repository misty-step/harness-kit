Role: fresh-context test/oracle critic.

Objective: Review `backlog.d/086-commit-qa-assistant.md` for blocking gaps in
the oracle, scope boundary, and acceptance evidence.

Scope:
- Read only `backlog.d/086-commit-qa-assistant.md` and the repo files it
  explicitly cites if needed.
- Do not edit files.
- Do not inspect any author reasoning beyond the ticket.

Output shape, max 500 words:
- Verdict: blocking / non-blocking.
- Blocking gaps, if any, with file/section evidence.
- Non-blocking improvements.
- One stronger executable oracle you would add, if needed.

Boundary: This is a shape critique only. Do not propose hosted VNC or
autonomous PR infrastructure unless required to satisfy the stated goal.
