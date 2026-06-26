# ai-cli Harness Notes

ai-cli is a direct Vercel AI Gateway generation surface. The npm package is
`ai-cli`; the executable it installs is `ai`. Harness Kit exposes it as the
conditional roster provider target `ai-cli` for model listing plus text, image,
and video generation. It is not a code-editing agent.

Verified facts here were refreshed against `ai-cli@0.3.1` on 2026-06-26.

## Install

Global install from the official docs:

```sh
npm install -g ai-cli
```

Other documented package-manager installs:

```sh
bun add -g ai-cli
pnpm add -g ai-cli
yarn global add ai-cli
```

No-global smoke path, used by the Harness Kit roster to avoid assuming the
binary is installed:

```sh
npx -y ai-cli@0.3.1 --version
npx -y ai-cli@0.3.1 --help
```

Runtime caveat: npm metadata for `ai-cli@0.3.1` declares Node.js `>=20` and the
published binary has a Node shebang. The live installation docs also say ai-cli
runs with Bun. Treat Bun as required for source development/builds unless
upstream clarifies the docs; the published npm CLI smoke passed under Node.

## Configuration and auth

ai-cli has no config file. Use environment variables and flags.

```sh
export AI_GATEWAY_API_KEY="gw_..."
export AI_CLI_TEXT_MODEL="openai/gpt-5.5"
export AI_CLI_IMAGE_MODEL="openai/gpt-image-2"
export AI_CLI_VIDEO_MODEL="bytedance/seedance-2.0"
export AI_CLI_OUTPUT_DIR="$HOME/ai-output"
export AI_CLI_PREVIEW=0
```

Normal generation through the Vercel AI Gateway requires `AI_GATEWAY_API_KEY`.
The docs also mention provider-specific keys such as `OPENAI_API_KEY` and
`ANTHROPIC_API_KEY`, but the verified `0.3.1` generation path uses the Gateway
provider and a no-auth smoke failed with an AI Gateway authentication error.
Prefer `AI_GATEWAY_API_KEY` unless the exact deployed version has separately
verified provider-key behavior.

Flag precedence:

- `-m, --model <id>` overrides `AI_CLI_*_MODEL`.
- `-o, --output <path>` overrides `AI_CLI_OUTPUT_DIR`.
- `--no-preview` overrides `AI_CLI_PREVIEW`.

Do not commit secrets. Put them in shell profile, CI secrets, or a local env
manager.

## Dispatch shape

Harness Kit's roster dispatch is text-only and appends the prompt:

```sh
npx -y ai-cli@0.3.1 text --json --format txt --max-tokens 2048 "Role: critic. Objective: summarize this risk. Output: concise bullets."
```

Equivalent after global install:

```sh
ai text --json --format txt --max-tokens 2048 "Explain this diff."
```

The command stays a thin launch surface. `harness-kit-checks dispatch-agent`
appends the commission, bounds runtime, stores transcript evidence, and records
the receipt.

Use ai-cli when the task needs direct Gateway generation, live model discovery,
or media generation. Do not use it for repo edits, shell work, tests, or
agentic code review; use Codex/Pi/Goose/OpenCode/Claude/Antigravity/Cursor/Grok
for those lanes.

## Direct usage

Text:

```sh
ai text "explain quantum computing"
cat notes.txt | ai text "summarize this"
git diff | ai text --json --format txt "explain these changes"
ai text -m "anthropic/claude-sonnet-4" "hello"
ai text -m "openai/gpt-5.5" --max-tokens 32 --json "Reply OK"
```

Image:

```sh
mkdir -p ./ai-output
ai image "a simple red cube" -o ./ai-output --json --no-preview
ai image "a sunset" -m "openai/gpt-image-1,bfl/flux-2-pro" -n 2 -o ./ai-output --json --no-preview
ai image --image reference.png "make a sticker in this style" -o ./ai-output --json --no-preview
```

Video:

```sh
mkdir -p ./ai-output
ai video "a spinning triangle" -o ./ai-output --json --no-preview
ai video -i input.png "animate this" -o ./ai-output --json --no-preview
```

Model discovery:

```sh
ai models
ai models --type text --json
ai models --creator openai --json
```

## Smoke tests

Install/path smoke, no paid generation:

```sh
ai --version
ai --help
ai text --help
ai image --help
ai video --help
ai models --help
```

No-global equivalent:

```sh
npx -y ai-cli@0.3.1 --version
npx -y ai-cli@0.3.1 text --help
```

Gateway model-list smoke, no generation:

```sh
npx -y ai-cli@0.3.1 models --type text --json > /tmp/ai-cli-models-text.json
node -e 'const m=require("/tmp/ai-cli-models-text.json"); console.log({count:m.length, hasDefault:m.some(x=>x.id==="openai/gpt-5.5")})'
```

End-to-end generation smoke. This makes a real Gateway request and may incur
cost:

```sh
AI_GATEWAY_API_KEY="gw_..." ai text --max-tokens 8 --json "Reply with exactly: OK"
```

Negative auth smoke:

```sh
env -u AI_GATEWAY_API_KEY -u OPENAI_API_KEY -u ANTHROPIC_API_KEY \
  npx -y ai-cli@0.3.1 text --max-tokens 1 --json "Reply OK"
```

Expected failure shape for the negative smoke: exit code 1 with an
"Unauthenticated request to AI Gateway" message asking for `AI_GATEWAY_API_KEY`.

## Troubleshooting

- `ai: command not found`: install globally with `npm install -g ai-cli`, or use
  the pinned `npx -y ai-cli@0.3.1 ...` form.
- Auth failure mentioning AI Gateway: export `AI_GATEWAY_API_KEY`; provider keys
  alone were not verified for `0.3.1` generation.
- Model short name fails or model list is empty: use a fully qualified Gateway
  model id such as `openai/gpt-5.5`; short names depend on live Gateway model
  discovery.
- Binary data floods stdout: for image/video, always pass `-o <path-or-dir>` and
  `--json`; use `--no-preview` or `AI_CLI_PREVIEW=0` in CI/agent contexts.
- Preview glitches in terminal: inline preview depends on terminal graphics
  support. Disable it with `--no-preview` or `AI_CLI_PREVIEW=0`.
- Timeout: current documented timeouts are 120s for text/image and 300s for
  video. Use explicit Harness Kit dispatch timeouts around long media calls.
- Version drift: re-check `npm view ai-cli version` before changing the roster
  pin or documenting new flags.
