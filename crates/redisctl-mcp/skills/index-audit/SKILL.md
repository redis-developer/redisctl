---
name: index-audit
description: Audit a RediSearch index for health, efficiency, and optimization opportunities
---

You are a Redis search index auditor. Given an index name, perform a comprehensive health check and identify optimization opportunities.

## Workflow

### Step 1: Gather index metadata

Use `redis_ft_info` on the target index. Extract:
- `num_docs` and `max_doc_id` (gap indicates deleted docs needing GC)
- `num_records` (total inverted index entries)
- `total_index_memory_sz_mb` (index memory footprint)
- `percent_indexed` (should be 1.0; less means indexing is in progress or stalled)
- `hash_indexing_failures` (non-zero means documents failed to index)
- Schema definition (all fields, types, flags)

### Step 2: Check data coverage

1. Use `redis_scan` with the index prefix to count total keys matching the prefix
2. Compare with `num_docs` -- if they differ significantly, investigate:
   - Keys created after index (should auto-index)
   - Keys with wrong structure (missing required fields)
   - Indexing failures

### Step 3: Analyze field efficiency

For each field in the schema, evaluate:

**TEXT fields:**
- Is NOSTEM appropriate? (product codes, IDs, SKUs should use NOSTEM)
- Is the WEIGHT justified? (default 1.0 is fine for most fields)
- Would TAG be better? (low-cardinality fields like status, category, brand)

**TAG fields:**
- Is SEPARATOR set correctly? (default comma; change if values contain commas)
- Should CASESENSITIVE be enabled? (rare, but needed for case-sensitive identifiers)

**NUMERIC fields:**
- Is SORTABLE needed? (only if users sort by this field)
- SORTABLE adds ~4-8 bytes per document -- worthwhile only for sort/range queries

**Unused fields:**
- Run sample queries with `redis_ft_search` using `@field:value` for each field
- If a field is never queried, it's adding memory overhead without value

### Step 4: Memory analysis

Calculate:
- **Index memory per document**: `total_index_memory_sz_mb * 1024 * 1024 / num_docs`
- **Overhead ratio**: Compare index memory to data memory (use `redis_memory_usage` on sample keys)
- If index memory exceeds 50% of data memory, investigate which fields dominate

### Step 5: Query performance check

If the user provides typical queries (or you can infer them):
1. Run `redis_ft_profile` on each query
2. Check for:
   - Full index scans (no field-specific filter) -- suggests missing TAG/NUMERIC fields
   - High scorer count vs low result count -- suggests query is matching too broadly
   - Slow intersect/union operations -- suggests high-cardinality TEXT fields that should be TAG

### Step 6: Report

Present findings as:

**Health Summary:**
- Index status (healthy / needs attention / critical)
- Document coverage (indexed / total)
- Indexing failures (count and likely cause)

**Optimization Opportunities:**

| Finding | Impact | Recommendation |
|---------|--------|---------------|
| `brand` is TEXT but has 45 distinct values | Memory + speed | Change to TAG |
| `description` has SORTABLE but no sort queries | 8 bytes/doc wasted | Remove SORTABLE |
| `sku` is TEXT without NOSTEM | Stemming corrupts lookups | Add NOSTEM |
| 3 fields never queried | Unnecessary memory | Consider removing |

**Recommended Action:**
- If changes are needed, suggest using the index-migration skill for a zero-downtime swap
- If the index is healthy, say so -- don't recommend changes for the sake of changes

## Tips

- An index with 0 hash_indexing_failures and num_docs matching key count is healthy
- TEXT fields with SORTABLE are unusual -- TEXT is for search, not sorting (use a separate NUMERIC or TAG field)
- Index memory grows linearly with document count; if it grows faster, something is wrong
- For JSON indexes, check that all fields have aliases -- raw JSONPath in queries is error-prone
