import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, test } from "bun:test";

import { QueryCache } from "../cache";
import { runResearch } from "../cli";
import { WebSearchOrchestrator } from "../orchestrator";
import type { ProviderAdapter, SearchRequest, SearchResult } from "../provider-adapter";
import { fetchWithTimeout, ProviderRequestError, XaiProvider } from "../providers";

const REQUEST: SearchRequest = {
  query: "latest harness news",
  command: "web",
};

function result(url: string, provider: ProviderAdapter["name"] = "exa"): SearchResult {
  return {
    title: url || "missing url",
    url,
    snippet: "",
    published_at: null,
    score: 1,
    source_provider: provider,
  };
}

describe("research runtime hardening", () => {
  test("cache preserves unrelated concurrent writes", async () => {
    const dir = await mkdtemp(path.join(os.tmpdir(), "research-cache-"));
    try {
      const cache = new QueryCache<SearchResult[]>({
        filePath: path.join(dir, "cache.json"),
        ttlMs: 60_000,
      });

      const requests = Array.from({ length: 20 }, (_, index) => ({
        query: `topic ${index}`,
        command: "web" as const,
      }));

      await Promise.all(
        requests.map((request, index) =>
          cache.set(request, [result(`https://example.com/${index}`)])
        )
      );

      const cached = await Promise.all(requests.map((request) => cache.get(request)));

      expect(cached.every((entry) => entry?.length === 1)).toBe(true);
      expect(new Set(cached.map((entry) => entry?.[0].url)).size).toBe(requests.length);
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });

  test("provider fetch timeout returns structured failure", async () => {
    const neverFetch = () => new Promise<Response>(() => {});

    const promise = fetchWithTimeout("exa", "https://example.com", {
      fetchImpl: neverFetch,
      timeoutMs: 1,
    });

    await expect(promise).rejects.toBeInstanceOf(ProviderRequestError);
    await expect(promise).rejects.toMatchObject({
      provider: "exa",
      kind: "timeout",
    });
  });

  test("synthesis failure degrades a successful deep research response", async () => {
    const provider: ProviderAdapter = {
      name: "exa",
      async search() {
        return [result("https://example.com/source")];
      },
    };

    const response = await runResearch(
      { command: "web-deep", query: "explain harness kit" },
      {
        providers: [provider],
        cache: null,
        logPath: null,
        synthesizer: {
          async synthesize() {
            throw new Error("synthesis unavailable");
          },
        },
      }
    );

    expect(response.results).toHaveLength(1);
    expect(response.agentic).toBeNull();
    expect(response.synthesis).toBeNull();
    expect(response.meta.degraded).toContain("synthesis failed: Error: synthesis unavailable");
  });

  test("agentic research output maps into an explicit response block", async () => {
    const provider: ProviderAdapter = {
      name: "exa",
      async search() {
        return [result("https://example.com/source")];
      },
    };

    const response = await runResearch(
      { command: "web-deep", query: "prior art landscape for agent skills" },
      {
        env: {
          EXA_API_KEY: "exa-key",
          EXA_AGENT_ENABLED: "1",
          EXA_AGENT_EFFORT: "low",
        },
        providers: [provider],
        cache: null,
        logPath: null,
        synthesizer: null,
        agenticProvider: {
          async run() {
            return {
              provider: "exa-agent",
              run_id: "run_123",
              status: "completed",
              effort: "low",
              private_context_allowed: false,
              stop_reason: "done",
              cost: 0.12,
              citations: [{ url: "https://example.com/agent", title: "Agent citation" }],
              structured_output: {
                summary: "Grounded agentic summary",
                findings: [{ claim: "useful" }],
                citations: [{ url: "https://example.com/agent", title: "Agent citation" }],
                open_questions: [],
              },
              degraded: [],
            };
          },
        },
      }
    );

    expect(response.results).toHaveLength(1);
    expect(response.agentic?.provider).toBe("exa-agent");
    expect(response.agentic?.run_id).toBe("run_123");
    expect(response.agentic?.private_context_allowed).toBe(false);
    expect(response.agentic?.citations[0].url).toBe("https://example.com/agent");
    expect(response.meta.degraded).toEqual([]);
  });

  test("agentic research records explicit private context consent", async () => {
    const provider: ProviderAdapter = {
      name: "exa",
      async search() {
        return [result("https://example.com/source")];
      },
    };
    let observedPrivateContextOk = false;

    const response = await runResearch(
      { command: "web-deep", query: "prior art landscape for agent skills" },
      {
        env: {
          EXA_API_KEY: "exa-key",
          EXA_AGENT_ENABLED: "1",
          EXA_AGENT_PRIVATE_CONTEXT_OK: "1",
        },
        providers: [provider],
        cache: null,
        logPath: null,
        synthesizer: null,
        agenticProvider: {
          async run(_request, options) {
            observedPrivateContextOk = options.privateContextOk;
            return {
              provider: "exa-agent",
              run_id: "run_private",
              status: "completed",
              effort: options.effort,
              private_context_allowed: options.privateContextOk,
              stop_reason: "done",
              cost: null,
              citations: [{ url: "https://example.com/private-ok" }],
              structured_output: {
                summary: "Private context was explicitly acknowledged",
                findings: [],
                citations: [{ url: "https://example.com/private-ok" }],
                open_questions: [],
              },
              degraded: [],
            };
          },
        },
      }
    );

    expect(observedPrivateContextOk).toBe(true);
    expect(response.agentic?.private_context_allowed).toBe(true);
  });

  test("agentic research failure degrades to ordinary retrieval", async () => {
    const provider: ProviderAdapter = {
      name: "exa",
      async search() {
        return [result("https://example.com/source")];
      },
    };

    const response = await runResearch(
      { command: "web-deep", query: "prior art landscape for agent skills" },
      {
        env: {
          EXA_API_KEY: "exa-key",
          EXA_AGENT_ENABLED: "1",
        },
        providers: [provider],
        cache: null,
        logPath: null,
        synthesizer: null,
        agenticProvider: {
          async run() {
            throw new Error("agent unavailable");
          },
        },
      }
    );

    expect(response.results).toHaveLength(1);
    expect(response.agentic).toBeNull();
    expect(response.meta.degraded).toContain("agentic failed: Error: agent unavailable");
  });

  test("xAI provider maps citations from Responses API payload", async () => {
    const originalFetch = globalThis.fetch;
    globalThis.fetch = (async () =>
      new Response(
        JSON.stringify({
          output_text: "Grounded answer",
          citations: ["https://x.ai/docs", "https://docs.x.ai/search"],
        }),
        { status: 200, headers: { "content-type": "application/json" } }
      )) as typeof fetch;

    try {
      const provider = new XaiProvider("test-key", "grok-test");
      const results = await provider.search({
        query: "what are people saying about Grok search",
        command: "web",
        limit: 2,
      });

      expect(results.map((item) => item.source_provider)).toEqual(["xai", "xai"]);
      expect(results.map((item) => item.url)).toEqual([
        "https://x.ai/docs",
        "https://docs.x.ai/search",
      ]);
      expect(results[0].snippet).toBe("Grounded answer");
    } finally {
      globalThis.fetch = originalFetch;
    }
  });

  test("empty post-dedupe results are not cached as success", async () => {
    const dir = await mkdtemp(path.join(os.tmpdir(), "research-empty-"));
    try {
      const provider: ProviderAdapter = {
        name: "exa",
        async search() {
          return [result("")];
        },
      };
      const cache = new QueryCache<SearchResult[]>({
        filePath: path.join(dir, "cache.json"),
        ttlMs: 60_000,
      });
      const orchestrator = new WebSearchOrchestrator([provider], { cache });

      await expect(orchestrator.searchWithMeta(REQUEST)).rejects.toThrow(
        "all providers returned no usable results"
      );
      await expect(cache.get(REQUEST)).resolves.toBeNull();
    } finally {
      await rm(dir, { recursive: true, force: true });
    }
  });
});
