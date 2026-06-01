import { mkdtemp, rm } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { describe, expect, test } from "bun:test";

import { QueryCache } from "../cache";
import { runResearch } from "../cli";
import { WebSearchOrchestrator } from "../orchestrator";
import type { ProviderAdapter, SearchRequest, SearchResult } from "../provider-adapter";
import { fetchWithTimeout, ProviderRequestError } from "../providers";

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
    expect(response.synthesis).toBeNull();
    expect(response.meta.degraded).toContain("synthesis failed: Error: synthesis unavailable");
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
