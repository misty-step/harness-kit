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
like here?" If the repo has its own `<repo>-qa` skill, defer to it: it
encodes the actual routes, commands, and golden paths, and it wins over this
generic method. If it doesn't, that absence is a harness gap — run the
protocol below to QA now, then **scaffold the repo-local skill** (see
"Scaffold a repo-local QA skill") so the next QA is cheaper. A repo you QA
more than once should own its QA skill.

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
the change plausibly broke. Capture evidence as you go (screenshot on
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

## Scaffold a repo-local QA skill

The durable form of QA is a `<repo>-qa` skill in the repo itself — the
project-specific knowledge this generic method structurally can't hold (exact
launch commands, ports, env, seed/auth, per-surface golden paths, gotchas).
This is the highest-leverage local agent investment; build it the moment a
repo earns a second QA pass.

1. **Explore the repo for the run surface.** `package.json` scripts/bin,
   `Cargo.toml` bin vs lib, `Makefile`, `docker-compose`, `.env.example`,
   `playwright.config.*`, `scripts/`, and any existing verify/smoke gate. The
   goal is the exact copy-pasteable launch command(s) + ports + required env.
2. **Interview the operator — the manual checks they run before merging ARE
   the spec.** What do they open, log into, click, curl, or run to believe a
   change works? Encode that sequence, not a generic protocol.
3. **Write it from the template**
   (`skills/qa/templates/repo-qa-skill.md`) into the repo's skill root
   (`.agents/skills/<repo>-qa/SKILL.md`, or `skills/` where the repo uses
   that). Name it `<repo>-qa`. Fill every placeholder with real commands;
   delete sections that don't apply.
4. **Start thin, refine through use.** A 20-line skill naming the real launch
   command and one golden path beats a 500-line speculative one. Grow it the
   next time QA is painful — not preemptively. Deterministic mass-scaffolding
   across many repos at once is the known failure mode; one grounded skill per
   repo, earned, is the pattern.
5. **Verify it works** by running your own new skill against the current diff
   before declaring it done.

To find which active repos still lack one, this is a `/harness-engineering`
repo-QA audit, not a blind blast.

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
- Browser tool selection and evidence conventions: `references/browser-tools.md`,
  `references/evidence-capture.md`.
