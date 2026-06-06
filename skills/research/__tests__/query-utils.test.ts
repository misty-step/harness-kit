import { describe, expect, test } from "bun:test";

import {
  canonicalizeUrl,
  dedupeByCanonicalUrl,
  inferRecencyDays,
  isDocsLookup,
  isSocialDiscourseQuery,
  isTimeSensitiveQuery,
  normalizeQuery,
} from "../query-utils";
import type { SearchResult } from "../provider-adapter";

describe("query-utils", () => {
  test("normalizeQuery trims, lowers, and collapses spaces", () => {
    expect(normalizeQuery("  Latest   Next.js   docs  ")).toBe("latest next.js docs");
  });

  test("isDocsLookup detects docs-like queries and explicit docs mode", () => {
    expect(isDocsLookup("react router docs", "web")).toBe(true);
    expect(isDocsLookup("weather in sf", "web-docs")).toBe(true);
    expect(isDocsLookup("weather in sf", "web")).toBe(false);
  });

  test("isTimeSensitiveQuery detects recency intents", () => {
    expect(isTimeSensitiveQuery("latest openai api updates", "web")).toBe(true);
    expect(isTimeSensitiveQuery("postgres indexing guide", "web-news")).toBe(true);
    expect(isTimeSensitiveQuery("postgres indexing guide", "web")).toBe(false);
  });

  test("isSocialDiscourseQuery detects X and sentiment intent", () => {
    expect(isSocialDiscourseQuery("what are people saying about Exa MCP")).toBe(true);
    expect(isSocialDiscourseQuery("X/Twitter discourse on Grok search")).toBe(true);
    expect(isSocialDiscourseQuery("postgres indexing guide")).toBe(false);
  });

  test("inferRecencyDays applies explicit value and defaults", () => {
    expect(inferRecencyDays({ query: "latest ai news", command: "web" })).toBe(30);
    expect(inferRecencyDays({ query: "new release", command: "web-news" })).toBe(7);
    expect(inferRecencyDays({ query: "evergreen topic", command: "web" })).toBeNull();
    expect(
      inferRecencyDays({
        query: "x",
        command: "web",
        recencyDays: 99999,
      })
    ).toBe(3650);
  });

  test("canonicalizeUrl strips tracking params and dedupes", () => {
    const inputs: SearchResult[] = [
      {
        title: "A",
        url: "https://example.com/docs?utm_source=test&id=1",
        snippet: "",
        published_at: null,
        score: 1,
        source_provider: "exa",
      },
      {
        title: "B",
        url: "https://example.com/docs?id=1&utm_medium=email",
        snippet: "",
        published_at: null,
        score: 0.8,
        source_provider: "brave",
      },
    ];

    expect(canonicalizeUrl(inputs[0].url)).toBe("https://example.com/docs?id=1");
    const deduped = dedupeByCanonicalUrl(inputs);
    expect(deduped).toHaveLength(1);
    expect(deduped[0].url).toBe("https://example.com/docs?id=1");
  });
});
