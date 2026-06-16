# Backlog delivery research sources

Date: 2026-06-16

## Source Matrix

| Source lane | Status | Evidence |
|---|---|---|
| Codebase | complete | Live backlog contains active `105-111`; implementation anchors inspected in `skills/research`, `registry.yaml`, `crates/harness-kit-checks`, shared references, and docs generation. |
| Docs | complete | Exa Agent docs at `https://exa.ai/docs/reference/agent-api-guide` say Agent is async, usage-based, returns text/structured JSON/grounding/cost, supports polling/events, effort modes, and is not ZDR. |
| Retrieval | complete | GitHub API and `git ls-remote` checked Ponytail and VoltAgent metadata. arXiv checked HarnessX paper status. |
| Extraction | complete | Raw Ponytail `skills/ponytail/SKILL.md` fetched from GitHub and inspected. |
| Recency / discourse | skipped | This delivery depends on official docs/repo metadata, not social sentiment. |
| Synthesis | complete | Lead synthesis maps sources to the seven backlog packet oracles. |
| Repo-aware critique | pending | Subagent lanes will critique implementation and doctrine boundaries before ship. |

## Current External Facts

- `DietrichGebert/ponytail`: public, MIT licensed, default branch `main`, observed `main` SHA `99139a25d07e3523d3f6871419798dda600db49a`, observed tag `v4.7.0` SHA `adad50d9b393926b2dd5ed7225dcb1848b9df408`, `stargazers_count` 24017 from GitHub API.
- Ponytail core skill: upstream `skills/ponytail/SKILL.md` defines the ladder: question need, stdlib, native platform, installed dependency, one line, then minimum code; it explicitly says not to simplify away security, validation, data-loss prevention, accessibility, or explicit requirements.
- `VoltAgent/awesome-agent-skills`: public, MIT licensed, homepage `https://officialskills.sh/`, observed `stargazers_count` 25547 from GitHub API, describes itself as a curated 1000+ agent skill collection across official teams and community.
- HarnessX arXiv `2606.14249`: submitted 2026-06-12, frames runtime harnesses as prompts/tools/memory/control flow, reports average gains, and says complete codebase will be open-sourced in a future release.
- Exa Agent docs: official docs describe async runs, structured JSON, grounding citations, cost breakdown, completed-run retrieval, event replay, continuation, fixed effort modes (`minimal`, `low`, `medium`, `high`, `xhigh`, `auto`), and a not-ZDR caveat.

## Residual Risk

- Exa Agent live smoke depends on `EXA_API_KEY` and explicit cost approval; mocked tests remain the deterministic gate.
- GitHub API counts and upstream repos can drift after this timestamp; registry pins are immutable.
