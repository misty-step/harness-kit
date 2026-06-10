---
name: sprites
description: |
  Run lane cards on Fly Sprites: remote, isolated, scale-to-zero sandboxes for
  heavy or parallel agent work. Golden-checkpoint provisioning so lanes start
  on a ready sprite with zero setup tokens. Use when: "run this on a sprite",
  "remote lane", "offload to a sandbox", "dispatch to sprites", "bake a
  sprite", "sprite fleet", heavy/long-running/parallel sub-agent work that
  should not run on this machine. Trigger: /sprites, /sprite-lane.
argument-hint: "[run|bake|status|fetch|reset] [sprite] [--repo --card]"
---

# /sprites

One primitive: `scripts/sprite-lane` runs one lane card on one sprite and
writes a local receipt. No daemon, no fleet state. Both ad-hoc dispatch (an
orchestrator calling this skill) and event-driven intake (a webhook or cron
caller) use the same primitive.

## Route a lane: local subagent or sprite?

| Lane shape | Run it |
|---|---|
| Quick exploration, repo Q&A, small review | Local native subagent |
| Massive parallel fan-out within one session (audits, migrations, cross-checked review) | The harness's own large-scale orchestration feature when it has one — sprites add network overhead for nothing there |
| Heavy build/test loops, big implementation tickets | Sprite |
| Long-running or detached work (hours, overnight) | Sprite, `--detach` |
| Parallel competing attempts on the same repo | One sprite per attempt |
| Anything needing isolation from this machine | Sprite |

Do not prescribe sprites for work a local subagent finishes faster; network
and sync overhead make sprites worse for sub-minute lanes.

## Commands

```bash
scripts/sprite-lane bake <sprite>                 # provision + golden checkpoint (idempotent)
scripts/sprite-lane run <sprite> --repo <owner/name|url> --card <file> \
    [--branch <b>] [--harness codex|claude] [--detach] [--fresh]
scripts/sprite-lane status <sprite>               # bake state + running lanes
scripts/sprite-lane fetch <sprite> <lane-id>      # pull a detached lane's log
scripts/sprite-lane reset <sprite>                # restore golden checkpoint
```

Receipts land in `~/.harness-kit/receipts/sprite-lane/<lane-id>.json` with the
remote log path, exit code, and timing.

## Provisioning model

Sprites cannot fork from another sprite's checkpoint, so determinism comes
from: an idempotent bake (harness CLIs, auth, git identity) plus a per-sprite
**golden checkpoint**. `run` auto-bakes an unbaked sprite; `--fresh` restores
golden first so the lane starts clean. Bumping `BAKE_VERSION` in the script
invalidates stale bakes everywhere. Details, auth flows, and failure modes:
`references/provisioning.md`.

## Lane cards

A lane card states: end state, success criteria, verification affordances,
boundaries, and expected output shape (template:
`templates/lane-card.md`). Big outcome-shaped tickets beat atomic tasks —
the agent on the sprite owns its own decomposition. Put the oracle in the
card; the sprite has no access to your local context.

## Gotchas

- The card is the entire context the remote agent gets. No local files, no
  conversation history. Inline or upload everything the lane needs.
- `--fresh` wipes uncommitted work from previous lanes on that sprite. Fetch
  anything you need first.
- Codex lanes reuse local `~/.codex/auth.json`; if lanes start failing with
  `refresh_token_reused` or `auth_error`, re-run `codex login` locally and
  re-bake.
- Claude lanes need `ANTHROPIC_API_KEY` exported locally at `run` time.
- A checkpoint with `mount_failed` health is unusable; re-bake and cut a new
  golden.
- Scale-to-zero is Fly's job. Do not build keepalives or reconcilers around
  sprites; wake-on-exec is the contract.
