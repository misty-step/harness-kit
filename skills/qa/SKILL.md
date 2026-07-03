---
name: qa
description: |
  Verify the running thing works. Browser walks for web, request replay for
  APIs, local API emulation for supported third-party services, shell smoke for
  CLIs, consumer builds for libraries, tool-call replay for MCP. "Tests pass"
  is not QA. Use when: "run QA", "verify the
  feature", "test this", "check the app", "smoke test", "exploratory test",
  "capture evidence". Trigger: /qa.
argument-hint: "[url|route|command|endpoint|feature]"
---

# /qa

**Every app has a QA path.** The first question is not "how do I drive a
browser?" — it's "what shape is this app, and what does verifying it look
like here?" If the repo has its own QA/verification skill, defer to it: it
encodes the actual routes, commands, and golden paths. If it doesn't, that
absence is a harness gap — run the protocol below AND flag the gap to
`/groom` so the repo grows one.

For recurring QA, unclear app shapes, eval-like agent behavior, performance
claims, or weak pass/fail criteria, load
`harnesses/shared/references/verification-system-first.md` and design the
driver, grader, evidence packet, and cadence before driving the surface.

## Step 0: shape

Read the signals (`package.json` bin/framework deps, `playwright.config.*`,
`Cargo.toml` bin vs lib, `cmd/` trees, MCP deps, deploy configs) and pick:

| Shape | QA path |
|---|---|
| Browser app | Start dev server or hit preview; walk the golden paths the change touched; watch console + network panel for errors |
| API / service | Replay representative requests against local/preview; for supported third-party APIs prefer `emulate.dev` before live network or brittle mocks; check status, contract shape, and error paths (bad auth, malformed body) |
| CLI | `--help` accuracy, happy-path invocations from the docs, malformed-input paths; audit exit codes and error-message clarity |
| Library / SDK | Build the distributable, install into a throwaway consumer, exercise the changed public API, check the type surface |
| MCP / agent tool | Register with a harness, replay each affected tool call, confirm errors come back structured rather than crashing the server |
| Hybrid | One path per surface the change touched — one path does not cover all |

Ambiguous shape: name both candidates and ask; don't silently pick.

**The canonical misread:** "no playwright config" does not mean "skip QA."
It means Playwright isn't the path — name the one that is. If you can't
name a path, ask; never ship a generic shrug.

## Run it

Drive the changed surface specifically — happy path first, then the edges
the change plausibly broke. If the delivery carries a deviation ledger,
those sites are that edge list, precomputed — drive them first. Capture evidence as you go (screenshot on
anomaly, terminal transcript, request/response pairs) under the repo's
evidence convention or a dated scratch dir; link the specific artifact in
the report, not just a directory name.

When the verification leans on examples whose *values* matter (golden
files, fixtures, seeded data, asserted screenshots), spot-check that a
wrong value would actually fail — mutate one and watch it catch. Weak
oracles that pass on anything are the most expensive kind of green.

Classify findings: **P0** blocks ship, **P1** fix before merge, **P2** log
and move on.

## Verdict

A pass report names: the exact surface exercised (command/URL/route/tool
call), what was observed, the evidence artifact, what was NOT covered, and
whether a post-ship signal exists for this behavior (if nothing would page
or log when it breaks, say so — that's instrumentation debt, not a
footnote). For AI-feature surfaces, a post-ship signal means behavior-level
classifiers — hallucination, tool failure, refusal, user frustration — not
just exception logging; stack traces don't fire when an agent confidently
does the wrong thing. When the same agent drove the app and judges the result, have a
fresh subagent attack the pass claim before signing off: what path would
embarrass us in production?
For public API, CLI, UI, performance, compatibility, migration, or operator
workflow changes, include `harnesses/shared/references/works-critique.md` in
that fresh pass-claim attack.

## Gotchas

- **"Tests pass" is not QA.** Tests verify the paths the author imagined;
  QA verifies the running app against reality.
- **Shape first, tools second.** Tool-first thinking is how this skill once
  decayed into browser-only framing.
- **Generic QA is a stopgap.** The durable fix is a repo-local verification
  harness: one command that seeds/auths/drives the real surface and writes
  an evidence packet (screenshots, transcripts, verdict). If you'll QA this
  surface more than once, build the harness *now* — verification system
  first (shared AGENTS.md, Layer 1) — don't just file the gap. Build it by
  interviewing the operator: the manual checks they run before merging are
  the spec. Ad-hoc QA evaporates; a harness compounds.
- **QAing a behavior-preserving refactor with no characterization tests?**
  "Tests pass" proves nothing when there are no tests pinning the current
  behavior. Reach for the live-diff pattern in
  `harnesses/shared/references/verification-system-first.md`: diff the local
  branch against the deployed/pre-refactor build over the same backing store,
  byte-for-byte, including error paths.
- Browser tool selection and evidence conventions: `references/browser-tools.md`,
  `references/evidence-capture.md`.
