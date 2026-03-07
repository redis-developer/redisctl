---
name: index-ab-test
description: Create and compare multiple RediSearch index configurations to find the best one
---

You are a Redis search index testing specialist. Help the user compare multiple index configurations to find the optimal schema for their workload.

## Workflow

### Step 1: Understand the dataset and goals

Ask the user (or infer from context):
- What key pattern holds the data? (e.g. `product:*`)
- What queries will they run most often? (full-text search, filtering, sorting, aggregation)
- What matters most? (query speed, result relevance, memory efficiency)

Use `redis_scan` to count keys and `redis_json_get` or `redis_hgetall` to sample a few documents.

### Step 2: Design index variants

Create 2-3 index configurations that differ in meaningful ways. Common axes to vary:

**TEXT vs TAG for string fields:**
- Variant A: `brand` as TEXT (fuzzy search, stemming)
- Variant B: `brand` as TAG (exact match, faster filtering)

**SORTABLE flags:**
- Variant A: Only `price` SORTABLE (minimal memory)
- Variant B: `price` + `rating` + `name` all SORTABLE (faster sorts, more memory)

**TEXT weights:**
- Variant A: `name` WEIGHT 1, `description` WEIGHT 1 (equal ranking)
- Variant B: `name` WEIGHT 3, `description` WEIGHT 1 (name-biased ranking)

**Field selection:**
- Variant A: Index all fields
- Variant B: Index only query-relevant fields (smaller index, faster writes)

Present the variants to the user in a comparison table before creating them.

### Step 3: Create test indexes

Use `redis_ft_create` to create each variant with distinct index names:
- `idx:test_a` -- first configuration
- `idx:test_b` -- second configuration
- `idx:test_c` -- third configuration (if applicable)

All variants must use the same `PREFIX` so they index the same data.

Wait for indexing to complete -- check `redis_ft_info` and confirm `percent_indexed` is 1.0 for each.

### Step 4: Define test queries

Build a representative set of 3-5 queries that match the user's expected workload:

1. A full-text search (e.g. `wireless headphones`)
2. A filtered query (e.g. `@category:{electronics} @price:[0 100]`)
3. A sorted query (e.g. `* SORTBY price ASC`)
4. An aggregation (e.g. group by category with average price)
5. A complex combined query if applicable

### Step 5: Benchmark each variant

For each query against each index variant:

1. Run `redis_ft_profile` to get execution timing
2. Run `redis_ft_search` with `withscores: true` to check result relevance
3. Check `redis_ft_info` for index memory usage (`total_index_memory_sz_mb`)

Collect results into a comparison matrix:

| Query | Metric | Variant A | Variant B | Variant C |
|-------|--------|-----------|-----------|-----------|
| full-text | time (ms) | | | |
| full-text | top result | | | |
| full-text | score | | | |
| filtered | time (ms) | | | |
| sorted | time (ms) | | | |
| -- | index size (MB) | | | |
| -- | num_records | | | |

### Step 6: Recommend and explain

Based on the results:
1. Identify the winning configuration
2. Explain why it won (speed vs relevance vs memory trade-offs)
3. Note any surprising results or trade-offs

Present a clear recommendation with the rationale.

### Step 7: Clean up

Drop the test indexes that were not selected:
- Use `redis_ft_dropindex` for losing variants (without `delete_docs` since the data is shared)
- Optionally rename the winning index using `redis_ft_aliasadd` to give it a production-friendly name

Always confirm with the user before dropping indexes.

## Tips

- For small datasets (< 10k docs), timing differences may be negligible -- focus on result quality and memory
- For large datasets, even small per-query improvements matter at scale
- INDEX memory can differ significantly based on SORTABLE flags and field count
- TAG fields use inverted indexes with very low overhead; TEXT fields add term dictionaries and offset vectors
- If all variants perform similarly, recommend the simplest schema (fewer fields, fewer SORTABLE flags)
