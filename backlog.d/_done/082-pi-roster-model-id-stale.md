# Fix stale pi roster model id (OpenRouter kimi-k2.6 not found)

Priority: P2
Status: done
Estimate: S
Shipped: 2026-05-29

## Resolution

Pinned `pi` to `moonshotai/kimi-k2.5` (was k2.6). Root cause: k2.6 is absent
from pi's native model registry, so pi routed it as a "custom model id" and
printed a benign `not found for provider openrouter` warning that the groom
session misread as a hard failure (the lane actually answered). k2.5 IS in
pi's registry with thinking + tools + 262K ctx — the only Kimi id pi
recognizes that supports both flags the lane uses. Verified: clean smoke
(transcript ` PI_SMOKE_OK`, no warning, succeeded receipt 69b2f6b3); system
roster updated automatically (it symlinks this file); bootstrap.sh:295-298
also propagates to ~/.harness-kit + legacy. opencode/DeepSeek lane id
(`deepseek/deepseek-v4-pro`) confirmed valid on OpenRouter — no fix needed.
dagger check 15/15. Delegation floor: config-only change grounded in
authoritative sources (OpenRouter API + pi registry + 2 live pi smokes) —
mechanical exception.

## Goal

The `pi` provider lane is dead: `.harness-kit/agents.yaml` pins
`--model moonshotai/kimi-k2.6` (`model: openrouter/moonshotai/kimi-k2.6`), and
OpenRouter rejects it — `Warning: Model "moonshotai/kimi-k2.6" not found for
provider "openrouter". Using custom model id.` — so the lane returns an empty
transcript and silently fails the delegation floor. Point `pi` at a currently
available SOTA open-weight OpenRouter model and verify it dispatches.

## Non-Goals

- Do NOT change other providers' configs.
- Do NOT add retry/fallback logic to the dispatcher — just fix the model id.

## Oracle
- [ ] `pi` `model`/`--model` in `.harness-kit/agents.yaml` resolves on
      OpenRouter (no "not found" warning).
- [ ] A smoke dispatch via `scripts/dispatch-agent.py --provider-target pi`
      with a trivial prompt produces a non-empty transcript and a `succeeded`
      receipt.
- [ ] System roster (`~/.harness-kit/agents.yaml`) updated to match after a
      `bash bootstrap.sh` re-run (or note that bootstrap propagates it).
- [ ] `dagger call check --source=.` green (config-only change).

## Notes

Surfaced live during the 2026-05-29 groom/research session: 3 of 4 provider
lanes (codex, agy, grok-build) succeeded; `pi` returned only the model-not-found
warning. The roster note already says "default to a current SOTA open-weight
lane and revisit the model regularly" — this is that revisit. Confirm the exact
current id against OpenRouter's model list or `pi` model listing before pinning;
do not guess. `opencode` (DeepSeek) is the complementary open-weight lane and
should be smoke-tested in the same pass.
