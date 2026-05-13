---
name: qa
description: |
  Verify a running app works — every app has a QA path; the right shape
  depends on the app. Browser walks for web apps, request-replay for APIs,
  shell smoke for CLIs, consumer-build checks for libraries, tool-call
  replay for MCP servers. Drive the running thing and confirm it behaves;
  "tests pass" is not QA. Use when: "run QA", "test this", "verify the
  feature", "exploratory test", "check the app", "QA this PR", "smoke
  test", "capture evidence", "manual testing", "scaffold qa",
  "generate qa skill". Trigger: /qa.
argument-hint: "[url|route|command|endpoint|feature|scaffold]"
---

# /qa

**Every app has a QA path.** The question this skill answers first is not
"how do I drive a browser?" — it's "what shape is this app, and what does
verifying it actually look like here?" A CLI is QA'd with shell
invocations and exit-code audits. An API is QA'd by replaying requests
against a preview deploy. A Next.js app is QA'd by walking golden paths
in a browser. A library is QA'd by installing it into a sandbox consumer.
An MCP server is QA'd by replaying tool calls through the harness that
registered it. None of these are optional; all of them are QA.

The skill's job is to route to the right shape, then either defer to a
project-local QA skill that encodes this repo's actual paths, or run a
quick protocol on the fly.

## Execution Stance

You are the executive orchestrator.
- Keep test scope, severity classification, and the final pass/fail call
  on the lead model.
- Delegate drive-the-app execution and evidence capture to focused
  subagents.
- When the same agent both executed and judged, dispatch an independent
  verifier before signing off on "pass".

## Step 0: Identify the app's shape

Before anything else, answer: **what kind of app is this, and what does
"verify it works" look like for that kind?** The shape determines the
path; the path determines the tools.

Signals to read (any of these; stop when you have enough):

- `.spellbook/repo-brief.md` — if present, its "Stack & boundaries"
  section is the spine. Use it.
- `package.json` — `"bin"` field → CLI. `"main"` + no bin + no
  framework deps → library. `next` / `remix` / `astro` / `vite` +
  framework routes → browser web app. `express` / `fastify` /
  `hono` + no SSR pages → API service.
- `playwright.config.*`, `cypress.config.*` → browser harness already
  wired; use it when the path is browser.
- `mcp/`, `servers/mcp/`, `@modelcontextprotocol/*` in deps → MCP server.
- `Dockerfile`, `fly.*.toml`, `vercel.json`, `.github/workflows/*deploy*`
  → deploy target hints; helps pick "which preview do I hit?"
- `Cargo.toml` (`[[bin]]` vs `[lib]`), `pyproject.toml` (`scripts` vs
  package), `go.mod` (`cmd/` tree), etc., for non-JS stacks.

Map the shape to the path:

| App shape | QA path |
|---|---|
| Browser web app (Next.js, Remix, SvelteKit, SPA) | Start dev server; walk golden paths; scan console errors; scan network panel for 4xx/5xx; optionally Playwright if `playwright.config.*` exists |
| API / serverless / backend service | Replay representative requests (`curl`, HTTPie, `.http` file, Postman export) against a preview deploy or local server; spot-check JSON contract; enumerate error statuses; verify auth paths |
| CLI | Shell-driven smoke: `--help`, key invocations with representative flags, malformed-input paths; audit exit codes; audit error messages for clarity |
| Library / SDK | Install into a sandbox consumer project (`npm pack && npm i <tarball>`, `cargo add --path`, `pip install -e .`); exercise the public API; check type surface; verify no runtime import of dev-only deps |
| MCP server / agent tool | Register with the harness (`claude mcp add` / Codex `/plugins`); replay tool calls; inspect responses; confirm error paths return structured failures, not crashes |
| Hybrid (e.g. Next.js app + MCP server + CLI) | Pick the path per surface touched by the change; do not pretend one path covers all |

If the shape is ambiguous, name both candidates and ask — do not silently
pick one.

## Routing

| Intent | Action |
|---|---|
| `"scaffold"` first arg, or "scaffold qa" / "generate qa skill" | Read `references/scaffold.md` and follow it |
| Project-local QA skill exists (`.agent/skills/qa/` or `.claude/skills/qa/` bridged to shared root) | Defer — it already encodes this repo's shape and paths |
| No project-local skill; need to verify something right now | Run the quick protocol below, routed by Step 0's shape |

## Quick One-Off QA (no scaffold)

Shaped by Step 0. Pick the matching sub-protocol.

### If the app is a browser web app

1. Start the dev server (or hit a preview URL).
2. Navigate to the affected routes.
3. Verify in order: happy path, edge cases, console errors, network
   panel failures.
4. Capture evidence to `/tmp/qa-{slug}/` — screenshots on anomaly,
   accessibility snapshot on ambiguity.
5. Classify findings: P0 (blocks ship), P1 (fix before merge),
   P2 (log and move).

For browser tool selection (Playwright MCP, Chrome MCP, agent-browser),
read `references/browser-tools.md`. For evidence capture conventions
across tools, read `references/evidence-capture.md`.

### If the app is an API / backend service

1. Identify the target: local server, preview URL, or staging.
2. Replay the representative request set (from `.http` files, Postman
   collection, README examples, or the endpoint list you mapped).
3. For each: check status code, response shape against the documented
   contract, and error-path behavior (bad auth, missing field,
   malformed body).
4. Capture evidence to `/tmp/qa-{slug}/`: request/response pairs as
   `.json` or `.http` transcripts, plus a short findings note.
5. Classify P0/P1/P2.

### If the app is a CLI

1. Run `<bin> --help` / `<bin> <subcommand> --help` — confirm help
   text matches the code's actual flags.
2. Exercise the happy-path invocations from README / docs.
3. Exercise malformed-input paths: missing required args, bad flag
   values, nonexistent files. Audit exit codes (should be non-zero)
   and error messages (should name the problem).
4. Capture evidence to `/tmp/qa-{slug}/`: terminal transcripts
   (`script -q` or tee'd stdout/stderr).
5. Classify P0/P1/P2.

### If the app is a library / SDK

1. Build the distributable (`npm pack`, `cargo build --release`,
   `python -m build`, etc.).
2. Install into a throwaway consumer project (`/tmp/qa-{slug}/consumer`).
3. Import the public API; call the entry points the change touched.
4. Check the type surface (TypeScript: `tsc --noEmit` in the consumer;
   Rust: consumer `cargo check`; Python: `mypy` against a stub).
5. Verify no runtime import of dev-only deps (inspect the tarball /
   built artifact if in doubt).
6. Classify P0/P1/P2.

### If the app is an MCP server / agent tool

1. Register the server with a harness (`claude mcp add <name> <cmd>`
   for Claude Code; equivalent `/plugins` flow for Codex).
2. From an agent session, invoke each affected tool with a
   representative payload.
3. Verify: success responses match the documented schema; error
   responses are structured (not thrown exceptions that kill the
   server); the server survives a malformed request.
4. Capture evidence to `/tmp/qa-{slug}/`: tool-call transcripts
   (input JSON, output JSON, server logs).
5. Classify P0/P1/P2.

## Tailoring guidance (for `/tailor` rewriters)

When `/tailor` rewrites this skill for a specific repo, the rewriter's
job is **not** to copy the full shape matrix above. It is to **name the
single QA path that applies to THIS codebase**, with the actual commands,
URLs, route list, endpoint list, or tool-call list embedded.

- If this repo is a Next.js app, the rewrite's body is browser-walk
  instructions with the actual golden-path routes named, the actual
  dev-server command, and the actual Playwright config path (if one
  exists). The other shapes disappear from the body.
- If this repo is a CLI, the rewrite's body is shell-smoke
  instructions with the actual binary name, the actual subcommand
  list, and the actual exit-code contract named. The other shapes
  disappear.
- If this repo is a hybrid (CLI + library, app + MCP server), the
  rewrite enumerates the paths that surface in this repo — **each
  one fully-concrete** — and names when to run which.

**Do not read "no `playwright.config.*` in the repo" as "skip QA."**
That reading is the canonical Norman-bug in this skill's history. The
correct reading is: "Playwright isn't the QA path here; name the one
that is." Every repo has a QA path. Find it. Write it. If you cannot
name the path, that is a signal to ask the user, not to ship a generic
rewrite.

## Gotchas

- **"Tests pass" is not QA.** Unit and integration tests verify code
  paths the author thought of. QA verifies the running app against the
  user's actual experience — including the paths no test covers.
- **Shape first, tools second.** Reaching for Playwright before
  confirming the app is browser-shaped is how this skill drifted
  into its old browser-only framing. Always answer Step 0 first.
- **This generic fallback is intentionally thin.** It cannot encode
  your app's routes, endpoints, CLI flags, tool contracts, or
  failure modes. Scaffold a project-local QA skill for durable
  coverage: `/qa scaffold`.
- **`/deliver` expects a scaffolded skill.** If `/deliver` invokes
  `/qa` and lands on this generic fallback, scaffold before
  continuing — the delivery gate wants repo-specific coverage.
- **Independent verification matters most when the same agent drove
  the app and judged it.** Dispatch a fresh-context verifier for any
  "pass" verdict that would otherwise be self-reported.

## References

- `references/scaffold.md` — project-local QA skill generator.
- `references/browser-tools.md` — long-form browser-automation guide;
  consulted only when Step 0 resolves to "browser web app".
- `references/evidence-capture.md` — cross-tool evidence conventions
  (screenshots, transcripts, GIFs, network logs).
