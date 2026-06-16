export type WebCommand = "web" | "web-deep" | "web-news" | "web-docs";

export type SearchProviderName = "context7" | "exa" | "xai" | "brave" | "perplexity";
export type AgenticProviderName = "exa-agent";
export type AgenticEffort = "minimal" | "low" | "medium" | "high" | "xhigh" | "auto";

export type ConfidenceLevel = "high" | "medium" | "low";

export interface SearchRequest {
  query: string;
  command: WebCommand;
  limit?: number;
  recencyDays?: number;
}

export interface SearchResult {
  title: string;
  url: string;
  snippet: string;
  published_at: string | null;
  score: number;
  source_provider: SearchProviderName;
}

export interface SearchMeta {
  query: string;
  normalized_query: string;
  command: WebCommand;
  provider_chain: SearchProviderName[];
  provider_used: SearchProviderName | null;
  cache_hit: boolean;
  time_sensitive: boolean;
  recency_days: number | null;
  confidence: ConfidenceLevel;
  uncertainty: string | null;
  degraded: string[];
}

export interface AgenticCitation {
  url: string;
  title?: string;
  snippet?: string;
}

export interface AgenticResearchOutput {
  summary: string;
  findings: unknown[];
  citations: AgenticCitation[];
  open_questions: string[];
  entities?: unknown[];
}

export interface AgenticResearchBlock {
  provider: AgenticProviderName;
  run_id: string | null;
  status: string;
  effort: AgenticEffort;
  private_context_allowed: boolean;
  stop_reason: string | null;
  cost: number | null;
  citations: AgenticCitation[];
  structured_output: AgenticResearchOutput | null;
  degraded: string[];
}

export interface SearchResponse {
  results: SearchResult[];
  meta: SearchMeta;
  agentic: AgenticResearchBlock | null;
  synthesis: {
    summary: string;
    citations: string[];
  } | null;
}

export interface ProviderAdapter {
  readonly name: SearchProviderName;
  search(request: SearchRequest): Promise<SearchResult[]>;
}
