# /harness-engineering lint

Validate a skill against quality gates.

## Quality gates

| Gate | Check | Fix |
|------|-------|-----|
| **Description triggers** | Does description include trigger phrases? | Add "Use when:" with concrete phrases |
| **Trigger alias** | Does description include `Trigger:` aliases? | Add explicit slash commands or natural-language aliases; `harness-kit-checks check-frontmatter` enforces this. |
| **Clean category** | Does the skill own one clear category: reference, verification, analysis, process automation, scaffold, review, CI/CD, runbook, or ops? | Split or compose instead of letting one skill straddle unrelated workflows. |
| **Size** | SKILL.md < 500 lines? | Extract to references/ |
| **Gotchas** | Does it enumerate failure modes? | Add a gotchas section |
| **Judgment test** | Does it encode judgment the model lacks? | If not, delete the skill |
| **Oracle** | Can you verify the skill worked? | Add success criteria |
| **Folder utility** | Does the skill use scripts/references/examples/templates/assets when that would prevent repeated reconstruction? | Move boilerplate, schemas, templates, examples, or helper code under the skill folder. |
| **Railroading check** | Does it over-prescribe steps when the repo situation should choose the path? | Replace procedural micromanagement with decision rules, constraints, and oracles. |
| **Freshness** | Do instructions match current model capabilities? | Strip non-load-bearing scaffold |
| **Mode bloat** | >4 modes with inline content, or any single mode >60 lines inline? | Extract mode content to references/mode-*.md; use router pattern (see /diagnose) |
| **Reference integrity** | Do all referenced local files in routing tables, gotchas, and examples exist? | Create the missing file, fix the path, or delete the stale reference |
| **Self-containment** | Do scripts source only paths under `skills/<name>/`? Do they resolve `SCRIPT_DIR` via `readlink -f` and `STATE_ROOT` from the invoking project? | Move shared libs into the skill tree; rewrite source paths to use `$SCRIPT_DIR/lib/…`; decouple state root from script dir. |
| **Delegation guidance** | Where a skill delegates, does it point at the shared Roster contract instead of restating it? | Point to `harnesses/shared/AGENTS.md` (Roster); delete restated doctrine. |

## Self-containment check

The skill must survive being symlinked into a foreign project. Two greps
catch most violations:

```bash
# Scripts that source files outside their own skill tree
rg -n 'source.*\$REPO_ROOT|source.*/scripts/lib/' skills/*/scripts/

# Scripts that walk up past skills/<name>/ via $SCRIPT_DIR/../..
rg -n 'SCRIPT_DIR/\.\./\.\.' skills/*/scripts/
```

Either match is a lint failure. The fix is structural, not a suppression.

Every scripted skill should also ship a distribution smoke test at
`skills/<name>/scripts/distribution_test.sh` that symlinks the skill into
a throwaway project and verifies `--help` works from there.

## Batch lint

Run on all skills: `for s in skills/*/SKILL.md; do /harness-engineering lint "$s"; done`

For duplicate skills, long descriptions, unused candidates, and prompt-budget
pressure, use `harness-kit-checks telemetry`, the registry, and the
first-party route graph; lint checks one skill's shape, not catalog tax.

For skill-design upgrades across the catalog, load
`references/skill-design-principles.md` and apply the gates there before
changing the skill surface.
