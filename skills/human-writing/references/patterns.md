# Human Writing Patterns

Use this when a draft still feels machine-made after a first pass.

Source scan: distilled from the MIT-licensed `hardikpandya/stop-slop`,
`humanizerai/agent-skills`, `stephenturner/skill-deslop`,
`conorbronsdon/avoid-ai-writing`, and existing Harness Kit design/showcase copy
rules. Do not copy external phrasing wholesale.

## Pattern Families

| Pattern | Symptom | Better move |
|---|---|---|
| Throat-clearing | The sentence announces the point before making it. | Start with the claim. |
| Formulaic contrast | The draft says "not X, but Y" or stages a reveal. | State Y directly. |
| Vague importance | "This matters" without naming who cares or what changes. | Name the consequence. |
| False agency | Data, markets, decisions, or systems act like people. | Name the person, team, or process. |
| Generic enthusiasm | Polished praise with no grounded observation. | Tie praise to a concrete property. |
| Jargon stack | Strategy words replace plain verbs. | Use the verb a person would say. |
| Rhythm monotony | Equal-length sentences, tidy triples, repeated paragraph endings. | Vary length and break the list shape. |
| Quote bait | A line sounds written for a screenshot. | Make it useful, not quotable. |
| Meta narration | The text explains its own structure. | Let the structure carry itself. |
| Hedged authority | "Often", "may", "can", and softeners dodge the claim. | Either make the claim or show uncertainty precisely. |

## Rewrite Tactics

- Start later. Most AI drafts spend the first sentence warming up.
- Cut summary sentences that repeat the paragraph.
- Replace abstractions with the object, person, command, route, date, or
  decision the reader can inspect.
- Prefer one strong example over three weak examples.
- Prefer short plain transitions over stylized pivots.
- Keep a useful imperfect sentence over a polished generic one.

## Context Rules

| Context | Rule |
|---|---|
| Docs | Preserve command names, config keys, warnings, and prerequisites. |
| Technical report | Keep caveats that affect correctness; cut caveats that only protect tone. |
| Marketing copy | Tie every claim to evidence, demo, customer outcome, or product surface. |
| Email | Keep the ask clear and the reason specific to the recipient. |
| UI copy | Match the user's next action; avoid explaining implementation or process. |
| Academic or compliance prose | Do not help hide AI use. Improve clarity and attribution. |

## Final Check

Score 1 to 5:

- Directness: the text states claims instead of announcing them.
- Specificity: claims name concrete subjects, actions, and evidence.
- Rhythm: sentence and paragraph shape varies without theatrics.
- Voice: the draft sounds like the intended writer or institution.
- Truth: the edit did not add unsupported detail or weaken required caveats.

Below 18/25: revise again or return an audit instead of a rewrite.
