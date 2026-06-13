# Evidence Gate

Use for public copy, launch pages, demo videos, case studies, decks, and
portfolio pages.

## Evidence Map

Every external-facing assertion gets one status:

| Status | Meaning |
|---|---|
| Proven | Backed by command, route, screenshot, video, CI, dogfood, release, or customer artifact |
| Demonstrated | Backed by deterministic fixture or replay that preserves the real constraint |
| Vision | Explicitly future-facing and not represented as current behavior |
| Remove | Unsupported, inflated, or not worth proving |

## Assertion Scan

Check:

- Hero headline and subhead.
- Feature bullets.
- Screenshot captions.
- Video narration.
- Architecture claims.
- Security/privacy/local-first claims.
- Performance/reliability claims.
- Consulting capability claims.

## Redaction and Privacy

- Public artifacts must not expose secrets, private prompts, private customer
  data, local filesystem paths, API keys, tokens, email addresses, or unrelated
  personal data.
- If local-first/privacy is a product claim, prove what is not captured.
- If using synthetic data, label it in internal evidence. Public copy can say
  "demo data" only when that matters to trust.

## Critic Prompt Shape

```markdown
Role: adversarial public-proof critic.
Objective: find unsupported, misleading, or trust-breaking assertions.
Scope: inspect the asset and the evidence map only.
Output: blockers first, then non-blocking polish findings.
Do not: rewrite the asset unless a finding requires replacement copy.
```
