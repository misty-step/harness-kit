# End-to-End Offline Validation

This is the manual proof path for Spellbook's git-native, offline-first
lifecycle. It is intentionally one protocol, not a platform and not a CI gate.

Use a disposable clone or worktree. The merge-gate step can create a real merge
commit when it succeeds.

## What This Proves

- A feature branch can be created and changed locally.
- Dagger can run from warm local cache.
- Evidence lands under `.evidence/<branch-slug>/<date>/`.
- Review proof can be stored and validated as a git ref.
- Non-fast-forward merge into `master` is blocked unless the pre-merge hook's
  Dagger check and verdict check pass.
- Network-bound pieces have explicit offline fallbacks.

## Online Warm-Up

Do this before disabling the network:

```sh
scripts/offline-validation-preflight.sh
DAGGER_NO_NAG=1 dagger call check --source=.
docker image ls 'registry.dagger.io/engine'
```

Dagger caches container layers, volumes, and function-call results. The Dagger
runner pulls images and repositories and manages its cache; the offline run is
only honest after the runner, base images, source, and function-call cache have
been warmed. When using a custom runner image, the image is
`registry.dagger.io/engine:<version>` and it still needs a local container
runtime plus a persistent volume/cache.

If warm-up cannot complete, stop here. The result is "offline validation
blocked: cache/runtime not ready", not "offline validation failed."

## Offline Run

Disable network access, then run the sequence below.

### 1. Create a Feature Branch

```sh
git checkout master
git checkout -b feat/027-offline-validation-probe
```

Make a local-only change. Keep it trivial and reversible:

```sh
printf 'offline validation probe\n' > meta/offline-validation-probe.txt
git add meta/offline-validation-probe.txt
git commit -m "test: offline validation probe"
```

### 2. Create Local Evidence

```sh
source scripts/lib/evidence.sh
EVIDENCE_DIR="$(evidence_mkdir)"
printf 'offline validation evidence\n' > "$EVIDENCE_DIR/qa-report.md"
```

### 3. Run Dagger From Warm Cache

```sh
DAGGER_NO_NAG=1 dagger call check --source=. 2>&1 \
  | tee "$EVIDENCE_DIR/dagger-check.txt"
```

If this tries to pull images or fetch remote repositories, the cache is not warm
enough. Re-enable the network, warm the missing dependency, and restart the
offline proof from a clean branch.

### 4. Write Review Evidence and Verdict Ref

When multi-provider review is unavailable offline, use a single local
fresh-context review and record that limitation in evidence:

```sh
cat > "$EVIDENCE_DIR/review-synthesis.md" <<'EOF'
# Review Synthesis

Mode: offline single-reviewer fallback
Verdict: ship
Limitations: multi-provider Thinktank/Codex/Gemini review unavailable offline.
EOF
```

Write the verdict ref:

```sh
source scripts/lib/verdicts.sh
branch="$(git rev-parse --abbrev-ref HEAD)"
git add "$EVIDENCE_DIR"
git commit -m "test: record offline evidence"
sha="$(git rev-parse HEAD)"
json="$(python3 - <<'PY' "$branch" "$sha"
import json, sys
branch, sha = sys.argv[1], sys.argv[2]
print(json.dumps({
    "branch": branch,
    "base": "master",
    "verdict": "ship",
    "reviewers": ["offline-local"],
    "scores": {"correctness": 8, "depth": 6, "simplicity": 8, "craft": 8},
    "sha": sha,
    "date": "offline-run"
}))
PY
)"
verdict_write "$branch" "$json"
verdict_validate "$branch"
verdict_read "$branch" > "$EVIDENCE_DIR/verdict.json"
```

Do not commit after `verdict_write` unless you immediately re-write the verdict
for the new `HEAD`. The ref is the authority; `verdict.json` is a browsable
copy for the local evidence directory.

### 5. Exercise the Pre-Merge Gate

Use a scratch clone/worktree if you do not want a real merge commit:

```sh
git checkout master
git merge --no-ff feat/027-offline-validation-probe
```

Expected behavior:

- The hook runs `dagger call check --source=.`.
- If Dagger fails, the merge is blocked.
- If Dagger passes but the verdict ref is missing, stale, or `dont-ship`, the
  merge is blocked.
- `SPELLBOOK_NO_REVIEW=1` bypasses only the verdict check; Dagger still runs.

### 6. Record the Outcome

Append the result to the evidence directory before keeping or discarding the
probe branch:

```sh
cat > "$EVIDENCE_DIR/offline-result.md" <<'EOF'
# Offline Validation Result

- Dagger from warm cache: pass/fail
- Evidence path: .evidence/<branch-slug>/<date>/
- Verdict ref validation: pass/fail
- Pre-merge hook: pass/fail
- Offline fallbacks used:
EOF
git add "$EVIDENCE_DIR/offline-result.md"
```

## Offline Boundaries and Fallbacks

| Component | Offline Reality | Fallback |
|---|---|---|
| Dagger | Works only after runner/base images/repos/function calls are warm in local cache. | Warm online first; if a pull occurs offline, mark validation blocked and record the missing dependency. |
| Dagger custom runner | `registry.dagger.io/engine:<version>` requires a local runtime and persistent volume/cache. | Pre-pull the runner image and preserve the cache volume before airplane mode. |
| Multi-provider review | Thinktank, Codex, Gemini, and remote model providers may require network. | Run one local/fresh-context review, write `review-synthesis.md`, and set verdict reviewer to `offline-local`. |
| GitHub PR checks/comments/releases | Network-bound and not required for this proof. | Use `.evidence/` and verdict refs. Publish later if useful. |
| git-bug or remote issue sync | May require installed tooling and/or remote sync. | Record issue/follow-up notes in `.evidence/` or `backlog.d/`; do not claim remote issue filing happened. |
| Git LFS hydration | Pointer files survive clone; full binary retrieval needs an LFS remote unless already cached. | For offline proof, verify pointer presence and keep text evidence sufficient. |

## Honest Outcomes

- **Pass:** All offline steps complete after warm-up, including Dagger, evidence,
  verdict ref, and pre-merge hook.
- **Blocked:** A required cache/runtime/tool was not available offline.
- **Partial:** Dagger and git-native pieces pass, but review/issue/provider
  systems needed fallbacks. Record the fallback, do not hide it.
