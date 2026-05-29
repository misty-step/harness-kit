# Fix stale pi roster model id (OpenRouter kimi-k2.6 not found)

Priority: P2
Status: ready
Estimate: S

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
