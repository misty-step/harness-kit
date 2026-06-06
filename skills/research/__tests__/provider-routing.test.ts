import { describe, expect, test } from "bun:test";

import { buildProviders } from "../cli";

describe("provider routing", () => {
  test("routes social discourse to xAI before Exa fallback", () => {
    const providers = buildProviders(
      { command: "web", query: "what are people saying about Exa MCP" },
      {
        EXA_API_KEY: "exa-key",
        XAI_API_KEY: "xai-key",
        BRAVE_API_KEY: "brave-key",
      }
    );

    expect(providers.map((provider) => provider.name)).toEqual(["xai", "exa", "brave"]);
  });

  test("uses xAI as recency corroboration after Exa for current queries", () => {
    const providers = buildProviders(
      { command: "web-news", query: "latest Grok search API changes" },
      {
        EXA_API_KEY: "exa-key",
        XAI_API_KEY: "xai-key",
        BRAVE_API_KEY: "brave-key",
      }
    );

    expect(providers.map((provider) => provider.name)).toEqual(["exa", "xai", "brave"]);
  });

  test("keeps docs lookups docs-first", () => {
    const providers = buildProviders(
      { command: "web-docs", query: "xAI API docs" },
      {
        CONTEXT7_API_KEY: "context-key",
        EXA_API_KEY: "exa-key",
        XAI_API_KEY: "xai-key",
      }
    );

    expect(providers.map((provider) => provider.name)).toEqual(["context7", "exa"]);
  });
});
