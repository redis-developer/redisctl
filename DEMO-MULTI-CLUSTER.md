# Multi-Cluster Enterprise Demo

## Quick Start

```bash
# Start all three clusters (takes ~2-3 minutes)
docker compose -f docker/docker-compose.multi-cluster.yml up -d

# Watch initialization
docker compose -f docker/docker-compose.multi-cluster.yml logs -f

# Verify all clusters are up
docker ps | grep cluster
```

## Configure Profiles

```bash
# West Region (port 9443)
redisctl profile create cluster-west \
  --type enterprise \
  --enterprise-url https://localhost:9443 \
  --enterprise-user admin@redis.local \
  --enterprise-password Redis123! \
  --enterprise-insecure

# East Region (port 9543)
redisctl profile create cluster-east \
  --type enterprise \
  --enterprise-url https://localhost:9543 \
  --enterprise-user admin@redis.local \
  --enterprise-password Redis123! \
  --enterprise-insecure

# Central Region (port 9643)
redisctl profile create cluster-central \
  --type enterprise \
  --enterprise-url https://localhost:9643 \
  --enterprise-user admin@redis.local \
  --enterprise-password Redis123! \
  --enterprise-insecure
```

## Verify Setup

```bash
# List profiles
redisctl profile list

# Quick check each cluster
redisctl -p cluster-west enterprise cluster get
redisctl -p cluster-east enterprise cluster get
redisctl -p cluster-central enterprise cluster get
```

## MCP Setup

The MCP server uses the configured profiles. It can switch between clusters using the `profile_set_default_enterprise` tool.

```bash
# Build if needed
cargo build --release -p redisctl-mcp

# Run MCP (write operations enabled for demo)
./target/release/redisctl-mcp --read-only=false
```

Or add to your `.mcp.json`:
```json
{
  "mcpServers": {
    "redisctl": {
      "command": "/path/to/redisctl-mcp",
      "args": ["--read-only=false"]
    }
  }
}
```

**How multi-cluster works:** The AI uses `profile_list` to discover clusters, then `profile_set_default_enterprise` to switch between them. Each query uses the current default profile.

---

## Demo Scenarios

### 1. Multi-Cluster Overview

**Prompt:** "Show me all my Redis Enterprise clusters"

Expected workflow:
1. AI calls `profile_list` to discover enterprise profiles
2. For each cluster, AI calls `profile_set_default_enterprise` then `get_cluster`
3. Aggregates and presents results

Shows:
- west-region (1 node, 2 databases)
- east-region (1 node, 3 databases)
- central-region (1 node, 3 databases)

### 2. Cross-Cluster Database Inventory

**Prompt:** "List all databases across all my clusters"

Expected: Uses `list_databases` for each profile, aggregates results showing all 8 databases with their clusters.

### 3. Total Memory Usage

**Prompt:** "What's my total memory allocation across all clusters?"

Expected: Sums memory_size from all databases:
- west: 100MB + 100MB = 200MB
- east: 200MB + 50MB + 100MB = 350MB
- central: 300MB + 150MB + 100MB = 550MB
- Total: ~1.1GB

### 4. License Check

**Prompt:** "Check the license status on all my clusters"

Expected: Uses `get_license` for each profile, shows expiration status and shard limits.

### 5. License Usage Audit

**Prompt:** "Show me license usage across all clusters - am I approaching any limits?"

Expected: Uses `get_license_usage` for each, compares against limits.

### 6. Find Databases Without Persistence

**Prompt:** "Which databases don't have persistence enabled?"

Expected: Lists databases where data_persistence is "disabled" - useful for compliance audits.

### 7. Cluster Health Check

**Prompt:** "Are there any alerts on my clusters?"

Expected: Uses `list_alerts` for each profile, reports any issues.

### 8. Stats Aggregation

**Prompt:** "What's the total CPU and memory utilization across all clusters?"

Expected: Uses `get_cluster_stats` for each, aggregates metrics.

### 9. Create Database (Write Demo)

**Prompt:** "Create a new 256MB database called 'analytics' on the central cluster with AOF persistence"

Expected: Uses `create_enterprise_database` with appropriate params.

### 10. Update License (Key Use Case)

**Prompt:** "Update the license on cluster-west with this license string: [paste license]"

Expected: Uses `update_license` tool - shows the license management workflow.

---

## CLI Demo Commands

### Cross-cluster queries (for comparison)

```bash
# All databases across clusters
for p in cluster-west cluster-east cluster-central; do
  echo "=== $p ==="
  redisctl -p $p enterprise database list -o json -q '[].{name: name, memory_mb: memory_size}'
done

# Total memory (bash)
total=0
for p in cluster-west cluster-east cluster-central; do
  mem=$(redisctl -p $p enterprise database list -o json -q '[].memory_size | sum(@)')
  echo "$p: $mem bytes"
  total=$((total + mem))
done
echo "Total: $((total / 1024 / 1024)) MB"

# License check across clusters
for p in cluster-west cluster-east cluster-central; do
  echo "=== $p ==="
  redisctl -p $p enterprise license get -o json -q '{expired: expired, shards_limit: shards_limit, expiration: expiration_date}'
done
```

---

## Cleanup

```bash
docker compose -f docker/docker-compose.multi-cluster.yml down -v
redisctl profile delete cluster-west
redisctl profile delete cluster-east
redisctl profile delete cluster-central
```

---

## Key Differentiators to Highlight

1. **Unified Management** - One tool, multiple clusters (RE UI is single-cluster)
2. **Programmatic Access** - CLI and MCP for automation
3. **AI-Powered Operations** - Natural language queries via MCP
4. **Cross-Cluster Visibility** - Aggregate stats, find issues across fleet
5. **License Management** - Check, update, validate across clusters
6. **Audit Capabilities** - Find non-compliant configs across all clusters
