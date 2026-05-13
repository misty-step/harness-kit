# Remotion — Programmatic Video Composition

React-based framework for rendering video programmatically. The strongest
option for agent-driven video composition.

## Why Remotion

- Video as code: React components define scenes, animations, timing
- Full programmatic control: data-driven, parameterized, batch-renderable
- Agent-friendly: official Agent Skills for Claude Code, AI-ready docs
- Rendering: local, server-side via `renderMedia()`, distributed via Lambda
- Outputs: MP4, WebM, GIF, still images, audio

## Quick Start

```bash
# One-off video project (non-interactive, for agent use)
npx create-video --yes --blank --tmp

# With Remotion Agent Skills (recommended for Claude Code)
npx remotion skills add
```

### Remotion Agent Skills

Remotion maintains Claude Code skills that define best practices for Remotion
projects. Install them for guided video creation:

```bash
npx remotion skills add remotion-dev/skills
```

These install to `.claude/skills/` and provide Remotion-specific guidance
that the agent loads automatically.

### AI-Ready Docs

Remotion docs are optimized for agent consumption:
- Add `.md` to any doc URL: `remotion.dev/docs/player.md`
- Content negotiation: `Accept: text/markdown` returns markdown
- Paste any Remotion doc link into Claude Code and it fetches markdown

## Core Concepts

### Composition (the video unit)

```tsx
import { useCurrentFrame, useVideoConfig, spring, AbsoluteFill } from 'remotion';

export const MyScene: React.FC<{ title: string }> = ({ title }) => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();

  const scale = spring({ fps, frame, config: { damping: 100 } });

  return (
    <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center' }}>
      <h1 style={{ transform: `scale(${scale})` }}>{title}</h1>
    </AbsoluteFill>
  );
};
```

### Key APIs

- `useCurrentFrame()` — current frame number
- `useVideoConfig()` — fps, width, height, durationInFrames
- `spring()` — physics-based animation
- `interpolate()` — map frame ranges to values
- `<Sequence>` — time-offset child compositions
- `<AbsoluteFill>` — full-frame positioned container
- `<Img>`, `<Video>`, `<Audio>` — media components
- `<OffthreadVideo>` — efficient video embedding

## Walkthrough Video Pattern

For assembling QA screenshots/recordings into a walkthrough:

```tsx
import { Composition } from 'remotion';

// Register the composition
<Composition
  id="walkthrough"
  component={Walkthrough}
  durationInFrames={300}  // 10 seconds at 30fps
  fps={30}
  width={1920}
  height={1080}
  defaultProps={{
    steps: [
      { image: '/tmp/qa-slug/01-dashboard.png', caption: 'Dashboard loads' },
      { image: '/tmp/qa-slug/02-create.png', caption: 'Create new item' },
      { image: '/tmp/qa-slug/03-success.png', caption: 'Success state' },
    ]
  }}
/>
```

### Scene structure for walkthroughs

```
Intro card (2s) → Step 1 screenshot + caption (3s) → transition →
Step 2 screenshot + caption (3s) → ... → Outro card (2s)
```

## Rendering

### Local render

```bash
npx remotion render MyComposition output.mp4
```

### Programmatic render (server-side)

```typescript
import { renderMedia, selectComposition } from '@remotion/renderer';

const composition = await selectComposition({
  serveUrl: '/path/to/bundle',
  id: 'walkthrough',
  inputProps: { steps: [...] },
});

await renderMedia({
  composition,
  serveUrl: '/path/to/bundle',
  codec: 'h264',
  outputLocation: 'output.mp4',
  inputProps: { steps: [...] },
});
```

### Still image render

```bash
npx remotion still MyComposition frame-50.png --frame=50
```

Useful for generating thumbnail/preview images from video compositions.

## Captions

Remotion has a captions ecosystem for auto-generated subtitles:

- Generate captions from audio via OpenAI Whisper
- Render word-by-word or sentence-by-sentence captions
- Sync captions to audio timing

```tsx
import { Caption } from '@remotion/captions';

// Captions component syncs text to audio timing
<Caption words={transcription.words} />
```

## Dataset Rendering (Batch)

Generate many videos from a JSON dataset — useful for repeatable demos
across different features or customers:

```typescript
import { data } from './dataset';

// Register one composition per data entry
data.map((entry) => (
  <Composition
    key={entry.id}
    id={entry.id}
    component={DemoVideo}
    defaultProps={entry}
    // ...
  />
));
```

Render all:
```bash
for id in $(npx remotion compositions --json | jq -r '.[].id'); do
  npx remotion render "$id" "output/${id}.mp4"
done
```

## Prompt-to-Motion-Graphics

Remotion offers a SaaS template for AI-generated animations:

```bash
npx create-video@latest --template prompt-to-motion-graphics
```

Users describe animations in natural language → app generates and previews
in real-time. Best for stylized explainers, not product walkthroughs.

## Recorder

Remotion Recorder captures webcam + screen as separate streams with
auto-generated captions. Good for talking-head product walkthroughs.

```bash
bun run dev  # Start the recorder
# Record webcam, screen, and audio
# Export as composed video with captions
```

## Integration with QA Evidence

Typical workflow: `/qa` captures evidence → `/demo` composes with Remotion

1. QA produces screenshots in `/tmp/qa-{slug}/`
2. Create a Remotion composition that sequences them
3. Add title cards, captions, transitions
4. Optionally add narration (see tts-narration.md)
5. Render to MP4
6. Upload via draft release or share directly
