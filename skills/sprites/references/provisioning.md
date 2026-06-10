# Sprite provisioning: golden-checkpoint pattern

Operational knowledge distilled from the bitterblossom conductor
(`conductor/lib/conductor/sprite.ex`, `launcher.ex`) and the Sprites platform
docs. The conductor is mothballed; this file is the surviving record.

## Platform facts

- Sprites are Ubuntu sandboxes with a writable overlay, persistent filesystem,
  and scale-to-zero (~30 s idle). Wake-on-exec; cold exec takes a few seconds.
- Checkpoints are copy-on-write filesystem snapshots: creation is milliseconds,
  restore is in-place. **There is no cross-sprite fork or base image** (top
  community wishlist item; sprites-docs issue #137). When forking ships,
  replace per-sprite bakes with one template sprite + forks.
- `sprite exec --file src:dest` uploads before exec; repeatable.
- The sprite user is `sprite`; home is `/home/sprite`.
- `--http-post` is the fallback transport when websocket exec misbehaves.

## Bake contents (BAKE_VERSION 1)

Idempotent; every step checks before acting:

1. node + git via apt if missing.
2. `@openai/codex` and `@anthropic-ai/claude-code` global installs.
3. `~/.codex/config.toml`: `cli_auth_credentials_store = "file"` — without
   this, codex on a headless sprite cannot use the uploaded `auth.json`.
4. Upload local `~/.codex/auth.json` → `/home/sprite/.codex/auth.json`.
5. Git identity + `credential.helper store`; GitHub token (from local
   `gh auth token`) written to `~/.git-credentials` for private clones.
6. Marker file `~/.sprite-lane-golden` containing the bake version.
7. Checkpoint with comment `sprite-lane golden v<N>`.

## Auth failure modes (from bitterblossom production)

- `refresh_token_reused` / `auth_error` in lane logs: the local and sprite
  copies of `auth.json` have diverged (token rotated on one side). Fix:
  `codex login` locally, then re-bake. Bitterblossom's conductor auto-detected
  these patterns before each launch; if lanes are run unattended, grep the
  lane log for them before retrying.
- Claude: no credential file sync in v1. Pass `ANTHROPIC_API_KEY` through the
  `run` environment. OAuth credential sync (`/home/sprite/.claude/`) is
  possible but rotation makes it fragile; API key is the stable path.

## Lifecycle lessons inherited from the conductor

- **Truthful state over assumed state**: always check the marker file, never
  trust that a sprite "should" be baked. Half-baked sprites converge on
  re-bake instead of erroring.
- **Preflight kill**: before reusing a sprite for a detached lane, check for
  stale pids (`lanes/*/pid`) and kill process groups, not single pids
  (`kill -- -<pgid>`), then `kill -9` survivors.
- **Detached starts**: `setsid nohup ... & echo $! > pid` is sufficient;
  conductor's supervised loops added nothing the receipts didn't already
  capture.
- **Repo sync**: `fetch + reset --hard + clean -fd` on reuse beats re-clone
  (sprites keep their overlay between wakes); `--fresh` exists for when you
  want the clean-room instead.

## Cost model

Scale-to-zero makes warm pools nearly free: a stopped sprite costs storage
only. Fly's estimate is ~$0.46 for a 4-hour intensive coding session. Prefer
a small set of named, baked sprites (e.g. `lane-1`..`lane-4`) reused across
lanes over creating/destroying per lane — bakes cost minutes, restores cost
milliseconds.
