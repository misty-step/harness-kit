---
name: human-writing
description: |
  Edit, audit, or rewrite prose so it sounds like a specific human wrote it,
  not a generic AI draft. Removes AI tells, filler, formulaic structure,
  fake polish, vague claims, and detector-bait phrasing while preserving truth,
  voice, and audience fit. Use when: "humanize this", "make this sound less
  AI", "remove AI slop", "de-slop this", "edit this prose", "make this
  sound natural", "fix the writing voice", "rewrite this copy". Trigger:
  /human-writing, /deslop.
argument-hint: "[text|file|draft] [--audit|--rewrite|--in-place] [--voice terse|warm|technical|plain|blunt]"
---

# /human-writing

Make prose sound authored, not generated. Preserve meaning and attribution.
Do not optimize for bypassing detectors or misrepresent authorship.

## Route

| Need | Action |
|---|---|
| Quick rewrite in chat | Rewrite directly; include a short change note. |
| File edit | Patch the file; preserve frontmatter, links, claims, code, and citations. |
| Audit only | Return findings ranked by severity with examples and fixes. |
| Public product copy | Compose with `/showcase` for evidence-gated claims. |
| UI copy | Compose with `/design`; visible copy must match the interface job. |
| Deep pattern check | Read `references/patterns.md`. |

## Contract

- Keep the author's claim, audience, and constraints intact.
- Cut filler before changing voice.
- Replace vague importance with specific evidence or delete it.
- Name actors where the draft hides them.
- Vary rhythm without adding theatrics.
- Keep technical terms, legal terms, quotes, citations, commands, and API names
  exact unless the user asks to change them.
- When facts are weak, flag them. Do not invent concrete details to sound
  human.

## Edit Pass

1. Identify the job: publishable copy, email, docs, report, social post,
   internal note, or UI text.
2. Infer or ask for voice only when it changes the edit. Default to plain,
   direct, and domain-specific.
3. Mark the top 3 AI tells: filler phrase, formulaic contrast, vague claim,
   false agency, over-polish, rhythm monotony, or generic enthusiasm.
4. Rewrite in the target voice. Preserve facts first, style second.
5. Run the final check: fewer words, clearer actors, no fake specificity, no
   detector-bypass framing.

## Severity

| Level | Meaning |
|---|---|
| P0 | Misleading, invented, overclaiming, or authorship/detector evasion risk. |
| P1 | Strong AI tell that hurts trust or publishability. |
| P2 | Style drag: filler, rhythm, jargon, or excess polish. |

## Output

For rewrites:

```markdown
[rewritten text]

Notes: cut <pattern>; preserved <claim/source>; residual risk <if any>.
```

For audits:

```markdown
Findings
- P1 <pattern>: "<short excerpt>" -> <fix>

Verdict: publish / revise / needs source.
```

## Gotchas

- **Detector laundering.** If the user asks to bypass GPTZero, Turnitin,
  Originality.ai, or similar detectors, reframe to honest editorial quality.
- **Fake human detail.** Do not add anecdotes, names, dates, citations, or
  sensory detail that the source does not support.
- **Voice flattening.** "Professional" often becomes generic. Preserve
  domain vocabulary and the writer's useful oddities.
- **Over-cutting.** Some passive voice and repetition are correct in legal,
  scientific, compliance, and API docs. Fix the writing job, not a checklist.
- **One-note bluntness.** Direct prose can still be kind, precise, or
  technical. Do not turn every draft into a hardboiled memo.

## Completion

Name what changed: patterns removed, claims preserved, any weak facts flagged,
and whether the result is ready to publish or needs source review.
