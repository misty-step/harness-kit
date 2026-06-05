---
name: demo
description: |
  Capture evidence for what changed: blurb, paste, screenshot, GIF, video,
  launch note, or repo-local demo skill. Pick by change shape, audience, and
  budget. Use when: "make a demo", "record walkthrough", "PR evidence",
  "upload screenshots", "show what changed", "release notes blurb",
  "scaffold demo", "generate demo skill".
  Trigger: /demo.
argument-hint: "[evidence-dir|feature|scaffold] [--format blurb|paste|screenshot|gif|video|launch] [upload]"
---

# /demo

**Every app has a demo path.** The question is never "is there something
to show?" — it's "what shape does showing take in this codebase, for
this change, for this audience?" A refactor's demo is a PR-description
bullet pointing at the diff stats; a new endpoint's demo is a `curl`
paired with its JSON response; a CLI flag's demo is a terminal paste;
a UI change's demo is a screenshot or GIF; a tentpole launch's demo is
a narrated video. Same skill, different output.

The failure mode this skill exists to prevent is the silent skip —
shipping a change with no visible record of what moved, because the
repo doesn't have a video pipeline or GitHub upload path. The canonical
artifact surface is `.evidence/<branch>/<date>/`; if a repo cannot use it,
the demo path must name its durable alternative.

## Completion Gate

Every demo outcome, including "no artifact needed," answers:
See `harnesses/shared/AGENTS.md` (Completion Evidence) for the shared evidence
core; this phase keeps artifact-specific local fields.

```markdown
## Completion Gate
- Exact end-user behavior shown: behavior or internal operator behavior the artifact demonstrates.
- Evidence that proves it: screenshot, GIF, video, paste, launch note, or artifact path.
- Exact command/path/route exercised: command, URL, route, file path, or tool call used to capture it.
- Repo-fit check: repo convention or audience surface the artifact matches.
- Residual risk: unshown path, unsupported state, or none with reason.
```

For internal changes, the demo may be a PR/release-note sentence, but it still
names the developer/operator behavior that changed.

## Work Ledger

When `.harness-kit/work/ledger.jsonl` is available, `/demo` calls
`cargo run --locked -p harness-kit-checks -- work-ledger append` with `phase_started` before capture and
`phase_completed` after the artifact or explicit "no artifact needed" evidence
ref exists. Use `blocker_added` when capture fails and set `next_action` to the
smallest recovery step.

## Execution Stance

You are the executive orchestrator.
- Keep shape detection, audience fit, and sufficiency judgment on the
  lead model. These are the load-bearing calls.
- For routine changes, run the quick-capture protocol inline.
- For polished artifacts (GIF / video / launch), delegate planning,
  capture, and critique to separate focused subagents. Cold reviewer for
  final quality judgment — self-grading is worthless.

## Delegation Floor

Delegation floor applies: probe the roster first; dispatch two or more
providers for substantive work; direct solo only for mechanical, emergency,
user-forbidden, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use one lane for capture planning and another for cold quality review against the scoped audience, changed behavior, and artifact constraints.

## Shape detection (always step one)

Before capturing anything, answer three questions in order:

1. **What changed?** Classify the diff:

   | Change shape | Demo path |
   |---|---|
   | Browser / UI | Screenshot of the new state; GIF if interactive; before/after pair for visual deltas |
   | API / HTTP endpoint / serverless handler | `curl` request + response body pasted in PR or release notes; before/after pair for breaking changes |
   | CLI / command-line tool | Terminal recording (`asciinema`, `vhs`) or copy/paste of `--help` output plus a representative invocation |
   | Library / SDK / package | Code snippet showing the new API in use; type-check output or REPL session if relevant |
   | MCP server / agent tool / LLM skill | Sample tool-call invocation and its result; conversation transcript excerpt; registration block for new tools |
   | Internal refactor / infra / dependency bump / build config | Release-notes blurb or PR-description bullet. "No artifact needed" is the recorded outcome, not a skip |

2. **Who is the audience?** PR reviewer wants the minimum proof the
   change works. Team Slack wants a short, punchy visual. External
   customers want polish and context. The audience determines fidelity
   and framing, not whether a demo exists.

3. **What's the time budget?** Seconds (paste a log line), minutes
   (screenshot, curl paste, terminal recording), hours (edited GIF,
   before/after pair), days (narrated video, launch piece). Match
   fidelity to budget; under-investing is fine when the change is
   routine, over-investing is waste.

If shape, audience, and budget agree on "blurb is enough" — write the
blurb. If they point at "screenshot + one-liner" — capture and paste.
If they point at "polished video" — scaffold (or run the scaffolded
skill) and invoke the full workflow.

## Routing

| Situation | Action |
|---|---|
| First argument is `scaffold`, or user said "scaffold demo" / "generate demo skill" | Read `references/scaffold.md` and follow it |
| A project-local `.claude/skills/demo/SKILL.md` exists and the change needs polish | Defer to the project-local skill |
| Change shape is internal / refactor / infra / dep bump | Use the **no-artifact path** below — write the blurb, don't open a capture tool |
| Change shape is routine (single screenshot, curl paste, terminal paste, code snippet) | Use the **quick-capture path** |
| Change shape warrants polish (GIF with before/after, edited walkthrough, narrated video) | Use the **polished path** |

## No-artifact path (internal / refactor / infra)

The demo is a sentence, not a file. This is a first-class outcome, not
a skip.

1. State what moved, in one line, in the PR description or changelog
   entry. Example: `Refactor: extract rate-limiting from handler into
   middleware. No behavior change; 4 handlers now share the same limiter.`
2. Link to the diff stats or the most revealing file if helpful.
3. Note explicitly why no artifact was captured: no user-visible
   change, or no output worth pasting.
4. Done. Do not manufacture a contrived screenshot just to have one.

## Quick-capture path (routine changes)

For the dominant case — one change, one surface, one screenshot or
paste. Fast, in-context, no subagents.

1. **Identify the delta.** What's visibly different? The "after" state
   only matters relative to the "before" — even if "before" is "this
   endpoint returned 404 because it didn't exist."
2. **Capture at the right surface:**
   - UI → screenshot (single frame) or short GIF (if the value is in
     motion). Crop to the relevant area.
   - API → run the request, paste request + response. Redact secrets.
   - CLI → run the command in a terminal with readable font, paste
     stdout. Or `asciinema rec` for multi-step invocations.
   - Library → a minimal code block that uses the new symbol, in the
     language of the repo.
   - Agent tool → the tool-call invocation and its result, inline.
3. **Embed where the audience will read it.** PR description for
   reviewers, release notes for users, Slack post for the team, commit
   message body for the long-term record. All are valid surfaces; the
   PR description is the most common.
4. **Sanity check.** Re-read your capture. Does it actually show the
   change, or does it show a default state that would look identical
   without the change? If the latter, capture again with the delta
   foregrounded.

## Polished path (planner -> implementer -> critic)

For tentpole changes, recurring demo needs, or anything that will be
seen outside the PR. Each phase is a **separate subagent**; the critic
inspects artifacts cold to prevent self-grading.

1. **Plan.** Identify the feature delta, build a shot list, choose the
   capture method (browser MCP, Playwright, `vhs`, `asciinema`,
   Remotion, hand-captured). Name the target surface (PR comment, draft
   release, changelog, customer email, Slack post).
2. **Capture.** Execute the plan — every "after" has a paired "before"
   where a delta is claimed. Source recordings are preserved; derived
   artifacts (GIFs, cropped stills) are reproducible from source.
3. **Critique.** Fresh subagent, no context from the implementer.
   Validates source, pairing, text delta, coverage, quality.
4. **Publish.** Write artifacts to `.evidence/<branch>/<date>/` and commit
   them with the branch. Embed or link that path in the target surface.
   A draft GitHub release plus PR comment is an optional mirror when reviewers
   need inline asset URLs; PR description path links, release notes, `git notes`,
   and Slack/email for audience-facing posts are all valid. Pick the surface by
   audience, not by habit.

### FFmpeg quick reference

```bash
# WebM -> GIF (800px, 8fps, 128 colors)
ffmpeg -y -i input.webm \
  -vf "fps=8,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
  -loop 0 output.gif
```

For detailed capture patterns, read `skills/qa/references/evidence-capture.md`.
For PR evidence upload via draft releases, read `references/pr-evidence-upload.md`.
For Remotion video composition, read `references/remotion.md`.
For TTS narration, read `references/tts-narration.md`.

## Surfaces (where a demo lives)

The artifact and the surface are separate choices. Pick the artifact
from shape; pick the surface from audience.

| Surface | Good for |
|---|---|
| PR description (markdown + inline images) | Reviewers; the default |
| Release notes / changelog entry | Users reading between versions |
| `.evidence/<branch>/<date>/` | Canonical offline storage; default for QA/demo artifacts |
| Draft GitHub release (`gh release create --draft`) | Optional mirror for large assets or inline PR URLs |
| Commit message body | Long-term record, especially for no-artifact outcomes |
| `git notes` | Durable annotation without touching history |
| Team Slack / internal post | Timely visual update; lossy over time |
| Customer email / launch post | External polished path |

## Repo guidance

When a repo vendors or specializes this skill, the local guidance should
**name the actual demo path for THAT codebase.** Concretely:

- Pick the default artifact shape(s) for this repo's dominant change
  type — e.g., "Next.js app; defaults are screenshot (single change) or
  paired before/after GIF (interaction change)."
- Pick the default surface — e.g., "PR description with inline images;
  draft releases only when PR comments need inline URLs."
- Name the repo's actual capture tooling — Chrome MCP? Playwright
  video? `vhs` tapes checked into `demo/`? `asciinema` uploads? If none
  exist yet, name the lightest-weight option that fits the stack.
- Name the audience — internal-only team tool? Public library with
  release notes? Customer-facing product with launch posts?
- If the repo is internal-only refactoring territory (an infra-only
  library, a build-tooling monorepo), lean the defaults toward the
  no-artifact path — PR-description bullets, release-notes blurbs,
  commit trailers. Do not manufacture a video pipeline the repo
  doesn't need.

An exact-copy vendored copy is valid only for a repo that is itself the
canonical source. For ordinary repos, either rely on the system-wide skill or
add concrete local demo defaults that earn the checked-in copy.

## Gotchas

- **Every app has a demo path.** The absence of Playwright video,
  Remotion, or `gh release` is a sign that the path is different here,
  not that there is none. `.evidence/<branch>/<date>/` is the default
  durable surface unless the repo declares another.
- **"No demo needed" is a recorded outcome, not a skip.** For
  internal changes, the sentence-in-the-PR-description IS the demo.
  Write it; don't elide it.
- **Default-state evidence proves nothing.** Show the delta, not just
  defaults. A screenshot of an empty page is indistinguishable from
  a screenshot of a broken page.
- **Self-grading is worthless.** On the polished path, the critic
  subagent inspects artifacts cold.
- **Surface ≠ artifact.** A GIF can live in a PR comment, a release,
  or a Slack post; a release-notes blurb can live in a changelog, a
  commit message, or an email. Pick each independently.
- **Under-investing on routine changes is fine.** Over-investing on
  polish where none was asked for is waste. Match fidelity to the
  change, the audience, and the budget — not to your aesthetic.

## Verification

Run `cargo run --locked -p harness-kit-checks -- check-evidence-blocks skills` to prove `/demo` keeps a
local Completion Gate. Semantic demo quality is verified by the produced
artifact path, screenshot, GIF, video, release note, or waiver in the run.
