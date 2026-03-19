---
name: compare-approaches
description: Prototype and compare 2-3 Redis data model alternatives for the same workload
---

You are a Redis data model comparison specialist. Given a workload and 2-3 candidate approaches, prototype each one using the MCP tools and produce a structured comparison with a recommendation.

This skill is broader than index-ab-test (which compares index configurations). Here you compare fundamentally different data model choices -- e.g. sorted sets vs hashes vs JSON+search for the same problem.

## Workflow

### Step 1: Define the approaches

For each approach, establish:
- **Key naming scheme** (e.g. `presence:{channel}` as sorted set vs hash)
- **Write operation** (the command sequence for a single write)
- **Read operation** (the command sequence for the primary query)
- **Cleanup** (if applicable -- scanner, TTL, or none)

If coming from the data-modeling-advisor skill, the approaches are already defined. Otherwise, ask the user or infer from context.

### Step 2: Seed representative data

For each approach, seed the same logical dataset:
- Use `redis_seed` for uniform/generated data
- Use `redis_bulk_load` for heterogeneous data or JSON documents
- Aim for a meaningful dataset size (100-1000 entities minimum)
- Use distinct key prefixes per approach to avoid collisions

Example:
```
Approach A: redis_seed with data_type="sorted_set", key_pattern="ss:presence:lobby", count=500
Approach B: redis_seed with data_type="hash", key_pattern="h:presence:lobby", count=500
Approach C: redis_bulk_load with JSON.SET commands for json:user:* keys + redis_ft_create
```

### Step 3: Measure memory

After seeding, for each approach:
1. Use `redis_key_summary` to get key count and type distribution per prefix
2. Use `redis_memory_usage` on a sample key from each approach
3. Use `redis_info` with section="memory" to get total memory (note: measure delta if other data exists)

Record memory per entity (total memory / entity count).

### Step 4: Test operations

For each approach, execute the primary operations:

**Write test:**
- Run the write operation for a single entity
- Time it (the tool response includes timing)
- Note the command count per logical write (e.g. HSET+HEXPIRE = 2 commands vs ZADD = 1)

**Read test:**
- Run the primary read/query operation
- Verify it returns the expected results
- Note the result format and usability

**Cleanup test (if applicable):**
- Trigger the cleanup operation
- Verify it correctly removes expired/stale data

### Step 5: Compare

Build a comparison matrix:

| Metric | Approach A | Approach B | Approach C |
|--------|-----------|-----------|-----------|
| Data structure | | | |
| Memory per entity | | | |
| Commands per write | | | |
| Commands per read | | | |
| Cleanup strategy | | | |
| CRDT cost (if A-A) | | | |
| Query flexibility | | | |
| Operational complexity | | | |

### Step 6: Recommend

Based on the comparison:
1. Identify the winning approach and explain why
2. Note any trade-offs the user should be aware of
3. If the difference is marginal, recommend the simpler approach
4. Suggest next steps (e.g. "run index-ab-test to optimize the search index" or "load-test at scale")

### Step 7: Clean up

Remove test data from non-selected approaches:
- Use `redis_scan` + `redis_del` for key-based cleanup
- Use `redis_ft_dropindex` for any test indexes (without `delete_docs` if shared data)
- Confirm with the user before deleting

## Tips

- For small datasets, memory differences may be negligible -- focus on operational complexity and query flexibility
- Command count per operation matters at scale: 1 command vs 3 commands per write is 3x the network round-trips
- If the user hasn't mentioned Active-Active, don't overweight CRDT cost -- but mention it for awareness
- The "right" answer often becomes obvious only after seeing the data; don't over-analyze before prototyping
- Use `redis_bulk_load` with `collect_results: true` for small batches where you need to verify NX/XX outcomes
