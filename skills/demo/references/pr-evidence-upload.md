# Optional PR Evidence Upload

How to mirror committed `.evidence/<branch>/<date>/` screenshots, GIFs, and
videos into GitHub PR comments.

## The Problem

Canonical QA/demo evidence lives in git under `.evidence/<branch>/<date>/`.
`gh pr comment` accepts markdown text but not file attachments. GitHub renders
images/GIFs via URLs, so inline PR comments need a URL mirror even though the
branch already contains the durable evidence.

## Default: Link Git Evidence

For most PRs, commit the evidence directory and link it:

```bash
source scripts/lib/evidence.sh
EVIDENCE_DIR="$(evidence_dir)"
git add "$EVIDENCE_DIR"
git commit -m "test(qa): add evidence"

gh pr comment "$PR_NUMBER" --body "QA evidence: \`$EVIDENCE_DIR\`"
```

This works offline up to the commit step. Binary files under `.evidence/` are
LFS-tracked when LFS is configured; fresh clones retain pointer files at
minimum.

## Optional Mirror: Draft Release Assets

Use this only when PR reviewers need inline screenshots or GIF URLs. Opt in
explicitly:

```bash
HARNESS_EVIDENCE_GITHUB=1
```

```bash
# 1. Capture evidence (screenshots, GIFs, videos)
PR_NUMBER=123
REPO=$(gh repo view --json nameWithOwner --jq .nameWithOwner)
source scripts/lib/evidence.sh
EVIDENCE_DIR="$(evidence_dir)"

# 2. Convert video to GIF for inline rendering (GitHub doesn't embed webm)
ffmpeg -y -i "$EVIDENCE_DIR/walkthrough.webm" \
  -vf "fps=8,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
  -loop 0 "$EVIDENCE_DIR/walkthrough.gif"

# 3. Upload to a draft release
test "${HARNESS_EVIDENCE_GITHUB:-}" = "1"
gh release create "qa-evidence-pr-${PR_NUMBER}" \
  --title "QA Evidence: PR #${PR_NUMBER}" \
  --notes "Visual QA evidence mirror for PR #${PR_NUMBER}. Canonical path: ${EVIDENCE_DIR}" \
  --draft \
  "$EVIDENCE_DIR/walkthrough.gif" \
  "$EVIDENCE_DIR/feature-demo.png"

# 4. Get asset URLs
RELEASE_TAG=$(
  gh release list --json tagName,isDraft \
    --jq '.[] | select(.isDraft) | .tagName' |
    grep -F -- "qa-evidence-pr-${PR_NUMBER}" |
    head -1
)
gh release view "$RELEASE_TAG" --json assets --jq '.assets[] | "\(.name): \(.url)"'

# 5. Embed in PR comment
RELEASE_BASE="https://github.com/${REPO}/releases/download/${RELEASE_TAG}"
gh pr comment "$PR_NUMBER" --body "$(cat <<EOF
## Visual QA Report

Canonical evidence: \`${EVIDENCE_DIR}\`

![walkthrough](${RELEASE_BASE}/walkthrough.gif)

| Route | Screenshot |
|-------|-----------|
| /dashboard | ![dash](${RELEASE_BASE}/feature-demo.png) |

[Mirrored assets](https://github.com/${REPO}/releases/tag/${RELEASE_TAG})
EOF
)"
```

## Rules

- **Commit `.evidence/<branch>/<date>/` first**; release assets are mirrors.
- **Always convert `.webm` to `.gif`** for inline rendering. GitHub markdown
  renders GIFs inline but not video files.
- **Use `--draft`** so the release doesn't appear in the public release list.
- **Tag naming**: `qa-evidence-pr-${PR_NUMBER}` or `qa-{feature-slug}` for easy identification.
- **Keep under 10MB per inline asset** even though GitHub releases allow larger files.
- **Link the canonical evidence path** in the comment so offline readers know
  where the branch stores proof.
- **For private repos**: draft release URLs require repo access — this is
  correct behavior for private QA evidence.

## When to Use

Use the GitHub mirror when inline visuals materially speed review. Otherwise,
link the committed evidence path.

| Change type | Default evidence |
|-------------|------------------|
| UI feature/fix | Committed GIF walkthrough + route screenshots |
| Visual change | Committed before/after screenshots |
| API/backend | Terminal output or response capture under `.evidence/` |
| Refactor with parity | Gate output and, if useful, committed smoke transcript |
| Config/infra | Terminal proof under `.evidence/` or PR text |

## Anti-Patterns

- Capturing evidence without committing or linking `.evidence/<branch>/<date>/`
- PR comments that describe what the reviewer should see instead of showing it
- Treating draft releases as canonical storage
- Using `raw.githubusercontent.com` URLs for private evidence
- Recording video of motionless screens (use screenshots)
- Uploading mirror assets without linking the canonical evidence path
