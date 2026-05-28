# Pin Codex `config.toml` schema in `/tailor`

Priority: high
Status: done
Estimate: S

## Goal

Update the `/tailor` skill so the `.codex/config.toml` it emits uses
keys Codex actually parses. Today the skill tells the model to emit
"per-harness settings" but leaves the schema unspecified; the model
hallucinates a `[permissions] allow = [...]` block (Claude Code's
shape), which Codex rejects at load time with
`invalid type: string "<cmd>", expected struct FilesystemPermissionsToml
in `permissions``. The config then fails to load on every Codex
invocation in the tailored repo.

## Non-Goals

- Redesign Codex's permissions model. We consume it as-is.
- Move the command allowlist into Codex. Codex uses top-level
  `approval_policy` + `sandbox_mode` (global in
  `~/.codex/config.toml`); there is no per-command allowlist in
  `config.toml`. Claude Code's `.claude/settings.local.json` remains
  the single source of truth for command allowlists.
- Migrate existing tailored repos. That is handled out-of-band when
  the error surfaces; this ticket prevents recurrence.

## Oracle

- [ ] `skills/tailor/SKILL.md` step 8 ("Write per-harness settings")
      names the Codex keys that are safe to emit (e.g. `[project]`,
      `[skills]`, `[conventions]`, `[gate]`, `[git]`) and explicitly
      forbids emitting a top-level `[permissions]` table with an
      `allow` array. One sentence naming the failure mode
      (`FilesystemPermissionsToml` schema collision) so future runs
      don't re-derive it.
- [ ] A fresh `/tailor` run against a clean repo emits a
      `.codex/config.toml` that loads without error:
      `codex exec --skip-git-repo-check "echo ok"` produces no
      `Error loading config.toml` line.
- [ ] The skill either (a) points Codex at the Claude allowlist via a
      cross-reference comment, or (b) documents that Codex inherits
      global `approval_policy`/`sandbox_mode` and no per-repo
      allowlist is needed. Pick one; don't emit an ineffective block
      either way.
- [ ] `./bin/validate` green (skill docs change only).

## Notes

**Symptom and repro.** From any repo tailored before this fix:

```
$ codex exec --skip-git-repo-check "echo ok"
Error loading config.toml: invalid type: string "./bin/validate",
expected struct FilesystemPermissionsToml
in `permissions`
```

Codex 0.122.0 then falls back to global config and proceeds, so the
error is soft — but it surfaces on every invocation and blocks
worktree / session setup UX for operators who treat errors as
blocking. Confirmed on canary (commit `9bdc962`, `.codex/config.toml`)
and harness-kit (own `.codex/config.toml`). Both were emitted by
`/tailor` and have since been hand-patched. This ticket prevents the
next tailored repo from re-introducing it.

**Root cause.** `skills/tailor/SKILL.md:325-334` instructs the model
to emit settings files for all three harnesses but only names the
*paths*, not the *schemas*. The model defaults to the Claude Code
shape (`[permissions] allow = [array of commands]`) because that's
the only permission-allowlist pattern in its working set. Codex's
`[permissions]` key is a different thing entirely — a
`FilesystemPermissionsToml` struct describing filesystem path
scopes, not commands — so the emitted TOML is schema-invalid.

**Norman Principle.** This is a harness bug, not an operator mistake.
The skill doesn't have the constraint, so the model re-derives the
wrong answer every run. The fix is in the skill.

**Fix shape (draft).** In `skills/tailor/SKILL.md` step 8, add
something like:

> **Codex `config.toml` schema.** Codex does not support a per-command
> allowlist in `config.toml`. Emit only these top-level keys:
> `[project]`, `[skills]`, `[conventions]`, `[gate]`, `[git]`. **Do not
> emit `[permissions]`** — Codex parses that key as a
> `FilesystemPermissionsToml` struct (filesystem path scopes), and a
> command-array shape fails config load. The command allowlist lives
> in `.claude/settings.local.json` (Claude Code's model); Codex uses
> global `approval_policy` / `sandbox_mode` from
> `~/.codex/config.toml`.

**Execution sketch (one PR, one commit).**

*Commit 1 — `docs(tailor): pin codex config.toml schema, forbid
[permissions] block`.* Single file change to
`skills/tailor/SKILL.md`. Optional follow-up: a one-liner test in
`scripts/` that lints every `.codex/config.toml` in the operator's
portfolio by invoking `codex` and grepping for the load error (out
of scope for this ticket; note for `#027`-style lint backlog).

**Risk list.**

- *Codex schema evolves.* If Codex adds a `config.toml` command
  allowlist later, the guidance goes stale. Mitigated by linking to
  Codex's current config docs in the comment the skill emits, so a
  future reader can verify against upstream.
- *Operators expect parity with Claude allowlist.* Addressed by the
  emitted comment pointing at `.claude/settings.local.json` as the
  canonical source, plus the `/tailor` write-up explaining that
  Codex inherits from `~/.codex/config.toml` globally.

**Lane.** Harness-conventions. No code path impact, skill docs only.
Ships whenever `/tailor` is next touched; high priority because every
re-tailored repo re-introduces the bug until this lands.

Source: `/diagnose` session on canary 2026-04-23; reproduced against
harness-kit from the same root cause. Both repos hand-patched; skill
not yet updated.
