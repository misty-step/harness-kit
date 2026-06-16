---
name: oracle
description: |
  Browser-mode-only Oracle consults: bundle a prompt plus selected files and ask
  a signed-in ChatGPT GPT-5.5 Pro browser session for a second opinion. Use
  when stuck, debugging hard bugs, reviewing an architecture plan, or
  cross-checking a substantive diff with large file context. Never use Oracle
  API mode from Harness Kit. Trigger: /oracle, /consult.
argument-hint: "[prompt + file globs]"
---

# /oracle

Use Oracle only as a browser-backed consult surface. It bundles the prompt and
selected files, opens a signed-in ChatGPT browser session, and stores the
answer under `~/.oracle/sessions`. The lead agent still owns synthesis,
verification, and final judgment.

## Hard Boundary

- Always pass `--engine browser`.
- Prefer `--model gpt-5.5-pro` unless the operator names another browser model.
- Never run `--engine api`, `--provider openai`, `--no-azure`, `--models`, or
  API preflight from this skill.
- If an Oracle command would need an API key or per-token billing, do not use
  Oracle for that work from Harness Kit; use the normal roster/research paths
  instead.

## Workflow

1. Pick the tightest file set that contains the truth. Exclude tests, snapshots,
   generated output, and fixtures unless they are the oracle.
2. Preview first:

   ```sh
   npx -y @steipete/oracle --engine browser --model gpt-5.5-pro \
     --dry-run summary --files-report \
     -p "<task>" --file "src/**" --file "!**/*.test.*"
   ```

3. Run only after the preview is sensible:

   ```sh
   npx -y @steipete/oracle --engine browser --model gpt-5.5-pro \
     --slug "<3-5-words>" \
     -p "<task>" --file "src/**" --file "!**/*.test.*"
   ```

4. If the run detaches or times out, do not re-run. Inspect the stored session:

   ```sh
   npx -y @steipete/oracle status --hours 72
   npx -y @steipete/oracle session <id> --render
   ```

## Good Prompts

Oracle starts cold. Include the repo goal, relevant commands, exact failure,
constraints, and desired output shape. For review, give the artifact and oracle,
not the author's reasoning trail.

Good output requests:

- `Return blockers only, with file/line evidence.`
- `Challenge this plan; list missing proof, wrong assumptions, and safer alternatives.`
- `Find the likely root cause and the smallest verification loop.`

## Safety

- Do not attach secrets, `.env`, key files, tokens, browser profiles, or private
  customer data.
- Use `--dry-run summary --files-report` before every real run.
- Treat Oracle output as advisory evidence. Verify with the repo's live oracle,
  tests, gate, browser route, or command before claiming done.
- Browser mode is cheaper than API billing but can be slow and subject to
  ChatGPT subscription limits.
