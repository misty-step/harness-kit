# Delete-First Sample

Requested raw idea: "Add a scheduled agent that rewrites the skill catalog every
night using marketplace trends."

Using `harnesses/shared/references/delete-first.md`:

- Requirement questioned: the real outcome is "notice useful external skills,"
  not "rewrite the catalog nightly."
- Deleted or simplified: delete unattended catalog mutation; run a dry-run
  `scout-skills` report when a curated input list changes.
- Only then optimized/automated because: a future Mode B loop may run the scout
  on a schedule after it has a stable state file, budget, verifier, and human
  approval boundary.

Result: reject the automation; keep a deterministic report and a shaped Mode B
handoff only if the report becomes recurring work.
