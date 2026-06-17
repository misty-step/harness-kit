Show the harness-kit work backlog and explain how to close an item.

`backlog.d/` is the source of truth for work (not GitHub Issues). Do this:

1. List active items: the `backlog.d/*.md` files (excluding `_done/`).
2. List recently completed items in `backlog.d/_done/`.
3. For each active item, read the first few lines to show: ID, title, status,
   priority, and size.

Closure protocol for an item: move it to `backlog.d/_done/` with `Status: done`,
add a `## What Was Built` note, and link it from the commit with a conventional
trailer — `Backlog: backlog.d/<id>-<slug>.md`, `Closes-backlog:`, or
`Ships-backlog:`.

Open high-signal debt starts at `backlog.d/023-*.md`; read the directory for
the current state.
