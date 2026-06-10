---
description: One targeted read-only critique through a named lens (ousterhout, carmack, grug, beck, cooper, …).
argument-hint: "--lens <name> --target <path> | --lenses"
---

Run a single-lens, read-only critique. `--lenses` lists the names in
`harnesses/shared/references/lenses.md`; `--lens <name>` applies that
rubric block to `--target <path>`.

Spawn one fresh read-only critic subagent with only the lens excerpt and
the target — no author reasoning trail. If a subagent can't be spawned,
adopt the lens yourself for one pass and label it `lens-adopted-by-primary`
(a fallback, not fresh-context separation).

Findings need `file:line` evidence and an impact level; if nothing
survives, say so and cite what was read. No edits, no Ship/Don't-Ship
verdict — that belongs to a full review. This is one focused signal, not a
merge gate.
