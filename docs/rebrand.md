# Harness Kit Rebrand

## Chosen Name

The project name is **Harness Kit**.

Harness Kit is plain, practical, and accurate: this repo provides reusable
skills, harness projections, bootstrap support, provider-roster conventions,
and local gates for AI-assisted engineering work. It does not imply a fantasy
metaphor, a productized control plane, or a single-vendor runtime.

## Why This Fits

- **Harness** names the job: connect agent runtimes to disciplined workflows.
- **Kit** keeps the scope intentionally small: primitives, references, scripts,
  and gates that operators can compose.
- The name works as a repo name, docs title, package/module prefix, and
  sentence in operator-facing documentation.
- It leaves room for Claude Code, Codex, Antigravity, Pi, and future harnesses
  without making any one provider feel canonical.

## Rejected Names

- **The former name**: memorable, but it keeps the repo anchored to a fantasy
  metaphor and makes static catalogs of "spells" feel natural when the product
  direction is dynamic harness guidance.
- **Agent Kit**: too broad and likely to be confused with agent SDKs or model
  vendor frameworks.
- **Workflow Kit**: accurate but too generic; it hides the cross-harness
  engineering focus.
- **Harness**: strong noun, but too overloaded as a standalone repo/product
  name.

## Migration Contract

Repo-controlled brand, docs, config roots, environment variables, and CI
package names use Harness Kit terminology. New runtime state lives under
`.harness-kit/` in repositories and `~/.harness-kit/` system-wide.

Follow-up migrations should be explicit backlog work with tests, not
opportunistic string replacement.
