# Demoability Audit

Use before building public assets. The question is not "can we market it?" It
is "can a serious outsider see a real product moment and trust it?"

## Read First

- README, product docs, screenshots, demos, release notes.
- The repo's main run command, gate, and QA evidence.
- Existing design docs or product principles.
- Any dogfood, CI, or production evidence the public story might cite.

## Classify

| Area | Pass signal | Common failure |
|---|---|---|
| Product truth | One clear job gets done end to end | Many features, no consequential scenario |
| Demo path | Command/route/fixture recreates the state | Manual setup lives in the operator's head |
| Evidence | Claims point to artifacts | Copy outruns what the app can prove |
| First screen | Product category and value are obvious | Internal project name plus vague AI copy |
| Visual polish | Screens look intentional and domain-fit | Generic SaaS cards, gradients, fake metrics |
| Buyer fit | Audience and next action are explicit | Portfolio piece reads like a dev diary |
| Reset | Demo can be rerun from clean state | Stale local state makes screenshots special |

## Verdict Shape

```markdown
## Demoability Audit
- Current verdict: demoable / nearly demoable / not yet demoable
- Strongest proof:
- Weakest public promise:
- Proof gap:
- Demo gap:
- Polish gap:
- Story gap:
- Smallest next slice:
- Verification path:
```

## Stop Conditions

- No runnable surface and no replayable artifact.
- Product's strongest claim depends on private local state that cannot be
  sanitized or reproduced.
- The visible product moment contradicts the story.
- The next action is "make a site" but no demo state exists.
