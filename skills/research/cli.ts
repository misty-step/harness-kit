import path from "node:path";

import { QueryCache } from "./cache";
import { assessConfidence } from "./confidence";
import { WebSearchOrchestrator } from "./orchestrator";
import {
  Context7Provider,
  BraveProvider,
  ExaAgentProvider,
  ExaProvider,
  PerplexitySynthesisProvider,
  XaiProvider,
} from "./providers";
import type {
  AgenticEffort,
  AgenticResearchBlock,
  ProviderAdapter,
  SearchResponse,
  SearchResult,
  WebCommand,
} from "./provider-adapter";
import {
  inferRecencyDays,
  isDocsLookup,
  isSocialDiscourseQuery,
  isTimeSensitiveQuery,
  normalizeQuery,
} from "./query-utils";

export interface CliInput {
  command: WebCommand;
  query: string;
}

interface RunResearchOptions {
  env?: Record<string, string | undefined>;
  providers?: ProviderAdapter[];
  agenticProvider?: Pick<ExaAgentProvider, "run"> | null;
  synthesizer?: Pick<PerplexitySynthesisProvider, "synthesize"> | null;
  cache?: QueryCache<SearchResult[]> | null;
  logPath?: string | null;
  configDir?: string;
  cacheTtlMs?: number;
  limit?: number;
}

export async function runResearch(
  input: CliInput,
  options: RunResearchOptions = {}
): Promise<SearchResponse> {
  const env = options.env ?? process.env;
  const configDir = options.configDir ?? env.PI_CONFIG_DIR ?? path.resolve(process.cwd(), "..", "..");
  const cacheTtlMs =
    options.cacheTtlMs ?? (Number(env.WEB_SEARCH_TTL_MS) || 30 * 60 * 1000);
  const limit = options.limit ?? (Number(env.WEB_SEARCH_MAX_RESULTS) || 5);

  const request = {
    query: input.query,
    command: input.command,
    limit,
    recencyDays: inferRecencyDays({
      query: input.query,
      command: input.command,
      limit,
    }),
  };

  const providers = options.providers ?? buildProviders(input, env);

  if (providers.length === 0) {
    throw new Error(
      "no retrieval providers configured; set CONTEXT7_API_KEY and/or EXA_API_KEY and/or XAI_API_KEY and/or BRAVE_API_KEY"
    );
  }

  const cache =
    options.cache === undefined
      ? new QueryCache<SearchResult[]>({
          filePath: path.join(configDir, "cache", "web-search-cache.json"),
          ttlMs: cacheTtlMs,
        })
      : options.cache ?? undefined;

  const orchestrator = new WebSearchOrchestrator(providers, {
    cache,
    logPath:
      options.logPath === undefined
        ? path.join(configDir, "logs", "web-search.ndjson")
        : options.logPath ?? undefined,
  });

  const { results, meta } = await orchestrator.searchWithMeta(request);
  const confidence = assessConfidence(request, results);

  let agentic: AgenticResearchBlock | null = null;
  let synthesis: SearchResponse["synthesis"] = null;
  const degraded: string[] = [];
  const agenticSelection = selectExaAgent(input, env);
  if (agenticSelection.configError) {
    degraded.push(agenticSelection.configError);
  }
  const agenticProvider =
    options.agenticProvider === undefined && agenticSelection.enabled && env.EXA_API_KEY
      ? new ExaAgentProvider(env.EXA_API_KEY)
      : options.agenticProvider;
  if (agenticSelection.enabled && agenticProvider && !agenticSelection.configError) {
    try {
      agentic = await agenticProvider.run(request, {
        effort: agenticSelection.effort,
        timeoutMs: boundedInt(env.EXA_AGENT_TIMEOUT_MS, 90_000, 5_000, 15 * 60_000),
        pollIntervalMs: boundedInt(env.EXA_AGENT_POLL_INTERVAL_MS, 2_000, 250, 30_000),
        privateContextOk: env.EXA_AGENT_PRIVATE_CONTEXT_OK === "1",
      });
      degraded.push(...agentic.degraded.map((item) => `agentic: ${item}`));
    } catch (error) {
      degraded.push(`agentic failed: ${String(error)}`);
    }
  }

  const synthesizer =
    options.synthesizer === undefined && env.PERPLEXITY_API_KEY
      ? new PerplexitySynthesisProvider(env.PERPLEXITY_API_KEY)
      : options.synthesizer;
  if (input.command === "web-deep" && synthesizer && results.length > 0) {
    try {
      const generated = await synthesizer.synthesize(input.query, results);
      synthesis = generated.citations.length > 0 ? generated : null;
    } catch (error) {
      degraded.push(`synthesis failed: ${String(error)}`);
    }
  }

  return {
    results,
    meta: {
      query: input.query,
      normalized_query: normalizeQuery(input.query),
      command: input.command,
      provider_chain: meta.providerChain,
      provider_used: meta.providerUsed,
      cache_hit: meta.cacheHit,
      time_sensitive: isTimeSensitiveQuery(input.query, input.command),
      recency_days: request.recencyDays ?? null,
      confidence: confidence.confidence,
      uncertainty: confidence.uncertainty,
      degraded,
    },
    agentic,
    synthesis,
  };
}

async function main(): Promise<void> {
  const input = parseArgs(process.argv.slice(2));
  const response = await runResearch(input);

  process.stdout.write(`${JSON.stringify(response, null, 2)}\n`);
}

export function buildProviders(
  input: CliInput,
  env: Record<string, string | undefined>
): ProviderAdapter[] {
  const providers: ProviderAdapter[] = [];
  const useContext7 = Boolean(env.CONTEXT7_API_KEY) && isDocsLookup(input.query, input.command);
  const useXai = Boolean(env.XAI_API_KEY) && isSocialDiscourseQuery(input.query);
  if (useContext7) {
    providers.push(new Context7Provider(env.CONTEXT7_API_KEY!));
  }
  if (useXai) {
    providers.push(new XaiProvider(env.XAI_API_KEY!));
  }
  if (env.EXA_API_KEY) {
    providers.push(new ExaProvider(env.EXA_API_KEY));
  }
  if (env.XAI_API_KEY && !useXai && isTimeSensitiveQuery(input.query, input.command)) {
    providers.push(new XaiProvider(env.XAI_API_KEY));
  }
  if (env.BRAVE_API_KEY) {
    providers.push(new BraveProvider(env.BRAVE_API_KEY));
  }
  return providers;
}

export function selectExaAgent(
  input: CliInput,
  env: Record<string, string | undefined>
): { enabled: boolean; effort: AgenticEffort; configError: string | null } {
  const effort = parseAgentEffort(env.EXA_AGENT_EFFORT);
  const expensiveEffort = effort === "high" || effort === "xhigh" || effort === "auto";
  const configError =
    expensiveEffort && env.EXA_AGENT_ALLOW_EXPENSIVE !== "1"
      ? `EXA_AGENT_EFFORT=${effort} requires EXA_AGENT_ALLOW_EXPENSIVE=1`
      : null;
  if (input.command !== "web-deep" || !env.EXA_API_KEY) {
    return { enabled: false, effort, configError: null };
  }
  if (isDocsLookup(input.query, input.command) || isSocialDiscourseQuery(input.query)) {
    return { enabled: false, effort, configError: null };
  }
  const explicitlyEnabled = env.EXA_AGENT_ENABLED === "1";
  const positiveSignal = hasAgenticResearchSignal(input.query);
  return {
    enabled: explicitlyEnabled || positiveSignal,
    effort,
    configError,
  };
}

function hasAgenticResearchSignal(query: string): boolean {
  return /\b(use exa agent|agentic research|build (a )?list|list prospects|enrich (these )?(entities|companies|people)|compare options across sources|prior art landscape|multi-entity|multi entity|landscape scan|market map)\b/i.test(
    query
  );
}

function parseAgentEffort(value: string | undefined): AgenticEffort {
  if (
    value === "minimal" ||
    value === "low" ||
    value === "medium" ||
    value === "high" ||
    value === "xhigh" ||
    value === "auto"
  ) {
    return value;
  }
  return "medium";
}

function boundedInt(
  value: string | undefined,
  fallback: number,
  min: number,
  max: number
): number {
  const parsed = Number(value);
  if (!Number.isFinite(parsed)) {
    return fallback;
  }
  return Math.max(min, Math.min(max, Math.floor(parsed)));
}

function parseArgs(args: string[]): CliInput {
  if (args.length < 2) {
    throw new Error("usage: web-search <web|web-deep|web-news|web-docs> <query>");
  }

  const [command, ...queryParts] = args;
  if (
    command !== "web" &&
    command !== "web-deep" &&
    command !== "web-news" &&
    command !== "web-docs"
  ) {
    throw new Error("command must be one of: web, web-deep, web-news, web-docs");
  }

  const query = queryParts.join(" ").trim();
  if (!query) {
    throw new Error("query must not be empty");
  }

  return {
    command,
    query,
  };
}

if (import.meta.main) {
  main().catch((error) => {
    process.stderr.write(`${String(error)}\n`);
    process.exit(1);
  });
}
