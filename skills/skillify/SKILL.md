---
name: skillify
description: |
  Turn proven agent-session patterns into first-party Harness Kit skills.
  Use when: "skillify this conversation", "make this into a skill",
  "generate a skill from current transcript", "extract reusable workflow".
  Trigger: /skillify.
argument-hint: "[--from-current] [--transcript <claude-jsonl>] [--name <skill-name>]"
---

# /skillify

Convert an agent conversation into a durable skill when the behavior is novel,
repeatable, and portable across harnesses.

## Scope

MVP supports Claude Code JSONL transcripts and `--from-current` style local
transcript extraction. Gemini, Codex batch ingestion, hosted transcript stores,
and automatic publication are future work.

## Delegation Floor

Delegation floor applies for novelty classification, portability critique, and
new skill design: probe the roster first; dispatch two or more providers;
direct solo only for deterministic parser/CRUD commands, emergency
preservation, user-forbidden delegation, or fewer-than-two-providers cases. See
`harnesses/shared/AGENTS.md` (Roster).

Local lane guidance: Use specialized lanes for transcript-pattern mining,
skill-design critique, portability/security review, and generated-skill
validator. Native in-thread subagents may supplement but do not satisfy the
roster floor.

## Workflow

1. Parse the transcript with
   `cargo run --locked -p harness-kit-checks -- skillify-parse-transcript`.
2. Classify novelty and repeatability with
   `cargo run --locked -p harness-kit-checks -- skillify-classify`.
   It dispatches two or more roster providers through the existing
   `harness-kit-checks dispatch-agent` boundary and records delegation
   receipts.
3. Create or update the candidate skill with
   `cargo run --locked -p harness-kit-checks -- skillify-skill-crud`.
4. Validate frontmatter, portability, and generated skill shape before
   bootstrap.
5. Run `cargo run --locked -p harness-kit-checks -- bootstrap` so the
   first-party skill catalog propagates to active harnesses.

## Portability Contract

Generated skills must be filesystem-first and cross-harness. Reject content
that depends on harness-private operations such as `SendUserMessage`, direct
tool names like `Edit`, or raw `bash` instructions without a fallback path.
Use the frontmatter schema in `references/frontmatter-schema.md`.

## Completion Gate

- Exact operator behavior changed: reusable transcript-to-skill primitive exists.
- Evidence that proves it: parser output, CRUD validation, classification
  receipt ids, bootstrap result, and Dagger gate.
- Exact command/path/route exercised: skillify Rust CLI commands, bootstrap, and Dagger:
  `cargo run --locked -p harness-kit-checks -- skillify-parse-transcript`,
  `cargo run --locked -p harness-kit-checks -- skillify-classify`,
  `cargo run --locked -p harness-kit-checks -- skillify-skill-crud`,
  `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`,
  `cargo run --locked -p harness-kit-checks -- bootstrap`,
  `dagger call check --source=.`
- Repo-fit check: self-contained first-party skill under `skills/skillify/`.
- Residual risk: Claude JSONL variants outside the MVP remain future work.

## Verification

Run `cargo run --locked -p harness-kit-checks -- check-frontmatter --repo .`
after CRUD output and `bash bootstrap.sh` before shipment; the generated skill
must also pass
`cargo run --locked -p harness-kit-checks -- skillify-skill-crud validate --name <name>`.
