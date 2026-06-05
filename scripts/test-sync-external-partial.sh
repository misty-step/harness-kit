#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/repo/scripts" "$TMP/repo/skills/.external/keep-me" "$TMP/bin"
cp "$ROOT/scripts/sync-external.sh" "$TMP/repo/scripts/sync-external.sh"

cat > "$TMP/repo/skills/.external/keep-me/SKILL.md" <<'DOC'
---
name: keep-me
description: Existing external skill that partial sync must preserve.
---

# Keep Me
DOC

cat > "$TMP/repo/registry.yaml" <<'DOC'
sources:
  - repo: example/keep
    ref: main
    pin: a111111111111111111111111111111111111111
    skills_path: skills
    alias_prefix: keep-
    include: [keep-me]
  - repo: example/new
    ref: main
    pin: b222222222222222222222222222222222222222
    skills_path: skills
    alias_prefix: new-
    include: [new-skill]
DOC

cat > "$TMP/bin/git" <<'SH'
#!/usr/bin/env bash
set -euo pipefail

cmd="$1"
shift

case "$cmd" in
  clone)
    dest="${@: -1}"
    mkdir -p "$dest/.git" "$dest/skills/new-skill"
    cat > "$dest/skills/new-skill/SKILL.md" <<'DOC'
---
name: new-skill
description: New external skill fixture.
---

# New Skill
DOC
    ;;
  -C)
    dir="$1"
    shift
    sub="$1"
    shift
    case "$sub" in
      sparse-checkout)
        exit 0
        ;;
      ls-remote)
        ref="${@: -1}"
        case "$ref" in
          *b222222222222222222222222222222222222222*)
            printf '%s\t%s\n' "b222222222222222222222222222222222222222" "$ref"
            ;;
          *a111111111111111111111111111111111111111*)
            printf '%s\t%s\n' "a111111111111111111111111111111111111111" "$ref"
            ;;
        esac
        ;;
      fetch|checkout)
        exit 0
        ;;
      *)
        echo "unexpected git -C $dir $sub" >&2
        exit 2
        ;;
    esac
    ;;
  *)
    echo "unexpected git $cmd" >&2
    exit 2
    ;;
esac
SH
chmod +x "$TMP/bin/git"

PATH="$TMP/bin:$PATH" bash "$TMP/repo/scripts/sync-external.sh" --only example/new > "$TMP/sync.out"

if [ ! -f "$TMP/repo/skills/.external/keep-me/SKILL.md" ]; then
  echo "partial sync removed unrelated external skill" >&2
  cat "$TMP/sync.out" >&2
  exit 1
fi

if [ ! -f "$TMP/repo/skills/.external/new-new-skill/SKILL.md" ]; then
  echo "partial sync did not install requested external skill" >&2
  cat "$TMP/sync.out" >&2
  exit 1
fi

grep -q "partial sync: skipping global orphan cleanup" "$TMP/sync.out" || {
  echo "partial sync should report skipped global orphan cleanup" >&2
  cat "$TMP/sync.out" >&2
  exit 1
}

echo "sync-external partial sync preserves unrelated externals"
