# Audit description tax: flag hand-only skills for user-invoked projection

Priority: P2 · Status: pending review · Estimate: S

## Goal

Stop paying always-loaded description context for first-party skills that only
ever fire by explicit operator trigger.

## Oracle

- [x] Telemetry run lists every first-party skill with zero model-initiated
      invocations over the sample window (or documents that telemetry cannot
      split invocation source, as a finding). **Telemetry cannot split
      invocation source — confirmed, with evidence, not just the anticipated
      open question.** `invocation_kind` (direct/routed/unknown) only exists
      on rows written since 2026-07-02 (24 of 378 rows in the aggregate
      `~/.claude/skill-invocations.jsonl` log, spanning ~3 months across ~20
      repos); every earlier row lacks the field. Worse, even where present,
      `crates/harness-kit-hooks/src/invocation_kind.rs`'s own doc comment
      admits "direct" covers both "the user typed `/some-skill`" and "a
      natural-language request that triggered this skill first" — it cannot
      tell those apart. A transcript-level deep-dive into 5 of the 24
      `direct` rows confirmed this in practice, not just in theory: `/design`
      was invoked from a human message containing the literal `/design`
      trigger; `/shape` was invoked from a pure natural-language brainstorm
      prompt with no trigger word anywhere; `/diagnose` and `/factory-apps`
      were both invoked in response to an automated `<task-notification>` /
      teammate message — no human typed anything in that turn at all. All
      three patterns are filed under the identical "direct" label. Zero
      first-party skills currently show a "some other skill's routing text
      quotes me but no fresh human turn ever reaches me directly" signature
      (`routed`-only with zero `direct`), which would have been the clean
      "only ever hand-triggered" signal this audit was hunting for.
- [x] Each candidate gets a verdict: keep model-invoked (with the story) or
      project as user-invoked where the harness supports it. **All 25
      first-party skills verdicted; table below.** 18 have nonzero
      invocations in the aggregate log → keep model-invoked (real usage,
      doctrine already mandates natural-language `Use when` triggers on all
      of them, and the transcript deep-dive shows organic and
      system-triggered invocation for exactly the skills (`design`,
      `diagnose`, `factory-apps`, `shape`) an uninformed skim might have
      guessed were hand-only). 7 have zero invocations in the 378-row
      sample: `oracle` and `todoist` are false negatives (their capability is
      reached through other surfaces — a manual browser session, the
      Todoist MCP tools directly — not through the Skill wrapper, so
      zero-count here does not mean zero real use) → keep with a note;
      `document`, `qa`, `showcase`, `skill-eval`, `sprites` show no signal at
      all in this sample → flagged for the skill-eval retirement-review
      path, not deleted.
- [x] At least one harness projection (Claude Code `disable-model-invocation`)
      applied, or explicitly rejected with reason. **Explicitly rejected,
      catalog-wide, for this pass.** No first-party skill has evidence of
      being "only ever hand-triggered" (bucket b in the lane brief) — the one
      real transcript check available showed the opposite for every skill
      it touched. Applying `disable-model-invocation` to any of them today
      would risk exactly the falsifier this ticket named up front: "a
      user-invoked projection breaks a real invocation path." Applying it to
      a zero-count skill instead doesn't fit either — zero-count is a
      retirement signal, not a hand-only signal, and hiding the description
      would make the zero permanent by construction. The lever itself is now
      real and tested (see Notes) and ready the moment a skill has genuine
      hand-only evidence; this pass just didn't produce one.

## Verification System

- Claim: some first-party skill descriptions pay context load with no
  autonomous-invocation payoff.
- Falsifier: telemetry shows model-initiated invocations for every skill, or
  a user-invoked projection breaks a real invocation path (skill stops firing
  when it should). **Neither triggered — no projection was applied, so
  nothing could break; the "shows model-initiated invocations for every
  skill" side of the falsifier tripped for the 18 nonzero-count skills. The 7
  zero-count skills are the residual, unfalsified case, and they're flagged,
  not projected.**
- Driver: `cargo run --locked -p harness-kit-checks -- telemetry --repo .`
  **Returns empty for this repo specifically** — the aggregate skill log has
  no `harness-kit` project rows at all (skills fire in consumer repos, not in
  the skill-defining repo itself). The real driver for this audit was the
  same command without `--repo`, against the full aggregate log.
- Grader: operator review of the per-skill invocation-source split.
- Evidence packet: telemetry output + per-skill verdict table in the groom
  report or this ticket (see table below).
- Cadence: once now; fold into `/groom audit` if it pays.

## Classification Table

| Skill | Count (378-row sample) | Verdict |
|---|---:|---|
| deliver | 36 | keep model-invoked |
| code-review | 33 | keep model-invoked |
| research | 27 | keep model-invoked |
| shape | 27 | keep model-invoked |
| groom | 25 | keep model-invoked |
| design | 24 | keep model-invoked |
| harness-engineering | 19 (+8 logged under its pre-rename alias `harness`, see below) | keep model-invoked |
| diagnose | 16 | keep model-invoked |
| next | 16 | keep model-invoked |
| refactor | 16 | keep model-invoked |
| ci | 12 | keep model-invoked |
| vision | 5 | keep model-invoked |
| artifact | 4 | keep model-invoked |
| human-writing | 4 | keep model-invoked |
| council | 2 | keep model-invoked |
| roster | 2 | keep model-invoked |
| factory-apps | 1 | keep model-invoked |
| orient | 1 | keep model-invoked |
| oracle | 0 | keep with note (used via manual browser session, not the Skill wrapper) |
| todoist | 0 | keep with note (used via direct Todoist MCP tools, not the Skill wrapper) |
| document | 0 | flag for skill-eval retirement-review path |
| qa | 0 | flag for skill-eval retirement-review path |
| showcase | 0 | flag for skill-eval retirement-review path |
| skill-eval | 0 | flag for skill-eval retirement-review path |
| sprites | 0 | flag for skill-eval retirement-review path |

All 25 first-party skills accounted for. The 8 `harness`-labeled rows predate
the spellbook→harness-kit rename (logged from `/Users/phaedrus/Development/
spellbook` in April, before this skill was called `harness-engineering`);
they are the same skill, invoked via its `/harness` trigger alias, not a
separate unused one.

## Notes

Source: Matt Pocock, "Writing Great Skills" — model-invoked skills pay
context load (description in the window every turn); user-invoked skills pay
cognitive load (the operator is the index). Cross-harness-first: user
invocation is a per-harness projection concern, never a source-skill change.
Open question: whether current telemetry distinguishes model-initiated from
operator-typed invocations per harness — if not, that gap is the first
finding. **Resolved: it does not, see Oracle above.** If hand-only skills
multiply past memory, Pocock's router-skill cure applies (one user-invoked
skill that names the others).

**Mechanism fix shipped alongside the audit, even though nothing needed it
yet:** `bootstrap --dry-run`'s description-byte tally
(`crates/harness-kit-checks/src/bundles.rs::description_bytes`) counted every
skill's description unconditionally — it did not know about
`disable-model-invocation` at all, so even if a future audit pass finds a
genuine hand-only skill and flags it, the operator would have had no way to
see the promised byte savings in the tool that is supposed to report them.
Fixed with a test (`description_bytes_excludes_disable_model_invocation_skills`)
proving a flagged skill's description drops out of the "full catalog" byte
count. Live catalog baseline as of this audit: 57 skills, ~23,260 description
bytes (`cargo run --locked -p harness-kit-checks -- bootstrap --dry-run --repo .`)
— unchanged by this ticket, since zero projections were applied; this number
is now the honest before-figure for whichever future ticket produces the
first real candidate.

**Follow-up worth filing, not done here (out of this ticket's S-estimate
scope):** `invocation_kind::classify` could split its "direct" bucket further
— explicit-trigger-in-message vs. organic natural-language vs.
system/agent-notification-driven — since this audit found all three
represented in a five-row sample. That finer split is what would eventually
let a future audit populate bucket (b) (user-invoked-only) with real
evidence instead of always landing on "keep" or "flag for zero-use," and it
also needs invocation_kind coverage to mature past the 24-row/2-day window it
has today before any verdict drawn from it should be trusted.
