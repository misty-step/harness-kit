export type WebCommand = "web" | "web-deep" | "web-news" | "web-docs";

export type SearchProviderName = "context7" | "exa" | "brave" | "perplexity";

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

export interface SearchResponse {
  results: SearchResult[];
  meta: SearchMeta;
  synthesis: {
    summary: string;
    citations: string[];
  } | null;
}

export interface ProviderAdapter {
  readonly name: SearchProviderName;
  search(request: SearchRequest): Promise<SearchResult[]>;
}
