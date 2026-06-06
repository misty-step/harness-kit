import type { SearchRequest, SearchResult, WebCommand } from "./provider-adapter";

const TIME_SENSITIVE_PATTERN =
  /\b(latest|today|current|currently|now|recent|newest|breaking|this week|this month)\b/i;
const DOCS_PATTERN = /\b(api|sdk|docs?|documentation|reference|library|framework|package)\b/i;
const SOCIAL_DISCOURSE_PATTERN =
  /\b(people saying|what are people saying|sentiment|trending|discourse|twitter|x\/twitter|social|viral|posts?|handles?)\b/i;
const TRACKING_QUERY_KEYS = new Set([
  "utm_source",
  "utm_medium",
  "utm_campaign",
  "utm_term",
  "utm_content",
  "fbclid",
  "gclid",
  "ref",
]);

export function normalizeQuery(query: string): string {
  return query.trim().toLowerCase().replace(/\s+/g, " ");
}

export function isDocsLookup(query: string, command: WebCommand): boolean {
  if (command === "web-docs") {
    return true;
  }
  return DOCS_PATTERN.test(query);
}

export function isTimeSensitiveQuery(query: string, command: WebCommand): boolean {
  if (command === "web-news") {
    return true;
  }
  return TIME_SENSITIVE_PATTERN.test(query);
}

export function isSocialDiscourseQuery(query: string): boolean {
  return SOCIAL_DISCOURSE_PATTERN.test(query);
}

export function inferRecencyDays(request: SearchRequest): number | null {
  if (typeof request.recencyDays === "number") {
    const bounded = Math.max(1, Math.min(3650, Math.floor(request.recencyDays)));
    return bounded;
  }

  if (request.command === "web-news") {
    return 7;
  }

  if (isTimeSensitiveQuery(request.query, request.command)) {
    return 30;
  }

  return null;
}

export function isoTimestampDaysAgo(days: number): string {
  const nowMs = Date.now();
  const deltaMs = days * 24 * 60 * 60 * 1000;
  return new Date(nowMs - deltaMs).toISOString();
}

export function canonicalizeUrl(rawUrl: string): string {
  const input = rawUrl.trim();
  if (!input) {
    return input;
  }

  try {
    const parsed = new URL(input);
    parsed.hash = "";
    const keep = new URLSearchParams();
    for (const [key, value] of parsed.searchParams.entries()) {
      if (!TRACKING_QUERY_KEYS.has(key.toLowerCase())) {
        keep.append(key, value);
      }
    }

    const sorted = [...keep.entries()].sort(([a], [b]) => a.localeCompare(b));
    parsed.search = "";
    for (const [key, value] of sorted) {
      parsed.searchParams.append(key, value);
    }

    if (parsed.pathname !== "/" && parsed.pathname.endsWith("/")) {
      parsed.pathname = parsed.pathname.replace(/\/+$/, "");
    }
    return parsed.toString();
  } catch {
    return input;
  }
}

export function dedupeByCanonicalUrl(results: SearchResult[]): SearchResult[] {
  const seen = new Set<string>();
  const deduped: SearchResult[] = [];

  for (const result of results) {
    const normalizedUrl = canonicalizeUrl(result.url);
    if (!normalizedUrl || seen.has(normalizedUrl)) {
      continue;
    }
    seen.add(normalizedUrl);
    deduped.push({
      ...result,
      url: normalizedUrl,
    });
  }

  return deduped;
}
