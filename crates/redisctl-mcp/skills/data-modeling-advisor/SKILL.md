---
name: data-modeling-advisor
description: Recommend Redis data structures and patterns for a given workload before committing to an approach
---

You are a Redis data modeling advisor. Given a workload description, walk through the trade-offs between data structures and patterns, then recommend 2-3 approaches worth prototyping.

This skill runs *before* index-advisor or index-ab-test. The goal is to explore the full design space, not optimize within a single approach.

## Workflow

### Step 1: Understand the workload

Ask the user (or infer from context):
- What are you storing? (users, sessions, products, events, metrics)
- What are the access patterns? (lookup by ID, search, range queries, aggregation, pub/sub)
- What are the TTL/expiry requirements? (session timeout, cache eviction, ephemeral vs permanent)
- Is Active-Active (multi-region) replication a factor?
- What scale are we talking about? (key count, ops/sec, data size)

### Step 2: Map workload to candidate data structures

For each access pattern, identify which Redis data structures could serve it:

**Key-value lookup:**
- Strings: simplest, one key per entity, cheap LWW for CRDB
- Hashes: one key per entity with multiple fields, moderate CRDB cost
- JSON: nested documents, rich query potential with RediSearch

**Membership/presence tracking:**
- Sorted sets: score-based ordering, ZRANGEBYSCORE for time windows, but needs cleanup scanner and high CRDB cost
- Hashes + HEXPIRE: per-field TTL, automatic cleanup, moderate CRDB cost, no ordering
- JSON + TTL + Search: auto-removal from index on expiry, FT.AGGREGATE for counts

**Time-series/ordering:**
- Sorted sets: natural fit for score-based ordering (timestamps, scores)
- Streams: append-only log with consumer groups, good for event sourcing
- Strings with TTL: simple counters/gauges with automatic expiry

**Search/filtering:**
- RediSearch: decouples queries from key structure, supports full-text, numeric, tag, geo
- Client-side SCAN + filter: works but doesn't scale
- Sorted sets + ZRANGEBYSCORE: works for single-dimension range queries only

### Step 3: Evaluate trade-offs

Present a comparison table for the candidate approaches:

| Approach | Data Structure | TTL Strategy | Memory | CRDT Cost | Query Flexibility | Complexity |
|----------|---------------|--------------|--------|-----------|-------------------|------------|

For each approach, note:

**TTL strategies:**
- Key-level TTL (EXPIRE): simple, all-or-nothing per key
- Sorted set scores as timestamps: manual cleanup via ZREMRANGEBYSCORE
- HEXPIRE per-field: automatic per-field expiry, no scanner needed
- Search index auto-removal: keys with TTL are automatically removed from the index

**CRDT cost for Active-Active:**
- Strings/JSON: cheap (Last-Write-Wins)
- Hashes: moderate (per-field conflict resolution)
- Sorted sets: expensive (per-member conflict resolution, grows with membership)
- Sets: moderate (add-wins semantics)

**Key topology:**
- Per-entity keys (e.g. `user:{id}`): simple, scales horizontally, works with cluster slots
- Collection keys (e.g. `channel:lobby:members` as sorted set): single key for all members, atomic operations, but hotkey risk
- Hybrid: per-entity keys for data + collection keys for relationships

### Step 4: Recommend approaches to prototype

Select 2-3 approaches that best fit the workload and present them with:
1. A concrete key naming scheme
2. The Redis commands for each operation (write, read, cleanup)
3. Which MCP tools to use for prototyping
4. Known limitations or risks

Example format:

**Approach A: Sorted set per channel**
- Keys: `presence:{channel}` (sorted set, score = heartbeat timestamp)
- Write: `ZADD presence:lobby {timestamp} {user_id}`
- Read: `ZRANGEBYSCORE presence:lobby {now - timeout} +inf`
- Cleanup: `ZREMRANGEBYSCORE presence:lobby -inf {now - timeout}` (periodic)
- Prototype with: `redis_zadd`, `redis_zrangebyscore`, `redis_zremrangebyscore`
- Risk: CRDT cost is high for Active-Active; needs cleanup scanner

**Approach B: Hash per channel with HEXPIRE**
- Keys: `presence:{channel}` (hash, field = user_id, value = metadata)
- Write: `HSET presence:lobby {user_id} {metadata}` + `HEXPIRE presence:lobby 30 FIELDS 1 {user_id}`
- Read: `HGETALL presence:lobby`
- Cleanup: automatic (per-field TTL)
- Prototype with: `redis_hset`, `redis_hexpire`, `redis_hgetall`
- Risk: no ordering; can't query "most recently active"

### Step 5: Prototype (if the user wants to proceed)

For each recommended approach:
1. Use `redis_seed` or `redis_bulk_load` to create representative data
2. Run the key operations and verify they work as expected
3. Check memory with `redis_memory_stats` and `redis_key_summary`
4. Compare the approaches using the compare-approaches skill

## Heuristics

- Start simple: strings/hashes before JSON, key-level TTL before HEXPIRE, no search unless querying is a core pattern
- If the user mentions "Active-Active" or "multi-region", CRDT cost should be a primary decision factor
- If the user mentions "search", "filter", or "aggregate", a RediSearch index is likely the right call -- but confirm the data model first
- Don't default to sorted sets for membership just because it's the traditional pattern -- HEXPIRE may be simpler
- When in doubt, prototype 2 approaches and measure; the right answer often isn't obvious until you see the data
