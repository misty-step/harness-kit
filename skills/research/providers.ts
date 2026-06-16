import type {
  AgenticCitation,
  AgenticEffort,
  AgenticResearchBlock,
  AgenticResearchOutput,
  ProviderAdapter,
  SearchRequest,
  SearchResult,
} from "./provider-adapter";
import { isoTimestampDaysAgo } from "./query-utils";

const DEFAULT_LIMIT = 5;
const CONTEXT7_BASE_URL = process.env.CONTEXT7_BASE_URL ?? "https://context7.com/api/v1";
const EXA_AGENT_BASE_URL = process.env.EXA_AGENT_BASE_URL ?? "https://api.exa.ai";
const PERPLEXITY_MODEL = process.env.PERPLEXITY_MODEL ?? "sonar";
const DEFAULT_PROVIDER_TIMEOUT_MS = 15_000;

export type ProviderFailureKind = "timeout" | "http" | "network";

export class ProviderRequestError extends Error {
  readonly provider: ProviderAdapter["name"] | "exa-agent";
  readonly kind: ProviderFailureKind;
  readonly status?: number;

  constructor(
    provider: ProviderAdapter["name"] | "exa-agent",
    kind: ProviderFailureKind,
    message: string,
    options: { status?: number; cause?: unknown } = {}
  ) {
    super(message);
    this.name = "ProviderRequestError";
    this.provider = provider;
    this.kind = kind;
    this.status = options.status;
    if (options.cause) {
      this.cause = options.cause;
    }
  }
}

export interface FetchWithTimeoutOptions extends RequestInit {
  timeoutMs?: number;
  fetchImpl?: typeof fetch;
}

export async function fetchWithTimeout(
  provider: ProviderAdapter["name"] | "exa-agent",
  input: string | URL | Request,
  options: FetchWithTimeoutOptions = {}
): Promise<Response> {
  const {
    timeoutMs = providerTimeoutMs(),
    fetchImpl = fetch,
    ...init
  } = options;
  const controller = new AbortController();
  let timedOut = false;
  let timeoutReject: (error: ProviderRequestError) => void = () => undefined;
  const timeout = new Promise<Response>((_, reject) => {
    timeoutReject = reject;
  });
  const timer = setTimeout(() => {
    timedOut = true;
    controller.abort();
    timeoutReject(
      new ProviderRequestError(
        provider,
        "timeout",
        `${provider} request timed out after ${timeoutMs}ms`
      )
    );
  }, timeoutMs);
  try {
    const request = fetchImpl(input, {
      ...init,
      signal: controller.signal,
    });
    request.catch(() => undefined);
    return await Promise.race([request, timeout]);
  } catch (error) {
    if (error instanceof ProviderRequestError) {
      throw error;
    }
    if (timedOut || controller.signal.aborted) {
      throw new ProviderRequestError(
        provider,
        "timeout",
        `${provider} request timed out after ${timeoutMs}ms`,
        { cause: error }
      );
    }
    throw new ProviderRequestError(provider, "network", `${provider} request failed`, {
      cause: error,
    });
  } finally {
    clearTimeout(timer);
  }
}

type Context7SearchItem = {
  id?: string;
  title?: string;
  name?: string;
  url?: string;
  description?: string;
  snippets?: string[];
  score?: number;
};

export class Context7Provider implements ProviderAdapter {
  readonly name = "context7" as const;
  private readonly apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async search(request: SearchRequest): Promise<SearchResult[]> {
    const payload = await this.searchPayload(request.query);

    const items = (payload.results ?? payload.data ?? []).slice(0, request.limit ?? DEFAULT_LIMIT);
    if (items.length === 0) {
      return [];
    }

    const mapped = items.map((item, index) => {
      const fallbackUrl = item.id ? `https://context7.com/${item.id}` : "";
      return {
        title: item.title ?? item.name ?? (fallbackUrl || "Context7 result"),
        url: item.url ?? fallbackUrl,
        snippet: item.description ?? firstSnippet(item.snippets),
        published_at: null,
        score: scoreFromRank(index),
        source_provider: "context7" as const,
      };
    });

    // For explicit docs mode, enrich the first result with actual documentation text.
    if (request.command === "web-docs" && items[0]?.id) {
      const docsSnippet = await this.fetchDocSnippet(items[0].id);
      if (docsSnippet) {
        mapped[0] = {
          ...mapped[0],
          snippet: docsSnippet,
        };
      }
    }

    return mapped.filter((item) => Boolean(item.url));
  }

  private async searchPayload(
    query: string
  ): Promise<{ results?: Context7SearchItem[]; data?: Context7SearchItem[] }> {
    const postResponse = await fetchWithTimeout(this.name, `${CONTEXT7_BASE_URL}/search`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        authorization: `Bearer ${this.apiKey}`,
      },
      body: JSON.stringify({
        query,
      }),
    });

    if (postResponse.ok) {
      return (await postResponse.json()) as { results?: Context7SearchItem[]; data?: Context7SearchItem[] };
    }

    // Context7 deployments differ; some only support GET.
    if (postResponse.status === 405) {
      const params = new URLSearchParams({ query });
      const getResponse = await fetchWithTimeout(this.name, `${CONTEXT7_BASE_URL}/search?${params}`, {
        method: "GET",
        headers: {
          authorization: `Bearer ${this.apiKey}`,
        },
      });
      if (getResponse.ok) {
        return (await getResponse.json()) as {
          results?: Context7SearchItem[];
          data?: Context7SearchItem[];
        };
      }
      throw new ProviderRequestError(this.name, "http", `context7 search failed: ${getResponse.status}`, {
        status: getResponse.status,
      });
    }

    throw new ProviderRequestError(this.name, "http", `context7 search failed: ${postResponse.status}`, {
      status: postResponse.status,
    });
  }

  private async fetchDocSnippet(context7Id: string): Promise<string | null> {
    try {
      const response = await fetchWithTimeout(this.name, `${CONTEXT7_BASE_URL}/${context7Id}`, {
        method: "POST",
        headers: {
          "content-type": "application/json",
          authorization: `Bearer ${this.apiKey}`,
        },
        body: JSON.stringify({
          tokens: 1800,
        }),
      });

      if (!response.ok) {
        return null;
      }

      const payload = (await response.json()) as Record<string, unknown>;
      return extractBestDocSnippet(payload);
    } catch {
      return null;
    }
  }
}

export class ExaProvider implements ProviderAdapter {
  readonly name = "exa" as const;
  private readonly apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async search(request: SearchRequest): Promise<SearchResult[]> {
    const recencyStart =
      typeof request.recencyDays === "number" ? isoTimestampDaysAgo(request.recencyDays) : null;

    const response = await fetchWithTimeout(this.name, "https://api.exa.ai/search", {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "x-api-key": this.apiKey,
      },
      body: JSON.stringify({
        query: request.query,
        numResults: request.limit ?? DEFAULT_LIMIT,
        ...(recencyStart ? { startPublishedDate: recencyStart } : {}),
      }),
    });

    if (!response.ok) {
      throw new ProviderRequestError(this.name, "http", `exa search failed: ${response.status}`, {
        status: response.status,
      });
    }

    const payload = (await response.json()) as {
      results?: Array<{
        title?: string;
        url?: string;
        text?: string;
        publishedDate?: string;
        score?: number;
      }>;
    };

    return (payload.results ?? [])
      .filter((item) => Boolean(item.url))
      .map((item) => ({
        title: item.title ?? item.url ?? "Untitled",
        url: item.url!,
        snippet: item.text ?? "",
        published_at: item.publishedDate ?? null,
        score: item.score ?? 0,
        source_provider: "exa" as const,
      }));
  }
}

export interface ExaAgentRunOptions {
  effort: AgenticEffort;
  timeoutMs: number;
  pollIntervalMs: number;
  privateContextOk: boolean;
}

export class ExaAgentProvider {
  readonly name = "exa-agent" as const;
  private readonly apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async run(request: SearchRequest, options: ExaAgentRunOptions): Promise<AgenticResearchBlock> {
    const run = await this.createRun(request, options);
    const runId = stringValue(run, ["id", "run_id", "runId"]) ?? null;
    if (!runId) {
      return this.blockFromPayload(run, request, options, "provider did not return a run id");
    }

    const terminal = isTerminalStatus(stringValue(run, ["status"]));
    const payload = terminal ? run : await this.pollRun(runId, options);
    return this.blockFromPayload(payload, request, options);
  }

  private async createRun(
    request: SearchRequest,
    options: ExaAgentRunOptions
  ): Promise<Record<string, unknown>> {
    const response = await fetchWithTimeout(this.name, `${EXA_AGENT_BASE_URL}/agent/runs`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        "x-api-key": this.apiKey,
      },
      timeoutMs: Math.min(options.timeoutMs, providerTimeoutMs()),
      body: JSON.stringify({
        prompt: buildAgentPrompt(request, options.privateContextOk),
        effort: options.effort,
        responseSchema: {
          type: "object",
          properties: {
            summary: { type: "string" },
            findings: { type: "array" },
            citations: { type: "array" },
            open_questions: { type: "array" },
            entities: { type: "array" },
          },
          required: ["summary", "findings", "citations", "open_questions"],
        },
      }),
    });

    if (!response.ok) {
      throw new ProviderRequestError(this.name, "http", `exa agent create run failed: ${response.status}`, {
        status: response.status,
      });
    }
    return (await response.json()) as Record<string, unknown>;
  }

  private async pollRun(runId: string, options: ExaAgentRunOptions): Promise<Record<string, unknown>> {
    const deadline = Date.now() + options.timeoutMs;
    let lastPayload: Record<string, unknown> | null = null;
    while (Date.now() < deadline) {
      const response = await fetchWithTimeout(this.name, `${EXA_AGENT_BASE_URL}/agent/runs/${encodeURIComponent(runId)}`, {
        method: "GET",
        headers: {
          "x-api-key": this.apiKey,
        },
        timeoutMs: Math.min(options.pollIntervalMs + 2_000, providerTimeoutMs()),
      });
      if (!response.ok) {
        throw new ProviderRequestError(this.name, "http", `exa agent get run failed: ${response.status}`, {
          status: response.status,
        });
      }
      lastPayload = (await response.json()) as Record<string, unknown>;
      if (isTerminalStatus(stringValue(lastPayload, ["status"]))) {
        return lastPayload;
      }
      await sleep(Math.max(250, options.pollIntervalMs));
    }
    const degraded = lastPayload ?? { id: runId, status: "timeout" };
    return { ...degraded, degraded_reason: `exa agent timed out after ${options.timeoutMs}ms` };
  }

  private blockFromPayload(
    payload: Record<string, unknown>,
    request: SearchRequest,
    options: ExaAgentRunOptions,
    forcedDegraded?: string
  ): AgenticResearchBlock {
    const degraded: string[] = [];
    if (forcedDegraded) {
      degraded.push(forcedDegraded);
    }
    const payloadDegraded = stringValue(payload, ["degraded_reason", "error", "message"]);
    if (payloadDegraded) {
      degraded.push(sanitizeProviderMessage(payloadDegraded));
    }
    const structuredOutput = extractAgenticOutput(payload);
    if (!structuredOutput) {
      degraded.push("exa agent response did not match expected structured output");
    }
    const status = stringValue(payload, ["status"]) ?? "unknown";
    if (!isSuccessfulTerminalStatus(status)) {
      degraded.push(`exa agent finished with status ${status}`);
    }

    return {
      provider: this.name,
      run_id: stringValue(payload, ["id", "run_id", "runId"]),
      status,
      effort: options.effort,
      private_context_allowed: options.privateContextOk,
      stop_reason: stringValue(payload, ["stop_reason", "stopReason", "reason"]),
      cost: numberValue(payload, ["cost", "cost_usd", "costUsd", "usage.cost", "usage.cost_usd"]),
      citations: structuredOutput?.citations ?? extractAgenticCitations(payload),
      structured_output: structuredOutput ?? {
        summary: stringValue(payload, ["answer", "output_text", "text", "summary"]) ?? "",
        findings: [],
        citations: extractAgenticCitations(payload),
        open_questions: [`No structured output was returned for: ${request.query}`],
      },
      degraded,
    };
  }
}

export class XaiProvider implements ProviderAdapter {
  readonly name = "xai" as const;
  private readonly apiKey: string;
  private readonly model: string;

  constructor(apiKey: string, model = process.env.XAI_SEARCH_MODEL ?? "grok-4.3") {
    this.apiKey = apiKey;
    this.model = model;
  }

  async search(request: SearchRequest): Promise<SearchResult[]> {
    const useXSearch = /\b(people saying|sentiment|trending|discourse|twitter|x\/twitter|social|viral|posts?|handles?)\b/i.test(
      request.query
    );
    const response = await fetchWithTimeout(this.name, "https://api.x.ai/v1/responses", {
      method: "POST",
      headers: {
        authorization: `Bearer ${this.apiKey}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model: this.model,
        input: [
          {
            role: "user",
            content: request.query,
          },
        ],
        tools: [{ type: useXSearch ? "x_search" : "web_search" }],
      }),
    });

    if (!response.ok) {
      throw new ProviderRequestError(this.name, "http", `xai search failed: ${response.status}`, {
        status: response.status,
      });
    }

    const payload = (await response.json()) as {
      citations?: string[];
      output_text?: string;
      output?: Array<{
        content?: Array<{
          text?: string;
        }>;
      }>;
    };
    const snippet =
      payload.output_text ??
      payload.output
        ?.flatMap((item) => item.content ?? [])
        .map((item) => item.text)
        .find((text) => typeof text === "string" && text.trim()) ??
      "";

    return dedupeUrls(payload.citations ?? [])
      .slice(0, request.limit ?? DEFAULT_LIMIT)
      .map((url, index) => ({
        title: "xAI citation",
        url,
        snippet,
        published_at: null,
        score: scoreFromRank(index),
        source_provider: "xai" as const,
      }));
  }
}

export class BraveProvider implements ProviderAdapter {
  readonly name = "brave" as const;
  private readonly apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async search(request: SearchRequest): Promise<SearchResult[]> {
    const query = new URLSearchParams({
      q: request.query,
      count: String(request.limit ?? DEFAULT_LIMIT),
    });

    const freshness = freshnessFromRecencyDays(request.recencyDays);
    if (freshness) {
      query.set("freshness", freshness);
    }

    const response = await fetchWithTimeout(
      this.name,
      `https://api.search.brave.com/res/v1/web/search?${query}`,
      {
      headers: {
        Accept: "application/json",
        "X-Subscription-Token": this.apiKey,
      },
      }
    );

    if (!response.ok) {
      throw new ProviderRequestError(this.name, "http", `brave search failed: ${response.status}`, {
        status: response.status,
      });
    }

    const payload = (await response.json()) as {
      web?: {
        results?: Array<{
          title?: string;
          url?: string;
          description?: string;
          age?: string;
        }>;
      };
    };

    return (payload.web?.results ?? [])
      .filter((item) => Boolean(item.url))
      .map((item, index) => ({
        title: item.title ?? item.url ?? "Untitled",
        url: item.url!,
        snippet: item.description ?? "",
        published_at: item.age ?? null,
        score: scoreFromRank(index),
        source_provider: "brave" as const,
      }));
  }
}

export class PerplexitySynthesisProvider implements ProviderAdapter {
  readonly name = "perplexity" as const;
  private readonly apiKey: string;

  constructor(apiKey: string) {
    this.apiKey = apiKey;
  }

  async search(request: SearchRequest): Promise<SearchResult[]> {
    const synthesis = await this.synthesize(request.query, []);
    return synthesis.citations.map((url, index) => ({
      title: "Perplexity citation",
      url,
      snippet: synthesis.summary,
      published_at: null,
      score: scoreFromRank(index),
      source_provider: "perplexity" as const,
    }));
  }

  async synthesize(
    query: string,
    sources: SearchResult[]
  ): Promise<{ summary: string; citations: string[] }> {
    const sourceLines = sources
      .map((result, index) => `${index + 1}. ${result.title} :: ${result.url}`)
      .join("\n");

    const response = await fetchWithTimeout(this.name, "https://api.perplexity.ai/chat/completions", {
      method: "POST",
      headers: {
        Authorization: `Bearer ${this.apiKey}`,
        "content-type": "application/json",
      },
      body: JSON.stringify({
        model: PERPLEXITY_MODEL,
        messages: [
          {
            role: "system",
            content:
              "Summarize using provided sources. Never invent URLs. Keep uncertainty explicit.",
          },
          {
            role: "user",
            content: [
              `Query: ${query}`,
              "Use these source URLs as ground truth:",
              sourceLines || "(No explicit sources were provided.)",
              "Return concise synthesis and include citations from sources.",
            ].join("\n"),
          },
        ],
      }),
    });

    if (!response.ok) {
      throw new ProviderRequestError(
        this.name,
        "http",
        `perplexity synthesis failed: ${response.status}`,
        { status: response.status }
      );
    }

    const payload = (await response.json()) as {
      choices?: Array<{
        message?: {
          content?: string;
        };
      }>;
      citations?: string[];
      search_results?: Array<{ url?: string }>;
    };

    const modelSummary = payload.choices?.[0]?.message?.content?.trim() ?? "";
    const citations = dedupeUrls([
      ...(payload.citations ?? []),
      ...((payload.search_results ?? []).map((item) => item.url).filter(Boolean) as string[]),
    ]);

    return {
      summary: modelSummary,
      citations,
    };
  }
}

function buildAgentPrompt(request: SearchRequest, privateContextOk: boolean): string {
  return [
    `Research query: ${request.query}`,
    `Command: ${request.command}`,
    "Return concise grounded research as JSON with summary, findings, citations, open_questions, and optional entities.",
    "Each citation should include a URL and any available title/snippet.",
    privateContextOk
      ? "Caller explicitly allowed private context in Agent input."
      : "Do not assume access to private repo/customer context; use public web evidence only.",
  ].join("\n");
}

function extractAgenticOutput(payload: Record<string, unknown>): AgenticResearchOutput | null {
  const candidate =
    objectValue(payload, ["structured_output"]) ??
    objectValue(payload, ["structuredOutput"]) ??
    objectValue(payload, ["output"]) ??
    objectValue(payload, ["result"]) ??
    objectValue(payload, ["data"]);

  if (!candidate) {
    return null;
  }
  const summary = stringValue(candidate, ["summary", "answer", "text", "output_text"]);
  const citations = extractAgenticCitations(candidate);
  if (!summary || citations.length === 0) {
    return null;
  }
  return {
    summary,
    findings: arrayValue(candidate, ["findings"]) ?? [],
    citations,
    open_questions: stringArrayValue(candidate, ["open_questions", "openQuestions"]) ?? [],
    ...(arrayValue(candidate, ["entities"]) ? { entities: arrayValue(candidate, ["entities"]) } : {}),
  };
}

function extractAgenticCitations(payload: Record<string, unknown>): AgenticCitation[] {
  const raw =
    arrayValue(payload, ["citations"]) ??
    arrayValue(payload, ["grounding", "citations"]) ??
    arrayValue(payload, ["groundings"]) ??
    arrayValue(payload, ["sources"]) ??
    [];
  return raw
    .map((item) => {
      if (typeof item === "string") {
        return { url: item };
      }
      if (!item || typeof item !== "object") {
        return null;
      }
      const record = item as Record<string, unknown>;
      const url = stringValue(record, ["url", "source", "href"]);
      if (!url) {
        return null;
      }
      return {
        url,
        ...(stringValue(record, ["title"]) ? { title: stringValue(record, ["title"])! } : {}),
        ...(stringValue(record, ["snippet", "text"]) ? { snippet: stringValue(record, ["snippet", "text"])! } : {}),
      };
    })
    .filter((item): item is AgenticCitation => Boolean(item?.url));
}

function sanitizeProviderMessage(message: string): string {
  return message.replace(
    /(x-api-key|api[_ -]?key|authorization)[:=]\s*[^\s,;]+/gi,
    "$1=[redacted]"
  );
}

function isTerminalStatus(status: string | null | undefined): boolean {
  if (!status) {
    return false;
  }
  return ["completed", "succeeded", "failed", "cancelled", "canceled", "timeout"].includes(
    status.toLowerCase()
  );
}

function isSuccessfulTerminalStatus(status: string): boolean {
  return ["completed", "succeeded"].includes(status.toLowerCase());
}

function stringValue(record: Record<string, unknown>, keys: string[]): string | null {
  for (const key of keys) {
    const value = valueAt(record, key);
    if (typeof value === "string" && value.trim()) {
      return value.trim();
    }
  }
  return null;
}

function numberValue(record: Record<string, unknown>, keys: string[]): number | null {
  for (const key of keys) {
    const value = valueAt(record, key);
    if (typeof value === "number" && Number.isFinite(value)) {
      return value;
    }
    if (typeof value === "string" && Number.isFinite(Number(value))) {
      return Number(value);
    }
  }
  return null;
}

function objectValue(record: Record<string, unknown>, keys: string[]): Record<string, unknown> | null {
  for (const key of keys) {
    const value = valueAt(record, key);
    if (value && typeof value === "object" && !Array.isArray(value)) {
      return value as Record<string, unknown>;
    }
    if (typeof value === "string" && value.trim().startsWith("{")) {
      try {
        const parsed = JSON.parse(value);
        if (parsed && typeof parsed === "object" && !Array.isArray(parsed)) {
          return parsed as Record<string, unknown>;
        }
      } catch {
        continue;
      }
    }
  }
  return null;
}

function arrayValue(record: Record<string, unknown>, keys: string[]): unknown[] | null {
  for (const key of keys) {
    const value = valueAt(record, key);
    if (Array.isArray(value)) {
      return value;
    }
  }
  return null;
}

function stringArrayValue(record: Record<string, unknown>, keys: string[]): string[] | null {
  const values = arrayValue(record, keys);
  if (!values) {
    return null;
  }
  return values.filter((value): value is string => typeof value === "string" && value.trim());
}

function valueAt(record: Record<string, unknown>, key: string): unknown {
  return key.split(".").reduce<unknown>((current, part) => {
    if (!current || typeof current !== "object" || Array.isArray(current)) {
      return undefined;
    }
    return (current as Record<string, unknown>)[part];
  }, record);
}

function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function scoreFromRank(index: number): number {
  const value = 1 - index * 0.05;
  return value > 0 ? value : 0;
}

function freshnessFromRecencyDays(recencyDays: number | undefined): "pd" | "pw" | "pm" | "py" | null {
  if (typeof recencyDays !== "number") {
    return null;
  }
  if (recencyDays <= 1) {
    return "pd";
  }
  if (recencyDays <= 7) {
    return "pw";
  }
  if (recencyDays <= 31) {
    return "pm";
  }
  return "py";
}

function firstSnippet(snippets: string[] | undefined): string {
  if (!snippets || snippets.length === 0) {
    return "";
  }
  return snippets[0];
}

function dedupeUrls(urls: string[]): string[] {
  const seen = new Set<string>();
  const deduped: string[] = [];
  for (const url of urls) {
    const normalized = url.trim();
    if (!normalized || seen.has(normalized)) {
      continue;
    }
    seen.add(normalized);
    deduped.push(normalized);
  }
  return deduped;
}

function extractBestDocSnippet(payload: Record<string, unknown>): string | null {
  const directText = payload.content;
  if (typeof directText === "string" && directText.trim()) {
    return truncateSnippet(directText);
  }

  const textField = payload.text;
  if (typeof textField === "string" && textField.trim()) {
    return truncateSnippet(textField);
  }

  const chunks = payload.chunks;
  if (Array.isArray(chunks)) {
    for (const chunk of chunks) {
      if (typeof chunk === "string" && chunk.trim()) {
        return truncateSnippet(chunk);
      }
      if (
        typeof chunk === "object" &&
        chunk &&
        "content" in chunk &&
        typeof (chunk as { content?: unknown }).content === "string"
      ) {
        return truncateSnippet((chunk as { content: string }).content);
      }
    }
  }

  return null;
}

function truncateSnippet(input: string): string {
  const trimmed = input.trim().replace(/\s+/g, " ");
  return trimmed.length > 480 ? `${trimmed.slice(0, 477)}...` : trimmed;
}

function providerTimeoutMs(): number {
  const raw = Number(process.env.WEB_SEARCH_PROVIDER_TIMEOUT_MS);
  if (!Number.isFinite(raw) || raw <= 0) {
    return DEFAULT_PROVIDER_TIMEOUT_MS;
  }
  return Math.min(Math.floor(raw), 120_000);
}
