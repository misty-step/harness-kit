# PR Evidence Upload

How to get screenshots, GIFs, and videos into PR comments on GitHub.

## The Problem

`gh pr comment` accepts markdown text but not file attachments. GitHub renders
images/GIFs via URLs, but the URLs must point to something GitHub can serve.
Binary assets sitting in `/tmp` are invisible to reviewers.

## The Solution: Draft Release Assets

Upload binary evidence to a draft GitHub release. Reference the download URLs
in PR comments and body. The release is private to the repo and can be cleaned
up later.

```bash
# 1. Capture evidence (screenshots, GIFs, videos)
mkdir -p /tmp/pr-evidence
# Evidence comes from /qa — screenshots, GIFs, videos in /tmp/qa-{slug}/

# 2. Convert video to GIF for inline rendering (GitHub doesn't embed webm)
ffmpeg -y -i /tmp/pr-evidence/walkthrough.webm \
  -vf "fps=8,scale=800:-1:flags=lanczos,split[s0][s1];[s0]palettegen=max_colors=128[p];[s1][p]paletteuse=dither=bayer" \
  -loop 0 /tmp/pr-evidence/walkthrough.gif

# 3. Upload to a draft release
gh release create qa-evidence-pr-{NUMBER} \
  --title "QA Evidence: PR #{NUMBER}" \
  --notes "Visual QA evidence for PR #{NUMBER}" \
  --draft \
  /tmp/pr-evidence/walkthrough.gif \
  /tmp/pr-evidence/feature-demo.png

# 4. Get asset URLs
RELEASE_TAG=$(gh release list --json tagName,isDraft --jq '.[] | select(.isDraft) | .tagName' | grep "qa-evidence-pr-{NUMBER}" | head -1)
gh release view "$RELEASE_TAG" --json assets --jq '.assets[] | "\(.name): \(.url)"'

# 5. Embed in PR comment
RELEASE_BASE="https://github.com/{OWNER}/{REPO}/releases/download/{TAG}"
gh pr comment {NUMBER} --body "$(cat <<EOF
## Visual QA Report

![walkthrough](${RELEASE_BASE}/walkthrough.gif)

| Route | Screenshot |
|-------|-----------|
| /dashboard | ![dash](${RELEASE_BASE}/feature-demo.png) |

[All evidence](https://github.com/{OWNER}/{REPO}/releases/tag/{TAG})
EOF
)"
```

## Rules

- **Always convert `.webm` to `.gif`** for inline rendering. GitHub markdown
  renders GIFs inline but not video files.
- **Use `--draft`** so the release doesn't appear in the public release list.
- **Tag naming**: `qa-evidence-pr-{NUMBER}` or `qa-{feature-slug}` for easy identification.
- **Keep under 10MB per asset** (GitHub release limit per file is 2GB, but
  keep GIFs reasonable for inline rendering).
- **Link the release** at the bottom of the comment so reviewers can download
  full-resolution assets.
- **For private repos**: draft release URLs require repo access — this is
  correct behavior for private QA evidence.
- **Cleanup**: delete draft releases after PR merge if desired, but they cost nothing.

## When to Use

Every PR with user-visible changes. No exceptions. The evidence should be:

| Change type | Required evidence |
|-------------|------------------|
| UI feature/fix | GIF walkthrough + route screenshots |
| Visual change | Before/after screenshots |
| API/backend | Terminal output pasted as code block (no upload needed) |
| Refactor with parity | GIF showing the app still works |
| Config/infra | Terminal proof (no upload needed) |

## Anti-Patterns

- Screenshots sitting in `/tmp` that never get uploaded
- PR comments that describe what the reviewer should see instead of showing it
- Committing binary evidence into the repo
- Using `raw.githubusercontent.com` URLs (breaks for private repos)
- Recording video of motionless screens (use screenshots)
- Uploading evidence without linking it in the PR comment
