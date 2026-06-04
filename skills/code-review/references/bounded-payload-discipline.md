# Bounded-Payload Discipline

Any API response that advertises a cap must enforce the cap before
materializing the collection, not by slicing an unbounded in-memory result.

Use this when a diff touches pagination, "top N" lists, newest/oldest lists,
truncated flags, summary totals, or bounded nested data. The concrete ORM
syntax differs by repo; the invariant does not.

## Antipattern

```text
fetch all rows -> slice in memory -> return "top N"
```

This is wrong even when the returned JSON looks capped. Memory, latency, and
database work still scale with the uncapped row count. It also tempts callers
to compute totals from the sliced view, so `total_count`, `has_more`, or
`truncated` fields can silently lie.

## Decision Tree

Choose one of two shapes.

### Shape A: Bounded Fetch

Use this when the caller only needs the bounded list and does not need a true
total or truncation flag.

```text
query rows with ORDER BY + LIMIT/take -> render list
```

The cap belongs in the data access call that fetches the rows.

### Shape B: Count Plus Bounded Fetch

Use this when the payload needs both a bounded list and a true total,
`has_more`, or `truncated` flag.

```text
count matching rows -> fetch ordered bounded rows -> compute metadata
```

Two bounded/aggregate queries are usually cheaper and clearer than one
unbounded read. The count answers the total question. The bounded fetch answers
the display question.

## Elixir + Ecto

Bad:

```elixir
incident =
  Incident
  |> Repo.get!(id)
  |> Repo.preload(:signals)

top_signals =
  incident.signals
  |> Enum.sort_by(& &1.inserted_at, {:desc, DateTime})
  |> Enum.take(limit)
```

The database can return every `signals` row before the cap is applied.

Also avoid presenting a preload query `limit` as a per-parent cap. Ecto's
preload query `limit` and `offset` apply to the whole preload result set, not
to each parent association. If you need top rows for a single parent/read
model, make that a bounded query for that parent:

```elixir
def fetch_top_signals(incident_id, limit) do
  Signal
  |> where([s], s.incident_id == ^incident_id)
  |> order_by([s], desc: s.inserted_at)
  |> limit(^limit)
  |> Repo.all()
end
```

If the response needs a true total:

```elixir
def signal_summary(incident_id, limit) do
  total =
    Signal
    |> where([s], s.incident_id == ^incident_id)
    |> Repo.aggregate(:count, :id)

  signals = fetch_top_signals(incident_id, limit)

  %{
    total_count: total,
    signals: signals,
    truncated: total > length(signals)
  }
end
```

For per-parent top-N across multiple parents, use a query shape that actually
expresses per-parent ranking, such as a windowed query, rather than relying on
a preload-level limit.

## TypeScript + Prisma

Bad:

```ts
const incident = await prisma.incident.findUnique({
  where: { id },
  include: { signals: true },
});

const topSignals = incident.signals
  .sort((a, b) => b.createdAt.getTime() - a.createdAt.getTime())
  .slice(0, limit);
```

The relation is loaded before the cap is applied.

Use an ordered bounded query:

```ts
const topSignals = await prisma.signal.findMany({
  where: { incidentId: id },
  orderBy: { createdAt: "desc" },
  take: limit,
});
```

If the response needs a true total:

```ts
const [total, topSignals] = await Promise.all([
  prisma.signal.count({ where: { incidentId: id } }),
  prisma.signal.findMany({
    where: { incidentId: id },
    orderBy: { createdAt: "desc" },
    take: limit,
  }),
]);

return {
  totalCount: total,
  signals: topSignals,
  truncated: total > topSignals.length,
};
```

The same rule applies to Drizzle or any query builder: push `limit` into the
SQL-producing query, and use a separate count when the API promises a true
total.

## Review Checklist

- Does the diff advertise a cap, top-N list, newest/oldest list, or truncated
  flag?
- Is the cap enforced in the query that reads rows?
- Are totals computed from an aggregate/count query rather than a sliced list?
- If a parent has nested children, is the limit per parent or across the whole
  result set? The code must make that distinction explicit.
- Does the test data scale beyond the cap so the bug would be visible?

## Assertion Pattern

Bounded read models need an assertion that fails when row count grows but the
physical work grows with it.

For Ecto, attach to the repo telemetry event used by the application, run the
same read with fixture sizes such as 5, 50, and 500 rows, and assert the query
count stays constant for the chosen shape. Also assert any `total_count` or
`truncated` metadata against the uncapped fixture size.

For Prisma, enable query-event logging or use the repo's database test helper
to count issued queries. The test should prove the handler does not fetch an
unbounded relation and slice it after the fact. When query counting is not
available, assert against generated SQL, a query-spy boundary, or a repository
method that exposes the bounded query as the public contract.

## Enforcement

- Static lint is repo-specific. Canary's `Canary.Checks.PreloadThenTake`
  catches one Ecto version of this shape.
- Review catalogs should cite this reference from local entries such as
  `P-07 - Preload-then-take on bounded read models`.
- Runtime or acceptance tests should attach telemetry/query-count evidence for
  bounded read models with totals.

## Primary References

- Ecto preload queries: https://hexdocs.pm/ecto/Ecto.Query.html#preload/3
- Ecto aggregate/count: https://hexdocs.pm/ecto/Ecto.Repo.html#aggregate/4
- Prisma filtering, sorting, and pagination: https://www.prisma.io/docs/orm/prisma-client/queries/filtering-and-sorting
- Prisma Client `findMany` and `count`: https://www.prisma.io/docs/orm/reference/prisma-client-reference
