import { describe, expect, test } from "bun:test";

import { buildProviders, selectExaAgent } from "../cli";

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

  test("keeps Exa Agent off by default for ordinary deep research", () => {
    const selection = selectExaAgent(
      { command: "web-deep", query: "explain harness kit" },
      { EXA_API_KEY: "exa-key" }
    );

    expect(selection.enabled).toBe(false);
  });

  test("selects Exa Agent only for opt-in or broad agentic signals", () => {
    expect(
      selectExaAgent(
        { command: "web-deep", query: "compare options across sources for agent skill marketplaces" },
        { EXA_API_KEY: "exa-key" }
      ).enabled
    ).toBe(true);
    expect(
      selectExaAgent(
        { command: "web-deep", query: "explain harness kit" },
        { EXA_API_KEY: "exa-key", EXA_AGENT_ENABLED: "1" }
      ).enabled
    ).toBe(true);
  });

  test("skips Exa Agent for missing key, non-deep commands, and docs lookups", () => {
    expect(
      selectExaAgent(
        { command: "web-deep", query: "prior art landscape for agent skills" },
        {}
      ).enabled
    ).toBe(false);
    expect(
      selectExaAgent(
        { command: "web", query: "prior art landscape for agent skills" },
        { EXA_API_KEY: "exa-key", EXA_AGENT_ENABLED: "1" }
      ).enabled
    ).toBe(false);
    expect(
      selectExaAgent(
        { command: "web-deep", query: "React documentation useEffect" },
        { EXA_API_KEY: "exa-key", EXA_AGENT_ENABLED: "1" }
      ).enabled
    ).toBe(false);
  });

  test("requires explicit expensive-effort acknowledgment", () => {
    expect(
      selectExaAgent(
        { command: "web-deep", query: "prior art landscape for agent skills" },
        { EXA_API_KEY: "exa-key", EXA_AGENT_EFFORT: "high" }
      ).configError
    ).toBe("EXA_AGENT_EFFORT=high requires EXA_AGENT_ALLOW_EXPENSIVE=1");
    expect(
      selectExaAgent(
        { command: "web-deep", query: "prior art landscape for agent skills" },
        { EXA_API_KEY: "exa-key", EXA_AGENT_EFFORT: "high", EXA_AGENT_ALLOW_EXPENSIVE: "1" }
      ).configError
    ).toBeNull();
  });
});
