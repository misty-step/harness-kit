#!/usr/bin/env bash
set -euo pipefail

# Spellbook External-Skill Sync
#
# Reads registry.yaml, fetches each declared external source at a pinned ref,
# installs selected skills into skills/.external/<alias>/.
#
# Idempotent: running twice produces the same filesystem state.
# Fail-loud: malformed registry, unreachable source, or floating ref (without
# --allow-floating / allow_floating: true) aborts non-zero.
# Orphan GC: any directory under skills/.external/ not declared in the registry
# is removed.
#
# Storage:
#   skills/.external/_checkouts/<org>__<repo>/   # raw git clone (reusable)
#   skills/.external/<alias>/                    # installed skill dir (flat)
#   skills/.external/<alias>/.sync-meta.json     # {source, rev, sha, fetched_at}
#
# The entire skills/.external/ tree is gitignored. Reproducibility lives in
# registry.yaml; sync is the one path by which externals land in the tree.
#
# Usage:
#   ./scripts/sync-external.sh           # sync all sources
#   ./scripts/sync-external.sh --check   # exit non-zero if sync would change anything
#   ./scripts/sync-external.sh --allow-floating  # permit ref=main/HEAD/branch
#   ./scripts/sync-external.sh --only anthropics/skills  # sync one source

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REGISTRY="$REPO_ROOT/registry.yaml"
EXTERNAL_ROOT="$REPO_ROOT/skills/.external"
CHECKOUT_ROOT="$EXTERNAL_ROOT/_checkouts"

LOCAL_SOURCE="misty-step/spellbook"

MODE="sync"          # sync | check
ALLOW_FLOATING=0     # 1 to permit branch/HEAD refs
ONLY_REPO=""         # if set, skip all other sources

info()  { printf '\033[0;34m%s\033[0m\n' "$*"; }
ok()    { printf '\033[0;32m%s\033[0m\n' "$*"; }
warn()  { printf '\033[0;33m%s\033[0m\n' "$*"; }
err()   { printf '\033[0;31mERROR: %s\033[0m\n' "$*" >&2; }

die() { err "$*"; exit 1; }

# --- argument parsing ---
while [ $# -gt 0 ]; do
  case "$1" in
    --check)          MODE="check"; shift ;;
    --allow-floating) ALLOW_FLOATING=1; shift ;;
    --only)           ONLY_REPO="$2"; shift 2 ;;
    -h|--help)
      sed -n '3,28p' "$0" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *) die "unknown argument: $1" ;;
  esac
done

# --- registry parsing ---
# We require PyYAML (or the stdlib-friendly fallback). The repo's other
# tooling (generate-embeddings.py) already leans on pyyaml, so this is fine.
command -v python3 >/dev/null 2>&1 || die "python3 required"
command -v git >/dev/null 2>&1 || die "git required"

[ -f "$REGISTRY" ] || die "registry not found: $REGISTRY"

# Emit registry entries as TAB-separated records:
#   repo \t ref \t pin \t skills_path \t include \t exclude \t alias_prefix \t allow_floating
# Where include/exclude are comma-joined and empty fields are "-".
parse_registry() {
  python3 - "$REGISTRY" <<'PY'
import sys, os
path = sys.argv[1]
try:
    import yaml
except ImportError:
    sys.stderr.write("ERROR: PyYAML required (pip install pyyaml)\n")
    sys.exit(2)

try:
    data = yaml.safe_load(open(path, "r", encoding="utf-8"))
except yaml.YAMLError as e:
    sys.stderr.write(f"ERROR: malformed registry.yaml: {e}\n")
    sys.exit(2)

if not isinstance(data, dict):
    sys.stderr.write("ERROR: registry.yaml must be a mapping at top level\n")
    sys.exit(2)

sources = data.get("sources", [])
if not isinstance(sources, list):
    sys.stderr.write("ERROR: registry.yaml: 'sources' must be a list\n")
    sys.exit(2)

def join(val):
    if not val: return "-"
    if isinstance(val, str): return val
    return ",".join(str(x) for x in val)

def bool_field(val):
    return "1" if val else "0"

for i, src in enumerate(sources):
    if not isinstance(src, dict):
        sys.stderr.write(f"ERROR: registry.yaml sources[{i}] must be a mapping\n")
        sys.exit(2)
    repo = src.get("repo")
    if not repo:
        sys.stderr.write(f"ERROR: registry.yaml sources[{i}] missing 'repo'\n")
        sys.exit(2)
    # Skip self and entries explicitly marked default (= local repo).
    if src.get("default"):
        continue
    # Skip inactive entries (declared for embeddings only).
    if src.get("active") is False:
        continue
    ref = src.get("ref") or src.get("rev") or "main"
    pin = src.get("pin") or "-"
    skills_path = src.get("skills_path") or "."
    include = join(src.get("include"))
    exclude = join(src.get("exclude"))
    alias_prefix = src.get("alias_prefix") or "-"
    allow_floating = bool_field(src.get("allow_floating"))
    print("\t".join([repo, ref, pin, skills_path, include, exclude, alias_prefix, allow_floating]))
PY
}

REGISTRY_TSV="$(parse_registry)" || die "failed to parse $REGISTRY"

# --- ref safety ---
# "Immutable" means the ref, once resolved to a sha, will not change upstream
# under our feet. Tags can technically be moved, but convention treats them as
# stable; full SHAs are truly immutable. Branch names (main/master/develop/...)
# and HEAD are floating — refuse unless the operator opts in.
is_immutable_ref() {
  local ref="$1"
  # 40-char hex sha
  if [[ "$ref" =~ ^[0-9a-f]{40}$ ]]; then
    return 0
  fi
  # anything starting with "v" followed by a digit is conventionally a tag
  if [[ "$ref" =~ ^v?[0-9]+\.[0-9]+(\.[0-9]+)?(-[A-Za-z0-9.-]+)?$ ]]; then
    return 0
  fi
  # explicitly floating refs
  case "$ref" in
    main|master|HEAD|develop|dev|trunk) return 1 ;;
  esac
  # unknown — treat as tag-ish (operator's call; git ls-remote verifies existence)
  return 0
}

# --- slug helpers ---
slugify_repo() {
  # "anthropics/skills" → "anthropics__skills"
  printf '%s\n' "$1" | tr '/' '_' | tr -c '[:alnum:]_.-' '_' | sed 's/__*/__/g'
}

# --- git operations ---
ensure_checkout() {
  local repo="$1"
  local dir="$CHECKOUT_ROOT/$(slugify_repo "$repo")"
  local url="https://github.com/$repo.git"

  if [ ! -d "$dir/.git" ]; then
    mkdir -p "$CHECKOUT_ROOT"
    # filter=blob:none for fast, tree-only fetch; sparse enables subdir selection.
    git clone --filter=blob:none --sparse "$url" "$dir" >/dev/null 2>&1 \
      || die "clone failed: $url (unreachable or auth required)"
  fi
  printf '%s\n' "$dir"
}

set_sparse() {
  local dir="$1"
  local path="$2"
  if [ "$path" = "." ] || [ -z "$path" ]; then
    git -C "$dir" sparse-checkout disable >/dev/null 2>&1 || true
  else
    git -C "$dir" sparse-checkout set "$path" >/dev/null 2>&1 \
      || die "sparse-checkout failed in $dir for $path"
  fi
}

resolve_ref_to_sha() {
  local dir="$1"
  local ref="$2"
  # Already a full sha?
  if [[ "$ref" =~ ^[0-9a-f]{40}$ ]]; then
    printf '%s\n' "$ref"; return 0
  fi
  # Try tag first, then branch via ls-remote (authoritative upstream view).
  local sha
  sha="$(git -C "$dir" ls-remote origin "refs/tags/$ref" 2>/dev/null | awk '{print $1}' | head -1)"
  [ -z "$sha" ] && sha="$(git -C "$dir" ls-remote origin "refs/heads/$ref" 2>/dev/null | awk '{print $1}' | head -1)"
  [ -z "$sha" ] && sha="$(git -C "$dir" ls-remote origin "$ref" 2>/dev/null | awk '{print $1}' | head -1)"
  [ -n "$sha" ] || die "cannot resolve ref '$ref' in $dir"
  printf '%s\n' "$sha"
}

checkout_sha() {
  local dir="$1" sha="$2"
  git -C "$dir" fetch --depth=1 --filter=blob:none origin "$sha" >/dev/null 2>&1 || \
    git -C "$dir" fetch --filter=blob:none origin >/dev/null 2>&1 || \
    die "fetch failed in $dir"
  git -C "$dir" checkout --quiet "$sha" 2>/dev/null || \
    git -C "$dir" checkout --quiet -B "spellbook-sync" "$sha" \
    || die "checkout $sha failed in $dir"
}

# --- skill discovery inside a checkout ---
# Echoes one skill name per line for a checked-out repo.
discover_skills() {
  local dir="$1" skills_path="$2"
  local root="$dir"
  [ "$skills_path" = "." ] || [ -z "$skills_path" ] || root="$dir/$skills_path"
  [ -d "$root" ] || return 0
  local d
  for d in "$root"/*/; do
    [ -f "$d/SKILL.md" ] || continue
    basename "$d"
  done
}

in_csv() {
  local needle="$1" csv="$2"
  [ "$csv" = "-" ] || [ -z "$csv" ] && return 1
  local IFS=,
  local x
  for x in $csv; do
    [ "$x" = "$needle" ] && return 0
  done
  return 1
}

# --- state ---
declare -a DECLARED_ALIASES=()
declare -A ALIAS_TO_SOURCE=()
CHANGES=0

note_change() { CHANGES=1; }

# --- install one skill (source path → alias) ---
install_alias() {
  local alias="$1" src_path="$2" repo="$3" sha="$4"
  local dest="$EXTERNAL_ROOT/$alias"

  if [ -n "${ALIAS_TO_SOURCE[$alias]:-}" ]; then
    die "alias collision: '$alias' declared by both '${ALIAS_TO_SOURCE[$alias]}' and '$repo' — set alias_prefix on the later source"
  fi
  ALIAS_TO_SOURCE[$alias]="$repo"
  DECLARED_ALIASES+=("$alias")

  # Compute desired content hash: repo + sha + src_path is sufficient proxy.
  local want_meta
  want_meta="$(printf '{"repo":"%s","sha":"%s","src_path":"%s"}' "$repo" "$sha" "$src_path")"

  local current_meta=""
  if [ -f "$dest/.sync-meta.json" ]; then
    current_meta="$(
      python3 - "$dest/.sync-meta.json" <<'PY'
import json
import sys

try:
    with open(sys.argv[1], "r", encoding="utf-8") as fh:
        data = json.load(fh)
except Exception:
    sys.exit(0)

sha = data.get("sha")
if isinstance(sha, str):
    print(f'"sha":"{sha}"')
PY
    )"
  fi
  local want_sha_frag="\"sha\":\"$sha\""

  if [ -d "$dest" ] && [ "$current_meta" = "$want_sha_frag" ]; then
    # Already up to date.
    return 0
  fi

  note_change
  [ "$MODE" = "check" ] && { warn "  would install/update: $alias ($repo @ ${sha:0:7})"; return 0; }

  rm -rf "$dest"
  mkdir -p "$dest"
  # Copy (not symlink) so the installed skill is self-contained and
  # resilient to checkout-dir churn. cp -R handles subdirs (references/, etc.).
  if [ -d "$src_path" ]; then
    # Use find to preserve hidden files while avoiding copying .git.
    (cd "$src_path" && tar cf - --exclude='.git' .) | (cd "$dest" && tar xf -)
  else
    die "source path missing: $src_path"
  fi

  # Write meta
  cat > "$dest/.sync-meta.json" <<EOF
{
  "repo": "$repo",
  "sha": "$sha",
  "src_path_suffix": "$(basename "$src_path")",
  "fetched_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF
  ok "  installed $alias ← $repo/$(basename "$src_path") @ ${sha:0:7}"
}

# --- orphan GC ---
cleanup_orphans() {
  [ -d "$EXTERNAL_ROOT" ] || return 0
  local entry base
  for entry in "$EXTERNAL_ROOT"/*/; do
    [ -d "$entry" ] || continue
    base="$(basename "$entry")"
    # Skip our internal checkout cache.
    [ "$base" = "_checkouts" ] && continue
    local found=0 a
    for a in "${DECLARED_ALIASES[@]:-}"; do
      [ "$a" = "$base" ] && found=1 && break
    done
    if [ "$found" -eq 0 ]; then
      note_change
      if [ "$MODE" = "check" ]; then
        warn "  would remove orphan: $base"
      else
        rm -rf "$entry"
        ok "  removed orphan: $base"
      fi
    fi
  done
}

cleanup_unused_checkouts() {
  [ -d "$CHECKOUT_ROOT" ] || return 0
  # Any checkout dir for a repo no longer in the registry → remove.
  local declared_slugs=()
  local line repo
  while IFS=$'\t' read -r repo _rest; do
    [ -z "$repo" ] && continue
    declared_slugs+=("$(slugify_repo "$repo")")
  done <<< "$REGISTRY_TSV"

  local entry base found s
  for entry in "$CHECKOUT_ROOT"/*/; do
    [ -d "$entry" ] || continue
    base="$(basename "$entry")"
    found=0
    for s in "${declared_slugs[@]:-}"; do
      [ "$s" = "$base" ] && found=1 && break
    done
    if [ "$found" -eq 0 ]; then
      note_change
      if [ "$MODE" = "check" ]; then
        warn "  would remove unused checkout: $base"
      else
        rm -rf "$entry"
        ok "  removed unused checkout: $base"
      fi
    fi
  done
}

# --- main loop ---
info "sync-external [$MODE] — reading $REGISTRY"

mkdir -p "$EXTERNAL_ROOT"

if [ -z "$REGISTRY_TSV" ]; then
  warn "no external sources declared in registry.yaml"
fi

while IFS=$'\t' read -r repo ref pin skills_path include exclude alias_prefix allow_floating_src; do
  [ -z "$repo" ] && continue
  if [ -n "$ONLY_REPO" ] && [ "$repo" != "$ONLY_REPO" ]; then
    continue
  fi

  info "→ $repo  (ref=$ref pin=${pin} path=$skills_path)"

  # ref safety
  if [ "$pin" = "-" ] && ! is_immutable_ref "$ref"; then
    if [ "$ALLOW_FLOATING" -ne 1 ] && [ "$allow_floating_src" != "1" ]; then
      die "refusing floating ref '$ref' for $repo — pin a sha/tag, set allow_floating: true, or pass --allow-floating"
    fi
    warn "  floating ref '$ref' allowed by operator override"
  fi

  checkout_dir="$(ensure_checkout "$repo")"

  # Sparse path
  set_sparse "$checkout_dir" "$skills_path"

  # Pick the sha to use. pin wins over ref (registry's own record of what we've locked to).
  want_ref="$ref"
  [ "$pin" != "-" ] && want_ref="$pin"

  sha="$(resolve_ref_to_sha "$checkout_dir" "$want_ref")"
  checkout_sha "$checkout_dir" "$sha"

  # Discover skills
  discovered="$(discover_skills "$checkout_dir" "$skills_path" || true)"
  if [ -z "$discovered" ]; then
    die "no skills found under $repo/$skills_path — upstream layout change? Update skills_path."
  fi

  # Apply include/exclude filters, install
  root="$checkout_dir"
  [ "$skills_path" = "." ] || [ -z "$skills_path" ] || root="$checkout_dir/$skills_path"

  while IFS= read -r skill_name; do
    [ -z "$skill_name" ] && continue
    if [ "$include" != "-" ] && ! in_csv "$skill_name" "$include"; then
      continue
    fi
    if [ "$exclude" != "-" ] && in_csv "$skill_name" "$exclude"; then
      continue
    fi
    alias="$skill_name"
    [ "$alias_prefix" != "-" ] && alias="${alias_prefix}${skill_name}"
    install_alias "$alias" "$root/$skill_name" "$repo" "$sha"
  done <<< "$discovered"

done <<< "$REGISTRY_TSV"

cleanup_orphans
cleanup_unused_checkouts

if [ "$MODE" = "check" ]; then
  if [ "$CHANGES" -eq 1 ]; then
    err "registry drift: sync would change the tree. Run ./scripts/sync-external.sh."
    exit 1
  fi
  ok "sync-external: clean (no changes needed)"
  exit 0
fi

ok "sync-external: done (${#DECLARED_ALIASES[@]} aliases installed)"
