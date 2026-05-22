# Cross-Harness Review

Invoke other AI coding CLIs for harness-diverse review. Each CLI brings its
own system prompt, tools, and AGENTS.md context — genuinely different from
the same model accessed via API. On Spellbook specifically this matters:
the thing being reviewed is often harness-scoped (Claude settings, Codex
plugins, Pi skills[] globs), and having a reviewer running *inside* a
different harness surfaces parity issues the marshal can't see from one
harness alone.

## Roster Selection

If `.spellbook/agents.yaml` exists, it is the source of truth. Probe the
roster, skip the provider/harness you are currently using, then dispatch
bounded read-only review lanes to available providers with review-capable
commands. For meaningful lanes, write sanitized delegation receipts through
the repo receipt tool; evidence references point to paths or ids, never raw
transcripts.

Use the roster's command templates instead of hard-coding provider names
here. If no roster exists, fall back to whichever read-only coding CLIs are
locally documented and installed.

## Harness Detection

Skip whichever CLI you ARE — you already have that model's perspective as
the marshal. The model knows which harness it's running in.

## Consuming Output

Read the full output from each lane. Extract findings with file:line
references and severity. Feed them into the marshal's synthesis alongside
thinktank and philosophy-bench results.

## Gotchas

- If a CLI is not installed or fails, mark that provider lane failed or
  skipped in its receipt. Don't block the review solely on provider
  availability.
- Cross-harness CLIs run in the current repo directory — they see the same
  files. No need to pipe the diff as stdin.
- Don't pipe the entire diff as stdin for large diffs. Let the CLI read
  the repo directly.
- A cross-harness reviewer flagging a parity issue the marshal missed is
  load-bearing signal — upgrade its severity, don't dismiss it as
  "different harness, different opinion."
