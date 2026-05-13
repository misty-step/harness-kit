# TTS Narration & Audio

Programmable voice and music for demo videos. Three providers, each with
a clear best-fit.

## Provider Selection

| Provider | Best for | Latency | Cost | Languages |
|----------|----------|---------|------|-----------|
| **ElevenLabs** | Quality, voice cloning, music | 100-200ms TTFB | ~$0.015/min | 30+ |
| **OpenAI TTS** | Simplest API, good-enough quality | ~200ms TTFB | $0.015/1K chars | ~50 |
| **Cartesia Sonic-3** | Ultra-low latency, real-time | 40-90ms TTFB | $0.006/min | 42 |

**Default recommendation:**
- Already using OpenAI? → **OpenAI TTS** (one endpoint, streaming, simple)
- Quality matters most? → **ElevenLabs** (best voices, cloning, emotional range)
- Latency matters most? → **Cartesia** (fastest TTFB, cheapest)
- Need generated music? → **ElevenLabs** (only provider with music generation)

## OpenAI TTS

Simplest integration. Six voices, one endpoint.

```bash
curl -s https://api.openai.com/v1/audio/speech \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tts-1-hd",
    "input": "Welcome to the feature walkthrough. Let me show you what we built.",
    "voice": "alloy"
  }' \
  --output /tmp/demo-slug/narration.mp3
```

### Voices
`alloy` (neutral), `echo` (male), `fable` (British), `onyx` (deep male),
`nova` (female), `shimmer` (soft female).

### Models
- `tts-1` — fast, lower quality
- `tts-1-hd` — slower, higher quality (use for launch videos)

## ElevenLabs

Best quality. Voice cloning from 1 minute of audio. Music generation.

```bash
curl -s https://api.elevenlabs.io/v1/text-to-speech/{voice_id} \
  -H "xi-api-key: $ELEVENLABS_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "text": "Welcome to the feature walkthrough.",
    "model_id": "eleven_multilingual_v2",
    "voice_settings": {
      "stability": 0.5,
      "similarity_boost": 0.75
    }
  }' \
  --output /tmp/demo-slug/narration.mp3
```

### Voice library
Browse voices at `api.elevenlabs.io/v1/voices`. Hundreds of pre-made voices
plus custom cloning.

### Music generation
ElevenLabs offers music generation — useful for background tracks:
```bash
# Check current API surface for music endpoints
# This is evolving rapidly
```

## Cartesia

Ultra-low latency. Best for real-time or interactive narration.

```bash
curl -s https://api.cartesia.ai/tts/bytes \
  -H "X-API-Key: $CARTESIA_API_KEY" \
  -H "Cartesia-Version: 2024-06-10" \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "sonic-3",
    "transcript": "Welcome to the feature walkthrough.",
    "voice": { "mode": "id", "id": "a0e99841-438c-4a64-b679-ae501e7d6091" },
    "output_format": { "container": "mp3", "bit_rate": 128000 }
  }' \
  --output /tmp/demo-slug/narration.mp3
```

## Narration Workflow

### Script generation

1. Describe the feature or provide the QA evidence
2. Generate a script: one sentence per scene/screenshot
3. Keep it concise — 10-15 words per scene, 2-3 seconds of speech

### Script → Audio → Video

```bash
# 1. Generate narration
curl -s https://api.openai.com/v1/audio/speech \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "tts-1-hd",
    "input": "'"$(cat /tmp/demo-slug/script.txt)"'",
    "voice": "nova"
  }' --output /tmp/demo-slug/narration.mp3

# 2. Get audio duration (for Remotion timing)
ffprobe -v error -show_entries format=duration \
  -of default=noprint_wrappers=1:nokey=1 \
  /tmp/demo-slug/narration.mp3

# 3. Compose in Remotion (set durationInFrames from audio duration)
# 4. Or mux directly with ffmpeg:
ffmpeg -y -i /tmp/demo-slug/walkthrough.mp4 \
  -i /tmp/demo-slug/narration.mp3 \
  -c:v copy -c:a aac -map 0:v:0 -map 1:a:0 \
  -shortest /tmp/demo-slug/narrated-walkthrough.mp4
```

## Auto-Captioning

### OpenAI Whisper (transcription → captions)

```bash
curl -s https://api.openai.com/v1/audio/transcriptions \
  -H "Authorization: Bearer $OPENAI_API_KEY" \
  -F file="@/tmp/demo-slug/narration.mp3" \
  -F model="whisper-1" \
  -F response_format="srt" \
  > /tmp/demo-slug/captions.srt
```

### Burn captions into video

```bash
ffmpeg -y -i /tmp/demo-slug/narrated-walkthrough.mp4 \
  -vf "subtitles=/tmp/demo-slug/captions.srt:force_style='FontSize=24,PrimaryColour=&HFFFFFF&'" \
  /tmp/demo-slug/final.mp4
```

### Remotion captions

Remotion's `@remotion/captions` package renders word-by-word synced captions
as React components — more polished than burned-in SRT subtitles.

## Background Music

### From file
```bash
# Mix narration + music (music at -20dB under narration)
ffmpeg -y -i /tmp/demo-slug/narration.mp3 \
  -i /tmp/demo-slug/music.mp3 \
  -filter_complex "[1:a]volume=0.1[music];[0:a][music]amix=inputs=2:duration=first" \
  /tmp/demo-slug/mixed-audio.mp3
```

### Generated
ElevenLabs music generation or royalty-free libraries. For launch videos,
generated music avoids licensing concerns.

## Cost Awareness

| Task | Provider | Typical cost |
|------|----------|-------------|
| 30s narration (~75 words) | OpenAI TTS | ~$0.01 |
| 30s narration | ElevenLabs | ~$0.02 |
| 30s narration | Cartesia | ~$0.003 |
| 2min launch video narration | OpenAI TTS | ~$0.05 |
| Auto-captioning (Whisper) | OpenAI | ~$0.01/min |

Don't generate narration for internal QA evidence — that's what screenshots
and GIFs are for. Reserve TTS for launch videos and external demos.
