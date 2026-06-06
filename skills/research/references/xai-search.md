# xAI Search

Grounded web search and X (Twitter) search via the xAI Grok API.

## Two Tools

### Web Search (`web_search`)
Grounded web search with optional image understanding.

### X Search (`x_search`)
Keyword, semantic, user, and thread search on X. Real-time social data.

## When to Use

| Need | Tool |
|------|------|
| Grounded live web retrieval | Web Search |
| Recency verification or contradiction check | Web Search |
| Social sentiment, public discourse | X Search |
| What people are saying about X | X Search |
| Trending topics, viral content | X Search |
| Specific user's posts/opinions | X Search with `allowed_x_handles` |
| Web search with image analysis | Web Search with `enable_image_understanding` |
| Video content analysis from X | X Search with `enable_video_understanding` |
| General web with domain filtering | Web Search with `allowed_domains` |

Default `/research` fanout uses xAI alongside Exa, not instead of it. Use
xAI/Grok for grounded web retrieval, recency verification, contradiction
checks, X-native discourse, and multimodal web/social evidence. Do not reduce
Grok to social sentiment only.

## API Access

Base URL: `https://api.x.ai/v1`
Auth: `Authorization: Bearer $XAI_API_KEY`
API: OpenAI Responses API compatible. Default model: `grok-4.3` unless the
environment overrides it.

## Web Search

```bash
curl https://api.x.ai/v1/responses \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -d '{
  "model": "grok-4.3",
  "input": [{"role": "user", "content": "What is the latest on AI regulation?"}],
  "tools": [{"type": "web_search"}]
}'
```

### Parameters

| Parameter | Description |
|-----------|-------------|
| `allowed_domains` | Only search within specific domains (max 5) |
| `excluded_domains` | Exclude specific domains from search (max 5) |
| `enable_image_understanding` | Analyze images found during browsing |

### Domain filtering
```json
{"type": "web_search", "filters": {"allowed_domains": ["arxiv.org", "github.com"]}}
```

### Image understanding
```json
{"type": "web_search", "enable_image_understanding": true}
```

## X Search

```bash
curl https://api.x.ai/v1/responses \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $XAI_API_KEY" \
  -d '{
  "model": "grok-4.3",
  "input": [{"role": "user", "content": "What are people saying about Claude 4?"}],
  "tools": [{"type": "x_search"}]
}'
```

### Parameters

| Parameter | Description |
|-----------|-------------|
| `allowed_x_handles` | Only posts from specific handles (max 10) |
| `excluded_x_handles` | Exclude posts from handles (max 10) |
| `from_date` | Start date, ISO8601 `YYYY-MM-DD` |
| `to_date` | End date, ISO8601 `YYYY-MM-DD` |
| `enable_image_understanding` | Analyze images in posts |
| `enable_video_understanding` | Analyze videos in posts (X Search only) |

### Handle filtering
```json
{"type": "x_search", "allowed_x_handles": ["kaborek", "AnthropicAI"]}
```

### Date range
```json
{"type": "x_search", "from_date": "2025-03-01", "to_date": "2025-03-15"}
```

### Multimodal
```json
{"type": "x_search", "enable_image_understanding": true, "enable_video_understanding": true}
```

## SDK Usage

### OpenAI-compatible Python
```python
from openai import OpenAI

client = OpenAI(api_key=os.getenv("XAI_API_KEY"), base_url="https://api.x.ai/v1")

response = client.responses.create(
    model="grok-4.3",
    input=[{"role": "user", "content": query}],
    tools=[{"type": "x_search"}],  # or "web_search"
)
```

### Vercel AI SDK
```typescript
import { xai } from '@ai-sdk/xai';
import { generateText } from 'ai';

const { text, sources } = await generateText({
  model: xai.responses('grok-4.3'),
  prompt: query,
  tools: {
    x_search: xai.tools.xSearch(),           // X search
    web_search: xai.tools.webSearch(),        // Web search
  },
});
```

### xAI Native SDK
```python
from xai_sdk import Client
from xai_sdk.chat import user
from xai_sdk.tools import web_search, x_search

client = Client(api_key=os.getenv("XAI_API_KEY"))
chat = client.chat.create(
    model="grok-4.3",
    tools=[web_search(), x_search()],
)
chat.append(user(query))
```

## Citations

Responses include `response.citations` with source URLs. Always cite them.

## Integration Notes

- Both tools can be used in the same request
- `enable_image_understanding` on Web Search also enables it for X Search
- `enable_video_understanding` is X Search only
- `allowed_domains` / `excluded_domains` cannot be combined in one request
- `allowed_x_handles` / `excluded_x_handles` cannot be combined in one request
- In the Harness Kit runtime, xAI is a retrieval/discourse provider when
  `XAI_API_KEY` is set. It is routed before Exa for social/discourse queries
  and after Exa for recency corroboration.
